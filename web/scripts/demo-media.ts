import { spawn, spawnSync } from "node:child_process";
import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";

import { chromium, type Browser, type BrowserContext, type Page } from "@playwright/test";

import {
  captureMediaFileRecord,
  captureInputSha256,
  commandVersionLine,
  compareOutputDirectories,
  ensureDirectory,
  ensureRegularFile,
  mp4Arguments,
  probeMedia,
  replaceOutputDirectory,
  validateMp4Probe,
  validateWebpProbe,
  webpArguments,
  writeTextAtomically,
} from "./demo-media/artifacts";
import {
  captureSceneFrames,
  installSyntheticAnimationClock,
} from "./demo-media/capture";
import {
  CAPTURE_PROFILE,
  SCENE_CAPTURE_PLANS,
  canonicalManifestJson,
  type DemoMediaManifest,
  type SceneCapturePlan,
  type SceneMediaRecord,
} from "./demo-media/model";

const PREVIEW_URL = "http://127.0.0.1:4173/liquidfun-rs/";
const PREVIEW_ORIGIN = "http://127.0.0.1:4173";
const READY_TIMEOUT_MILLISECONDS = 30_000;

type DemoMediaMode = "generate" | "check";

type PlaywrightIdentity = {
  readonly packageVersion: string;
  readonly chromiumRevision: string;
  readonly chromiumVersion: string;
  readonly executablePath: string;
};

type EncodingContext = {
  readonly nextDirectory: string;
  readonly page: Page;
  readonly maybeFailSceneId: SceneCapturePlan["id"] | undefined;
};

type PreviewProcess = ReturnType<typeof spawn>;
type CleanupAction = {
  readonly label: string;
  readonly run: () => Promise<void>;
};
type CleanupFailure = {
  readonly label: string;
  readonly error: Error;
};
type TerminatePreviewProcessDependencies = {
  readonly getExitCode: () => number | null;
  readonly kill: (signal: NodeJS.Signals) => void;
  readonly waitForExit: (timeoutMilliseconds: number) => Promise<boolean>;
};
type OutputDirectoryAction =
  | { readonly kind: "noop" }
  | { readonly kind: "replace" }
  | { readonly kind: "fail"; readonly message: string };
type WaitForOwnedPreviewReadinessDependencies = {
  readonly readySignal: Promise<void>;
  readonly exitSignal: Promise<number | null>;
  readonly getExitCode: () => number | null;
  readonly diagnostics: () => string;
  readonly expectedIndexHtml: string;
  readonly fetchIndexHtml: () => Promise<string>;
  readonly readyTimeoutMilliseconds: number;
};
type FinalizeDemoMediaRunDependencies = {
  readonly maybePrimaryError?: unknown;
  readonly cleanupFailures: readonly CleanupFailure[];
  readonly commitOutputs: () => Promise<void>;
};
type CommitStagedOutputsDependencies = {
  readonly compareDirectories?: typeof compareOutputDirectories;
  readonly ensureTargetParent?: (path: string) => Promise<void>;
  readonly replaceDirectory?: typeof replaceOutputDirectory;
};
type StagedOutputDirectories = {
  readonly nextDirectory: string;
  readonly targetDirectory: string;
};

function parseMode(value: string | undefined): DemoMediaMode {
  if (value === "generate" || value === "check") {
    return value;
  }

  throw new Error("usage: bun scripts/demo-media.ts <generate|check>");
}

