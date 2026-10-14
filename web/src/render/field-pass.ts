/**
 * One WebGL2 context per particle canvas, shared by shaded blob and water
 * (D-04). The surface owns the context, the particle buffers both modes splat
 * from, and a slot per render pass with its own availability (D-13, D-14).
 *
 * Context loss and restore reset everything the surface holds. The reset
 * listeners are registered when the surface is created, so they run before
 * any later listener that repaints.
 */
import type { RenderFrame } from "../physics/frame";
import type { Camera } from "./camera";
import { cappedDevicePixelRatio } from "./canvas";
import { requireBuffer } from "./gl-program";
import { createParticleMetadataCache } from "./particle-metadata";
import { fillProjectedParticleArrays } from "./projected-particle";

/** Context attributes for every particle canvas. */
export const PARTICLE_CONTEXT_ATTRIBUTES: Readonly<WebGLContextAttributes> = {
  alpha: false,
  antialias: false,
  depth: false,
  stencil: false,
  premultipliedAlpha: false,
};

/**
 * Reports whether RGBA16F render targets may be color-renderable (D-13).
 *
 * Calling `getExtension` enables the extension; either one makes RGBA16F
 * renderable. Framebuffer completeness is still checked at every resize.
 */
export function supportsHalfFloatTargets(gl: WebGL2RenderingContext): boolean {
  return (
    gl.getExtension("EXT_color_buffer_float") !== null ||
    gl.getExtension("EXT_color_buffer_half_float") !== null
  );
}

/** A framebuffer with one or two RGBA16F color attachments. */
export type FieldTarget = {
  readonly framebuffer: WebGLFramebuffer;
  readonly textures: readonly WebGLTexture[];
  width: number;
  height: number;
};

/**
 * Creates an unsized field target. Call `resizeFieldTarget` before drawing;
 * a zero size never matches a real canvas, so the first draw allocates.
 */
export function createFieldTarget(
  gl: WebGL2RenderingContext,
  attachmentCount: 1 | 2,
): FieldTarget | undefined {
  const framebuffer = gl.createFramebuffer();
  if (framebuffer === null) {
    return undefined;
  }
  const textures: WebGLTexture[] = [];
  for (let index = 0; index < attachmentCount; index++) {
    const texture = gl.createTexture();
    if (texture === null) {
      return undefined;
    }
    textures.push(texture);
  }
  return { framebuffer, textures, width: 0, height: 0 };
}

/**
 * Allocates RGBA16F storage for every attachment and reports completeness.
 * The recorded size changes only when the framebuffer is complete.
 */
export function resizeFieldTarget(
  gl: WebGL2RenderingContext,
  target: FieldTarget,
  width: number,
  height: number,
): boolean {
  const attachments: number[] = [];
  gl.bindFramebuffer(gl.FRAMEBUFFER, target.framebuffer);
  target.textures.forEach((texture, index) => {
    gl.bindTexture(gl.TEXTURE_2D, texture);
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
    const attachment =
      index === 0 ? gl.COLOR_ATTACHMENT0 : gl.COLOR_ATTACHMENT1;
    gl.framebufferTexture2D(
      gl.FRAMEBUFFER,
      attachment,
      gl.TEXTURE_2D,
      texture,
      0,
    );
    attachments.push(attachment);
  });
  if (attachments.length === 2) {
    gl.drawBuffers(attachments);
  }
  if (gl.checkFramebufferStatus(gl.FRAMEBUFFER) !== gl.FRAMEBUFFER_COMPLETE) {
    return false;
  }
  target.width = width;
  target.height = height;
  return true;
}

/** Particle buffers both modes bind; refilled once per drawn frame. */
export type SharedParticleBuffers = {
  readonly gl: WebGL2RenderingContext;
  readonly position: WebGLBuffer;
  readonly radius: WebGLBuffer;
  readonly color: WebGLBuffer;
  readonly maxPointSize: number;
};

/** Backing-store size of the particle canvas and its capped pixel ratio. */
export type FieldSize = {
  readonly width: number;
  readonly height: number;
  readonly pixelRatio: number;
};

/** The shared GL state of one particle canvas. */
export interface ParticleGlSurface {
  /** The live context, or undefined when unavailable, lost or unsupported. */
  context(): WebGL2RenderingContext | undefined;
  /** Shared particle buffers, created on first use after each reset. */
  buffers(): SharedParticleBuffers | undefined;
  /** Sizes the canvas backing store for the camera viewport (D-09). */
  sizeFor(camera: Camera, devicePixelRatio: number): FieldSize | undefined;
  /** Uploads projected particles; radii and colors only when they changed. */
  upload(frame: RenderFrame, camera: Camera): number;
  /**
   * Returns the pass stored under `key`, creating it once. An undefined
   * result marks only that key unavailable until the next reset.
   */
  pass<T>(
    key: symbol,
    create: (buffers: SharedParticleBuffers) => T | undefined,
  ): T | undefined;
  /** Marks one pass unavailable until the next reset. */
  disablePass(key: symbol): void;
}

type BufferState = {
  readonly shared: SharedParticleBuffers;
  readonly metadata: ReturnType<typeof createParticleMetadataCache>;
};

function createBufferState(
  gl: WebGL2RenderingContext,
): BufferState | undefined {
  const position = requireBuffer(gl);
  const radius = requireBuffer(gl);
  const color = requireBuffer(gl);
  if (position === undefined || radius === undefined || color === undefined) {
    return undefined;
  }
  const range = gl.getParameter(
    gl.ALIASED_POINT_SIZE_RANGE,
  ) as Float32Array | null;
  return {
    shared: {
      gl,
      position,
      radius,
      color,
      maxPointSize: range?.[1] ?? 64,
    },
    // Owned by the buffers so a restore re-uploads every radius and color.
    metadata: createParticleMetadataCache(),
  };
}

