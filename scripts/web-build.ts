import {
  mkdir,
  readFile,
  readdir,
  realpath,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { basename, relative, resolve } from "node:path";

import { captureSourceIdentity } from "./phase16/source-identity";

const BUN_VERSION = "1.4.2";
const RUST_VERSION = "1.97.0";
const WASM_PACK_VERSION = "0.15.0";
const PLAYWRIGHT_VERSION = "1.63.0";
const GENERATED_RELATIVE_PATH = "web/src/generated/liquidfun-wasm";
const CLOSURE_RELATIVE_PATH = "target/phase16";

type BuildMode = "wasm" | "build" | "smoke" | "player-smoke";
type BuildStatus = "passed" | "failed";

type CommandRecord = {
  readonly command: string;
  readonly status: BuildStatus;
  readonly exitCode: number;
};

type PlaywrightIdentity = {
  readonly packageVersion: string;
  readonly chromiumRevision: string;
  readonly chromiumVersion: string;
};

const repoRoot = resolve(import.meta.dir, "..");
const webDirectory = resolve(repoRoot, "web");
const generatedDirectory = resolve(repoRoot, GENERATED_RELATIVE_PATH);
const summaryDirectory = resolve(repoRoot, "target/web-build");
const closureDirectory = resolve(repoRoot, CLOSURE_RELATIVE_PATH);
let logPath = resolve(summaryDirectory, "web-build.log");
const commandRecords: CommandRecord[] = [];

function parseMode(value: string | undefined): BuildMode {
  if (
    value === "wasm" ||
    value === "build" ||
    value === "smoke" ||
    value === "player-smoke"
  ) {
    return value;
  }

  throw new Error(
    "usage: bun scripts/web-build.ts <wasm|build|smoke|player-smoke>",
  );
}

function commandText(command: readonly string[]): string {
  return command.join(" ");
}

async function breadcrumb(message: string): Promise<void> {
  const line = `[web-build] ${message}`;
  console.log(line);
  await Bun.write(logPath, `${await Bun.file(logPath).text()}${line}\n`);
}

function commandError(command: string, exitCode: number): Error {
  return Object.assign(
    new Error(`command failed (${exitCode}): ${command}`),
    { exitCode },
  );
}

async function runCommand(
  command: readonly string[],
  cwd: string,
  environment?: Record<string, string | undefined>,
): Promise<void> {
  const displayCommand = commandText(command);
  await breadcrumb(`run ${displayCommand}`);

  const result = Bun.spawnSync({
    cmd: [...command],
    cwd,
    ...(environment === undefined ? {} : { env: environment }),
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = result.exitCode === 0 ? "passed" : "failed";
  commandRecords.push({
    command: displayCommand,
    status,
    exitCode: result.exitCode,
  });

  if (result.exitCode !== 0) {
    throw commandError(displayCommand, result.exitCode);
  }
}

function captureCommand(command: readonly string[]): string {
  const displayCommand = commandText(command);
  const result = Bun.spawnSync({
    cmd: [...command],
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const status = result.exitCode === 0 ? "passed" : "failed";
  commandRecords.push({
    command: displayCommand,
    status,
    exitCode: result.exitCode,
  });

  if (result.exitCode !== 0) {
    const diagnostic = result.stderr.toString().trim();
    throw commandError(`${displayCommand}: ${diagnostic}`, result.exitCode);
  }

  return result.stdout.toString().trim();
}

function verifyTools(): void {
  if (process.versions.bun !== BUN_VERSION) {
    throw new Error(
      `Bun ${BUN_VERSION} is required; found ${process.versions.bun ?? "unknown"}`,
    );
  }

  const rustVersion = captureCommand(["rustc", "--version"]);
  if (!rustVersion.startsWith(`rustc ${RUST_VERSION} `)) {
    throw new Error(`rustc ${RUST_VERSION} is required; found ${rustVersion}`);
  }

  const wasmPackVersion = captureCommand(["wasm-pack", "--version"]);
  if (wasmPackVersion !== `wasm-pack ${WASM_PACK_VERSION}`) {
    throw new Error(
      `wasm-pack ${WASM_PACK_VERSION} is required; found ${wasmPackVersion}`,
    );
  }
}

function validateGeneratedDirectory(): void {
  const relativePath = relative(repoRoot, generatedDirectory);
  if (relativePath !== GENERATED_RELATIVE_PATH) {
    throw new Error(`refusing to remove non-canonical generated path: ${relativePath}`);
  }
}

async function regenerateWasm(): Promise<void> {
  validateGeneratedDirectory();
  await breadcrumb(`remove ${GENERATED_RELATIVE_PATH}`);
  await rm(generatedDirectory, { recursive: true, force: true });
  await runCommand(
    [
      "wasm-pack",
      "build",
      "crates/liquidfun-wasm",
      "--target",
      "web",
      "--release",
      "--out-dir",
      "../../web/src/generated/liquidfun-wasm",
      "--out-name",
      "liquidfun_wasm",
    ],
    repoRoot,
  );
}

function maybeProcessEnv(name: string): string | undefined {
  const maybeValue = process.env[name];
  if (maybeValue === undefined) {
    return undefined;
  }

  const trimmed = maybeValue.trim();
  return trimmed.length === 0 ? undefined : trimmed;
}

async function collectViteProvenanceEnv(): Promise<Record<string, string>> {
  const packageJson = JSON.parse(
    await readFile(resolve(webDirectory, "package.json"), "utf8"),
  ) as { version?: unknown };
  if (typeof packageJson.version !== "string" || packageJson.version.length === 0) {
    throw new Error("web/package.json is missing a version");
  }

  const gitSha =
    maybeProcessEnv("GITHUB_SHA") ?? captureCommand(["git", "rev-parse", "HEAD"]);
  const buildId = maybeProcessEnv("GITHUB_RUN_ID") ?? new Date().toISOString();
  const maybeServerUrl = maybeProcessEnv("GITHUB_SERVER_URL");
  const maybeRepository = maybeProcessEnv("GITHUB_REPOSITORY");
  const maybeRunId = maybeProcessEnv("GITHUB_RUN_ID");
  const maybeBuildUrl =
    maybeServerUrl === undefined ||
    maybeRepository === undefined ||
    maybeRunId === undefined
      ? undefined
      : `${maybeServerUrl}/${maybeRepository}/actions/runs/${maybeRunId}`;

  return {
    VITE_APP_VERSION: packageJson.version,
    VITE_GIT_SHA: gitSha,
    VITE_BUILD_ID: buildId,
    ...(maybeBuildUrl === undefined ? {} : { VITE_BUILD_URL: maybeBuildUrl }),
  };
}

async function assertProductionAssetPaths(): Promise<void> {
  const validationName = "assert production dist asset paths";
  const distDirectory = resolve(webDirectory, "dist");
  try {
    const indexHtml = await readFile(resolve(distDirectory, "index.html"), "utf8");
    if (!indexHtml.includes("/liquidfun-rs/assets/")) {
      throw new Error("production dist is missing /liquidfun-rs/assets/");
    }

    const entries = await readdir(distDirectory, { recursive: true });
    const hasWasm = entries.some((entry) => entry.endsWith(".wasm"));
    if (!hasWasm) {
      throw new Error("production dist is missing a .wasm asset");
    }

    commandRecords.push({
      command: validationName,
      status: "passed",
      exitCode: 0,
    });
    await breadcrumb(`${validationName} passed`);
  } catch (error) {
    commandRecords.push({
      command: validationName,
      status: "failed",
      exitCode: 1,
    });
    throw error;
  }
}

async function runFrontendBuild(): Promise<void> {
  await runCommand(["bun", "install", "--frozen-lockfile"], webDirectory);
  await runCommand(["bun", "run", "typecheck"], webDirectory);
  await runCommand(["bun", "run", "test:unit"], webDirectory);
  const provenanceEnv = await collectViteProvenanceEnv();
  await breadcrumb(
    `inject VITE_APP_VERSION=${provenanceEnv.VITE_APP_VERSION} VITE_GIT_SHA=${provenanceEnv.VITE_GIT_SHA} VITE_BUILD_ID=${provenanceEnv.VITE_BUILD_ID}`,
  );
  await runCommand(["bun", "run", "build:app"], webDirectory, {
    ...process.env,
    ...provenanceEnv,
  });
  await assertProductionAssetPaths();
}

async function runCompleteBuild(): Promise<void> {
  verifyTools();
  await regenerateWasm();
  await runFrontendBuild();
}

async function allocateClosureAttempt(): Promise<string> {
  await mkdir(closureDirectory, { recursive: true });

  for (let attemptNumber = 1; ; attemptNumber += 1) {
    const attemptDirectory = resolve(
      closureDirectory,
      `closure-attempt-${attemptNumber}`,
    );
    try {
      await mkdir(attemptDirectory);
      return await realpath(attemptDirectory);
    } catch (error) {
      if (
        error instanceof Error &&
        "code" in error &&
        error.code === "EEXIST"
      ) {
        continue;
      }
      throw error;
    }
  }
}

async function atomicWriteJson(
  destination: string,
  value: unknown,
): Promise<void> {
  const temporaryPath = `${destination}.tmp-${process.pid}`;
  await writeFile(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, {
    flag: "wx",
  });
  await rename(temporaryPath, destination);
}

async function parsePlaywrightIdentity(): Promise<PlaywrightIdentity> {
  const packagePath = resolve(
    webDirectory,
    "node_modules/@playwright/test/package.json",
  );
  const browsersPath = resolve(
    webDirectory,
    "node_modules/playwright-core/browsers.json",
  );
  const packageJson = JSON.parse(await readFile(packagePath, "utf8")) as {
    version?: unknown;
  };
  const browsersJson = JSON.parse(await readFile(browsersPath, "utf8")) as {
    browsers?: Array<{
      name?: unknown;
      revision?: unknown;
      browserVersion?: unknown;
    }>;
  };
  if (packageJson.version !== PLAYWRIGHT_VERSION) {
    throw new Error(
      `@playwright/test ${PLAYWRIGHT_VERSION} is required; found ${String(packageJson.version)}`,
    );
  }

  const chromium = browsersJson.browsers?.find(
    (browser) => browser.name === "chromium",
  );
  if (
    typeof chromium?.revision !== "string" ||
    typeof chromium.browserVersion !== "string"
  ) {
    throw new Error("package-pinned Chromium identity is unavailable");
  }

  return {
    packageVersion: PLAYWRIGHT_VERSION,
    chromiumRevision: chromium.revision,
    chromiumVersion: chromium.browserVersion,
  };
}

async function runSmoke(attemptDirectory: string): Promise<void> {
  await runCompleteBuild();
  const playwright = await parsePlaywrightIdentity();
  const source = await captureSourceIdentity(repoRoot);
  await atomicWriteJson(resolve(attemptDirectory, "provenance.json"), {
    schemaVersion: 1,
    attemptIdentity: basename(attemptDirectory),
    source,
    tools: {
      bun: BUN_VERSION,
      rust: RUST_VERSION,
      wasmPack: WASM_PACK_VERSION,
      playwright,
    },
  });

  const environment = {
    ...process.env,
    PHASE16_CLOSURE_ATTEMPT_DIR: attemptDirectory,
  };
  await runCommand(
    ["bun", "run", "browser:install"],
    webDirectory,
    environment,
  );
  await runCommand(["bun", "run", "test:browser"], webDirectory, environment);
  await validatePlaywrightAttachments(attemptDirectory);
}

function collectAttachmentNames(
  value: unknown,
  names: Set<string>,
): void {
  if (Array.isArray(value)) {
    for (const item of value) {
      collectAttachmentNames(item, names);
    }
    return;
  }
  if (value === null || typeof value !== "object") {
    return;
  }

  const record = value as Record<string, unknown>;
  if (
    typeof record.name === "string" &&
    typeof record.contentType === "string"
  ) {
    names.add(record.name);
  }
  for (const child of Object.values(record)) {
    collectAttachmentNames(child, names);
  }
}

async function validatePlaywrightAttachments(
  attemptDirectory: string,
): Promise<void> {
  const validationName = "validate Playwright passing attachments";
  const reportPath = resolve(attemptDirectory, "playwright-report.json");
  try {
    const report = JSON.parse(await readFile(reportPath, "utf8")) as unknown;
    const names = new Set<string>();
    collectAttachmentNames(report, names);
    for (const requiredName of [
      "canvas-initial.png",
      "canvas-moving.png",
      "canvas-disposed.png",
      "browser-proof.json",
    ]) {
      if (!names.has(requiredName)) {
        throw new Error(`missing passing attachment: ${requiredName}`);
      }
    }
    commandRecords.push({
      command: validationName,
      status: "passed",
      exitCode: 0,
    });
    await breadcrumb(`${validationName} passed`);
  } catch (error) {
    commandRecords.push({
      command: validationName,
      status: "failed",
      exitCode: 1,
    });
    throw error;
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "unknown web build failure";
}

function errorExitCode(error: unknown): number {
  if (
    error instanceof Error &&
    "exitCode" in error &&
    typeof error.exitCode === "number"
  ) {
    return error.exitCode;
  }

  return 1;
}

async function writeSummary(
  mode: BuildMode,
  status: BuildStatus,
  startedAt: string,
  maybeAttemptDirectory?: string,
  maybeError?: unknown,
): Promise<void> {
  const completedAt = new Date().toISOString();
  const summary = {
    mode,
    status,
    startedAt,
    completedAt,
    generatedDirectory: GENERATED_RELATIVE_PATH,
    commands: commandRecords,
    ...(maybeAttemptDirectory === undefined
      ? {}
      : {
          attemptDirectory: relative(repoRoot, maybeAttemptDirectory),
        }),
    ...(maybeError === undefined ? {} : { error: errorMessage(maybeError) }),
  };
  const text = [
    `mode: ${mode}`,
    `status: ${status}`,
    `started: ${startedAt}`,
    `completed: ${completedAt}`,
    `generated: ${GENERATED_RELATIVE_PATH}`,
    ...(maybeAttemptDirectory === undefined
      ? []
      : [`attempt: ${relative(repoRoot, maybeAttemptDirectory)}`]),
    ...(maybeError === undefined ? [] : [`error: ${errorMessage(maybeError)}`]),
    "",
  ].join("\n");

  await atomicWriteJson(resolve(summaryDirectory, "summary.json"), summary);
  await writeFile(resolve(summaryDirectory, "summary.txt"), text);

  if (maybeAttemptDirectory !== undefined) {
    await atomicWriteJson(
      resolve(maybeAttemptDirectory, "smoke-summary.json"),
      summary,
    );
  }
}

async function main(): Promise<void> {
  const mode = parseMode(process.argv[2]);
  const startedAt = new Date().toISOString();
  let maybeAttemptDirectory: string | undefined;
  await mkdir(summaryDirectory, { recursive: true });
  if (mode === "smoke") {
    maybeAttemptDirectory = await allocateClosureAttempt();
    logPath = resolve(maybeAttemptDirectory, "smoke.log");
    await writeFile(logPath, "", { flag: "wx" });
  } else {
    await writeFile(logPath, "");
  }

  try {
    await breadcrumb(`start ${mode}`);
    if (mode === "smoke" && maybeAttemptDirectory !== undefined) {
      await breadcrumb(
        `retain ${relative(repoRoot, maybeAttemptDirectory)}`,
      );
      await runSmoke(maybeAttemptDirectory);
    } else {
      verifyTools();
      await regenerateWasm();
      if (mode === "build" || mode === "player-smoke") {
        await runFrontendBuild();
      }
      if (mode === "player-smoke") {
        await runCommand(["bun", "run", "browser:install"], webDirectory);
        await runCommand(["bun", "run", "test:player"], webDirectory);
      }
    }
    await writeSummary(
      mode,
      "passed",
      startedAt,
      maybeAttemptDirectory,
    );
    await breadcrumb(`complete ${mode}`);
  } catch (error) {
    await writeSummary(
      mode,
      "failed",
      startedAt,
      maybeAttemptDirectory,
      error,
    );
    console.error(`[web-build] ${errorMessage(error)}`);
    process.exitCode = errorExitCode(error);
  }
}

await main();
