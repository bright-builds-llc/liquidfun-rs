import type { RenderFrame } from "../physics/frame";
import { cappedDevicePixelRatio } from "./canvas";
import type { Camera } from "./camera";
import { fillProjectedParticleArrays } from "./projected-particle";
import { createBlobUniforms } from "./blob-uniforms";
import { createParticleMetadataCache } from "./particle-metadata";

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

type GlBuffer = WebGLBuffer;

type WebglResources = {
  readonly gl: WebGL2RenderingContext;
  readonly pointProgram: WebGLProgram;
  readonly compositeProgram: WebGLProgram;
  readonly pointVao: WebGLVertexArrayObject;
  readonly compositeVao: WebGLVertexArrayObject;
  readonly positionBuffer: GlBuffer;
  readonly radiusBuffer: GlBuffer;
  readonly colorBuffer: GlBuffer;
  readonly fieldTexture: WebGLTexture;
  readonly fieldFramebuffer: WebGLFramebuffer;
  readonly maxPointSize: number;
  readonly uniforms: ReturnType<typeof createBlobUniforms>;
  readonly metadata: ReturnType<typeof createParticleMetadataCache>;
};

type WebglSurface = {
  draw: (
    canvas: HTMLCanvasElement,
    frame: RenderFrame,
    camera: Camera,
    devicePixelRatio: number,
    densityShading: boolean,
  ) => boolean;
};

function compileShader(
  gl: WebGL2RenderingContext,
  type: number,
  source: string,
): WebGLShader | undefined {
  const shader = gl.createShader(type);
  if (shader === null) {
    return undefined;
  }
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (gl.getShaderParameter(shader, gl.COMPILE_STATUS) !== true) {
    gl.deleteShader(shader);
    return undefined;
  }
  return shader;
}

function linkProgram(
  gl: WebGL2RenderingContext,
  vertexSource: string,
  fragmentSource: string,
): WebGLProgram | undefined {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  if (vertex === undefined || fragment === undefined) {
    return undefined;
  }
  const program = gl.createProgram();
  if (program === null) {
    return undefined;
  }
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  gl.deleteShader(vertex);
  gl.deleteShader(fragment);
  if (gl.getProgramParameter(program, gl.LINK_STATUS) !== true) {
    gl.deleteProgram(program);
    return undefined;
  }
  return program;
}

function requireBuffer(gl: WebGL2RenderingContext): GlBuffer | undefined {
  return gl.createBuffer() ?? undefined;
}

