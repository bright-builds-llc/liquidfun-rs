/**
 * Decides whether this device can run the Water render mode (D-05).
 *
 * The decision is a pure function of observed facts. The probe gathers those
 * facts on a detached canvas, never the particle canvas (context attributes
 * are fixed by the first `getContext`), releases its context, and caches the
 * verdict for the page.
 */
import {
  createFieldTarget,
  PARTICLE_CONTEXT_ATTRIBUTES,
  resizeFieldTarget,
  supportsHalfFloatTargets,
} from "./field-pass";
import { linkWaterPrograms } from "./water-programs";

/** Everything Water needs from the device. */
export type WaterSupportFacts = {
  readonly webgl2: boolean;
  /** EXT_color_buffer_float or EXT_color_buffer_half_float (D-13). */
  readonly floatTargets: boolean;
  readonly maxDrawBuffers: number;
  /** Fragment HIGH_FLOAT precision bits; 0 when highp is missing (D-08). */
  readonly highpFragment: number;
  /** A 2-attachment RGBA16F framebuffer is complete. */
  readonly mrtComplete: boolean;
  readonly programsLinked: boolean;
};

const MIN_DRAW_BUFFERS = 2;
const PROBE_TARGET_SIZE = 4;

/** Returns true only when every Water requirement holds. */
export function waterSupportFrom(facts: WaterSupportFacts): boolean {
  return (
    facts.webgl2 &&
    facts.floatTargets &&
    facts.maxDrawBuffers >= MIN_DRAW_BUFFERS &&
    facts.highpFragment > 0 &&
    facts.mrtComplete &&
    facts.programsLinked
  );
}

function maxDrawBuffers(gl: WebGL2RenderingContext): number {
  const value: unknown = gl.getParameter(gl.MAX_DRAW_BUFFERS);
  return typeof value === "number" ? value : 0;
}

function highpFragment(gl: WebGL2RenderingContext): number {
  return (
    gl.getShaderPrecisionFormat(gl.FRAGMENT_SHADER, gl.HIGH_FLOAT)
      ?.precision ?? 0
  );
}

/**
 * Checks the draw-buffer and precision limits on a live context, so forced
 * limits affect drawing the same way they affect the probe.
 */
export function liveWaterLimitsOk(gl: WebGL2RenderingContext): boolean {
  return maxDrawBuffers(gl) >= MIN_DRAW_BUFFERS && highpFragment(gl) > 0;
}

const NO_SUPPORT: WaterSupportFacts = {
  webgl2: false,
  floatTargets: false,
  maxDrawBuffers: 0,
  highpFragment: 0,
  mrtComplete: false,
  programsLinked: false,
};

function mrtComplete(gl: WebGL2RenderingContext): boolean {
  const maybeTarget = createFieldTarget(gl, 2);
  return (
    maybeTarget !== undefined &&
    resizeFieldTarget(gl, maybeTarget, PROBE_TARGET_SIZE, PROBE_TARGET_SIZE)
  );
}

/** Gathers facts in order and stops at the first missing one. */
function gatherFacts(gl: WebGL2RenderingContext): WaterSupportFacts {
  const facts = { ...NO_SUPPORT, webgl2: true };
  facts.floatTargets = supportsHalfFloatTargets(gl);
  if (!facts.floatTargets) return facts;
  facts.maxDrawBuffers = maxDrawBuffers(gl);
  if (facts.maxDrawBuffers < MIN_DRAW_BUFFERS) return facts;
  facts.highpFragment = highpFragment(gl);
  if (facts.highpFragment <= 0) return facts;
  facts.mrtComplete = mrtComplete(gl);
  if (!facts.mrtComplete) return facts;
  facts.programsLinked = linkWaterPrograms(gl) !== undefined;
  return facts;
}

/**
 * Probes Water support on a fresh detached canvas and releases its context.
 * Any thrown error counts as unsupported.
 */
export function probeWaterSupport(
  createCanvas: () => HTMLCanvasElement | undefined,
): boolean {
  let maybeGl: WebGL2RenderingContext | undefined;
  try {
    maybeGl =
      createCanvas()?.getContext("webgl2", PARTICLE_CONTEXT_ATTRIBUTES) ??
      undefined;
    if (maybeGl === undefined || maybeGl.isContextLost()) {
      return false;
    }
    return waterSupportFrom(gatherFacts(maybeGl));
  } catch {
    return false;
  } finally {
    maybeGl?.getExtension("WEBGL_lose_context")?.loseContext();
  }
}

function defaultProbeCanvas(): HTMLCanvasElement | undefined {
  return typeof document === "undefined"
    ? undefined
    : document.createElement("canvas");
}

let maybeCachedSupport: boolean | undefined;

/** Probes once per page and returns the cached verdict afterwards. */
export function cachedWaterSupport(
  createCanvas: () => HTMLCanvasElement | undefined = defaultProbeCanvas,
): boolean {
  maybeCachedSupport ??= probeWaterSupport(createCanvas);
  return maybeCachedSupport;
}

/** Clears the per-page cache so each test probes afresh. */
export function resetWaterSupportCacheForTests(): void {
  maybeCachedSupport = undefined;
}
