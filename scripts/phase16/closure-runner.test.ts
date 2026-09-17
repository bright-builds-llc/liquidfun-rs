import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  realpath,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { resolve } from "node:path";

import { afterEach, describe, expect, test } from "bun:test";

import { runClosureAttempt } from "./closure-runner";

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

async function createMalformedAttempt(): Promise<{
  readonly repoRoot: string;
  readonly attemptDirectory: string;
}> {
  const repoRoot = await realpath(
    await mkdtemp(resolve(tmpdir(), "phase16-closure-")),
  );
  temporaryDirectories.push(repoRoot);
  runGit(repoRoot, ["init", "--quiet"]);
  runGit(repoRoot, ["config", "user.email", "phase16@example.invalid"]);
  runGit(repoRoot, ["config", "user.name", "Phase 16 Test"]);
  await writeFile(resolve(repoRoot, ".gitignore"), "target/\n");
  runGit(repoRoot, ["add", ".gitignore"]);
  runGit(repoRoot, ["commit", "--quiet", "-m", "initial"]);

  const attemptDirectory = resolve(
    repoRoot,
    "target/phase16/closure-attempt-10",
  );
  await mkdir(resolve(attemptDirectory, "browser"), { recursive: true });
  await writeFile(
    resolve(attemptDirectory, "browser/browser-proof.json"),
    "{}\n",
  );
  return { repoRoot, attemptDirectory };
}

afterEach(async () => {
  await Promise.all(
    temporaryDirectories.splice(0).map((path) =>
      rm(path, { recursive: true, force: true }),
    ),
  );
});

describe("Phase 16 closure transaction", () => {
  test("retains one bounded failure summary for malformed evidence", async () => {
    // Arrange
    const fixture = await createMalformedAttempt();
    const summaryPath = resolve(
      fixture.attemptDirectory,
      "closure/closure-summary.json",
    );

    // Act
    const closure = runClosureAttempt(
      fixture.repoRoot,
      fixture.attemptDirectory,
      [],
    );

    // Assert
    await expect(closure).rejects.toThrow(
      "browser proof has unknown or missing entries",
    );
    const summary = JSON.parse(
      await readFile(summaryPath, "utf8"),
    ) as Record<string, unknown>;
    const summaryStatus = await stat(summaryPath);
    const closureEntries = await readdir(
      resolve(fixture.attemptDirectory, "closure"),
    );
    expect(summary.status).toBe("failed");
    expect(summary.error).toBe(
      "browser proof has unknown or missing entries",
    );
    expect(summaryStatus.size).toBeLessThanOrEqual(128_000);
    expect(
      closureEntries.filter((entry) => entry === "closure-summary.json"),
    ).toHaveLength(1);
  });

  test("does not overwrite an existing failure summary", async () => {
    // Arrange
    const fixture = await createMalformedAttempt();
    const summaryPath = resolve(
      fixture.attemptDirectory,
      "closure/closure-summary.json",
    );
    await expect(
      runClosureAttempt(
        fixture.repoRoot,
        fixture.attemptDirectory,
        [],
      ),
    ).rejects.toThrow();
    const originalSummary = await readFile(summaryPath);

    // Act
    const rerun = runClosureAttempt(
      fixture.repoRoot,
      fixture.attemptDirectory,
      [],
    );

    // Assert
    await expect(rerun).rejects.toThrow();
    expect(await readFile(summaryPath)).toEqual(originalSummary);
  });
});
