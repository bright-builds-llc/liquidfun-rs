import type { RenderFrame } from "../../src/physics/frame";

export const BENCHMARK_VERSION = "tesla-valve-v1";
export const SCHEMA_VERSION = 1;

export type BenchmarkCase = {
  readonly id: "forward" | "reverse";
  readonly direction: "1" | "-1";
  readonly warmupSteps: number;
};

export type BenchmarkWorkload = {
  readonly purpose: "canonical" | "smoke";
  readonly sceneId: "tesla-valve";
  readonly rate: 1440;
  readonly timestepSeconds: number;
  readonly particleIterations: 4;
  readonly rigidVelocityIterations: 8;
  readonly rigidPositionIterations: 3;
  readonly particleRadius: 0.005;
  readonly gravity: { readonly x: 0; readonly y: -10 };
  readonly inletSpeed: 2; readonly particleDamping: 0.2; readonly wallFriction: 0.05;
  readonly maximumParticleCount: 16384; readonly channelClearWidth: 0.06;
  readonly physicalSettingsProvenance: "declared from frozen production source; source manifest and semantic checkpoints retained";
  readonly viewport: { readonly width: number; readonly height: number };
  readonly cameraBounds: { readonly minX: number; readonly minY: number; readonly maxX: number; readonly maxY: number };
  readonly cameraView: { readonly zoom: 1; readonly panX: 0; readonly panY: 0 };
  readonly devicePixelRatio: number;
  readonly renderMode: "shaded-blob";
  readonly densityShading: true;
  readonly maxRenderedParticles: 16384;
  readonly wireframeStrokeWidth: 0.3;
  readonly sampleCount: number;
  readonly liveDurationMs: number;
  readonly repetitions: number;
  readonly cases: readonly BenchmarkCase[];
};

export const DEFAULT_WORKLOAD: BenchmarkWorkload = {
  purpose: "canonical", sceneId: "tesla-valve", rate: 1440,
  timestepSeconds: 1 / 60, particleIterations: 4, particleRadius: 0.005,
  rigidVelocityIterations: 8, rigidPositionIterations: 3,
  gravity: { x: 0, y: -10 }, inletSpeed: 2, particleDamping: 0.2, wallFriction: 0.05,
  maximumParticleCount: 16384, channelClearWidth: 0.06,
  physicalSettingsProvenance: "declared from frozen production source; source manifest and semantic checkpoints retained",
  viewport: { width: 1280, height: 960 }, devicePixelRatio: 1,
  cameraBounds: { minX: -0.7, minY: -0.12, maxX: 0.58, maxY: 4.32 }, cameraView: { zoom: 1, panX: 0, panY: 0 },
  renderMode: "shaded-blob", densityShading: true,
  maxRenderedParticles: 16384, wireframeStrokeWidth: 0.3,
  sampleCount: 40, liveDurationMs: 2000, repetitions: 3,
  cases: [
    { id: "forward", direction: "1", warmupSteps: 360 },
    { id: "reverse", direction: "-1", warmupSteps: 384 },
  ],
};

export const SMOKE_WORKLOAD: BenchmarkWorkload = {
  ...DEFAULT_WORKLOAD, purpose: "smoke", sampleCount: 4, liveDurationMs: 200, repetitions: 1,
  cases: DEFAULT_WORKLOAD.cases.map((scenario) => ({ ...scenario, warmupSteps: 8 })),
};

/** Sorts named semantic fields; it never serializes object or WASM memory. */
export function canonicalJson(value: unknown): string {
  if (typeof value === "number" && !Number.isFinite(value)) throw new Error("JSON numbers must be finite");
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
  if (value !== null && typeof value === "object") {
    const object = value as Record<string, unknown>;
    return `{${Object.keys(object).sort().map((key) =>
      `${JSON.stringify(key)}:${canonicalJson(object[key])}`).join(",")}}`;
  }
  const encoded = JSON.stringify(value);
  if (encoded === undefined) throw new Error("Cannot encode an unavailable JSON field");
  return encoded;
}

/** Producer hashes intentionally differ between optimization stages. */
export function comparisonMismatchReasons(
  before: { readonly workload: unknown; readonly environment: unknown },
  after: { readonly workload: unknown; readonly environment: unknown },
): string[] {
  const reasons: string[] = [];
  if (canonicalJson(before.workload) !== canonicalJson(after.workload)) reasons.push("workload differs");
  if (canonicalJson(before.environment) !== canonicalJson(after.environment)) reasons.push("runtime or hardware differs");
  return reasons;
}

export type Distribution = {
  readonly count: number; readonly mean: number; readonly median: number;
  readonly p95: number; readonly min: number; readonly max: number;
};

/** Adjacent trajectory samples are correlated; this is descriptive, not confidence. */
export function summarize(values: readonly number[]): Distribution {
  if (!values.length || values.some((value) => !Number.isFinite(value) || value < 0)) {
    throw new Error("Measurements must be nonempty finite nonnegative values");
  }
  const sorted = [...values].sort((first, second) => first - second);
  const percentile = (fraction: number) => sorted[Math.max(0, Math.ceil(sorted.length * fraction) - 1)]!;
  const middle = Math.floor(sorted.length / 2);
  const median = sorted.length % 2 === 0
    ? (sorted[middle - 1]! + sorted[middle]!) * 0.5 : sorted[middle]!;
  return {
    count: values.length, mean: values.reduce((total, value) => total + value, 0) / values.length,
    median, p95: percentile(0.95), min: sorted[0]!, max: sorted[sorted.length - 1]!,
  };
}

function semanticFloats(values: Float32Array): readonly (number | "negative-zero")[] {
  return Array.from(values, (value) => Object.is(value, -0) ? "negative-zero" : value);
}

export function semanticState(frame: RenderFrame): Record<string, unknown> {
  return {
    stepIndex: frame.stepIndex, particleCount: frame.particleCount,
    rigidShapeCount: frame.rigidShapeCount, maxSpeed: frame.maxSpeed,
    stuckCandidateCount: frame.stuckCandidateCount, bodyContactCount: frame.bodyContactCount,
    particlePositions: semanticFloats(frame.particlePositions),
    particleColors: Array.from(frame.particleColors), particleRadii: semanticFloats(frame.particleRadii),
    rigidSegments: semanticFloats(frame.rigidSegments), rigidCircles: semanticFloats(frame.rigidCircles),
    circleLabels: frame.circleLabels,
  };
}
