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
type WaitForPreviewReadinessDependencies = {
  readonly isReady: () => Promise<boolean>;
  readonly getExitCode: () => number | null;
  readonly diagnostics: () => string;
  readonly delayMilliseconds: number;
  readonly delay: (milliseconds: number) => Promise<void>;
  readonly now: () => number;
  readonly timeoutAt: number;
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
  await ensureRegularFile(resolve(distDirectory, "index.html"), "web/dist");

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

  try {
    await waitForPreview(previewProcess, PREVIEW_URL);
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

    const diagnostics = await compareOutputDirectories(targetDirectory, nextDirectory);
    const action = decideOutputDirectoryAction(mode, diagnostics);
    if (action.kind === "fail") {
      throw new Error(action.message);
    }
    if (action.kind === "replace") {
      await ensureDirectory(dirname(targetDirectory));
      await replaceOutputDirectory(nextDirectory, targetDirectory);
    }
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
  maybeRethrowWithCleanupFailures(maybePrimaryError, cleanupFailures);
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
): Promise<void> {
  const output = collectProcessOutput(previewProcess);
  await waitForPreviewReadiness({
    isReady: async () => {
      try {
        const response = await fetch(url);
        return response.ok;
      } catch {
        return false;
      }
    },
    getExitCode: () => previewProcess.exitCode,
    diagnostics: output,
    delayMilliseconds: 250,
    delay,
    now: Date.now,
    timeoutAt: Date.now() + READY_TIMEOUT_MILLISECONDS,
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

function collectProcessOutput(
  processToWatch: PreviewProcess,
): () => string {
  let stdout = "";
  let stderr = "";

  const { stdout: stdoutStream, stderr: stderrStream } = processToWatch;
  if (stdoutStream === null || stderrStream === null) {
    throw new Error("Preview process streams are unavailable");
  }

  stdoutStream.on("data", (chunk: string) => {
    stdout += chunk;
  });
  stderrStream.on("data", (chunk: string) => {
    stderr += chunk;
  });

  return () => {
    const combined = `${stdout}\n${stderr}`.trim();
    return combined.length === 0 ? "no preview output" : combined;
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

export async function waitForPreviewReadiness(
  dependencies: WaitForPreviewReadinessDependencies,
): Promise<void> {
  while (dependencies.now() < dependencies.timeoutAt) {
    const exitCodeBeforeCheck = dependencies.getExitCode();
    if (exitCodeBeforeCheck !== null) {
      throw new Error(
        `preview process exited early (${exitCodeBeforeCheck}): ${dependencies.diagnostics()}`,
      );
    }

    const isReady = await dependencies.isReady();
    const exitCodeAfterCheck = dependencies.getExitCode();
    if (exitCodeAfterCheck !== null) {
      throw new Error(
        `preview process exited early (${exitCodeAfterCheck}): ${dependencies.diagnostics()}`,
      );
    }
    if (isReady) {
      return;
    }

    await dependencies.delay(dependencies.delayMilliseconds);
  }

  throw new Error(`preview did not become ready: ${dependencies.diagnostics()}`);
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
