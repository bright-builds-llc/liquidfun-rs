import type { BenchmarkReport } from "./contracts";
import { BENCHMARK_VERSION, canonicalJson, DEFAULT_WORKLOAD, SCHEMA_VERSION, SMOKE_WORKLOAD, summarize } from "./model";
import { caseReplicateMeans, semanticConsistency } from "./compare";
import type { CaseReplicate } from "./contracts";

function object(value: unknown, label: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) throw new Error(`${label} must be an object`);
  return value as Record<string, unknown>;
}

function string(value: unknown, label: string): string {
  if (typeof value !== "string" || !value.length) throw new Error(`${label} must be a nonempty string`);
  return value;
}

function finite(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) throw new Error(`${label} must be finite and nonnegative`);
  return value;
}

function integer(value: unknown, label: string, maximum = Number.MAX_SAFE_INTEGER): number {
  const number = finite(value, label);
  if (!Number.isSafeInteger(number) || number > maximum) throw new Error(`${label} exceeds its integer bound`);
  return number;
}

function array(value: unknown, label: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
  return value;
}

function hash(value: unknown, label: string): void {
  if (!/^[a-f0-9]{64}$/.test(string(value, label))) throw new Error(`${label} must be SHA-256`);
}

function checkpoint(value: unknown): Record<string, unknown> {
  const point = object(value, "checkpoint");
  integer(point.stepIndex, "stepIndex"); integer(point.particleCount, "particleCount", 16384);
  integer(point.bodyContactCount, "bodyContactCount"); integer(point.stuckCandidateCount, "stuckCandidateCount"); finite(point.maxSpeed, "maxSpeed");
  if (point.rigidShapeCount !== 468) throw new Error("Tesla geometry must retain 468 segments");
  for (const key of ["sha256", "positionsSha256", "colorsSha256", "radiiSha256", "geometrySha256"]) hash(point[key], key);
  return point;
}

function renderer(value: unknown): void {
  const gl = object(value, "renderer");
  if (typeof gl.contextLost !== "boolean" && gl.contextLost !== "unavailable") throw new Error("Renderer must record actual context-loss availability");
  if (gl.backend !== "webgl2" && gl.backend !== "canvas-metaball") throw new Error("Unknown render backend");
  for (const key of ["vendor", "renderer", "version", "unmaskedVendor", "unmaskedRenderer"]) string(gl[key], key);
  if (!["software", "not-identified-as-software", "unavailable"].includes(string(gl.softwareClassification, "software classification"))) throw new Error("Unknown software classification");
}

function validateProducer(value: unknown): void {
  const producer = object(value, "producer");
  if (!/^[a-f0-9]{40}$/.test(string(producer.revision, "revision"))) throw new Error("Producer revision must be a complete git revision");
  for (const key of ["sourceDiffSha256", "sourceFilesSha256", "wasmSha256", "generatedBindingsSha256", "benchmarkBundleSha256"]) hash(producer[key], key);
  string(producer.rustToolchain, "rustToolchain");
  for (const key of ["wasmPackVersion", "rustflags", "cargoEncodedRustflags"]) string(producer[key], key);
  if (producer.buildProfile !== "release" || producer.wasmTarget !== "wasm32-unknown-unknown") throw new Error("Unknown production WASM build profile");
  if (producer.executionBackend !== "direct" && producer.executionBackend !== "worker") throw new Error("Unknown execution backend");
  const paths = array(producer.sourcePaths, "sourcePaths");
  if (!paths.length) throw new Error("Source identity requires declared paths");
  paths.forEach((path) => string(path, "source path"));
}

function validateEnvironment(value: unknown): void {
  const environment = object(value, "environment");
  for (const key of ["platform", "architecture", "osRelease", "cpuModel", "bun", "nodeCompatibility", "playwright", "chromium", "userAgent"]) string(environment[key], key);
  for (const key of ["logicalCpuCount", "totalMemoryBytes", "hardwareConcurrency"]) {
    if (integer(environment[key], key) < 1) throw new Error(`${key} must be positive`);
  }
  renderer(environment.renderer);
  string(environment.chromiumExecutable, "chromium executable");
  hash(environment.chromiumExecutableSha256, "chromium executable digest");
  if (environment.chromiumChannel !== "chromium" && environment.chromiumChannel !== "headless-shell") throw new Error("Unknown Chromium channel");
}