async function main(): Promise<void> {
  const mode = parseMode(process.argv[2]);
  const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
  const webDirectory = resolve(repoRoot, "web");
  const distDirectory = resolve(webDirectory, "dist");
  const targetDirectory = resolve(repoRoot, "docs/assets/demos");
  const attemptsRoot = resolve(repoRoot, "target/demo-media");
  const maybeFailSceneId = readInjectedFailureSceneId();
  const ffmpegVersionLine = commandVersionLine("ffmpeg");
  commandVersionLine("ffprobe");
  const indexHtmlPath = resolve(distDirectory, "index.html");
  await ensureRegularFile(indexHtmlPath, "web/dist");
  const expectedIndexHtml = await readFile(indexHtmlPath, "utf8");

  const playwrightIdentity = await readPlaywrightIdentity(webDirectory);
  await ensureRegularFile(
    playwrightIdentity.executablePath,
    "Playwright Chromium executable",
  );

  const attemptDirectory = await allocateAttemptDirectory(attemptsRoot);
  const nextDirectory = resolve(attemptDirectory, "next");
  await ensureDirectory(nextDirectory);

  const previewProcess = startPreviewProcess(webDirectory);
  let maybeBrowser: Browser | undefined;
  let maybeContext: BrowserContext | undefined;
  let maybePrimaryError: unknown;
  let maybeStagedOutput: StagedOutputDirectories | undefined;

  try {
    await waitForPreview(previewProcess, PREVIEW_URL, expectedIndexHtml);
    const inputSha256 = await captureInputSha256(repoRoot);

    maybeBrowser = await chromium.launch({ headless: true });
    maybeContext = await maybeBrowser.newContext({
      baseURL: PREVIEW_ORIGIN,
      viewport: CAPTURE_PROFILE.viewport,
      deviceScaleFactor: CAPTURE_PROFILE.deviceScaleFactor,
    });
    const page = await maybeContext.newPage();
    await installSyntheticAnimationClock(page);

    const scenes: SceneMediaRecord[] = [];
    for (const plan of SCENE_CAPTURE_PLANS) {
      scenes.push(
        await encodeSceneMedia({
          nextDirectory,
          page,
          maybeFailSceneId,
        }, plan),
      );
    }

    const manifest: DemoMediaManifest = {
      schemaVersion: 1,
      captureProfile: {
        ...CAPTURE_PROFILE,
        inputSha256,
        platform: process.platform,
        playwrightPackageVersion: playwrightIdentity.packageVersion,
        chromiumRevision: playwrightIdentity.chromiumRevision,
        expectedChromiumVersion: playwrightIdentity.chromiumVersion,
        runtimeChromiumVersion: maybeBrowser.version(),
        ffmpegVersionLine,
      },
      scenes,
    };
    await writeTextAtomically(
      resolve(nextDirectory, "manifest.json"),
      canonicalManifestJson(manifest),
    );
    maybeStagedOutput = {
      nextDirectory,
      targetDirectory,
    };
  } catch (error) {
    maybePrimaryError = error;
  }

  const cleanupFailures = await runCleanupActions([
    ...(maybeContext === undefined
      ? []
      : [
          {
            label: "close browser context",
            run: async () => {
              await maybeContext.close();
            },
          } satisfies CleanupAction,
        ]),
    ...(maybeBrowser === undefined
      ? []
      : [
          {
            label: "close browser",
            run: async () => {
              await maybeBrowser.close();
            },
          } satisfies CleanupAction,
        ]),
    {
      label: "stop preview process",
      run: async () => {
        await stopPreviewProcess(previewProcess);
      },
    },
  ]);
  await finalizeDemoMediaRun({
    maybePrimaryError,
    cleanupFailures,
    commitOutputs: async () => {
      if (maybeStagedOutput === undefined) {
        throw new Error("Staged demo-media output is unavailable");
      }
      const warnings = await commitStagedOutputs(
        mode,
        maybeStagedOutput.nextDirectory,
        maybeStagedOutput.targetDirectory,
      );
      for (const warning of warnings) {
        try {
          process.emitWarning(warning);
        } catch {
          // Post-commit warning reporting must never turn a successful install into failure.
        }
      }
    },
  });
}

export function decideOutputDirectoryAction(
  mode: DemoMediaMode,
  diagnostics: readonly string[],
): OutputDirectoryAction {
  if (diagnostics.length === 0) {
    return { kind: "noop" };
  }
  if (mode === "check") {
    return {
      kind: "fail",
      message: `demo media outputs differ:\n${diagnostics.join("\n")}`,
    };
  }
  return { kind: "replace" };
}

