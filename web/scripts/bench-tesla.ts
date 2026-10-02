import { randomUUID } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { openBenchmarkBrowser } from "./bench-tesla/browser";
import {
  caseReplicateMeans,
  compareReports,
  semanticConsistency,
} from "./bench-tesla/compare";
import type {
  BenchmarkReport,
  CaseReplicate,
  EnvironmentIdentity,
} from "./bench-tesla/contracts";
import {
  assertSameProducer,
  hostIdentity,
  producerIdentity,
  rebuildWasm,
  sha256,
} from "./bench-tesla/identity";
import {
  BENCHMARK_VERSION,
  canonicalJson,
  DEFAULT_WORKLOAD,
  SCHEMA_VERSION,
  SMOKE_WORKLOAD,
} from "./bench-tesla/model";
import { HELP, parseOptions } from "./bench-tesla/options";
import { reserveRunDirectory, writeImmutableJson } from "./bench-tesla/results";
import {
  assertAcceptedReference,
  assertBenchmarkReport,
} from "./bench-tesla/validation";

const repoRoot = resolve(import.meta.dirname, "../..");

async function main(): Promise<void> {
  const options = parseOptions(process.argv.slice(2));
  if (options.help) {
    console.log(HELP);
    return;
  }
  const createdAtUtc = new Date().toISOString();
  const runId =
    options.maybeRunId ??
    `${createdAtUtc.replace(/[:.]/g, "-").toLowerCase()}-${options.stage}-${options.role}-${randomUUID().slice(0, 8)}`;
  const root =
    options.maybeOutputRoot === undefined
      ? resolve(repoRoot, "docs/benchmarks/tesla-valve/runs")
      : resolve(options.maybeOutputRoot);
  const directory = await reserveRunDirectory(root, runId);
  const workload =
    options.stage === "smoke" ? SMOKE_WORKLOAD : DEFAULT_WORKLOAD;
  const replicates: CaseReplicate[] = [];
  let maybeBrowser:
    Awaited<ReturnType<typeof openBenchmarkBrowser>> | undefined;
  try {
    let maybePrevious: BenchmarkReport | undefined;
    let previousSha256: string | undefined;
    if (options.maybePrevious !== undefined) {
      const bytes = await readFile(resolve(options.maybePrevious), "utf8");
      const parsed: unknown = JSON.parse(bytes);
      assertBenchmarkReport(parsed);
      assertAcceptedReference(parsed);
      maybePrevious = parsed;
      previousSha256 = sha256(bytes);
    }
    console.log(`[bench:tesla] ${runId}: rebuilding actual production WASM`);
    await rebuildWasm(repoRoot, resolve(directory, "wasm-build.log"));
    const snapshot = await producerIdentity(repoRoot);
    const { sourceManifest, ...source } = snapshot;
    await writeImmutableJson(
      resolve(directory, "source-manifest.json"),
      sourceManifest,
    );
    const browser = await openBenchmarkBrowser(
      resolve(repoRoot, "web"),
      resolve(repoRoot, "target/tesla-benchmark-build", runId),
      workload,
      options.browserChannel,
    );
    maybeBrowser = browser;
    if (browser.builtWasmSha256 !== source.wasmSha256)
      throw new Error(
        "Browser bundle WASM does not match the rebuilt producer",
      );
    const playwrightPackage: { version: string } = JSON.parse(
      await readFile(
        resolve(repoRoot, "web/node_modules/@playwright/test/package.json"),
        "utf8",
      ),
    );
    const liveBackend =
      options.maybeLiveBackend ??
      (options.role === "before"
        ? (maybePrevious?.producer.executionBackend ?? "direct")
        : options.stage === "stage4" || options.stage === "stage5"
          ? "worker"
          : "direct");
    const producer = {
      ...source,
      executionBackend: liveBackend,
      benchmarkBundleSha256: browser.benchmarkBundleSha256,
    };
    await writeImmutableJson(resolve(directory, "identity.json"), {
      schemaVersion: SCHEMA_VERSION,
      benchmarkVersion: BENCHMARK_VERSION,
      runId,
      stage: options.stage,
      role: options.role,
      createdAtUtc,
      workload,
      producer,
      sourceDiffInterpretation:
        "SHA-256 of canonical tracked git diff plus untracked source file path/content hashes; sourceFilesSha256 covers every declared source path",
    });
    for (const scenario of workload.cases) {
      for (
        let repetition = 1;
        repetition <= workload.repetitions;
        repetition += 1
      ) {
        console.log(
          `[bench:tesla] ${scenario.id} repeat ${repetition}/${workload.repetitions}: warm${scenario.warmupSteps}, fixed${workload.sampleCount}, live${workload.liveDurationMs}ms`,
        );
        const replicate = await browser.runCase(
          workload,
          scenario.id,
          repetition,
          liveBackend,
        );
        replicates.push(replicate);
        await writeImmutableJson(
          resolve(directory, `${scenario.id}-${repetition}.json`),
          replicate,
        );
        if (
          canonicalJson(replicate.renderer) !==
          canonicalJson(replicate.live.renderer)
        )
          throw new Error(
            `Live renderer changed; actual fixed/live identities are preserved in ${scenario.id}-${repetition}.json`,
          );
        console.log(
          `[bench:tesla] ${scenario.id} repeat${repetition}: physics mean${replicate.summary.advanceMs.mean.toFixed(3)}ms p95${replicate.summary.advanceMs.p95.toFixed(3)}ms, paints${replicate.live.presentedFps.toFixed(1)}/s sim${replicate.live.simulationStepsPerSecond.toFixed(1)}/s, liveN${replicate.live.minimumParticleCount}..${replicate.live.maximumParticleCount}`,
        );
      }
    }
    const first = replicates[0];
    if (first === undefined) throw new Error("Benchmark collected no cases");
    const environment: EnvironmentIdentity = {
      ...hostIdentity(repoRoot),
      playwright: playwrightPackage.version,
      chromium: browser.chromiumVersion,
      chromiumChannel: options.browserChannel,
      chromiumExecutable: browser.chromiumExecutable,
      chromiumExecutableSha256: browser.chromiumExecutableSha256,
      userAgent: browser.userAgent,
      hardwareConcurrency: browser.hardwareConcurrency,
      renderer: first.renderer,
    };
    const report: BenchmarkReport = {
      schemaVersion: SCHEMA_VERSION,
      benchmarkVersion: BENCHMARK_VERSION,
      runId,
      stage: options.stage,
      role: options.role,
      status: "complete",
      createdAtUtc,
      producer,
      environment,
      workload,
      replicates,
      semanticConsistency: semanticConsistency(replicates),
      previousRun:
        maybePrevious === undefined
          ? null
          : {
              runId: maybePrevious.runId,
              stage: maybePrevious.stage,
              role: maybePrevious.role,
              reportSha256: previousSha256!,
            },
      comparison: null,
      uncertainty: {
        unit: "sequential-replicate",
        note: "Adjacent trajectory samples are correlated. Replicate means describe run-to-run variation; no confidence interval is inferred from 40 adjacent steps.",
        caseReplicateMeans: caseReplicateMeans(replicates),
      },
      timingInterpretation: {
        render: "CPU submission; GPU completion is not timed",
        fps: "actual production frame-loop paint submissions per elapsed wall second",
      },
    };
    const complete =
      maybePrevious === undefined
        ? report
        : { ...report, comparison: compareReports(maybePrevious, report) };
    assertBenchmarkReport(complete);
    const afterSource = await producerIdentity(repoRoot);
    assertSameProducer(source, afterSource);
    maybeBrowser = undefined;
    await browser.close();
    await writeImmutableJson(resolve(directory, "report.json"), complete);
    console.log(
      `[bench:tesla] immutable report: ${resolve(directory, "report.json")}`,
    );
    if (
      complete.comparison !== null &&
      (!complete.comparison.comparable ||
        complete.comparison.semanticDifferences.length)
    ) {
      console.error(
        `[bench:tesla] comparison requires investigation: ${JSON.stringify(complete.comparison)}`,
      );
      process.exitCode = 2;
    }
    if (
      complete.semanticConsistency.withinRunDifferences.length ||
      complete.semanticConsistency.fixedVsLiveWarmupDifferences.length
    ) {
      console.error(
        `[bench:tesla] semantic consistency requires investigation: ${JSON.stringify(complete.semanticConsistency)}`,
      );
      process.exitCode = 2;
    }
  } catch (error) {
    await writeImmutableJson(resolve(directory, "failure.json"), {
      runId,
      stage: options.stage,
      role: options.role,
      createdAtUtc,
      failedAtUtc: new Date().toISOString(),
      workload,
      completedReplicates: replicates.length,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  } finally {
    if (maybeBrowser !== undefined) await maybeBrowser.close();
  }
}

if (
  process.argv[1] !== undefined &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
}
