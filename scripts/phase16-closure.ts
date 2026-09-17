import { createHash } from "node:crypto";
import {
  mkdir,
  readFile,
  realpath,
  rename,
  writeFile,
} from "node:fs/promises";
import { basename, relative, resolve } from "node:path";

type CommandSpec = {
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

type BrowserProof = {
  readonly attemptIdentity: string;
  readonly assertions: Record<string, boolean>;
  readonly artifacts: Record<
    string,
    {
      readonly path: string;
      readonly sha256: string;
      readonly byteLength: number;
      readonly dimensions: {
        readonly width: number;
        readonly height: number;
      };
    }
  >;
};

const commands: readonly CommandSpec[] = [
  {
    id: "cargo-fmt",
    argv: ["cargo", "fmt", "--all", "--check"],
  },
  {
    id: "wasm-tests",
    argv: ["cargo", "test", "-p", "liquidfun-wasm"],
  },
  {
    id: "wasm-clippy",
    argv: [
      "cargo",
      "clippy",
      "-p",
      "liquidfun-wasm",
      "--all-targets",
      "--",
      "-D",
      "warnings",
    ],
  },
  {
    id: "web-build",
    argv: ["just", "web-build"],
  },
  {
    id: "native-build",
    argv: ["cargo", "build", "-p", "liquidfun"],
  },
  {
    id: "native-tests",
    argv: [
      "cargo",
      "test",
      "-p",
      "liquidfun",
      "--all-features",
    ],
  },
  {
    id: "default-build",
    argv: ["cargo", "build"],
  },
  {
    id: "package-verify",
    argv: ["cargo", "xtask", "package", "verify"],
  },
  {
    id: "aggregate-check",
    argv: ["just", "check"],
  },
  {
    id: "markdown-check",
    argv: ["just", "markdown-check"],
  },
  {
    id: "bright-builds",
    argv: ["bun", "scripts/bright-builds-check.ts", "all"],
  },
  {
    id: "git-diff-check",
    argv: ["git", "diff", "--check"],
  },
  {
    id: "native-tree",
    argv: ["cargo", "tree", "-p", "liquidfun", "--edges", "normal"],
  },
  {
    id: "cargo-metadata",
    argv: ["cargo", "metadata", "--format-version", "1", "--no-deps"],
  },
  {
    id: "package-list",
    argv: [
      "cargo",
      "package",
      "-p",
      "liquidfun",
      "--list",
      "--allow-dirty",
    ],
  },
];

const requiredAttachments = [
  "canvas-initial.png",
  "canvas-moving.png",
  "canvas-disposed.png",
  "browser-proof.json",
] as const;
const repoRoot = resolve(import.meta.dir, "..");

function sha256(bytes: Uint8Array | string): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function commandText(argv: readonly string[]): string {
  return argv.join(" ");
}

function requireAttemptPath(maybePath: string | undefined): string {
  if (maybePath === undefined) {
    throw new Error(
      "usage: bun scripts/phase16-closure.ts target/phase16/closure-attempt-N",
    );
  }
  const path = resolve(repoRoot, maybePath);
  const relativePath = relative(repoRoot, path);
  if (!/^target\/phase16\/closure-attempt-\d+$/.test(relativePath)) {
    throw new Error(`invalid Phase 16 closure attempt: ${relativePath}`);
  }
  return path;
}

async function atomicWriteJson(path: string, value: unknown): Promise<void> {
  const temporaryPath = `${path}.tmp-${process.pid}`;
  await writeFile(temporaryPath, `${JSON.stringify(value, null, 2)}\n`, {
    flag: "wx",
  });
  await rename(temporaryPath, path);
}

async function runCommand(
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
  const result = {
    id: spec.id,
    command: commandText(spec.argv),
    status,
    exitCode,
    stdoutLog: relative(repoRoot, stdoutPath),
    stderrLog: relative(repoRoot, stderrPath),
  } satisfies CommandResult;
  return { result, stdout };
}

function collectAttachmentNames(value: unknown, names: Set<string>): void {
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

async function validateBrowserEvidence(
  attemptDirectory: string,
): Promise<BrowserProof> {
  const proofPath = resolve(attemptDirectory, "browser/browser-proof.json");
  const proof = JSON.parse(
    await readFile(proofPath, "utf8"),
  ) as BrowserProof;
  if (proof.attemptIdentity !== basename(attemptDirectory)) {
    throw new Error("browser proof attempt identity mismatch");
  }
  if (!Object.values(proof.assertions).every(Boolean)) {
    throw new Error("browser proof contains a failed assertion");
  }

  for (const artifact of Object.values(proof.artifacts)) {
    const expectedPrefix = `${relative(repoRoot, attemptDirectory)}/browser/`;
    if (!artifact.path.startsWith(expectedPrefix)) {
      throw new Error(`browser artifact escapes attempt: ${artifact.path}`);
    }
    const bytes = await readFile(resolve(repoRoot, artifact.path));
    if (
      bytes.length !== artifact.byteLength ||
      sha256(bytes) !== artifact.sha256
    ) {
      throw new Error(`browser artifact hash mismatch: ${artifact.path}`);
    }
  }

  const report = JSON.parse(
    await readFile(
      resolve(attemptDirectory, "playwright-report.json"),
      "utf8",
    ),
  ) as unknown;
  const attachmentNames = new Set<string>();
  collectAttachmentNames(report, attachmentNames);
  for (const requiredName of requiredAttachments) {
    if (!attachmentNames.has(requiredName)) {
      throw new Error(`missing Playwright attachment: ${requiredName}`);
    }
  }
  return proof;
}

function validateIsolation(
  outputs: ReadonlyMap<string, string>,
): {
  readonly defaultMember: string;
  readonly normalDependencies: readonly string[];
  readonly packageFileCount: number;
  readonly forbiddenPackageEntries: readonly string[];
} {
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

async function workingTreeIdentity(): Promise<{
  readonly revision: string;
  readonly diffSha256: string;
  readonly status: string;
}> {
  const revisionProcess = Bun.spawnSync({
    cmd: ["git", "rev-parse", "HEAD"],
    cwd: repoRoot,
    stdout: "pipe",
  });
  const statusProcess = Bun.spawnSync({
    cmd: ["git", "status", "--porcelain=v1"],
    cwd: repoRoot,
    stdout: "pipe",
  });
  const diffProcess = Bun.spawnSync({
    cmd: ["git", "diff", "--binary", "HEAD"],
    cwd: repoRoot,
    stdout: "pipe",
  });
  if (
    revisionProcess.exitCode !== 0 ||
    statusProcess.exitCode !== 0 ||
    diffProcess.exitCode !== 0
  ) {
    throw new Error("could not capture closure source identity");
  }
  return {
    revision: revisionProcess.stdout.toString().trim(),
    diffSha256: sha256(diffProcess.stdout),
    status: statusProcess.stdout.toString(),
  };
}

async function main(): Promise<void> {
  const attemptDirectory = await realpath(
    requireAttemptPath(process.argv[2]),
  );
  const closureDirectory = resolve(attemptDirectory, "closure");
  const logsDirectory = resolve(closureDirectory, "logs");
  await mkdir(closureDirectory);
  await mkdir(logsDirectory);
  const startedAt = new Date().toISOString();
  const source = await workingTreeIdentity();
  const browserProof = await validateBrowserEvidence(attemptDirectory);
  const results: CommandResult[] = [];
  const outputs = new Map<string, string>();

  try {
    for (const [index, spec] of commands.entries()) {
      const execution = await runCommand(spec, index, logsDirectory);
      results.push(execution.result);
      outputs.set(spec.id, execution.stdout);
      if (execution.result.status === "failed") {
        throw new Error(`closure command failed: ${execution.result.command}`);
      }
    }
    const isolation = validateIsolation(outputs);
    await atomicWriteJson(resolve(closureDirectory, "closure-summary.json"), {
      schemaVersion: 1,
      status: "passed",
      startedAt,
      completedAt: new Date().toISOString(),
      attemptIdentity: basename(attemptDirectory),
      source,
      browserProof: {
        path: relative(
          repoRoot,
          resolve(attemptDirectory, "browser/browser-proof.json"),
        ),
        sha256: sha256(
          await readFile(
            resolve(attemptDirectory, "browser/browser-proof.json"),
          ),
        ),
        artifacts: browserProof.artifacts,
      },
      commands: results,
      isolation,
    });
    console.log(
      `[phase16-closure] passed ${relative(repoRoot, attemptDirectory)}`,
    );
  } catch (error) {
    await atomicWriteJson(resolve(closureDirectory, "closure-summary.json"), {
      schemaVersion: 1,
      status: "failed",
      startedAt,
      completedAt: new Date().toISOString(),
      attemptIdentity: basename(attemptDirectory),
      source,
      browserProof: {
        artifacts: browserProof.artifacts,
      },
      commands: results,
      error:
        error instanceof Error ? error.message : "unknown closure failure",
    });
    throw error;
  }
}

await main();
