import { mkdir, rm, writeFile } from "node:fs/promises";
import { relative, resolve } from "node:path";

const BUN_VERSION = "1.4.2";
const RUST_VERSION = "1.97.0";
const WASM_PACK_VERSION = "0.15.0";
const GENERATED_RELATIVE_PATH = "web/src/generated/liquidfun-wasm";

type BuildMode = "wasm" | "build";
type BuildStatus = "passed" | "failed";

type CommandRecord = {
  readonly command: string;
  readonly status: BuildStatus;
  readonly exitCode: number;
};

const repoRoot = resolve(import.meta.dir, "..");
const webDirectory = resolve(repoRoot, "web");
const generatedDirectory = resolve(repoRoot, GENERATED_RELATIVE_PATH);
const summaryDirectory = resolve(repoRoot, "target/web-build");
const logPath = resolve(summaryDirectory, "web-build.log");
const commandRecords: CommandRecord[] = [];

function parseMode(value: string | undefined): BuildMode {
  if (value === "wasm" || value === "build") {
    return value;
  }

  throw new Error("usage: bun scripts/web-build.ts <wasm|build>");
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
): Promise<void> {
  const displayCommand = commandText(command);
  await breadcrumb(`run ${displayCommand}`);

  const result = Bun.spawnSync({
    cmd: [...command],
    cwd,
    stdout: "inherit",
    stderr: "inherit",
  });
  const status = result.exitCode === 0 ? "passed" : "failed";
  commandRecords.push({ command: displayCommand, status, exitCode: result.exitCode });

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
  commandRecords.push({ command: displayCommand, status, exitCode: result.exitCode });

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

async function runFrontendBuild(): Promise<void> {
  await runCommand(["bun", "install", "--frozen-lockfile"], webDirectory);
  await runCommand(["bun", "run", "typecheck"], webDirectory);
  await runCommand(["bun", "run", "test:unit"], webDirectory);
  await runCommand(["bun", "run", "build:app"], webDirectory);
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
    ...(maybeError === undefined ? {} : { error: errorMessage(maybeError) }),
  };
  const text = [
    `mode: ${mode}`,
    `status: ${status}`,
    `started: ${startedAt}`,
    `completed: ${completedAt}`,
    `generated: ${GENERATED_RELATIVE_PATH}`,
    ...(maybeError === undefined ? [] : [`error: ${errorMessage(maybeError)}`]),
    "",
  ].join("\n");

  await writeFile(
    resolve(summaryDirectory, "summary.json"),
    `${JSON.stringify(summary, null, 2)}\n`,
  );
  await writeFile(resolve(summaryDirectory, "summary.txt"), text);
}

async function main(): Promise<void> {
  const mode = parseMode(process.argv[2]);
  const startedAt = new Date().toISOString();
  await mkdir(summaryDirectory, { recursive: true });
  await writeFile(logPath, "");

  try {
    await breadcrumb(`start ${mode}`);
    verifyTools();
    await regenerateWasm();
    if (mode === "build") {
      await runFrontendBuild();
    }
    await writeSummary(mode, "passed", startedAt);
    await breadcrumb(`complete ${mode}`);
  } catch (error) {
    await writeSummary(mode, "failed", startedAt, error);
    console.error(`[web-build] ${errorMessage(error)}`);
    process.exitCode = errorExitCode(error);
  }
}

await main();
