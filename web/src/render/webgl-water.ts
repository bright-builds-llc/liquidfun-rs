/**
 * The Water render mode on the shared particle surface (D-06).
 *
 * A field pass splats every particle into two RGBA16F attachments (color
 * field and speed-weighted foam). A highp shade pass then refracts the walls
 * backdrop by the field gradient and adds Beer-Lambert tint, Blinn-Phong
 * highlights and foam. Water resources are created lazily on the first
 * `drawWater`, so canvases that never draw water keep shaded blob's exact
 * program and uniform counts. A water failure disables only the water pass.
 */
import type { RenderFrame } from "../physics/frame";
import type { Camera } from "./camera";
import { CANVAS_RGB } from "./canvas";
import {
  createFieldTarget,
  deleteFieldTarget,
  type FieldSize,
  type FieldTarget,
  particleSurfaceFor,
  resizeFieldTarget,
  type SharedParticleBuffers,
} from "./field-pass";
import { requireBuffer } from "./gl-program";
import {
  type BackdropCanvas,
  createWaterBackdrop,
  type WaterBackdrop,
} from "./water-backdrop";
import { liveWaterLimitsOk } from "./water-capability";
import { linkWaterPrograms, type WaterPrograms } from "./water-programs";
import { createWaterUniforms } from "./water-uniforms";

/** A backdrop canvas that WebGL can also upload as a texture. */
export type WaterBackdropCanvas = BackdropCanvas & TexImageSource;

const CLIP_QUAD = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);
// Shown until the first walls raster uploads; avoids sampling an empty texture.
const PLACEHOLDER_TEXEL = new Uint8Array([...CANVAS_RGB, 255]);

const WATER_PASS = Symbol("water");

type WaterPass = {
  readonly programs: WaterPrograms;
  readonly pointVao: WebGLVertexArrayObject;
  readonly shadeVao: WebGLVertexArrayObject;
  readonly speedBuffer: WebGLBuffer;
  readonly clipBuffer: WebGLBuffer;
  readonly speedLocation: number;
  readonly target: FieldTarget;
  readonly backdropTexture: WebGLTexture;
  readonly backdrop: WaterBackdrop<WaterBackdropCanvas>;
  readonly maxPointSize: number;
  readonly uniforms: ReturnType<typeof createWaterUniforms>;
  uploadedRevision: number;
};

type AttributeLocations = {
  readonly position: number;
  readonly radius: number;
  readonly color: number;
  readonly speed: number;
  readonly clip: number;
};

function attributeLocations(
  gl: WebGL2RenderingContext,
  programs: WaterPrograms,
): AttributeLocations | undefined {
  const locations = {
    position: gl.getAttribLocation(programs.point, "aPosition"),
    radius: gl.getAttribLocation(programs.point, "aRadius"),
    color: gl.getAttribLocation(programs.point, "aColor"),
    speed: gl.getAttribLocation(programs.point, "aSpeed"),
    clip: gl.getAttribLocation(programs.shade, "aClip"),
  };
  return Object.values(locations).some((location) => location < 0)
    ? undefined
    : locations;
}

function bindAttribute(
  gl: WebGL2RenderingContext,
  buffer: WebGLBuffer,
  location: number,
  size: number,
): void {
  gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.enableVertexAttribArray(location);
  gl.vertexAttribPointer(location, size, gl.FLOAT, false, 0, 0);
}

