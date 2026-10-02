import type { BenchmarkReport, CaseReplicate, ReportComparison } from "./contracts";
import { comparisonMismatchReasons, summarize } from "./model";

export const TIMING_METRICS = ["advanceMs", "captureMs", "copyValidateMs", "renderSubmitMs"] as const;

export function caseReplicateMeans(replicates: readonly CaseReplicate[]) {
  return ["forward", "reverse"].map((caseId) => {
    const cases = replicates.filter((entry) => entry.caseId === caseId);
    return { caseId,
      advanceMs: summarize(cases.map((entry) => entry.summary.advanceMs.mean)),
      captureMs: summarize(cases.map((entry) => entry.summary.captureMs.mean)),
      copyValidateMs: summarize(cases.map((entry) => entry.summary.copyValidateMs.mean)),
      renderSubmitMs: summarize(cases.map((entry) => entry.summary.renderSubmitMs.mean)),
    };
  });
}

export function semanticConsistency(replicates: readonly CaseReplicate[]) {
  const withinRunDifferences: string[] = [], fixedVsLiveWarmupDifferences: string[] = [];
  for (const replicate of replicates) {
    const first = replicates.find((entry) => entry.caseId === replicate.caseId);
    if (first !== undefined) for (const point of ["start", "end"] as const) {
      if (first[point].sha256 !== replicate[point].sha256) withinRunDifferences.push(`${replicate.caseId}/${replicate.repetition}/${point}`);
    }
    if (replicate.start.sha256 !== replicate.live.start.sha256) fixedVsLiveWarmupDifferences.push(`${replicate.caseId}/${replicate.repetition}`);
  }
  return { withinRunDifferences, fixedVsLiveWarmupDifferences };
}

export function compareReports(before: BenchmarkReport, after: BenchmarkReport): ReportComparison {
  const reasons = comparisonMismatchReasons(before, after);
  for (const key of ["rustToolchain", "wasmPackVersion", "buildProfile", "wasmTarget", "rustflags", "cargoEncodedRustflags"] as const) {
    if (before.producer[key] !== after.producer[key]) reasons.push(`producer toolchain/configuration differs: ${key}`);
  }
  if (before.schemaVersion !== after.schemaVersion || before.benchmarkVersion !== after.benchmarkVersion) reasons.push("benchmark methodology differs");
  const semanticDifferences: string[] = [];
  for (const replicate of after.replicates) {
    const previous = before.replicates.find((entry) => entry.caseId === replicate.caseId && entry.repetition === replicate.repetition);
    if (previous === undefined) { semanticDifferences.push(`${replicate.caseId}/${replicate.repetition}: missing checkpoint`); continue; }
    for (const checkpoint of ["start", "end"] as const) {
      for (const lane of ["sha256", "positionsSha256", "colorsSha256", "radiiSha256", "geometrySha256"] as const) {
        if (replicate[checkpoint][lane] !== previous[checkpoint][lane]) semanticDifferences.push(`${replicate.caseId}/${replicate.repetition}/${checkpoint}/${lane}`);
      }
    }
  }
  const previousMeans = caseReplicateMeans(before.replicates), currentMeans = caseReplicateMeans(after.replicates);
  const timingChanges = reasons.length ? [] : currentMeans.flatMap((entry, index) => TIMING_METRICS.map((metric) => {
    const old = previousMeans[index]![metric], current = entry[metric];
    return { caseId: entry.caseId, metric, beforeMedianReplicateMeanMs: old.median,
      afterMedianReplicateMeanMs: current.median, afterOverBeforeRatio: old.median === 0 ? null : current.median / old.median,
      beforeMeanRangeMs: [old.min, old.max] as const, afterMeanRangeMs: [current.min, current.max] as const };
  }));
  return { comparable: reasons.length === 0, reasons, semanticDifferences, timingChanges,
    backendTransition: `${before.producer.executionBackend} -> ${after.producer.executionBackend}` };
}
