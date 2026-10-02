import type { BenchmarkWorkload, Distribution } from "./model";

export type BenchmarkStage = "original" | "stage1" | "stage2" | "stage3" | "stage4" | "stage5" | "smoke";
export type RunRole = "baseline" | "before" | "after" | "smoke";
export type RendererIdentity = {
  readonly backend: "webgl2" | "canvas-metaball";
  readonly contextLost: boolean | "unavailable";
  readonly vendor: string; readonly renderer: string; readonly version: string;
  readonly unmaskedVendor: string; readonly unmaskedRenderer: string;
  readonly softwareClassification: "software" | "not-identified-as-software" | "unavailable";
};

export type EnvironmentIdentity = {
  readonly platform: string; readonly architecture: string; readonly osRelease: string;
  readonly cpuModel: string; readonly logicalCpuCount: number; readonly totalMemoryBytes: number;
  readonly bun: string; readonly nodeCompatibility: string; readonly playwright: string;
  readonly chromium: string; readonly userAgent: string; readonly hardwareConcurrency: number;
  readonly chromiumChannel: "chromium" | "headless-shell"; readonly chromiumExecutable: string;
  readonly chromiumExecutableSha256: string;
  readonly renderer: RendererIdentity;
};

export type ProducerIdentity = {
  readonly revision: string;
  readonly sourceDiffSha256: string;
  readonly sourceFilesSha256: string;
  readonly sourcePaths: readonly string[];
  readonly wasmSha256: string;
  readonly generatedBindingsSha256: string;
  readonly benchmarkBundleSha256: string;
  readonly rustToolchain: string;
  readonly buildProfile: "release"; readonly wasmTarget: "wasm32-unknown-unknown";
  readonly wasmPackVersion: string; readonly rustflags: string; readonly cargoEncodedRustflags: string;
  readonly executionBackend: "direct" | "worker";
};

export type SemanticCheckpoint = {
  readonly stepIndex: number; readonly particleCount: number; readonly rigidShapeCount: number;
  readonly bodyContactCount: number; readonly stuckCandidateCount: number; readonly maxSpeed: number;
  readonly sha256: string; readonly positionsSha256: string; readonly colorsSha256: string;
  readonly radiiSha256: string; readonly geometrySha256: string;
};

export type TimedSample = {
  readonly stepIndex: number; readonly particleCount: number;
  readonly advanceMs: number; readonly captureMs: number; readonly copyValidateMs: number;
  readonly renderSubmitMs: number;
};

export type LivePaint = {
  readonly presentedAtMs: number; readonly rafTimestampMs: number;
  readonly simSteps: number; readonly stepIndex: number; readonly particleCount: number;
  readonly bodyContactCount: number;
  readonly workerComputeMs: number | null; readonly workerCaptureMs: number | null;
  readonly workerRoundTripMs: number | null;
};

export type LiveCadence = {
  readonly driver: "production-frame-loop";
  readonly executionBackend: "direct" | "worker";
  readonly renderer: RendererIdentity;
  readonly start: SemanticCheckpoint; readonly end: SemanticCheckpoint;
  readonly elapsedMs: number; readonly presentedFps: number; readonly simulationStepsPerSecond: number;
  readonly startedAtMs: number;
  readonly paints: readonly LivePaint[]; readonly timerTaskLatenessMs: readonly number[];
  readonly responsivenessProxy: "16ms-main-thread-timer-lateness";
  readonly includesProductChrome: false;
  readonly minimumParticleCount: number; readonly maximumParticleCount: number;
};

export type CaseReplicate = {
  readonly caseId: "forward" | "reverse"; readonly repetition: number;
  readonly fixedStepBackend: "direct";
  readonly renderer: RendererIdentity;
  readonly start: SemanticCheckpoint; readonly end: SemanticCheckpoint;
  readonly samples: readonly TimedSample[];
  readonly summary: {
    readonly advanceMs: Distribution; readonly captureMs: Distribution;
    readonly copyValidateMs: Distribution; readonly renderSubmitMs: Distribution;
  };
  readonly live: LiveCadence;
};

export type ReportComparison = {
  readonly comparable: boolean; readonly reasons: readonly string[];
  readonly semanticDifferences: readonly string[];
  readonly backendTransition: string;
  readonly timingChanges: readonly {
    readonly caseId: string; readonly metric: string;
    readonly beforeMedianReplicateMeanMs: number; readonly afterMedianReplicateMeanMs: number;
    readonly afterOverBeforeRatio: number | null;
    readonly beforeMeanRangeMs: readonly [number, number]; readonly afterMeanRangeMs: readonly [number, number];
  }[];
};

export type BenchmarkReport = {
  readonly schemaVersion: 1; readonly benchmarkVersion: "tesla-valve-v1";
  readonly runId: string; readonly stage: BenchmarkStage; readonly role: RunRole; readonly createdAtUtc: string;
  readonly status: "complete";
  readonly producer: ProducerIdentity; readonly environment: EnvironmentIdentity;
  readonly workload: BenchmarkWorkload;
  readonly previousRun: null | { readonly runId: string; readonly stage: BenchmarkStage; readonly role: RunRole; readonly reportSha256: string };
  readonly comparison: ReportComparison | null;
  readonly semanticConsistency: {
    readonly withinRunDifferences: readonly string[]; readonly fixedVsLiveWarmupDifferences: readonly string[];
  };
  readonly replicates: readonly CaseReplicate[];
  readonly uncertainty: {
    readonly unit: "sequential-replicate";
    readonly note: string;
    readonly caseReplicateMeans: readonly {
      readonly caseId: string; readonly advanceMs: Distribution; readonly captureMs: Distribution;
      readonly copyValidateMs: Distribution; readonly renderSubmitMs: Distribution;
    }[];
  };
  readonly timingInterpretation: {
    readonly render: "CPU submission; GPU completion is not timed";
    readonly fps: "actual production frame-loop paint submissions per elapsed wall second";
  };
};
