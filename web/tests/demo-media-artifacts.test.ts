import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import {
  mkdtemp,
  mkdir,
  readFile,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, relative, resolve } from "node:path";

import { describe, expect, it } from "vitest";

import {
  captureInputSha256,
  compareOutputDirectories,
  mp4Arguments,
  parseMediaProbeJson,
  replaceOutputDirectory,
  validateMp4Probe,
  validateWebpProbe,
  webpArguments,
} from "../scripts/demo-media/artifacts";

describe("demo media artifacts", () => {
  it("hashes sorted capture inputs and ignores readme and media outputs", async () => {
    // Arrange
    const repoRoot = await mkdtemp(join(tmpdir(), "liquidfun-demo-media-hash-"));
    const includedFiles = new Map<string, string>([
      ["web/scripts/demo-media/model.ts", "model bytes\n"],
      ["Cargo.lock", "cargo lock bytes\n"],
      ["web/src/index.ts", "console.log('scene');\n"],
      ["web/package.json", '{"name":"liquidfun-web"}\n'],
      ["Cargo.toml", "[workspace]\n"],
      ["rust-toolchain.toml", "[toolchain]\nchannel = \"stable\"\n"],
      ["crates/liquidfun/src/lib.rs", "pub fn engine() {}\n"],
      ["crates/liquidfun-wasm/src/lib.rs", "pub fn wasm() {}\n"],
      ["web/vite.config.ts", "export default {};\n"],
      ["web/bun.lock", "bun lock bytes\n"],
      ["web/scripts/demo-media.ts", "console.log('cli');\n"],
    ]);
    const excludedFiles = new Map<string, string>([
      ["README.md", "initial readme\n"],
      ["docs/assets/demos/dam-break.mp4", "binary mp4 bytes\n"],
      ["web/src/generated/liquidfun_wasm.js", "generated output\n"],
      ["target/demo-media/tmp.txt", "temporary output\n"],
    ]);

    try {
      await writeRepositoryFiles(repoRoot, includedFiles);
      await writeRepositoryFiles(repoRoot, excludedFiles);
      initializeGitRepository(repoRoot);
      stageAllFiles(repoRoot);
      const expectedHash = hashCaptureInputs(includedFiles);

      // Act
      const initialHash = await captureInputSha256(repoRoot);
      await writeFile(resolve(repoRoot, "README.md"), "updated readme\n");
      await writeFile(
        resolve(repoRoot, "docs/assets/demos/dam-break.mp4"),
        "updated binary mp4 bytes\n",
      );
      const afterExcludedChanges = await captureInputSha256(repoRoot);

      // Assert
      expect(initialHash).toBe(expectedHash);
      expect(afterExcludedChanges).toBe(expectedHash);
    } finally {
      await rm(repoRoot, { recursive: true, force: true });
    }
  });

  it("builds fixed mp4 ffmpeg arguments", () => {
    // Arrange
    const framesDirectory = "/tmp/frames";
    const outputPath = "/tmp/dam-break.mp4";

    // Act
    const actual = mp4Arguments(framesDirectory, outputPath);

    // Assert
    expect(actual).toEqual([
      "-hide_banner",
      "-loglevel",
      "error",
      "-framerate",
      "30",
      "-i",
      "/tmp/frames/frame-%04d.png",
      "-frames:v",
      "240",
      "-an",
      "-c:v",
      "libx264",
      "-preset",
      "slow",
      "-crf",
      "20",
      "-pix_fmt",
      "yuv420p",
      "-movflags",
      "+faststart",
      "-map_metadata",
      "-1",
      "-metadata",
      "creation_time=1970-01-01T00:00:00Z",
      "-fflags",
      "+bitexact",
      "-flags:v",
      "+bitexact",
      "-y",
      outputPath,
    ]);
  });

  it("builds fixed webp ffmpeg arguments", () => {
    // Arrange
    const framesDirectory = "/tmp/frames";
    const outputPath = "/tmp/dam-break.webp";

    // Act
    const actual = webpArguments(framesDirectory, outputPath);

    // Assert
    expect(actual).toEqual([
      "-hide_banner",
      "-loglevel",
      "error",
      "-framerate",
      "30",
      "-i",
      "/tmp/frames/frame-%04d.png",
      "-frames:v",
      "240",
      "-an",
      "-vf",
      "scale=640:-2:flags=lanczos",
      "-c:v",
      "libwebp",
      "-compression_level",
      "6",
      "-q:v",
      "70",
      "-loop",
      "0",
      "-map_metadata",
      "-1",
      "-fflags",
      "+bitexact",
      "-y",
      outputPath,
    ]);
  });

  it("reports missing extra and changed files between output directories", async () => {
    // Arrange
    const expectedDirectory = await mkdtemp(
      join(tmpdir(), "liquidfun-demo-media-expected-"),
    );
    const actualDirectory = await mkdtemp(
      join(tmpdir(), "liquidfun-demo-media-actual-"),
    );

    try {
      await writeRepositoryFiles(expectedDirectory, new Map([
        ["dam-break.mp4", "expected mp4\n"],
        ["manifest.json", "expected manifest\n"],
        ["water-wheel.webp", "shared preview\n"],
      ]));
      await writeRepositoryFiles(actualDirectory, new Map([
        ["manifest.json", "changed manifest\n"],
        ["old.webp", "unexpected extra file\n"],
        ["water-wheel.webp", "shared preview\n"],
      ]));

      // Act
      const actual = await compareOutputDirectories(
        expectedDirectory,
        actualDirectory,
      );

      // Assert
      expect(actual).toEqual([
        "missing: dam-break.mp4",
        "changed: manifest.json",
        "extra: old.webp",
      ]);
    } finally {
      await rm(expectedDirectory, { recursive: true, force: true });
      await rm(actualDirectory, { recursive: true, force: true });
    }
  });

  it("restores the original target directory after a replacement rename failure", async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), "liquidfun-demo-media-replace-"));
    const targetDirectory = resolve(root, "target");
    const nextDirectory = resolve(root, "next");

    try {
      await writeRepositoryFiles(targetDirectory, new Map([
        ["manifest.json", "original manifest\n"],
        ["dam-break.mp4", "original video bytes\n"],
      ]));
      await writeRepositoryFiles(nextDirectory, new Map([
        ["manifest.json", "replacement manifest\n"],
        ["dam-break.mp4", "replacement video bytes\n"],
      ]));
      const expectedSnapshot = await directorySnapshot(targetDirectory);
      const renameCalls: Array<{ readonly from: string; readonly to: string }> = [];
      const renamePath = async (from: string, to: string): Promise<void> => {
        renameCalls.push({ from, to });
        if (from === nextDirectory && to === targetDirectory) {
          throw new Error("injected rename failure");
        }
        await rename(from, to);
      };

      // Act / Assert
      await expect(
        replaceOutputDirectory(nextDirectory, targetDirectory, renamePath),
      ).rejects.toThrow("injected rename failure");
      expect(await directorySnapshot(targetDirectory)).toEqual(expectedSnapshot);
      expect(renameCalls).toHaveLength(3);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("keeps the new target installed when backup cleanup fails after commit", async () => {
    // Arrange
    const root = await mkdtemp(join(tmpdir(), "liquidfun-demo-media-cleanup-"));
    const targetDirectory = resolve(root, "target");
    const nextDirectory = resolve(root, "next");

    try {
      await writeRepositoryFiles(targetDirectory, new Map([
        ["manifest.json", "original manifest\n"],
        ["dam-break.mp4", "original video bytes\n"],
      ]));
      await writeRepositoryFiles(nextDirectory, new Map([
        ["manifest.json", "replacement manifest\n"],
        ["dam-break.mp4", "replacement video bytes\n"],
      ]));
      const warnings: string[] = [];

      // Act
      await expect(
        replaceOutputDirectory(nextDirectory, targetDirectory, {
          removeDirectory: async () => {
            throw new Error("backup cleanup failed");
          },
          onWarning: (warning) => {
            warnings.push(warning);
          },
        }),
      ).resolves.toBeUndefined();

      // Assert
      expect(await directorySnapshot(targetDirectory)).toEqual({
        "dam-break.mp4": Buffer.from("replacement video bytes\n", "utf8").toString("hex"),
        "manifest.json": Buffer.from("replacement manifest\n", "utf8").toString("hex"),
      });
      expect(warnings).toEqual([
        expect.stringContaining("backup cleanup failed"),
      ]);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });

  it("parses ffprobe JSON into a media probe", () => {
    // Arrange
    const jsonText = JSON.stringify({
      streams: [
        {
          codec_name: "h264",
          width: 1280,
          height: 720,
          avg_frame_rate: "30/1",
          nb_frames: "240",
        },
      ],
      format: {
        duration: "8.0",
      },
    });

    // Act
    const actual = parseMediaProbeJson("dam-break.mp4", jsonText);

    // Assert
    expect(actual).toEqual({
      codecName: "h264",
      width: 1280,
      height: 720,
      averageFrameRate: "30/1",
      maybeFrameCount: 240,
      durationSeconds: 8,
    });
  });

  it("rejects invalid ffprobe JSON and validation mismatches", () => {
    // Arrange
    const invalidJsonText = JSON.stringify({
      streams: [
        {
          codec_name: "webp",
          width: 640,
          height: 360,
          avg_frame_rate: "not-a-rational",
          nb_frames: "240",
        },
      ],
      format: {
        duration: "8.0",
      },
    });
    const wrongCodecProbe = {
      codecName: "vp9",
      width: 640,
      height: 360,
      averageFrameRate: "30/1",
      maybeFrameCount: 240,
      durationSeconds: 8,
    };
    const wrongDurationProbe = {
      codecName: "webp",
      width: 640,
      height: 360,
      averageFrameRate: "30/1",
      maybeFrameCount: 240,
      durationSeconds: 8.5,
    };

    // Act / Assert
    expect(() => parseMediaProbeJson("bad.webp", invalidJsonText)).toThrow(
      "ffprobe avg_frame_rate is invalid for bad.webp",
    );
    expect(() => validateMp4Probe("bad.mp4", wrongCodecProbe)).toThrow(
      "Expected H.264 MP4 output, found vp9 for bad.mp4",
    );
    expect(() => validateWebpProbe("bad.webp", wrongDurationProbe)).toThrow(
      "Expected ~8 second duration, found 8.5 for bad.webp",
    );
  });
});