function createParticleSurface(canvas: HTMLCanvasElement): ParticleGlSurface {
  let maybeGl: WebGL2RenderingContext | undefined;
  let unsupported = false;
  let maybeBufferState: BufferState | undefined;
  const passes = new Map<symbol, unknown>();
  const unavailablePasses = new Set<symbol>();
  let positions = new Float32Array(0);
  let radii = new Float32Array(0);
  let colors = new Float32Array(0);

  const reset = () => {
    maybeGl = undefined;
    unsupported = false;
    maybeBufferState = undefined;
    passes.clear();
    unavailablePasses.clear();
  };
  canvas.addEventListener("webglcontextlost", (event) => {
    event.preventDefault();
    reset();
  });
  canvas.addEventListener("webglcontextrestored", reset);

  const context = (): WebGL2RenderingContext | undefined => {
    if (unsupported) {
      return undefined;
    }
    if (maybeGl !== undefined) {
      return maybeGl.isContextLost() ? undefined : maybeGl;
    }
    const gl = canvas.getContext("webgl2", PARTICLE_CONTEXT_ATTRIBUTES);
    if (gl === null || gl.isContextLost()) {
      return undefined;
    }
    if (!supportsHalfFloatTargets(gl)) {
      unsupported = true;
      return undefined;
    }
    maybeGl = gl;
    return gl;
  };

  const bufferState = (): BufferState | undefined => {
    if (maybeBufferState !== undefined) {
      return maybeBufferState;
    }
    const gl = context();
    if (gl === undefined) {
      return undefined;
    }
    maybeBufferState = createBufferState(gl);
    return maybeBufferState;
  };

  return {
    context,
    buffers: () => bufferState()?.shared,
    sizeFor(camera, devicePixelRatio) {
      let pixelRatio = 1;
      try {
        pixelRatio = cappedDevicePixelRatio(devicePixelRatio);
      } catch {
        return undefined;
      }
      const width = Math.max(1, Math.round(camera.viewport.width * pixelRatio));
      const height = Math.max(
        1,
        Math.round(camera.viewport.height * pixelRatio),
      );
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      return { width, height, pixelRatio };
    },
    upload(frame, camera) {
      const state = maybeBufferState;
      if (state === undefined) {
        return 0;
      }
      const countNeeded = frame.particleCount;
      if (positions.length < countNeeded * 2) {
        positions = new Float32Array(countNeeded * 2);
        radii = new Float32Array(countNeeded);
        colors = new Float32Array(countNeeded * 4);
      }
      const { gl, position, radius, color } = state.shared;
      const { radiiChanged, colorsChanged } = state.metadata.changes(
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
      gl.bindBuffer(gl.ARRAY_BUFFER, position);
      gl.bufferData(
        gl.ARRAY_BUFFER,
        positions.subarray(0, count * 2),
        gl.DYNAMIC_DRAW,
      );
      if (radiiChanged) {
        gl.bindBuffer(gl.ARRAY_BUFFER, radius);
        gl.bufferData(
          gl.ARRAY_BUFFER,
          radii.subarray(0, count),
          gl.DYNAMIC_DRAW,
        );
      }
      if (colorsChanged) {
        gl.bindBuffer(gl.ARRAY_BUFFER, color);
        gl.bufferData(
          gl.ARRAY_BUFFER,
          colors.subarray(0, count * 4),
          gl.DYNAMIC_DRAW,
        );
      }
      return count;
    },
    pass<T>(
      key: symbol,
      create: (buffers: SharedParticleBuffers) => T | undefined,
    ): T | undefined {
      if (unavailablePasses.has(key)) {
        return undefined;
      }
      if (passes.has(key)) {
        // Each key is private to the module that creates its pass, so the
        // stored value always has that module's pass type.
        return passes.get(key) as T;
      }
      const state = bufferState();
      if (state === undefined) {
        return undefined;
      }
      const maybePass = create(state.shared);
      if (maybePass === undefined) {
        unavailablePasses.add(key);
        return undefined;
      }
      passes.set(key, maybePass);
      return maybePass;
    },
    disablePass(key) {
      passes.delete(key);
      unavailablePasses.add(key);
    },
  };
}

const surfaces = new WeakMap<HTMLCanvasElement, ParticleGlSurface>();

/**
 * Returns the shared GL surface of a particle canvas, creating it (and its
 * loss/restore reset listeners) on first use.
 */
export function particleSurfaceFor(
  canvas: HTMLCanvasElement,
): ParticleGlSurface {
  const maybeSurface = surfaces.get(canvas);
  if (maybeSurface !== undefined) {
    return maybeSurface;
  }
  const surface = createParticleSurface(canvas);
  surfaces.set(canvas, surface);
  return surface;
}

/**
 * Repaints after the surface reset, so a paused view falls back on loss and
 * rebuilds on restore (D-14). Never draws with dead GL objects: the surface,
 * and so its reset listeners, exist before these listeners are added, and DOM
 * listeners run in registration order. Returns a function that detaches.
 */
export function watchParticleSurface(
  canvas: HTMLCanvasElement,
  repaint: () => void,
): () => void {
  particleSurfaceFor(canvas);
  const listener = () => repaint();
  canvas.addEventListener("webglcontextlost", listener);
  canvas.addEventListener("webglcontextrestored", listener);
  return () => {
    canvas.removeEventListener("webglcontextlost", listener);
    canvas.removeEventListener("webglcontextrestored", listener);
  };
}