export async function commitStagedOutputs(
  mode: DemoMediaMode,
  nextDirectory: string,
  targetDirectory: string,
  dependencies: CommitStagedOutputsDependencies = {},
): Promise<readonly string[]> {
  const compareDirectories =
    dependencies.compareDirectories ?? compareOutputDirectories;
  const ensureTargetParent = dependencies.ensureTargetParent ?? ensureDirectory;
  const replaceDirectory = dependencies.replaceDirectory ?? replaceOutputDirectory;
  const diagnostics = await compareDirectories(targetDirectory, nextDirectory);
  const action = decideOutputDirectoryAction(mode, diagnostics);
  if (action.kind === "fail") {
    throw new Error(action.message);
  }
  if (action.kind === "noop") {
    return [];
  }

  await ensureTargetParent(dirname(targetDirectory));
  return await replaceDirectory(nextDirectory, targetDirectory);
}

async function encodeSceneMedia(
  context: EncodingContext,
  plan: SceneCapturePlan,
): Promise<SceneMediaRecord> {
  const framesDirectory = resolve(context.nextDirectory, "..", "frames", plan.id);
  const mp4Path = resolve(context.nextDirectory, `${plan.id}.mp4`);
  const webpPath = resolve(context.nextDirectory, `${plan.id}.webp`);

  await captureSceneFrames({
    page: context.page,
    plan,
    framesDirectory,
  });

  if (context.maybeFailSceneId === plan.id) {
    throw new Error(`Injected demo media failure for scene ${plan.id}`);
  }

  runCommand("ffmpeg", mp4Arguments(framesDirectory, mp4Path));
  runCommand("ffmpeg", webpArguments(framesDirectory, webpPath));

  const [mp4Probe, webpProbe] = await Promise.all([
    probeMedia(mp4Path),
    probeMedia(webpPath),
  ]);
  validateMp4Probe(mp4Path, mp4Probe);
  validateWebpProbe(webpPath, webpProbe);

  const files = await Promise.all([
    captureMediaFileRecord(context.nextDirectory, mp4Path),
    captureMediaFileRecord(context.nextDirectory, webpPath),
  ]);

  return {
    id: plan.id,
    route: plan.route,
    interactionStep: plan.interactionStep,
    files,
  };
}

async function readPlaywrightIdentity(webDirectory: string): Promise<PlaywrightIdentity> {
  const packageJson = JSON.parse(
    await readFile(
      resolve(webDirectory, "node_modules/@playwright/test/package.json"),
      "utf8",
    ),
  ) as {
    version?: unknown;
  };
  const browsersJson = JSON.parse(
    await readFile(
      resolve(webDirectory, "node_modules/playwright-core/browsers.json"),
      "utf8",
    ),
  ) as {
    browsers?: Array<{
      name?: unknown;
      revision?: unknown;
      browserVersion?: unknown;
    }>;
  };
  const chromiumBrowser = browsersJson.browsers?.find(
    (browser) => browser.name === "chromium",
  );

  if (typeof packageJson.version !== "string" || packageJson.version.length === 0) {
    throw new Error("Playwright package version is unavailable");
  }
  if (
    typeof chromiumBrowser?.revision !== "string" ||
    typeof chromiumBrowser.browserVersion !== "string"
  ) {
    throw new Error("Pinned Chromium identity is unavailable");
  }

  return {
    packageVersion: packageJson.version,
    chromiumRevision: chromiumBrowser.revision,
    chromiumVersion: chromiumBrowser.browserVersion,
    executablePath: chromium.executablePath(),
  };
}

function startPreviewProcess(webDirectory: string): PreviewProcess {
  const previewProcess = spawn(
    "bun",
    ["run", "preview", "--", "--strictPort"],
    {
      cwd: webDirectory,
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  previewProcess.stdout.setEncoding("utf8");
  previewProcess.stderr.setEncoding("utf8");
  return previewProcess;
}

async function waitForPreview(
  previewProcess: PreviewProcess,
  url: string,
  expectedIndexHtml: string,
): Promise<void> {
  const monitor = createPreviewProcessMonitor(previewProcess);
  await waitForOwnedPreviewReadiness({
    readySignal: monitor.readySignal,
    exitSignal: monitor.exitSignal,
    getExitCode: () => previewProcess.exitCode,
    diagnostics: monitor.diagnostics,
    expectedIndexHtml,
    fetchIndexHtml: async () => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`preview responded with HTTP ${response.status}`);
      }
      return await response.text();
    },
    readyTimeoutMilliseconds: READY_TIMEOUT_MILLISECONDS,
  });
}

