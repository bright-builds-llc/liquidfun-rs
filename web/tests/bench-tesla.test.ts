import { describe, expect, it } from "vitest";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  canonicalJson, comparisonMismatchReasons, DEFAULT_WORKLOAD, summarize, semanticState,
} from "../scripts/bench-tesla/model";
import { reserveRunDirectory, writeImmutableJson } from "../scripts/bench-tesla/results";
import { assertAcceptedReference, assertBenchmarkReport } from "../scripts/bench-tesla/validation";
import { reportFixture } from "./bench-tesla-fixture";
import { compareReports, semanticConsistency } from "../scripts/bench-tesla/compare";
import { parseOptions } from "../scripts/bench-tesla/options";
import { assertSameProducer } from "../scripts/bench-tesla/identity";

describe("Tesla benchmark comparability", () => {
  it("compares different optimization producers while requiring the same workload and runtime", () => {
    // Arrange
    const before = {
      workload: DEFAULT_WORKLOAD,
      environment: { cpu: "test CPU", browser: "Chromium test", renderer: "test GL" },
      producer: { revision: "before", wasmSha256: "old" },
    };
    const after = { ...before, producer: { revision: "after", wasmSha256: "new" } };

    // Act / Assert
    expect(comparisonMismatchReasons(before, after)).toEqual([]);
    expect(comparisonMismatchReasons(before, {
      ...after,
      workload: { ...after.workload, devicePixelRatio: 2 },
    })).toContain("workload differs");
    expect(comparisonMismatchReasons(before, {
      ...after,
      environment: { ...after.environment, renderer: "different GL" },
    })).toContain("runtime or hardware differs");
  });
});

describe("immutable Tesla benchmark evidence", () => {
  it("rejects existing run IDs and files without changing their original evidence", async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), "tesla-benchmark-test-"));
    try {
      const directory = await reserveRunDirectory(root, "original-test");
      await writeImmutableJson(join(directory, "report.json"), { result: "original" });

      // Act / Assert
      await expect(reserveRunDirectory(root, "original-test")).rejects.toThrow();
      await expect(writeImmutableJson(join(directory, "report.json"), { result: "replacement" })).rejects.toThrow();
      expect(JSON.parse(await readFile(join(directory, "report.json"), "utf8"))).toEqual({ result: "original" });
      await expect(reserveRunDirectory(root, "../outside")).rejects.toThrow("run ID");
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
});

describe("Tesla benchmark measurements", () => {
  it("summarizes measured milliseconds without inventing an FPS or independent-sample confidence interval", () => {
    // Arrange
    const milliseconds = [100, 1, 4, 2, 3];

    // Act
    const summary = summarize(milliseconds);

    // Assert
    expect(summary).toEqual({ count: 5, mean: 22, median: 3, p95: 100, min: 1, max: 100 });
    expect(() => summarize([1, Number.NaN])).toThrow("finite");
  });

  it("fingerprints named semantic lanes and preserves signed zero independently of object storage", () => {
    // Arrange
    const frame = {
      stepIndex: 4, particleCount: 1, rigidShapeCount: 0,
      maxSpeed: 2, stuckCandidateCount: 0, bodyContactCount: 0,
      particlePositions: new Float32Array([-0, 2]),
      particleColors: new Uint8Array([77, 163, 255, 255]),
      particleRadii: new Float32Array([0.005]),
      rigidSegments: new Float32Array(), rigidCircles: new Float32Array(), circleLabels: [],
    };
    const clone = { ...frame, particlePositions: new Float32Array([-0, 2]) };

    // Act / Assert
    const original = canonicalJson(semanticState(frame));
    expect(original).toContain('"particlePositions":["negative-zero",2]');
    expect(canonicalJson(semanticState(clone))).toBe(original);
    expect(canonicalJson(semanticState({
      ...clone, particlePositions: new Float32Array([0, 2]),
    }))).not.toBe(original);
  });
});

describe("Tesla benchmark report boundary", () => {
  it("rejects incomplete trajectories, nonfinite timings and anonymous producers", () => {
    // Arrange
    const report = reportFixture();

    // Act / Assert
    expect(() => assertBenchmarkReport(report)).not.toThrow();
    expect(() => assertBenchmarkReport({ ...report, producer: { ...report.producer, wasmSha256: "" } })).toThrow();
    const first = report.replicates[0]!;
    expect(() => assertBenchmarkReport({ ...report, replicates: [{ ...first, samples: [] }, ...report.replicates.slice(1)] })).toThrow("sample");
    expect(() => assertBenchmarkReport({ ...report, replicates: [{ ...first, samples: [{ ...first.samples[0]!, advanceMs: Number.NaN }, ...first.samples.slice(1)] }, ...report.replicates.slice(1)] })).toThrow("finite");
    expect(() => assertBenchmarkReport({ ...report, previousRun: { runId: "unknown", stage: "stage5", reportSha256: "1".repeat(64) } })).toThrow("original");
  });
});

