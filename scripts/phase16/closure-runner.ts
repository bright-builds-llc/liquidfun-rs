import { createHash } from "node:crypto";
import {
  link,
  mkdir,
  readFile,
  realpath,
  unlink,
  writeFile,
} from "node:fs/promises";
import { basename, relative, resolve } from "node:path";

import {
  validateBrowserEvidence,
  type BrowserProof,
} from "./browser-evidence";
import {
  captureSourceIdentity,
  parseSourceIdentity,
  requireMatchingSourceIdentity,
  type SourceIdentity,
} from "./source-identity";

export type CommandSpec = {
  readonly id: string;
  readonly argv: readonly string[];
};

type CommandResult = {
  readonly id: string;
  readonly command: string;
  readonly status: "passed" | "failed";
  readonly exitCode: number;
  readonly stdoutLog: string;
  readonly stderrLog: string;
};

type IsolationResult = {
  readonly defaultMember: string;
  readonly normalDependencies: readonly string[];
  readonly packageFileCount: number;
  readonly forbiddenPackageEntries: readonly string[];
};

const MAX_FAILURE_SUMMARY_BYTES = 128_000;
const MAX_SUMMARY_BYTES = 2_000_000;
const MAX_FAILURE_ERROR_LENGTH = 4_096;
const MAX_FAILURE_STATUS_LENGTH = 16_384;

