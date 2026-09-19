import { spawnSync } from "node:child_process";
import { mkdir, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
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
import {
  startPreviewProcess,
  stopPreviewProcess,
  waitForPreview,
} from "./demo-media/preview";

const PREVIEW_URL = "http://127.0.0.1:4173/liquidfun-rs/";
const PREVIEW_ORIGIN = "http://127.0.0.1:4173";
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

type CleanupAction = {
  readonly label: string;
  readonly run: () => Promise<void>;
};
type CleanupFailure = {
  readonly label: string;
  readonly error: Error;
};
type OutputDirectoryAction =
  | { readonly kind: "noop" }
  | { readonly kind: "replace" }
  | { readonly kind: "fail"; readonly message: string };
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