function configureVertexArrays(
  buffers: SharedParticleBuffers,
  pointVao: WebGLVertexArrayObject,
  shadeVao: WebGLVertexArrayObject,
  speedBuffer: WebGLBuffer,
  clipBuffer: WebGLBuffer,
  locations: AttributeLocations,
): void {
  const gl = buffers.gl;
  gl.bindVertexArray(pointVao);
  bindAttribute(gl, buffers.position, locations.position, 2);
  bindAttribute(gl, buffers.radius, locations.radius, 1);
  bindAttribute(gl, buffers.color, locations.color, 4);
  // Enabled per frame only while the speed lane is present.
  bindAttribute(gl, speedBuffer, locations.speed, 1);
  gl.bindVertexArray(shadeVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, clipBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, CLIP_QUAD, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(locations.clip);
  gl.vertexAttribPointer(locations.clip, 2, gl.FLOAT, false, 0, 0);
  gl.bindVertexArray(null);
}

function createBackdropTexture(
  gl: WebGL2RenderingContext,
): WebGLTexture | undefined {
  const texture = gl.createTexture();
  if (texture === null) {
    return undefined;
  }
  gl.bindTexture(gl.TEXTURE_2D, texture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA,
    1,
    1,
    0,
    gl.RGBA,
    gl.UNSIGNED_BYTE,
    PLACEHOLDER_TEXEL,
  );
  return texture;
}

/** GL objects of a Water pass; a failed creation may lack some of them. */
type WaterObjects = {
  readonly programs: WaterPrograms;
  readonly buffers: readonly (WebGLBuffer | undefined)[];
  readonly vertexArrays: readonly (WebGLVertexArrayObject | null)[];
  readonly maybeTarget: FieldTarget | undefined;
  readonly maybeBackdropTexture: WebGLTexture | undefined;
};

/** Frees Water GL objects, skipping any that were never created. */
function deleteWaterObjects(
  gl: WebGL2RenderingContext,
  objects: WaterObjects,
): void {
  gl.deleteProgram(objects.programs.point);
  gl.deleteProgram(objects.programs.shade);
  objects.buffers.forEach((buffer) => gl.deleteBuffer(buffer ?? null));
  objects.vertexArrays.forEach((vao) => gl.deleteVertexArray(vao));
  if (objects.maybeTarget !== undefined) {
    deleteFieldTarget(gl, objects.maybeTarget);
  }
  gl.deleteTexture(objects.maybeBackdropTexture ?? null);
}

function deleteWaterPass(gl: WebGL2RenderingContext, water: WaterPass): void {
  deleteWaterObjects(gl, {
    programs: water.programs,
    buffers: [water.speedBuffer, water.clipBuffer],
    vertexArrays: [water.pointVao, water.shadeVao],
    maybeTarget: water.target,
    maybeBackdropTexture: water.backdropTexture,
  });
}

function createWaterPass(
  buffers: SharedParticleBuffers,
  createBackdropCanvas: () => WaterBackdropCanvas | undefined,
): WaterPass | undefined {
  const gl = buffers.gl;
  if (!liveWaterLimitsOk(gl)) {
    return undefined;
  }
  const programs = linkWaterPrograms(gl);
  if (programs === undefined) {
    return undefined;
  }
  const speedBuffer = requireBuffer(gl);
  const clipBuffer = requireBuffer(gl);
  const pointVao = gl.createVertexArray();
  const shadeVao = gl.createVertexArray();
  const target = createFieldTarget(gl, 2);
  const backdropTexture = createBackdropTexture(gl);
  const locations = attributeLocations(gl, programs);
  if (
    speedBuffer === undefined ||
    clipBuffer === undefined ||
    pointVao === null ||
    shadeVao === null ||
    target === undefined ||
    backdropTexture === undefined ||
    locations === undefined
  ) {
    deleteWaterObjects(gl, {
      programs,
      buffers: [speedBuffer, clipBuffer],
      vertexArrays: [pointVao, shadeVao],
      maybeTarget: target,
      maybeBackdropTexture: backdropTexture,
    });
    return undefined;
  }
  configureVertexArrays(
    buffers,
    pointVao,
    shadeVao,
    speedBuffer,
    clipBuffer,
    locations,
  );
  return {
    programs,
    pointVao,
    shadeVao,
    speedBuffer,
    clipBuffer,
    speedLocation: locations.speed,
    target,
    backdropTexture,
    backdrop: createWaterBackdrop(createBackdropCanvas),
    maxPointSize: buffers.maxPointSize,
    uniforms: createWaterUniforms(gl, programs),
    // A fresh pass (also after a context restore) uploads the next raster.
    uploadedRevision: -1,
  };
}

/** Feeds the speed lane, or a constant 0 foam when it is absent (Pitfall 8). */
function bindSpeeds(
  gl: WebGL2RenderingContext,
  water: WaterPass,
  frame: RenderFrame,
  count: number,
): void {
  const maybeSpeeds = frame.maybeParticleSpeeds;
  gl.bindVertexArray(water.pointVao);
  if (maybeSpeeds === undefined || maybeSpeeds.length < count) {
    gl.disableVertexAttribArray(water.speedLocation);
    gl.vertexAttrib1f(water.speedLocation, 0);
    return;
  }
  gl.bindBuffer(gl.ARRAY_BUFFER, water.speedBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, maybeSpeeds.subarray(0, count), gl.DYNAMIC_DRAW);
  gl.enableVertexAttribArray(water.speedLocation);
}

/** Uploads the walls raster only when its revision changed (D-07). */
function syncBackdrop(
  gl: WebGL2RenderingContext,
  water: WaterPass,
  frame: RenderFrame,
  camera: Camera,
  size: FieldSize,
): boolean {
  const backdrop = water.backdrop;
  backdrop.update(frame, camera, size);
  const maybeRaster = backdrop.maybeRaster;
  if (maybeRaster !== undefined && backdrop.revision !== water.uploadedRevision) {
    gl.activeTexture(gl.TEXTURE2);
    gl.bindTexture(gl.TEXTURE_2D, water.backdropTexture);
    gl.texImage2D(
      gl.TEXTURE_2D,
      0,
      gl.RGBA,
      gl.RGBA,
      gl.UNSIGNED_BYTE,
      maybeRaster,
    );
    water.uploadedRevision = backdrop.revision;
  }
  return backdrop.matchesView;
}

function drawField(
  gl: WebGL2RenderingContext,
  water: WaterPass,
  camera: Camera,
  size: FieldSize,
  count: number,
): void {
  gl.bindFramebuffer(gl.FRAMEBUFFER, water.target.framebuffer);
  gl.viewport(0, 0, size.width, size.height);
  gl.clearColor(0, 0, 0, 0);
  gl.clear(gl.COLOR_BUFFER_BIT);
  if (count === 0) {
    return;
  }
  gl.enable(gl.BLEND);
  gl.blendFunc(gl.ONE, gl.ONE);
  gl.useProgram(water.programs.point);
  gl.bindVertexArray(water.pointVao);
  water.uniforms.point(
    camera.viewport.width,
    camera.viewport.height,
    size.pixelRatio,
    water.maxPointSize,
  );
  gl.drawArrays(gl.POINTS, 0, count);
  gl.disable(gl.BLEND);
}

function drawShade(
  gl: WebGL2RenderingContext,
  water: WaterPass,
  size: FieldSize,
  densityShading: boolean,
  hasBackdrop: boolean,
): void {
  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.viewport(0, 0, size.width, size.height);
  gl.useProgram(water.programs.shade);
  gl.bindVertexArray(water.shadeVao);
  const [field, foam] = water.target.textures;
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, field ?? null);
  gl.activeTexture(gl.TEXTURE1);
  gl.bindTexture(gl.TEXTURE_2D, foam ?? null);
  gl.activeTexture(gl.TEXTURE2);
  gl.bindTexture(gl.TEXTURE_2D, water.backdropTexture);
  water.uniforms.shade({ densityShading, ...size, hasBackdrop });
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  gl.bindVertexArray(null);
  gl.activeTexture(gl.TEXTURE0);
}