function createResources(
  gl: WebGL2RenderingContext,
): WebglResources | undefined {
  if (gl.getExtension("EXT_color_buffer_float") === null) {
    return undefined;
  }
  const pointProgram = linkProgram(gl, POINT_VERTEX, POINT_FRAGMENT);
  const compositeProgram = linkProgram(
    gl,
    COMPOSITE_VERTEX,
    COMPOSITE_FRAGMENT,
  );
  const positionBuffer = requireBuffer(gl);
  const radiusBuffer = requireBuffer(gl);
  const colorBuffer = requireBuffer(gl);
  const clipBuffer = requireBuffer(gl);
  const pointVao = gl.createVertexArray();
  const compositeVao = gl.createVertexArray();
  const fieldTexture = gl.createTexture();
  const fieldFramebuffer = gl.createFramebuffer();
  if (
    pointProgram === undefined ||
    compositeProgram === undefined ||
    positionBuffer === undefined ||
    radiusBuffer === undefined ||
    colorBuffer === undefined ||
    clipBuffer === undefined ||
    pointVao === null ||
    compositeVao === null ||
    fieldTexture === null ||
    fieldFramebuffer === null
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
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.enableVertexAttribArray(positionLocation);
  gl.vertexAttribPointer(positionLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, radiusBuffer);
  gl.enableVertexAttribArray(radiusLocation);
  gl.vertexAttribPointer(radiusLocation, 1, gl.FLOAT, false, 0, 0);
  gl.bindBuffer(gl.ARRAY_BUFFER, colorBuffer);
  gl.enableVertexAttribArray(colorLocation);
  gl.vertexAttribPointer(colorLocation, 4, gl.FLOAT, false, 0, 0);

  gl.bindVertexArray(compositeVao);
  gl.bindBuffer(gl.ARRAY_BUFFER, clipBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, CLIP_QUAD, gl.STATIC_DRAW);
  gl.enableVertexAttribArray(clipLocation);
  gl.vertexAttribPointer(clipLocation, 2, gl.FLOAT, false, 0, 0);
  gl.bindVertexArray(null);

  const range = gl.getParameter(
    gl.ALIASED_POINT_SIZE_RANGE,
  ) as Float32Array | null;
  const maxPointSize = range?.[1] ?? 64;

  return {
    gl,
    pointProgram,
    compositeProgram,
    pointVao,
    compositeVao,
    positionBuffer,
    radiusBuffer,
    colorBuffer,
    fieldTexture,
    fieldFramebuffer,
    maxPointSize,
    uniforms: createBlobUniforms(gl, pointProgram, compositeProgram),
    metadata: createParticleMetadataCache(),
  };
}

function resizeField(
  resources: WebglResources,
  width: number,
  height: number,
): boolean {
  const gl = resources.gl;
  gl.bindTexture(gl.TEXTURE_2D, resources.fieldTexture);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
  gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
  gl.texImage2D(
    gl.TEXTURE_2D,
    0,
    gl.RGBA16F,
    width,
    height,
    0,
    gl.RGBA,
    gl.HALF_FLOAT,
    null,
  );
  gl.bindFramebuffer(gl.FRAMEBUFFER, resources.fieldFramebuffer);
  gl.framebufferTexture2D(
    gl.FRAMEBUFFER,
    gl.COLOR_ATTACHMENT0,
    gl.TEXTURE_2D,
    resources.fieldTexture,
    0,
  );
  return gl.checkFramebufferStatus(gl.FRAMEBUFFER) === gl.FRAMEBUFFER_COMPLETE;
}

function uploadParticles(
  resources: WebglResources,
  frame: RenderFrame,
  camera: Camera,
  positions: Float32Array,
  radii: Float32Array,
  colors: Float32Array,
): number {
  const gl = resources.gl;
  const { radiiChanged, colorsChanged } = resources.metadata.changes(
    frame,
    camera.scale,
  );
  const count = fillProjectedParticleArrays(
    frame,
    camera,
    positions,
    radii,
    colors,
    radiiChanged,
    colorsChanged,
  );

  gl.bindBuffer(gl.ARRAY_BUFFER, resources.positionBuffer);
  gl.bufferData(
    gl.ARRAY_BUFFER,
    positions.subarray(0, count * 2),
    gl.DYNAMIC_DRAW,
  );
  if (radiiChanged) {
    gl.bindBuffer(gl.ARRAY_BUFFER, resources.radiusBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, radii.subarray(0, count), gl.DYNAMIC_DRAW);
  }
  if (colorsChanged) {
    gl.bindBuffer(gl.ARRAY_BUFFER, resources.colorBuffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      colors.subarray(0, count * 4),
      gl.DYNAMIC_DRAW,
    );
  }
  return count;
}

function createWebglSurface(canvas: HTMLCanvasElement): WebglSurface {
  let maybeResources: WebglResources | undefined;
  let fieldWidth = 0;
  let fieldHeight = 0;
  let positions = new Float32Array(0);
  let radii = new Float32Array(0);
  let colors = new Float32Array(0);
  let shadersAvailable = true;
  const reset = () => {
    maybeResources = undefined;
    fieldWidth = 0;
    fieldHeight = 0;
    shadersAvailable = true;
  };
  canvas.addEventListener("webglcontextlost", (event) => {
    event.preventDefault();
    reset();
  });
  canvas.addEventListener("webglcontextrestored", reset);

  function resourcesFor(canvas: HTMLCanvasElement): WebglResources | undefined {
    if (!shadersAvailable) {
      return undefined;
    }
    if (maybeResources !== undefined && !maybeResources.gl.isContextLost()) {
      return maybeResources;
    }

    const gl = canvas.getContext("webgl2", {
      alpha: false,
      antialias: false,
      depth: false,
      stencil: false,
      premultipliedAlpha: false,
    });
    if (gl === null) {
      return undefined;
    }
    if (gl.isContextLost()) return undefined;
    const created = createResources(gl);
    if (created === undefined) {
      shadersAvailable = false;
      return undefined;
    }
    maybeResources = created;
    fieldWidth = 0;
    fieldHeight = 0;
    return created;
  }

  return {
    draw(canvas, frame, camera, devicePixelRatio, densityShading) {
      const resources = resourcesFor(canvas);
      if (resources === undefined) {
        return false;
      }

      let pixelRatio = 1;
      try {
        pixelRatio = cappedDevicePixelRatio(devicePixelRatio);
      } catch {
        return false;
      }
      const width = Math.max(1, Math.round(camera.viewport.width * pixelRatio));
      const height = Math.max(
        1,
        Math.round(camera.viewport.height * pixelRatio),
      );
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
        fieldWidth = 0;
      }
      if (fieldWidth !== width || fieldHeight !== height) {
        if (!resizeField(resources, width, height)) {
          shadersAvailable = false;
          maybeResources = undefined;
          return false;
        }
        fieldWidth = width;
        fieldHeight = height;
      }

      const countNeeded = frame.particleCount;
      if (positions.length < countNeeded * 2) {
        positions = new Float32Array(countNeeded * 2);
        radii = new Float32Array(countNeeded);
        colors = new Float32Array(countNeeded * 4);
      }

      const gl = resources.gl;
      const count = uploadParticles(
        resources,
        frame,
        camera,
        positions,
        radii,
        colors,
      );
      gl.bindFramebuffer(gl.FRAMEBUFFER, resources.fieldFramebuffer);
      gl.viewport(0, 0, width, height);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      if (count > 0) {
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.ONE, gl.ONE);
        gl.useProgram(resources.pointProgram);
        gl.bindVertexArray(resources.pointVao);
        resources.uniforms.point(
          camera.viewport.width,
          camera.viewport.height,
          pixelRatio,
          resources.maxPointSize,
        );
        gl.drawArrays(gl.POINTS, 0, count);
        gl.disable(gl.BLEND);
      }

      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, width, height);
      gl.useProgram(resources.compositeProgram);
      gl.bindVertexArray(resources.compositeVao);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, resources.fieldTexture);
      resources.uniforms.composite(densityShading);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      gl.bindVertexArray(null);
      return true;
    },
  };
}

const surfaces = new WeakMap<HTMLCanvasElement, WebglSurface>();

/**
 * Draws the shaded blob into the particle canvas.
 *
 * Packed overlap darkens the recovered particle color. Returns false when
 * WebGL2 or a float framebuffer is unavailable so the caller can fall back
 * to the canvas metaball.
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

  let maybeSurface = surfaces.get(canvas);
  if (maybeSurface === undefined) {
    maybeSurface = createWebglSurface(canvas);
    surfaces.set(canvas, maybeSurface);
  }
  return maybeSurface.draw(
    canvas,
    frame,
    camera,
    devicePixelRatio,
    densityShading,
  );
}