function sha256(bytes: Uint8Array | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function commandText(argv: readonly string[]): string {
  return argv.join(" ");
}

function boundedText(value: string, maximumLength: number): string {
  if (value.length <= maximumLength) {
    return value;
  }
  return `${value.slice(0, maximumLength)}…`;
}

function errorMessage(error: unknown): string {
  const message =
    error instanceof Error ? error.message : "unknown closure failure";
  return boundedText(message, MAX_FAILURE_ERROR_LENGTH);
}

async function writeJsonIdentityLast(
  path: string,
  value: unknown,
  maximumBytes: number,
): Promise<void> {
  const bytes = Buffer.from(`${JSON.stringify(value, null, 2)}\n`);
  if (bytes.length > maximumBytes) {
    throw new Error(`closure summary exceeds ${maximumBytes} bytes`);
  }
  const temporaryPath = `${path}.tmp-${process.pid}`;
  await writeFile(temporaryPath, bytes, { flag: "wx" });
  try {
    await link(temporaryPath, path);
  } finally {
    await unlink(temporaryPath).catch(() => undefined);
  }
}

async function runCommand(
  repoRoot: string,
  spec: CommandSpec,
  index: number,
  logsDirectory: string,
): Promise<{ readonly result: CommandResult; readonly stdout: string }> {
  console.log(`[phase16-closure] run ${commandText(spec.argv)}`);
  const processResult = Bun.spawn({
    cmd: [...spec.argv],
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(processResult.stdout).text(),
    new Response(processResult.stderr).text(),
    processResult.exited,
  ]);
  process.stdout.write(stdout);
  process.stderr.write(stderr);

  const prefix = `${String(index + 1).padStart(2, "0")}-${spec.id}`;
  const stdoutPath = resolve(logsDirectory, `${prefix}.stdout.log`);
  const stderrPath = resolve(logsDirectory, `${prefix}.stderr.log`);
  await writeFile(stdoutPath, stdout, { flag: "wx" });
  await writeFile(stderrPath, stderr, { flag: "wx" });
  const status = exitCode === 0 ? "passed" : "failed";
  return {
    result: {
      id: spec.id,
      command: commandText(spec.argv),
      status,
      exitCode,
      stdoutLog: relative(repoRoot, stdoutPath),
      stderrLog: relative(repoRoot, stderrPath),
    },
    stdout,
  };
}

async function readProvenanceSource(
  attemptDirectory: string,
): Promise<SourceIdentity> {
  const provenanceValue = JSON.parse(
    await readFile(resolve(attemptDirectory, "provenance.json"), "utf8"),
  ) as unknown;
  if (
    provenanceValue === null ||
    typeof provenanceValue !== "object" ||
    Array.isArray(provenanceValue)
  ) {
    throw new Error("provenance must be an object");
  }
  return parseSourceIdentity(
    (provenanceValue as Record<string, unknown>).source,
    "provenance source",
  );
}

function validateIsolation(
  outputs: ReadonlyMap<string, string>,
): IsolationResult {
  const tree = outputs.get("native-tree");
  const metadataText = outputs.get("cargo-metadata");
  const packageList = outputs.get("package-list");
  if (
    tree === undefined ||
    metadataText === undefined ||
    packageList === undefined
  ) {
    throw new Error("isolation command output is incomplete");
  }

  const forbiddenTreeTerms = [
    "wasm-bindgen",
    "solid",
    "vite",
    "playwright",
    "renderer",
    "protocol",
    "differential",
    "benchmark",
    "testbed",
  ];
  const loweredTree = tree.toLowerCase();
  for (const term of forbiddenTreeTerms) {
    if (loweredTree.includes(term)) {
      throw new Error(`native dependency tree contains ${term}`);
    }
  }

  const metadata = JSON.parse(metadataText) as {
    packages: Array<{
      id: string;
      name: string;
      dependencies: Array<{ name: string }>;
    }>;
    workspace_default_members: string[];
  };
  if (metadata.workspace_default_members.length !== 1) {
    throw new Error("workspace must have exactly one default member");
  }
  const defaultPackage = metadata.packages.find(
    (item) => item.id === metadata.workspace_default_members[0],
  );
  if (defaultPackage?.name !== "liquidfun") {
    throw new Error("liquidfun must be the sole default member");
  }

  const packageEntries = packageList
    .split("\n")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
  const forbiddenPackageTerms = [
    "crates/liquidfun-wasm",
    "web/",
    "node_modules",
    "chromium",
    "third_party",
    "liquidfun-test-protocol",
    "liquidfun-differential",
    "liquidfun-benchmarks",
    "liquidfun-testbed",
    "generated",
  ];
  const forbiddenPackageEntries = packageEntries.filter((entry) => {
    const loweredEntry = entry.toLowerCase();
    return (
      /\.(?:js|d\.ts|wasm)$/.test(loweredEntry) ||
      forbiddenPackageTerms.some((term) =>
        loweredEntry.includes(term.toLowerCase()),
      )
    );
  });
  if (forbiddenPackageEntries.length > 0) {
    throw new Error(
      `packaged crate contains forbidden entries: ${forbiddenPackageEntries.join(", ")}`,
    );
  }

  return {
    defaultMember: defaultPackage.name,
    normalDependencies: defaultPackage.dependencies.map(
      (dependency) => dependency.name,
    ),
    packageFileCount: packageEntries.length,
    forbiddenPackageEntries,
  };
}

function boundedFailureSource(
  source: SourceIdentity,
): SourceIdentity & { readonly statusTruncated?: true } {
  if (source.status.length <= MAX_FAILURE_STATUS_LENGTH) {
    return source;
  }
  return {
    revision: source.revision,
    workingTreeSha256: source.workingTreeSha256,
    status: boundedText(source.status, MAX_FAILURE_STATUS_LENGTH),
    statusTruncated: true,
  };
}

async function writeFailureSummary(
  summaryPath: string,
  startedAt: string,
  attemptDirectory: string,
  results: readonly CommandResult[],
  error: unknown,
  maybeSource: SourceIdentity | undefined,
  maybeBrowserProof: BrowserProof | undefined,
): Promise<void> {
  await writeJsonIdentityLast(
    summaryPath,
    {
      schemaVersion: 1,
      status: "failed",
      startedAt,
      completedAt: new Date().toISOString(),
      attemptIdentity: basename(attemptDirectory),
      ...(maybeSource === undefined
        ? {}
        : { source: boundedFailureSource(maybeSource) }),
      ...(maybeBrowserProof === undefined
        ? {}
        : { browserProof: { artifacts: maybeBrowserProof.artifacts } }),
      commands: results,
      error: errorMessage(error),
    },
    MAX_FAILURE_SUMMARY_BYTES,
  );
}

export async function runClosureAttempt(
  repoRoot: string,
  requestedAttemptDirectory: string,
  commands: readonly CommandSpec[],
): Promise<void> {
  const startedAt = new Date().toISOString();
  let maybeAttemptDirectory: string | undefined;
  let maybeSource: SourceIdentity | undefined;
  let maybeBrowserProof: BrowserProof | undefined;
  const results: CommandResult[] = [];
  const outputs = new Map<string, string>();

  try {
    maybeAttemptDirectory = await realpath(requestedAttemptDirectory);
    const closureDirectory = resolve(maybeAttemptDirectory, "closure");
    const logsDirectory = resolve(closureDirectory, "logs");
    await mkdir(closureDirectory);
    await mkdir(logsDirectory);

    maybeSource = await captureSourceIdentity(repoRoot);
    maybeBrowserProof = await validateBrowserEvidence(
      repoRoot,
      maybeAttemptDirectory,
    );
    const provenanceSource = await readProvenanceSource(
      maybeAttemptDirectory,
    );
    requireMatchingSourceIdentity(
      maybeSource,
      maybeBrowserProof.source,
      "browser proof",
    );
    requireMatchingSourceIdentity(
      maybeSource,
      provenanceSource,
      "provenance",
    );

    for (const [index, spec] of commands.entries()) {
      const execution = await runCommand(
        repoRoot,
        spec,
        index,
        logsDirectory,
      );
      results.push(execution.result);
      outputs.set(spec.id, execution.stdout);
      if (execution.result.status === "failed") {
        throw new Error(`closure command failed: ${execution.result.command}`);
      }
    }
    const isolation = validateIsolation(outputs);
    const completedSource = await captureSourceIdentity(repoRoot);
    requireMatchingSourceIdentity(
      maybeSource,
      completedSource,
      "post-command checkout",
    );
    const proofPath = resolve(
      maybeAttemptDirectory,
      "browser/browser-proof.json",
    );
    await writeJsonIdentityLast(
      resolve(closureDirectory, "closure-summary.json"),
      {
        schemaVersion: 1,
        status: "passed",
        startedAt,
        completedAt: new Date().toISOString(),
        attemptIdentity: basename(maybeAttemptDirectory),
        source: maybeSource,
        browserProof: {
          path: relative(repoRoot, proofPath),
          sha256: sha256(await readFile(proofPath)),
          artifacts: maybeBrowserProof.artifacts,
        },
        commands: results,
        isolation,
      },
      MAX_SUMMARY_BYTES,
    );
    console.log(
      `[phase16-closure] passed ${relative(repoRoot, maybeAttemptDirectory)}`,
    );
  } catch (error) {
    if (maybeAttemptDirectory !== undefined) {
      const summaryPath = resolve(
        maybeAttemptDirectory,
        "closure/closure-summary.json",
      );
      try {
        await writeFailureSummary(
          summaryPath,
          startedAt,
          maybeAttemptDirectory,
          results,
          error,
          maybeSource,
          maybeBrowserProof,
        );
      } catch (summaryError) {
        console.error(
          `[phase16-closure] could not retain failure summary: ${errorMessage(summaryError)}`,
        );
      }
    }
    throw error;
  }
}