async function stopPreviewProcess(
  previewProcess: PreviewProcess,
): Promise<void> {
  await terminatePreviewProcess({
    getExitCode: () => previewProcess.exitCode,
    kill: (signal) => {
      previewProcess.kill(signal);
    },
    waitForExit: async (timeoutMilliseconds) =>
      await waitForExit(previewProcess, timeoutMilliseconds),
  });
}

function createPreviewProcessMonitor(
  processToWatch: PreviewProcess,
): {
  readonly readySignal: Promise<void>;
  readonly exitSignal: Promise<number | null>;
  readonly diagnostics: () => string;
} {
  let stdout = "";
  let stderr = "";
  let resolveReadySignal!: () => void;
  let readyResolved = false;
  const readySignal = new Promise<void>((resolvePromise) => {
    resolveReadySignal = resolvePromise;
  });
  const exitSignal = new Promise<number | null>((resolvePromise) => {
    if (processToWatch.exitCode !== null) {
      resolvePromise(processToWatch.exitCode);
      return;
    }

    processToWatch.once("exit", (exitCode) => {
      resolvePromise(exitCode);
    });
  });

  const { stdout: stdoutStream, stderr: stderrStream } = processToWatch;
  if (stdoutStream === null || stderrStream === null) {
    throw new Error("Preview process streams are unavailable");
  }

  const maybeResolveReady = (): void => {
    if (readyResolved) {
      return;
    }
    const output = `${stdout}\n${stderr}`;
    if (!hasOwnedPreviewReadySignal(output)) {
      return;
    }

    readyResolved = true;
    resolveReadySignal();
  };
  stdoutStream.on("data", (chunk: string) => {
    stdout += chunk;
    maybeResolveReady();
  });
  stderrStream.on("data", (chunk: string) => {
    stderr += chunk;
    maybeResolveReady();
  });

  return {
    readySignal,
    exitSignal,
    diagnostics: () => {
      const combined = `${stdout}\n${stderr}`.trim();
      return combined.length === 0 ? "no preview output" : combined;
    },
  };
}

async function waitForExit(
  processToWatch: PreviewProcess,
  timeoutMilliseconds: number,
): Promise<boolean> {
  const maybeExitCode = processToWatch.exitCode;
  if (maybeExitCode !== null) {
    return true;
  }

  return await new Promise<boolean>((resolvePromise) => {
    const timeout = setTimeout(() => {
      cleanup();
      resolvePromise(false);
    }, timeoutMilliseconds);

    const handleExit = (): void => {
      cleanup();
      resolvePromise(true);
    };

    const cleanup = (): void => {
      clearTimeout(timeout);
      processToWatch.off("exit", handleExit);
    };

    processToWatch.once("exit", handleExit);
  });
}

export function hasOwnedPreviewReadySignal(output: string): boolean {
  return /Local:\s+http:\/\/127\.0\.0\.1:4173(?:\/|\/liquidfun-rs\/)/.test(output);
}

export async function waitForOwnedPreviewReadiness(
  dependencies: WaitForOwnedPreviewReadinessDependencies,
): Promise<void> {
  await Promise.race([
    dependencies.readySignal,
    dependencies.exitSignal.then((exitCode) => {
      throw new Error(
        `preview process exited early (${exitCode ?? "unknown"}): ${dependencies.diagnostics()}`,
      );
    }),
    delay(dependencies.readyTimeoutMilliseconds).then(() => {
      throw new Error(`preview did not become ready: ${dependencies.diagnostics()}`);
    }),
  ]);

  const exitCodeBeforeFetch = dependencies.getExitCode();
  if (exitCodeBeforeFetch !== null) {
    throw new Error(
      `preview process exited early (${exitCodeBeforeFetch}): ${dependencies.diagnostics()}`,
    );
  }

  const servedIndexHtml = await dependencies.fetchIndexHtml();
  if (servedIndexHtml !== dependencies.expectedIndexHtml) {
    throw new Error("preview index html does not match current web/dist/index.html");
  }

  const exitCodeAfterFetch = dependencies.getExitCode();
  if (exitCodeAfterFetch !== null) {
    throw new Error(
      `preview process exited early (${exitCodeAfterFetch}): ${dependencies.diagnostics()}`,
    );
  }
}

