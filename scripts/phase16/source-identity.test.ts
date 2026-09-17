import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

import { afterEach, describe, expect, test } from "bun:test";

import {
  captureSourceIdentity,
  requireMatchingSourceIdentity,
} from "./source-identity";

const temporaryDirectories: string[] = [];

function runGit(repoRoot: string, args: readonly string[]): void {
  const result = Bun.spawnSync({
    cmd: ["git", ...args],
    cwd: repoRoot,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) {
    throw new Error(
      `git ${args.join(" ")} failed: ${result.stderr.toString().trim()}`,
    );
  }
}

async function createRepository(): Promise<string> {
  const repoRoot = await mkdtemp(resolve(tmpdir(), "phase16-source-"));
  temporaryDirectories.push(repoRoot);
  runGit(repoRoot, ["init", "--quiet"]);
  runGit(repoRoot, ["config", "user.email", "phase16@example.invalid"]);
  runGit(repoRoot, ["config", "user.name", "Phase 16 Test"]);
  await writeFile(resolve(repoRoot, "tracked.txt"), "initial\n");
  runGit(repoRoot, ["add", "tracked.txt"]);
  runGit(repoRoot, ["commit", "--quiet", "-m", "initial"]);
  return repoRoot;
}

afterEach(async () => {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) =>
      rm(path, { recursive: true, force: true }),
    ),
  );
});

describe("Phase 16 source identity", () => {
  test("rejects a staged mutation made after smoke capture", async () => {
    // Arrange
    const repoRoot = await createRepository();
    const smokeSource = await captureSourceIdentity(repoRoot);
    await writeFile(resolve(repoRoot, "tracked.txt"), "staged mutation\n");
    runGit(repoRoot, ["add", "tracked.txt"]);

    // Act
    const closureSource = await captureSourceIdentity(repoRoot);
    const compare = (): void =>
      requireMatchingSourceIdentity(
        smokeSource,
        closureSource,
        "browser proof",
      );

    // Assert
    expect(compare).toThrow("browser proof source identity mismatch");
  });

  test("rejects an untracked mutation made after smoke capture", async () => {
    // Arrange
    const repoRoot = await createRepository();
    const smokeSource = await captureSourceIdentity(repoRoot);
    await writeFile(resolve(repoRoot, "untracked.txt"), "untracked mutation\n");

    // Act
    const closureSource = await captureSourceIdentity(repoRoot);
    const compare = (): void =>
      requireMatchingSourceIdentity(
        smokeSource,
        closureSource,
        "browser proof",
      );

    // Assert
    expect(compare).toThrow("browser proof source identity mismatch");
  });
});
