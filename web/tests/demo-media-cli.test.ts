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
  preflightDemoMediaDependencies,
  runCleanupActions,
} from "../scripts/demo-media";

describe("demo media cli helpers", () => {
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

  it("preflights ffmpeg ffprobe and webpmux before generation", () => {
    // Arrange
    const calls: Array<{
      readonly command: string;
      readonly installHint: string | undefined;
    }> = [];

    // Act
    const actual = preflightDemoMediaDependencies({
      readVersionLine: (command, installHint) => {
        calls.push({ command, installHint });
        return `${command} version`;
      },
    });

    // Assert
    expect(actual).toEqual({
      ffmpegVersionLine: "ffmpeg version",
      ffprobeVersionLine: "ffprobe version",
      webpmuxVersionLine: "webpmux version",
    });
    expect(calls).toEqual([
      { command: "ffmpeg", installHint: undefined },
      { command: "ffprobe", installHint: undefined },
      {
        command: "webpmux",
        installHint:
          "Install WebP tools so `webpmux` is available on PATH before running demo-media generation.",
      },
    ]);
  });

  it("surfaces actionable webpmux prerequisite failures", () => {
    // Act / Assert
    expect(() =>
      preflightDemoMediaDependencies({
        readVersionLine: (command, installHint) => {
          if (command === "webpmux") {
            throw new Error(`missing ${command}: ${installHint}`);
          }
          return `${command} version`;
        },
      }),
    ).toThrow(
      "missing webpmux: Install WebP tools so `webpmux` is available on PATH before running demo-media generation.",
    );
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
