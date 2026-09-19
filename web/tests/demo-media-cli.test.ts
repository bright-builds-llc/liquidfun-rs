import {
  mkdtemp,
  mkdir,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

import { describe, expect, it } from "vitest";

import {
  commitStagedOutputs,
  finalizeDemoMediaRun,
  formatCleanupFailure,
  maybeRethrowWithCleanupFailures,
  runCleanupActions,
  terminatePreviewProcess,
  waitForOwnedPreviewReadiness,
} from "../scripts/demo-media";

describe("demo media cli helpers", () => {
  it("fails when no owned ready signal arrives and the child exits", async () => {
    // Arrange
    let fetchCalls = 0;

    // Act / Assert
    await expect(
      waitForOwnedPreviewReadiness({
        readySignal: new Promise<void>(() => undefined),
        exitSignal: Promise.resolve(1),
        getExitCode: () => 1,
        diagnostics: () => "EADDRINUSE",
        expectedIndexHtml: "<html>current</html>",
        fetchIndexHtml: async () => {
          fetchCalls += 1;
          return "<html>stale</html>";
        },
        readyTimeoutMilliseconds: 1_000,
      }),
    ).rejects.toThrow("preview process exited early (1): EADDRINUSE");
    expect(fetchCalls).toBe(0);
  });

  it("accepts owned readiness only after served index matches current dist", async () => {
    // Arrange
    let fetchCalls = 0;

    // Act / Assert
    await expect(
      waitForOwnedPreviewReadiness({
        readySignal: Promise.resolve(),
        exitSignal: new Promise<number | null>(() => undefined),
        getExitCode: () => null,
        diagnostics: () => "no diagnostics",
        expectedIndexHtml: "<html>current</html>",
        fetchIndexHtml: async () => {
          fetchCalls += 1;
          return "<html>current</html>";
        },
        readyTimeoutMilliseconds: 1_000,
      }),
    ).resolves.toBeUndefined();
    expect(fetchCalls).toBe(1);
  });

  it("aggregates cleanup failures with the primary error first", async () => {
    // Arrange
    const cleanupFailures = await runCleanupActions([
      {
        label: "close browser",
        run: async () => {
          throw new Error("browser refused to close");
        },
      },
      {
        label: "stop preview",
        run: async () => {
          throw new Error("preview refused to stop");
        },
      },
    ]);

    // Act / Assert
    expect(cleanupFailures.map((failure) => formatCleanupFailure(failure))).toEqual([
      "close browser: browser refused to close",
      "stop preview: preview refused to stop",
    ]);
    expect(() =>
      maybeRethrowWithCleanupFailures(
        new Error("primary failure"),
        cleanupFailures,
      ),
    ).toThrowError(AggregateError);
    try {
      maybeRethrowWithCleanupFailures(
        new Error("primary failure"),
        cleanupFailures,
      );
    } catch (error) {
      const aggregate = error as AggregateError;
      expect(aggregate.errors).toHaveLength(3);
      expect((aggregate.errors[0] as Error).message).toBe("primary failure");
      expect((aggregate.errors[1] as Error).message).toBe(
        "close browser: browser refused to close",
      );
      expect((aggregate.errors[2] as Error).message).toBe(
        "stop preview: preview refused to stop",
      );
    }
  });

  it("does not commit staged outputs after cleanup failure", async () => {
    // Arrange
    const cleanupFailures = await runCleanupActions([
      {
        label: "stop preview",
        run: async () => {
          throw new Error("preview refused to stop");
        },
      },
    ]);
    let commitCalls = 0;

    // Act / Assert
    await expect(
      finalizeDemoMediaRun({
        cleanupFailures,
        commitOutputs: async () => {
          commitCalls += 1;
        },
      }),
    ).rejects.toThrowError(AggregateError);
    expect(commitCalls).toBe(0);
  });

  it("waits for actual exit after SIGKILL and fails if the process survives", async () => {
    // Arrange
    const killSignals: string[] = [];
    const waitResults = [false, false];

    // Act / Assert
    await expect(
      terminatePreviewProcess({
        getExitCode: () => null,
        kill: (signal) => {
          killSignals.push(signal);
        },
        waitForExit: async () => waitResults.shift() ?? false,
      }),
    ).rejects.toThrow("Preview process did not exit after SIGKILL");
    expect(killSignals).toEqual(["SIGTERM", "SIGKILL"]);
  });

  it("leaves committed output unchanged for identical check outputs", async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), "liquidfun-demo-media-check-clean-"));
    const targetDirectory = resolve(root, "target");
    const nextDirectory = resolve(root, "next");

    try {
      await writeDirectoryFiles(targetDirectory, new Map([
        ["manifest.json", "shared manifest\n"],
        ["dam-break.mp4", "shared video bytes\n"],
      ]));
      await writeDirectoryFiles(nextDirectory, new Map([
        ["manifest.json", "shared manifest\n"],
        ["dam-break.mp4", "shared video bytes\n"],
      ]));
      const before = await readDirectorySnapshot(targetDirectory);

      // Act
      const warnings = await commitStagedOutputs("check", nextDirectory, targetDirectory);

      // Assert
      expect(warnings).toEqual([]);
      expect(await readDirectorySnapshot(targetDirectory)).toEqual(before);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("fails check mismatches without mutating committed output", async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), "liquidfun-demo-media-check-dirty-"));
    const targetDirectory = resolve(root, "target");
    const nextDirectory = resolve(root, "next");

    try {
      await writeDirectoryFiles(targetDirectory, new Map([
        ["manifest.json", "target manifest\n"],
        ["dam-break.mp4", "target video bytes\n"],
      ]));
      await writeDirectoryFiles(nextDirectory, new Map([
        ["manifest.json", "changed manifest\n"],
        ["dam-break.mp4", "target video bytes\n"],
      ]));
      const before = await readDirectorySnapshot(targetDirectory);

      // Act / Assert
      await expect(
        commitStagedOutputs("check", nextDirectory, targetDirectory),
      ).rejects.toThrow("demo media outputs differ:\nchanged: manifest.json");
      expect(await readDirectorySnapshot(targetDirectory)).toEqual(before);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
});

async function writeDirectoryFiles(
  root: string,
  files: ReadonlyMap<string, string>,
): Promise<void> {
  for (const [relativePath, contents] of files) {
    const absolutePath = resolve(root, relativePath);
    await mkdir(resolve(absolutePath, ".."), { recursive: true });
    await writeFile(absolutePath, contents);
  }
}

async function readDirectorySnapshot(root: string): Promise<Record<string, string>> {
  return {
    "dam-break.mp4": await readFile(resolve(root, "dam-break.mp4"), "utf8"),
    "manifest.json": await readFile(resolve(root, "manifest.json"), "utf8"),
  };
}
