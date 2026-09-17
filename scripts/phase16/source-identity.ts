import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

export type SourceIdentity = {
  readonly revision: string;
  readonly workingTreeSha256: string;
  readonly status: string;
};

const SHA256_PATTERN = /^[0-9a-f]{64}$/;
const MAX_REVISION_LENGTH = 64;
const MAX_STATUS_LENGTH = 1_000_000;

function commandText(command: readonly string[]): string {
  return command.join(" ");
}

function captureGitBytes(
  repoRoot: string,
  command: readonly string[],
): Uint8Array {
  const result = Bun.spawnSync({
    cmd: [...command],
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    const diagnostic = result.stderr.toString().trim();
    throw new Error(
      `source identity command failed (${result.exitCode}): ${commandText(command)}${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
    );
  }
  return result.stdout;
}

function updateSegment(
  hash: ReturnType<typeof createHash>,
  name: string,
  bytes: Uint8Array,
): void {
  hash.update(name);
  hash.update("\0");
  hash.update(String(bytes.length));
  hash.update("\0");
  hash.update(bytes);
  hash.update("\0");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function requireExactKeys(
  record: Record<string, unknown>,
  expectedKeys: readonly string[],
  label: string,
): void {
  const actualKeys = Object.keys(record).sort();
  const sortedExpectedKeys = [...expectedKeys].sort();
  if (
    actualKeys.length !== sortedExpectedKeys.length ||
    actualKeys.some((key, index) => key !== sortedExpectedKeys[index])
  ) {
    throw new Error(`${label} has unknown or missing fields`);
  }
}

export function parseSourceIdentity(
  value: unknown,
  label: string,
): SourceIdentity {
  if (!isRecord(value)) {
    throw new Error(`${label} must be an object`);
  }
  requireExactKeys(
    value,
    ["revision", "workingTreeSha256", "status"],
    label,
  );
  if (
    typeof value.revision !== "string" ||
    value.revision.length === 0 ||
    value.revision.length > MAX_REVISION_LENGTH
  ) {
    throw new Error(`${label}.revision is invalid`);
  }
  if (
    typeof value.workingTreeSha256 !== "string" ||
    !SHA256_PATTERN.test(value.workingTreeSha256)
  ) {
    throw new Error(`${label}.workingTreeSha256 is invalid`);
  }
  if (
    typeof value.status !== "string" ||
    value.status.length > MAX_STATUS_LENGTH
  ) {
    throw new Error(`${label}.status is invalid`);
  }
  return {
    revision: value.revision,
    workingTreeSha256: value.workingTreeSha256,
    status: value.status,
  };
}

export function requireMatchingSourceIdentity(
  expected: SourceIdentity,
  actual: SourceIdentity,
  label: string,
): void {
  if (
    expected.revision !== actual.revision ||
    expected.workingTreeSha256 !== actual.workingTreeSha256 ||
    expected.status !== actual.status
  ) {
    throw new Error(`${label} source identity mismatch`);
  }
}

export async function captureSourceIdentity(
  repoRoot: string,
): Promise<SourceIdentity> {
  const revisionBytes = captureGitBytes(repoRoot, [
    "git",
    "rev-parse",
    "HEAD",
  ]);
  const statusBytes = captureGitBytes(repoRoot, [
    "git",
    "status",
    "--porcelain=v1",
    "-z",
  ]);
  const unstagedDiffBytes = captureGitBytes(repoRoot, [
    "git",
    "diff",
    "--binary",
  ]);
  const stagedDiffBytes = captureGitBytes(repoRoot, [
    "git",
    "diff",
    "--binary",
    "--cached",
    "HEAD",
  ]);
  const untrackedPathsBytes = captureGitBytes(repoRoot, [
    "git",
    "ls-files",
    "--others",
    "--exclude-standard",
    "-z",
  ]);
  const untrackedPaths = new TextDecoder()
    .decode(untrackedPathsBytes)
    .split("\0")
    .filter((path) => path.length > 0)
    .sort();

  const hash = createHash("sha256");
  updateSegment(hash, "porcelain-status", statusBytes);
  updateSegment(hash, "unstaged-diff", unstagedDiffBytes);
  updateSegment(hash, "staged-diff", stagedDiffBytes);
  for (const path of untrackedPaths) {
    updateSegment(hash, "untracked-path", new TextEncoder().encode(path));
    updateSegment(
      hash,
      "untracked-content",
      await readFile(resolve(repoRoot, path)),
    );
  }

  return {
    revision: new TextDecoder().decode(revisionBytes).trim(),
    workingTreeSha256: hash.digest("hex"),
    status: new TextDecoder().decode(statusBytes).replaceAll("\0", "\n"),
  };
}