function validateLive(value: unknown, expectedDurationMs: number, warmupSteps: number): void {
  const live = object(value, "live cadence");
  renderer(live.renderer);
  if (live.executionBackend !== "direct" && live.executionBackend !== "worker") throw new Error("Unknown actual live backend");
  if (live.driver !== "production-frame-loop" || live.includesProductChrome !== false || live.responsivenessProxy !== "16ms-main-thread-timer-lateness") throw new Error("Unknown live cadence methodology");
  const start = checkpoint(live.start), end = checkpoint(live.end);
  if (start.stepIndex !== warmupSteps) throw new Error("Live warmup checkpoint differs");
  const elapsed = finite(live.elapsedMs, "elapsedMs");
  const started = finite(live.startedAtMs, "startedAtMs");
  if (elapsed < expectedDurationMs) throw new Error("Live measurement must span declared wall time");
  const paints = array(live.paints, "paints");
  if (!paints.length) throw new Error("Live measurement must include actual paints");
  let previous = warmupSteps, steps = 0, lastTime = -1;
  const counts = [integer(start.particleCount, "start count"), integer(end.particleCount, "end count")];
  for (const entry of paints) {
    const paint = object(entry, "paint");
    const advanced = integer(paint.simSteps, "simSteps", 4);
    const at = finite(paint.presentedAtMs, "presentedAtMs");
    finite(paint.rafTimestampMs, "rafTimestampMs"); counts.push(integer(paint.particleCount, "particleCount", 16384));
    integer(paint.bodyContactCount, "bodyContactCount");
    for (const key of ["workerComputeMs", "workerCaptureMs", "workerRoundTripMs"]) if (paint[key] !== null) finite(paint[key], key);
    if (at < lastTime || paint.stepIndex !== previous + advanced) throw new Error("Live paints must remain ordered by actual simulation steps");
    previous += advanced; steps += advanced; lastTime = at;
  }
  if (end.stepIndex !== previous) throw new Error("Live end checkpoint differs");
  if (Math.abs(lastTime - started - elapsed) > 1e-6) throw new Error("Live duration differs from actual start/last paint timestamps");
  if (live.minimumParticleCount !== Math.min(...counts) || live.maximumParticleCount !== Math.max(...counts)) throw new Error("Live particle range differs from observations");
  if (Math.abs(finite(live.presentedFps, "presentedFps") - paints.length * 1000 / elapsed) > 1e-6 ||
      Math.abs(finite(live.simulationStepsPerSecond, "simulationStepsPerSecond") - steps * 1000 / elapsed) > 1e-6) throw new Error("FPS/TPS must derive from actual live cadence");
  array(live.timerTaskLatenessMs, "timer lateness").forEach((value) => finite(value, "timer lateness"));
}