describe("stage-to-stage Tesla benchmark attribution", () => {
  it("reports comparable per-case timing ratios while flagging exact semantic differences", () => {
    // Arrange
    const before = reportFixture();
    const after = { ...before, producer: { ...before.producer, revision: "2".repeat(40), wasmSha256: "2".repeat(64) },
      replicates: before.replicates.map((replicate) => ({ ...replicate,
        samples: replicate.samples.map((sample) => ({ ...sample, advanceMs: 0.5 })),
        summary: { ...replicate.summary, advanceMs: summarize(Array.from({ length: 40 }, () => 0.5)) },
        end: { ...replicate.end, positionsSha256: "2".repeat(64) },
      })) };

    // Act
    const comparison = compareReports(before, after);

    // Assert
    expect(comparison.comparable).toBe(true);
    expect(comparison.timingChanges.find((change) => change.caseId === "forward" && change.metric === "advanceMs")?.afterOverBeforeRatio).toBe(0.5);
    expect(comparison.semanticDifferences).toContain("forward/1/end/positionsSha256");
    const differentRenderer = compareReports(before, { ...after,
      environment: { ...after.environment, chromium: "different runtime" },
    });
    expect(differentRenderer.comparable).toBe(false);
    expect(differentRenderer.timingChanges).toEqual([]);
  });

  it("allows fresh BEFORE to AFTER links at the same stage without accepting skipped stage identities", () => {
    // Arrange
    const original = reportFixture();
    const before = { ...original, runId: "stage1-before", stage: "stage1" as const, role: "before" as const,
      previousRun: { runId: original.runId, stage: original.stage, role: original.role, reportSha256: "1".repeat(64) },
      comparison: compareReports(original, original) };
    const after = { ...before, runId: "stage1-after", role: "after" as const,
      previousRun: { runId: before.runId, stage: before.stage, role: before.role, reportSha256: "2".repeat(64) } };

    // Act / Assert
    expect(() => assertBenchmarkReport(before)).not.toThrow();
    expect(() => assertBenchmarkReport(after)).not.toThrow();
    expect(() => assertBenchmarkReport({ ...after, previousRun: { ...after.previousRun, stage: "stage3" } })).toThrow("previous");
    const divergent = { ...original.replicates[0]!, end: { ...original.replicates[0]!.end, sha256: "2".repeat(64) } };
    expect(semanticConsistency([original.replicates[0]!, { ...divergent, repetition: 2 }]).withinRunDifferences).toEqual(["forward/2/end"]);
  });
});

describe("runnable Tesla benchmark profiles", () => {
  it("keeps canonical workload fixed and smoke evidence explicitly separate", () => {
    // Arrange / Act / Assert
    expect(DEFAULT_WORKLOAD).toMatchObject({ rate: 1440, particleIterations: 4, particleRadius: 0.005,
      rigidVelocityIterations: 8, rigidPositionIterations: 3,
      gravity: { x: 0, y: -10 }, inletSpeed: 2, particleDamping: 0.2, wallFriction: 0.05,
      maximumParticleCount: 16384, channelClearWidth: 0.06,
      sampleCount: 40, repetitions: 3, liveDurationMs: 2000, viewport: { width: 1280, height: 960 }, devicePixelRatio: 1,
      cases: [{ id: "forward", direction: "1", warmupSteps: 360 }, { id: "reverse", direction: "-1", warmupSteps: 384 }] });
    expect(parseOptions(["--", "--help"]).help).toBe(true);
    expect(parseOptions(["--smoke"]).role).toBe("smoke");
    expect(parseOptions([]).browserChannel).toBe("chromium");
    expect(() => parseOptions(["--stage", "stage1"])).toThrow("previous");
    expect(() => parseOptions(["--unknown"])).toThrow("Unknown");
    expect(() => parseOptions(["--browser-channel", "user-browser"])).toThrow("channel");
  });
});

describe("benchmark evidence review guards", () => {
  it("rejects generated bindings or dependent source drift during the same run", () => {
    // Arrange
    const producer = reportFixture().producer;

    // Act / Assert
    expect(() => assertSameProducer(producer, producer)).not.toThrow();
    expect(() => assertSameProducer(producer, { ...producer, generatedBindingsSha256: "2".repeat(64) })).toThrow("generatedBindingsSha256");
    expect(() => assertSameProducer(producer, { ...producer, sourceFilesSha256: "2".repeat(64) })).toThrow("sourceFilesSha256");
  });

  it("never promotes complete but divergent or incomparable evidence into a new accepted reference", () => {
    // Arrange
    const original = reportFixture();
    const stage = { ...original, stage: "stage1" as const, role: "after" as const,
      previousRun: { runId: original.runId, stage: original.stage, role: original.role, reportSha256: "1".repeat(64) },
      comparison: compareReports(original, original) };

    // Act / Assert
    expect(() => assertAcceptedReference(original)).not.toThrow();
    expect(() => assertAcceptedReference({ ...stage, comparison: { ...stage.comparison,
      semanticDifferences: ["forward/1/end/positionsSha256"] } })).toThrow("accepted");
    expect(() => assertAcceptedReference({ ...stage, comparison: { ...stage.comparison,
      comparable: false, reasons: ["runtime differs"], timingChanges: [] } })).toThrow("accepted");
    const changed = original.replicates.map((replicate, index) => index === 1
      ? { ...replicate, end: { ...replicate.end, sha256: "2".repeat(64) } } : replicate);
    expect(() => assertAcceptedReference({ ...original, replicates: changed,
      semanticConsistency: semanticConsistency(changed) })).toThrow("accepted");
  });

  it("rejects a live fallback or lost GPU context instead of inheriting the fixed-phase Metal label", () => {
    // Arrange
    const original = reportFixture();
    const first = original.replicates[0]!;
    const changed = { ...first, live: { ...first.live,
      renderer: { ...first.renderer, backend: "canvas-metaball" as const, contextLost: true } } };

    // Act / Assert
    expect(() => assertBenchmarkReport({ ...original,
      replicates: [changed, ...original.replicates.slice(1)] })).toThrow("Live renderer");
    expect(() => assertBenchmarkReport({ ...original,
      environment: { ...original.environment, chromiumExecutableSha256: "" } })).toThrow("digest");
  });
});
