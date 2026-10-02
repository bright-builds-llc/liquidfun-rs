import type { BenchmarkReport, CaseReplicate } from "../scripts/bench-tesla/contracts";
import { DEFAULT_WORKLOAD, summarize } from "../scripts/bench-tesla/model";
import { caseReplicateMeans, semanticConsistency } from "../scripts/bench-tesla/compare";

export function reportFixture(): BenchmarkReport {
  const sha = "1".repeat(64);
  const renderer = { backend: "webgl2" as const, contextLost: false, vendor: "fixture", renderer: "fixture", version: "fixture",
    unmaskedVendor: "unavailable", unmaskedRenderer: "unavailable", softwareClassification: "unavailable" as const };
  const checkpoint = (stepIndex: number) => ({ stepIndex, particleCount: 1000, rigidShapeCount: 468,
    bodyContactCount: 100, stuckCandidateCount: 0, maxSpeed: 2,
    sha256: sha, positionsSha256: sha, colorsSha256: sha, radiiSha256: sha, geometrySha256: sha });
  const summary = summarize(Array.from({ length: 40 }, () => 1));
  const replicates: CaseReplicate[] = DEFAULT_WORKLOAD.cases.flatMap((scenario) =>
    [1, 2, 3].map((repetition) => {
      const start = checkpoint(scenario.warmupSteps);
      const end = checkpoint(scenario.warmupSteps + 40);
      return { caseId: scenario.id, repetition, fixedStepBackend: "direct", renderer, start, end,
        samples: Array.from({ length: 40 }, (_, index) => ({ stepIndex: start.stepIndex + index + 1,
          particleCount: 1000, advanceMs: 1, captureMs: 1, copyValidateMs: 1, renderSubmitMs: 1 })),
        summary: { advanceMs: summary, captureMs: summary, copyValidateMs: summary, renderSubmitMs: summary },
        live: { driver: "production-frame-loop", executionBackend: "direct", renderer, start, end, elapsedMs: 2000, startedAtMs: 0, presentedFps: 20,
          simulationStepsPerSecond: 20, minimumParticleCount: 1000, maximumParticleCount: 1000,
          paints: Array.from({ length: 40 }, (_, index) => ({
            presentedAtMs: (index + 1) * 50, rafTimestampMs: index * 50,
            simSteps: 1, stepIndex: start.stepIndex + index + 1, particleCount: 1000,
            bodyContactCount: 100, workerComputeMs: null, workerCaptureMs: null, workerRoundTripMs: null,
          })), timerTaskLatenessMs: [1, 2, 3], responsivenessProxy: "16ms-main-thread-timer-lateness",
          includesProductChrome: false },
      };
    }));
  return { schemaVersion: 1, benchmarkVersion: "tesla-valve-v1", runId: "original-fixture", stage: "original", role: "baseline",
    createdAtUtc: "2026-10-02T18:00:00.000Z", status: "complete", workload: DEFAULT_WORKLOAD,
    previousRun: null, comparison: null, replicates,
    semanticConsistency: semanticConsistency(replicates),
    producer: { revision: "1".repeat(40), sourceDiffSha256: sha, sourceFilesSha256: sha, sourcePaths: ["web"],
      wasmSha256: sha, generatedBindingsSha256: sha, benchmarkBundleSha256: sha, rustToolchain: "fixture", executionBackend: "direct",
      buildProfile: "release", wasmTarget: "wasm32-unknown-unknown", wasmPackVersion: "fixture", rustflags: "unset", cargoEncodedRustflags: "unset" },
    environment: { platform: "fixture", architecture: "fixture", osRelease: "fixture", cpuModel: "fixture", logicalCpuCount: 1,
      totalMemoryBytes: 1000, bun: "fixture", nodeCompatibility: "fixture", playwright: "fixture", chromium: "fixture",
      userAgent: "fixture", hardwareConcurrency: 1, renderer, chromiumChannel: "chromium", chromiumExecutable: "fixture", chromiumExecutableSha256: sha },
    uncertainty: { unit: "sequential-replicate", note: "trajectory samples are correlated", caseReplicateMeans: caseReplicateMeans(replicates) },
    timingInterpretation: { render: "CPU submission; GPU completion is not timed",
      fps: "actual production frame-loop paint submissions per elapsed wall second" },
  };
}