/** Guards loaded reports and browser results before accepting their evidence. */
export function assertBenchmarkReport(value: unknown): asserts value is BenchmarkReport {
  const report = object(value, "report");
  if (report.schemaVersion !== SCHEMA_VERSION || report.benchmarkVersion !== BENCHMARK_VERSION || report.status !== "complete") throw new Error("Unknown or incomplete benchmark schema");
  if (!/^[a-z0-9][a-z0-9-]{0,119}$/.test(string(report.runId, "runId"))) throw new Error("Invalid runId");
  if (!Number.isFinite(Date.parse(string(report.createdAtUtc, "createdAtUtc")))) throw new Error("Invalid creation time");
  const stages = ["original", "stage1", "stage2", "stage3", "stage4", "stage5", "smoke"];
  const stage = string(report.stage, "stage");
  if (!stages.includes(stage)) throw new Error("Unknown benchmark stage");
  validateProducer(report.producer); validateEnvironment(report.environment);
  const workload = object(report.workload, "workload");
  const expected = stage === "smoke" ? SMOKE_WORKLOAD : DEFAULT_WORKLOAD;
  if (canonicalJson(workload) !== canonicalJson(expected)) throw new Error("Workload differs from the declared benchmark profile");
  if (stage === "original" || stage === "smoke") {
    if (report.role !== (stage === "original" ? "baseline" : "smoke")) throw new Error("Invalid original/smoke role");
    if (report.previousRun !== null || report.comparison !== null) throw new Error("An original or smoke run cannot have a previous stage");
  } else {
    const previous = object(report.previousRun, "previousRun");
    string(previous.runId, "previous runId"); hash(previous.reportSha256, "previous report");
    const index = stages.indexOf(stage);
    if (report.role !== "before" && report.role !== "after") throw new Error("Optimization stage needs before/after role");
    const preceding = previous.stage === stages[index - 1] && ["baseline", "after"].includes(string(previous.role, "previous role"));
    const sameStageBefore = report.role === "after" && previous.stage === stage && previous.role === "before";
    if (!preceding && !sameStageBefore) throw new Error("Stage must link its before record or immediately previous accepted stage");
    const comparison = object(report.comparison, "comparison");
    if (typeof comparison.comparable !== "boolean") throw new Error("Comparison must disclose comparability");
    array(comparison.reasons, "comparison reasons").forEach((item) => string(item, "reason"));
    array(comparison.semanticDifferences, "semantic differences").forEach((item) => string(item, "difference"));
    if (!/^(direct|worker) -> (direct|worker)$/.test(string(comparison.backendTransition, "backend transition"))) throw new Error("Invalid backend transition");
    const changes = array(comparison.timingChanges, "timing changes");
    if ((comparison.comparable && changes.length !== 8) || (!comparison.comparable && changes.length !== 0)) throw new Error("Timing comparisons require matching workloads/runtime");
    for (const entry of changes) {
      const change = object(entry, "timing change");
      if (!["forward", "reverse"].includes(string(change.caseId, "comparison case")) ||
          !["advanceMs", "captureMs", "copyValidateMs", "renderSubmitMs"].includes(string(change.metric, "comparison metric"))) throw new Error("Unknown comparison case/metric");
      const before = finite(change.beforeMedianReplicateMeanMs, "before median");
      const after = finite(change.afterMedianReplicateMeanMs, "after median");
      if (before === 0 ? change.afterOverBeforeRatio !== null :
          Math.abs(finite(change.afterOverBeforeRatio, "timing ratio") - after / before) > 1e-6) throw new Error("Timing ratio differs from recorded medians");
      for (const key of ["beforeMeanRangeMs", "afterMeanRangeMs"]) {
        const range = array(change[key], "replicate range");
        if (range.length !== 2 || finite(range[0], "minimum mean") > finite(range[1], "maximum mean")) throw new Error("Invalid replicate range");
      }
    }
  }
  const replicates = array(report.replicates, "replicates");
  if (replicates.length !== expected.cases.length * expected.repetitions) throw new Error("Replicate count differs");
  const seen = new Set<string>();
  for (const value of replicates) {
    const replicate = object(value, "replicate");
    if (replicate.fixedStepBackend !== "direct") throw new Error("Primary physics timing must use direct fixed-step WASM");
    const scenario = expected.cases.find((scenario) => scenario.id === replicate.caseId);
    const repeat = integer(replicate.repetition, "repetition", expected.repetitions);
    if (scenario === undefined || repeat < 1 || seen.has(`${scenario.id}-${repeat}`)) throw new Error("Invalid or duplicate replicate");
    seen.add(`${scenario.id}-${repeat}`); renderer(replicate.renderer);
    if (canonicalJson(replicate.renderer) !== canonicalJson(object(report.environment, "environment").renderer)) throw new Error("Renderer changed within the run");
    const start = checkpoint(replicate.start), end = checkpoint(replicate.end);
    if (start.stepIndex !== scenario.warmupSteps || end.stepIndex !== scenario.warmupSteps + expected.sampleCount) throw new Error("Fixed-step checkpoints differ");
    const samples = array(replicate.samples, "samples");
    if (samples.length !== expected.sampleCount) throw new Error("Timed sample count differs");
    const rows = samples.map((entry, index) => {
      const row = object(entry, "sample");
      if (row.stepIndex !== scenario.warmupSteps + index + 1) throw new Error("Fixed-step sample sequence differs");
      integer(row.particleCount, "sample count", 16384);
      for (const metric of ["advanceMs", "captureMs", "copyValidateMs", "renderSubmitMs"]) finite(row[metric], metric);
      return row;
    });
    if (end.particleCount !== rows[rows.length - 1]?.particleCount) throw new Error("Final checkpoint count differs from last measured sample");
    const summary = object(replicate.summary, "summary");
    for (const metric of ["advanceMs", "captureMs", "copyValidateMs", "renderSubmitMs"]) {
      if (canonicalJson(summary[metric]) !== canonicalJson(summarize(rows.map((row) => finite(row[metric], metric))))) throw new Error("Timing summary differs from raw samples");
    }
    validateLive(replicate.live, expected.liveDurationMs, scenario.warmupSteps);
    if (canonicalJson(object(replicate.live, "live").renderer) !== canonicalJson(replicate.renderer)) throw new Error("Live renderer changed or lost its fixed-phase context");
    if (object(replicate.live, "live").executionBackend !== object(report.producer, "producer").executionBackend) throw new Error("Actual live backend differs from producer claim");
  }
  const parsedReplicates = replicates as unknown as readonly CaseReplicate[];
  if (canonicalJson(report.semanticConsistency) !== canonicalJson(semanticConsistency(parsedReplicates))) throw new Error("Semantic consistency flags differ from checkpoints");
  const uncertainty = object(report.uncertainty, "uncertainty");
  if (uncertainty.unit !== "sequential-replicate") throw new Error("Uncertainty must use replicate units");
  string(uncertainty.note, "uncertainty note");
  if (canonicalJson(uncertainty.caseReplicateMeans) !== canonicalJson(caseReplicateMeans(parsedReplicates))) throw new Error("Replicate uncertainty must stay separate by case");
  const timing = object(report.timingInterpretation, "timingInterpretation");
  if (timing.render !== "CPU submission; GPU completion is not timed" || timing.fps !== "actual production frame-loop paint submissions per elapsed wall second") throw new Error("Timing interpretation must remain explicit");
}

export function assertAcceptedReference(report: BenchmarkReport): void {
  assertBenchmarkReport(report);
  if (report.semanticConsistency.withinRunDifferences.length || report.semanticConsistency.fixedVsLiveWarmupDifferences.length ||
      (report.comparison !== null && (!report.comparison.comparable || report.comparison.semanticDifferences.length))) {
    throw new Error("Previous report has unresolved comparability or semantic divergence and cannot become an accepted stage reference");
  }
}