async function writeRepositoryFiles(
  root: string,
  files: ReadonlyMap<string, string>,
): Promise<void> {
  for (const [relativePath, contents] of files) {
    const absolutePath = resolve(root, relativePath);
    await mkdir(resolve(absolutePath, ".."), { recursive: true });
    await writeFile(absolutePath, contents);
  }
}

function initializeGitRepository(repoRoot: string): void {
  execFileSync("git", ["init"], { cwd: repoRoot, stdio: "ignore" });
}

function stageAllFiles(repoRoot: string): void {
  execFileSync("git", ["add", "-A"], { cwd: repoRoot, stdio: "ignore" });
}

function hashCaptureInputs(files: ReadonlyMap<string, string>): string {
  const hash = createHash("sha256");

  for (const relativePath of [...files.keys()].sort()) {
    const bytes = Buffer.from(files.get(relativePath) ?? "", "utf8");
    hash.update(relativePath);
    hash.update("\0");
    hash.update(String(bytes.length));
    hash.update("\0");
    hash.update(bytes);
  }

  return hash.digest("hex");
}

async function directorySnapshot(
  root: string,
): Promise<Record<string, string>> {
  const snapshot: Record<string, string> = {};
  for (const relativePath of ["dam-break.mp4", "manifest.json"]) {
    const bytes = await readFile(resolve(root, relativePath));
    snapshot[relative(root, resolve(root, relativePath))] = bytes.toString("hex");
  }
  return snapshot;
}
