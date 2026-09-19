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
  readonly repoRoot: string;
  readonly nextDirectory: string;
  readonly page: Page;
  readonly maybeFailSceneId: SceneCapturePlan["id"] | undefined;
};

type PreviewProcess = ReturnType<typeof spawn>;

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
          repoRoot,
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
    if (mode === "check") {
      if (diagnostics.length > 0) {
        throw new Error(
          `demo media outputs differ:\n${diagnostics.join("\n")}`,
        );
      }
      return;
    }

    if (diagnostics.length === 0) {
      return;
    }

    await ensureDirectory(dirname(targetDirectory));
    await replaceOutputDirectory(nextDirectory, targetDirectory);
  } finally {
    await Promise.allSettled([
      maybeContext?.close(),
      maybeBrowser?.close(),
    ]);
    await stopPreviewProcess(previewProcess);
  }
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
  const deadline = Date.now() + READY_TIMEOUT_MILLISECONDS;

  while (Date.now() < deadline) {
    if (previewProcess.exitCode !== null) {
      throw new Error(
        `preview process exited early (${previewProcess.exitCode}): ${output()}`,
      );
    }

    try {
      const response = await fetch(url);
      if (response.ok) {
        return;
      }
    } catch {
      // Keep polling until the timeout expires.
    }

    await delay(250);
  }

  throw new Error(`preview did not become ready: ${output()}`);
}

async function stopPreviewProcess(
  previewProcess: PreviewProcess,
): Promise<void> {
  if (previewProcess.exitCode !== null) {
    return;
  }

  previewProcess.kill("SIGTERM");
  const exited = await waitForExit(previewProcess, 5_000);
  if (exited) {
    return;
  }

  previewProcess.kill("SIGKILL");
  await waitForExit(previewProcess, 5_000);
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

await main();
