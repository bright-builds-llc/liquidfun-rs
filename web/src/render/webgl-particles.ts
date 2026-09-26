import type { RenderFrame } from "../physics/frame";
import { cappedDevicePixelRatio } from "./canvas";
import type { Camera } from "./camera";
import { eachProjectedParticle } from "./projected-particle";

/** Sprite diameter scale that places the shaded iso-surface near the particle radius. */
const INFLUENCE_DIAMETER = 1.85 * 2;

const CANVAS_BACKGROUND: readonly [number, number, number] = [
  7 / 255,
  16 / 255,
  24 / 255,
];

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
in vec2 vUv;
out vec4 fragColor;
void main() {
  vec4 sum = texture(uField, vUv);
  float edge = smoothstep(0.42, 0.62, sum.a);
  vec3 color = sum.rgb / max(sum.a, 0.0001);
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
};

type WebglSurface = {
  draw: (
    canvas: HTMLCanvasElement,
    frame: RenderFrame,
    camera: Camera,
    devicePixelRatio: number,
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

function createResources(gl: WebGL2RenderingContext): WebglResources | undefined {
  if (gl.getExtension("EXT_color_buffer_float") === null) {
    return undefined;
  }
  const pointProgram = linkProgram(gl, POINT_VERTEX, POINT_FRAGMENT);
  const compositeProgram = linkProgram(gl, COMPOSITE_VERTEX, COMPOSITE_FRAGMENT);
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

  const range = gl.getParameter(gl.ALIASED_POINT_SIZE_RANGE) as Float32Array | null;
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
  let count = 0;
  eachProjectedParticle(
    frame,
    camera,
    Number.POSITIVE_INFINITY,
    (x, y, radius, red, green, blue, alpha) => {
      const positionIndex = count * 2;
      const colorIndex = count * 4;
      positions[positionIndex] = x;
      positions[positionIndex + 1] = y;
      radii[count] = radius;
      colors[colorIndex] = red / 255;
      colors[colorIndex + 1] = green / 255;
      colors[colorIndex + 2] = blue / 255;
      colors[colorIndex + 3] = alpha;
      count += 1;
    },
  );

  gl.bindBuffer(gl.ARRAY_BUFFER, resources.positionBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, positions.subarray(0, count * 2), gl.DYNAMIC_DRAW);
  gl.bindBuffer(gl.ARRAY_BUFFER, resources.radiusBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, radii.subarray(0, count), gl.DYNAMIC_DRAW);
  gl.bindBuffer(gl.ARRAY_BUFFER, resources.colorBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, colors.subarray(0, count * 4), gl.DYNAMIC_DRAW);
  return count;
}

function createWebglSurface(): WebglSurface {
  let maybeCanvas: HTMLCanvasElement | undefined;
  let maybeResources: WebglResources | undefined;
  let fieldWidth = 0;
  let fieldHeight = 0;
  let positions = new Float32Array(0);
  let radii = new Float32Array(0);
  let colors = new Float32Array(0);
  let shadersAvailable = true;

  function resourcesFor(canvas: HTMLCanvasElement): WebglResources | undefined {
    if (!shadersAvailable) {
      return undefined;
    }
    if (
      maybeResources !== undefined &&
      maybeCanvas === canvas &&
      !maybeResources.gl.isContextLost()
    ) {
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
    const created = createResources(gl);
    if (created === undefined) {
      shadersAvailable = false;
      return undefined;
    }
    maybeCanvas = canvas;
    maybeResources = created;
    fieldWidth = 0;
    fieldHeight = 0;
    return created;
  }

  return {
    draw(canvas, frame, camera, devicePixelRatio) {
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
      const height = Math.max(1, Math.round(camera.viewport.height * pixelRatio));
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
      const count = uploadParticles(resources, frame, camera, positions, radii, colors);
      gl.bindFramebuffer(gl.FRAMEBUFFER, resources.fieldFramebuffer);
      gl.viewport(0, 0, width, height);
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
      if (count > 0) {
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.ONE, gl.ONE);
        gl.useProgram(resources.pointProgram);
        gl.bindVertexArray(resources.pointVao);
        const resolution = gl.getUniformLocation(resources.pointProgram, "uResolution");
        const pointScale = gl.getUniformLocation(resources.pointProgram, "uPointScale");
        const maxPoint = gl.getUniformLocation(resources.pointProgram, "uMaxPointSize");
        gl.uniform2f(resolution, camera.viewport.width, camera.viewport.height);
        gl.uniform1f(pointScale, INFLUENCE_DIAMETER * pixelRatio);
        gl.uniform1f(maxPoint, resources.maxPointSize);
        gl.drawArrays(gl.POINTS, 0, count);
        gl.disable(gl.BLEND);
      }

      gl.bindFramebuffer(gl.FRAMEBUFFER, null);
      gl.viewport(0, 0, width, height);
      gl.useProgram(resources.compositeProgram);
      gl.bindVertexArray(resources.compositeVao);
      gl.activeTexture(gl.TEXTURE0);
      gl.bindTexture(gl.TEXTURE_2D, resources.fieldTexture);
      const field = gl.getUniformLocation(resources.compositeProgram, "uField");
      const background = gl.getUniformLocation(resources.compositeProgram, "uBackground");
      gl.uniform1i(field, 0);
      gl.uniform3f(
        background,
        CANVAS_BACKGROUND[0],
        CANVAS_BACKGROUND[1],
        CANVAS_BACKGROUND[2],
      );
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
      gl.bindVertexArray(null);
      return true;
    },
  };
}

let maybeSurface: WebglSurface | undefined;

/**
 * Draws the shaded blob into the particle canvas.
 *
 * Returns false when WebGL2 or a float framebuffer is unavailable so the
 * caller can fall back to the canvas metaball.
 */
export function drawShadedBlob(
  canvas: HTMLCanvasElement | undefined,
  frame: RenderFrame,
  camera: Camera,
  devicePixelRatio: number,
): boolean {
  if (canvas === undefined) {
    return false;
  }

  maybeSurface ??= createWebglSurface();
  return maybeSurface.draw(canvas, frame, camera, devicePixelRatio);
}