function defaultBackdropCanvas(): WaterBackdropCanvas | undefined {
  return typeof document === "undefined"
    ? undefined
    : document.createElement("canvas");
}

/**
 * Draws the Water look into the particle canvas.
 *
 * Returns false whenever Water cannot draw (no canvas, lost context, missing
 * float targets, draw-buffer or highp limits, link failure, incomplete
 * 2-attachment target) so the caller falls back to shaded blob, then the
 * canvas (D-12). A water failure never disables shaded blob.
 */
export function drawWater(
  canvas: HTMLCanvasElement | undefined,
  frame: RenderFrame,
  camera: Camera,
  devicePixelRatio: number,
  densityShading = true,
  maybeCreateBackdropCanvas: () =>
    | WaterBackdropCanvas
    | undefined = defaultBackdropCanvas,
): boolean {
  if (canvas === undefined) {
    return false;
  }
  const surface = particleSurfaceFor(canvas);
  const gl = surface.context();
  if (gl === undefined) {
    return false;
  }
  const size = surface.sizeFor(camera, devicePixelRatio);
  if (size === undefined) {
    return false;
  }
  const water = surface.pass(WATER_PASS, (buffers) =>
    createWaterPass(buffers, maybeCreateBackdropCanvas),
  );
  if (water === undefined) {
    return false;
  }
  const target = water.target;
  if (target.width !== size.width || target.height !== size.height) {
    // Research Pitfall 4: MRT can be incomplete where one target is not.
    if (!resizeFieldTarget(gl, target, size.width, size.height)) {
      deleteWaterPass(gl, water);
      surface.disablePass(WATER_PASS);
      return false;
    }
  }

  const count = surface.upload(frame, camera);
  bindSpeeds(gl, water, frame, count);
  const hasBackdrop = syncBackdrop(gl, water, frame, camera, size);
  drawField(gl, water, camera, size, count);
  drawShade(gl, water, size, densityShading, hasBackdrop);
  return true;
}
