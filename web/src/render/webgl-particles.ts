import type { RenderFrame } from "../physics/frame";
import type { Camera } from "./camera";
import { createBlobUniforms } from "./blob-uniforms";
import {
  createFieldTarget,
  type FieldTarget,
  particleSurfaceFor,
  resizeFieldTarget,
  type SharedParticleBuffers,
} from "./field-pass";
import { linkProgram, requireBuffer } from "./gl-program";

const POINT_VERTEX = `#version 300 es
in vec2 aPosition;
in float aRadius;
in vec4 aColor;
uniform vec2 uResolution;
uniform float uPointScale;
uniform float uMaxPointSize;
out vec4 vColor;
void main() {
  vec2 clip = (aPosition / uResolution) * 2.0 - 1.0;
  clip.y = -clip.y;
  gl_Position = vec4(clip, 0.0, 1.0);
  gl_PointSize = clamp(aRadius * uPointScale, 1.0, uMaxPointSize);
  vColor = aColor;
}
`;

const POINT_FRAGMENT = `#version 300 es
precision mediump float;
in vec4 vColor;
out vec4 fragColor;
void main() {
  vec2 point = gl_PointCoord * 2.0 - 1.0;
  float dist2 = dot(point, point);
  if (dist2 > 1.0) {
    discard;
  }
  float weight = 1.0 - dist2;
  weight = weight * weight;
  float alpha = vColor.a * weight;
  fragColor = vec4(vColor.rgb * alpha, alpha);
}
`;

const COMPOSITE_VERTEX = `#version 300 es
in vec2 aClip;
out vec2 vUv;
void main() {
  vUv = aClip * 0.5 + 0.5;
  gl_Position = vec4(aClip, 0.0, 1.0);
}
`;

const COMPOSITE_FRAGMENT = `#version 300 es
precision mediump float;
uniform sampler2D uField;
uniform vec3 uBackground;
uniform float uShadeStart;
uniform float uShadeEnd;
uniform float uShadeFloor;
in vec2 vUv;
out vec4 fragColor;
void main() {
  vec4 sum = texture(uField, vUv);
  float edge = smoothstep(0.42, 0.62, sum.a);
  vec3 color = sum.rgb / max(sum.a, 0.0001);
  // Same curve as densityShade(): packed overlap darkens the recovered color.
  float packed = smoothstep(uShadeStart, uShadeEnd, sum.a);
  color *= mix(1.0, uShadeFloor, packed);
  fragColor = vec4(mix(uBackground, color, edge), 1.0);
}
`;

const CLIP_QUAD = new Float32Array([-1, -1, 1, -1, -1, 1, 1, 1]);

const BLOB_PASS = Symbol("shaded-blob");

type BlobPass = {
  readonly pointProgram: WebGLProgram;
  readonly compositeProgram: WebGLProgram;
  readonly pointVao: WebGLVertexArrayObject;
  readonly compositeVao: WebGLVertexArrayObject;
  readonly target: FieldTarget;
  readonly maxPointSize: number;
  readonly uniforms: ReturnType<typeof createBlobUniforms>;
};

function createBlobPass(buffers: SharedParticleBuffers): BlobPass | undefined {
  const gl = buffers.gl;
  const pointProgram = linkProgram(gl, POINT_VERTEX, POINT_FRAGMENT);
  const compositeProgram = linkProgram(
    gl,
    COMPOSITE_VERTEX,
    COMPOSITE_FRAGMENT,
  );
  const clipBuffer = requireBuffer(gl);
  const pointVao = gl.createVertexArray();
  const compositeVao = gl.createVertexArray();
  const target = createFieldTarget(gl, 1);
  if (
    pointProgram === undefined ||
    compositeProgram === undefined ||
    clipBuffer === undefined ||
    pointVao === null ||
    compositeVao === null ||
    target === undefined
  ) {
    return undefined;
  }

  const positionLocation = gl.getAttribLocation(pointProgram, "aPosition");
  const radiusLocation = gl.getAttribLocation(pointProgram, "aRadius");
  const colorLocation = gl.getAttribLocation(pointProgram, "aColor");
  const clipLocation = gl.getAttribLocation(compositeProgram, "aClip");
  if (
    positionLocation < 0 ||
    radiusLocation < 0 ||
    colorLocation < 0 ||
    clipLocation < 0
  ) {
    return undefined;
  }

  gl.bindVertexArray(pointVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, buffers.position);
  gl.enableVertexAttribArray(positionLocation);
  gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, buffers.radius);
  gl.enableVertexAttribArray(radiusLocation);
  gl.vertexAttribPointer(radiusLocation, 1, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, buffers.color);
  gl.enableVertexAttribArray(colorLocation);
  gl.vertexAttribPointer(colorLocation, 4, gl.FLOAT, false, 0, 0);

  gl.bindVertexArray(compositeVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, clipBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, CLIP_QUAD, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(clipLocation);
  gl.vertexAttribPointer(clipLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindVertexArray(null);

  return {
    pointProgram,
    compositeProgram,
    pointVao,
    compositeVao,
    target,
    maxPointSize: buffers.maxPointSize,
    uniforms: createBlobUniforms(gl, pointProgram, compositeProgram),
  };
}

/**
 * Draws the shaded blob into the particle canvas.
 *
 * Packed overlap darkens the recovered particle color. Returns false when
 * WebGL2 or a float or half-float framebuffer is unavailable so the caller
 * can fall back to the canvas metaball.
 */
export function drawShadedBlob(
  canvas: HTMLCanvasElement | undefined,
  frame: RenderFrame,
  camera: Camera,
  devicePixelRatio: number,
  densityShading = true,
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
  const blob = surface.pass(BLOB_PASS, createBlobPass);
  if (blob === undefined) {
    return false;
  }
  const { width, height, pixelRatio } = size;
  const target = blob.target;
  if (target.width !== width || target.height !== height) {
    if (!resizeFieldTarget(gl, target, width, height)) {
      surface.disablePass(BLOB_PASS);
      return false;
    }
  }

  const count = surface.upload(frame, camera);
  gl.bindFramebuffer(gl.FRAMEBUFFER, target.framebuffer);
  gl.viewport(0, 0, width, height);
  gl.clearColor(0, 0, 0, 0);
  gl.clear(gl.COLOR_BUFFER_BIT);
  if (count > 0) {
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.useProgram(blob.pointProgram);
    gl.bindVertexArray(blob.pointVao);
    blob.uniforms.point(
      camera.viewport.width,
      camera.viewport.height,
      pixelRatio,
      blob.maxPointSize,
    );
    gl.drawArrays(gl.POINTS, 0, count);
    gl.disable(gl.BLEND);
  }

  gl.bindFramebuffer(gl.FRAMEBUFFER, null);
  gl.viewport(0, 0, width, height);
  gl.useProgram(blob.compositeProgram);
  gl.bindVertexArray(blob.compositeVao);
  gl.activeTexture(gl.TEXTURE0);
  gl.bindTexture(gl.TEXTURE_2D, target.textures[0] ?? null);
  blob.uniforms.composite(densityShading);
  gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
  gl.bindVertexArray(null);
  return true;
}