export async function terminatePreviewProcess(
  dependencies: TerminatePreviewProcessDependencies,
): Promise<void> {
  if (dependencies.getExitCode() !== null) {
    return;
  }

  dependencies.kill("SIGTERM");
  const exitedAfterTerminate = await dependencies.waitForExit(5_000);
  if (exitedAfterTerminate) {
    return;
  }

  dependencies.kill("SIGKILL");
  const exitedAfterKill = await dependencies.waitForExit(5_000);
  if (!exitedAfterKill) {
    throw new Error("Preview process did not exit after SIGKILL");
  }
}

export async function runCleanupActions(
  actions: readonly CleanupAction[],
): Promise<readonly CleanupFailure[]> {
  const failures: CleanupFailure[] = [];

  for (const action of actions) {
    try {
      await action.run();
    } catch (error) {
      failures.push({
        label: action.label,
        error: asError(error),
      });
    }
  }

  return failures;
}

export function formatCleanupFailure(failure: CleanupFailure): string {
  return `${failure.label}: ${failure.error.message}`;
}

export function maybeRethrowWithCleanupFailures(
  maybePrimaryError: unknown,
  cleanupFailures: readonly CleanupFailure[],
): void {
  if (cleanupFailures.length === 0) {
    if (maybePrimaryError !== undefined) {
      throw maybePrimaryError;
    }
    return;
  }

  const cleanupErrors = cleanupFailures.map((failure) =>
    new Error(formatCleanupFailure(failure), { cause: failure.error }),
  );
  if (maybePrimaryError === undefined) {
    throw new AggregateError(cleanupErrors, "demo media cleanup failed");
  }

  const primaryError = asError(maybePrimaryError);
  throw new AggregateError(
    [primaryError, ...cleanupErrors],
    "demo media command failed and cleanup also failed",
    { cause: primaryError },
  );
}

export async function finalizeDemoMediaRun(
  dependencies: FinalizeDemoMediaRunDependencies,
): Promise<void> {
  if (
    dependencies.maybePrimaryError !== undefined ||
    dependencies.cleanupFailures.length > 0
  ) {
    maybeRethrowWithCleanupFailures(
      dependencies.maybePrimaryError,
      dependencies.cleanupFailures,
    );
  }

  await dependencies.commitOutputs();
}

function readInjectedFailureSceneId(): SceneCapturePlan["id"] | undefined {
  const maybeSceneId = process.env.DEMO_MEDIA_FAIL_SCENE?.trim();
  if (maybeSceneId === undefined || maybeSceneId.length === 0) {
    return undefined;
  }

  const matchingPlan = SCENE_CAPTURE_PLANS.find((plan) => plan.id === maybeSceneId);
  if (matchingPlan === undefined) {
    throw new Error(`Unknown DEMO_MEDIA_FAIL_SCENE: ${maybeSceneId}`);
  }

  return matchingPlan.id;
}

async function allocateAttemptDirectory(attemptsRoot: string): Promise<string> {
  await mkdir(attemptsRoot, { recursive: true });

  for (let attemptNumber = 1; ; attemptNumber += 1) {
    const attemptDirectory = resolve(
      attemptsRoot,
      `attempt-${String(attemptNumber).padStart(3, "0")}`,
    );
    try {
      await mkdir(attemptDirectory);
      return attemptDirectory;
    } catch (error) {
      if (isExistsError(error)) {
        continue;
      }
      throw error;
    }
  }
}

function runCommand(command: string, argumentsList: readonly string[]): void {
  const result = spawnSync(command, [...argumentsList], {
    encoding: "utf8",
  });
  if (result.status === 0) {
    return;
  }

  const diagnostic = result.stderr?.trim() ?? "";
  throw new Error(
    `command failed (${result.status}): ${command} ${argumentsList.join(" ")}${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
  );
}

function isExistsError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    error.code === "EEXIST"
  );
}

function asError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

function isDirectExecution(): boolean {
  const maybeEntryPath = process.argv[1];
  if (maybeEntryPath === undefined) {
    return false;
  }

  return resolve(maybeEntryPath) === fileURLToPath(import.meta.url);
}

if (isDirectExecution()) {
  await main();
}
