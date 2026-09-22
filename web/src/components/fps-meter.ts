/** Playback rates for the debug overlay. */

export const FPS_BASELINE = 60;
export const FPS_HISTORY_MS = 5_000;
export const FPS_COUNTER_WINDOW_MS = 1_000;
export const FPS_GRAPH_BUCKETS = 40;
/** Empty slots between the newest bar and the bars fading out. */
export const FPS_GRAPH_GAP_BARS = 4;
/** Bars beyond the gap that fade out before they are replaced. */
export const FPS_GRAPH_FADE_BARS = 14;

const FPS_RETAIN_MS = FPS_HISTORY_MS + FPS_COUNTER_WINDOW_MS;
const GOOD_FPS = 55;
const FAIR_FPS = 30;

export type FpsTick = {
  readonly timeMs: number;
  readonly simSteps: number;
};

export type FpsRates = {
  readonly renderFps: number;
  readonly simFps: number;
};

export type FpsTone = "good" | "fair" | "poor";

export type FpsBar = {
  readonly fps: number;
  readonly opacity: number;
};

export type FpsHistory = {
  readonly render: readonly FpsBar[];
  readonly sim: readonly FpsBar[];
  readonly cursor: number;
};

/** Keeps presented frames from the last 6 seconds so a 5-second graph can use a 1-second rate. */
export function appendFpsTick(
  ticks: readonly FpsTick[],
  tick: FpsTick,
): readonly FpsTick[] {
  if (!Number.isFinite(tick.timeMs) || !Number.isFinite(tick.simSteps) || tick.simSteps < 0) {
    return ticks;
  }

  const cutoff = tick.timeMs - FPS_RETAIN_MS;
  const kept = ticks.filter((item) => item.timeMs >= cutoff);
  kept.push({ timeMs: tick.timeMs, simSteps: tick.simSteps });
  return kept;
}

/** Paints and simulation steps per second over the trailing window. */
export function fpsOverWindow(
  ticks: readonly FpsTick[],
  nowMs: number,
  windowMs: number,
): FpsRates {
  if (!Number.isFinite(nowMs) || !Number.isFinite(windowMs) || windowMs <= 0) {
    return { renderFps: 0, simFps: 0 };
  }

  const start = nowMs - windowMs;
  let paints = 0;
  let steps = 0;
  for (const tick of ticks) {
    if (tick.timeMs < start || tick.timeMs > nowMs) {
      continue;
    }
    paints += 1;
    steps += tick.simSteps;
  }

  const seconds = windowMs / 1000;
  return {
    renderFps: paints / seconds,
    simFps: steps / seconds,
  };
}

const FADE_FLOOR = 0.18;

/** One fixed slot per bucket. Only the current slot moves; older slots keep the rate they were given. */
export function fpsHistory(ticks: readonly FpsTick[], nowMs: number): FpsHistory {
  const bucketMs = FPS_HISTORY_MS / FPS_GRAPH_BUCKETS;
  const currentBucket = Math.floor(nowMs / bucketMs);
  const cursor = positiveMod(currentBucket, FPS_GRAPH_BUCKETS);
  const render: FpsBar[] = [];
  const sim: FpsBar[] = [];
  for (let index = 0; index < FPS_GRAPH_BUCKETS; index += 1) {
    const bucketsAgo = positiveMod(cursor - index, FPS_GRAPH_BUCKETS);
    const sampleBucket = currentBucket - bucketsAgo;
    const bucketEnd = (sampleBucket + 1) * bucketMs;
    const rates = fpsOverWindow(ticks, Math.min(nowMs, bucketEnd), FPS_COUNTER_WINDOW_MS);
    const opacity = overwriteOpacity(index, cursor);
    render.push({ fps: rates.renderFps, opacity });
    sim.push({ fps: rates.simFps, opacity });
  }
  return { render, sim, cursor };
}

function overwriteOpacity(index: number, cursor: number): number {
  const slotsAhead = positiveMod(index - cursor - 1, FPS_GRAPH_BUCKETS);
  if (slotsAhead < FPS_GRAPH_GAP_BARS) {
    return 0;
  }

  const fadeIndex = slotsAhead - FPS_GRAPH_GAP_BARS;
  if (fadeIndex >= FPS_GRAPH_FADE_BARS) {
    return 1;
  }
  return FADE_FLOOR + (1 - FADE_FLOOR) * (fadeIndex / FPS_GRAPH_FADE_BARS);
}

function positiveMod(value: number, divisor: number): number {
  return ((value % divisor) + divisor) % divisor;
}

export function graphScaleMax(samples: readonly number[]): number {
  let peak = FPS_BASELINE;
  for (const sample of samples) {
    if (Number.isFinite(sample) && sample > peak) {
      peak = sample;
    }
  }
  return peak;
}

/** Rates at or above the 60 fps baseline are healthy, including a simulation running faster than realtime. */
export function fpsTone(fps: number): FpsTone {
  if (!Number.isFinite(fps) || fps < FAIR_FPS) {
    return "poor";
  }
  if (fps < GOOD_FPS) {
    return "fair";
  }
  return "good";
}

export function formatFps(fps: number): string {
  if (!Number.isFinite(fps) || fps <= 0) {
    return "0";
  }
  return String(Math.round(fps));
}
