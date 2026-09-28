import { spawn, type ChildProcess } from "node:child_process";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Writable } from "node:stream";

import {
  describeAnimatedWebp,
  frameDurationsMs,
  isWebpPreset,
  prepareRecordedWebp,
} from "./webp";

export type AnimatedWebpEncoderOptions = {
  readonly width: number;
  readonly height: number;
  readonly frameCount: number;
  readonly framesPerSecond: number;
  readonly quality: number;
  readonly preset: string;
  /** Executable used to encode frames. Tests pass a missing name. */
  readonly command?: string;
};

export type AnimatedWebpEncoder = {
  writeFrame(pixels: Uint8Array): Promise<void>;
  finish(): Promise<Buffer>;
  abort(): Promise<void>;
};

/**
 * Streams raw RGBA frames into one looping WebP.
 *
 * ffmpeg writes the animation itself. A pipe cannot patch the RIFF size, so
 * the encoder uses a temporary file and reads it back after ffmpeg exits.
 */
export async function startAnimatedWebpEncoder(
  options: AnimatedWebpEncoderOptions,
): Promise<AnimatedWebpEncoder> {
  assertEncodeOptions(options);
  const directory = await mkdtemp(join(tmpdir(), "readme-webp-"));
  const outputPath = join(directory, "preview.webp");
  const expectedBytes = options.width * options.height * 4;
  const expectedDurations = frameDurationsMs(options.frameCount, options.framesPerSecond);
  const child = spawn(options.command ?? "ffmpeg", ffmpegArguments(options, outputPath), {
    stdio: ["pipe", "ignore", "pipe"],
  });
  const stdin = child.stdin;
  const stderrStream = child.stderr;
  if (stdin === null || stderrStream === null) {
    child.kill("SIGKILL");
    await rm(directory, { recursive: true, force: true });
    throw new Error("ffmpeg pipes are missing.");
  }
  const stderr = collect(stderrStream);
  void stderr.catch(() => undefined);
  const exitCode = waitForExit(child);
  void exitCode.catch(() => {
    // A failed write kills ffmpeg. This catches that close so it does not
    // surface as an unhandled rejection.
  });
  const spawnFailure = new Promise<never>((_, reject) => {
    child.once("error", (error) => {
      const failure = ffmpegSpawnError(error);
      stdin.destroy(failure);
      reject(failure);
    });
  });
  void spawnFailure.catch(() => undefined);

  let written = 0;
  let settled = false;

  const fail = async (error: unknown): Promise<never> => {
    await stopEncoder(child, directory, () => {
      settled = true;
    }, settled);
    throw error;
  };

  return {
    async writeFrame(pixels) {
      if (settled) {
        throw new Error("Animated WebP encoder is already closed.");
      }
      if (pixels.byteLength !== expectedBytes) {
        return fail(
          new Error(
            `Frame ${written} is ${pixels.byteLength} bytes, expected ${expectedBytes}.`,
          ),
        );
      }
      if (written >= options.frameCount) {
        return fail(
          new Error(`Animated WebP received more than ${options.frameCount} frames.`),
        );
      }
      try {
        const write = writeAll(stdin, Buffer.from(pixels));
        void write.catch(() => undefined);
        await Promise.race([write, spawnFailure]);
        written += 1;
      } catch (error) {
        return fail(error);
      }
    },
    async finish() {
      if (settled) {
        throw new Error("Animated WebP encoder is already closed.");
      }
      if (written !== options.frameCount) {
        return fail(
          new Error(`Animated WebP wrote ${written} frames, expected ${options.frameCount}.`),
        );
      }
      settled = true;
      stdin.end();
      const code = await Promise.race([exitCode, spawnFailure]);
      if (code !== 0) {
        const detail = (await stderr).toString("utf8").trim().slice(-2000);
        await rm(directory, { recursive: true, force: true });
        throw new Error(`ffmpeg WebP encode failed (${code}): ${detail}`);
      }
      const webp = prepareRecordedWebp(await readFile(outputPath));
      await rm(directory, { recursive: true, force: true });
      assertEncodedWebp(webp, options, expectedDurations);
      return webp;
    },
    async abort() {
      if (settled) {
        return;
      }
      settled = true;
      child.kill("SIGKILL");
      await exitCode.catch(() => undefined);
      await rm(directory, { recursive: true, force: true });
    },
  };
}

function assertEncodeOptions(options: AnimatedWebpEncoderOptions): void {
  if (options.width % 2 !== 0 || options.height % 2 !== 0) {
    throw new Error(
      `WebP size must be even because the encoder uses 4:2:0, got ${options.width}x${options.height}.`,
    );
  }
  if (!Number.isInteger(options.quality) || options.quality < 0 || options.quality > 100) {
    throw new Error("WebP quality must be an integer from 0 through 100.");
  }
  if (!isWebpPreset(options.preset)) {
    throw new Error(`Unsupported WebP preset "${options.preset}".`);
  }
  if (!Number.isInteger(options.frameCount) || options.frameCount <= 0) {
    throw new Error("WebP frame count must be a positive integer.");
  }
}

function assertEncodedWebp(
  webp: Buffer,
  options: AnimatedWebpEncoderOptions,
  expectedDurations: readonly number[],
): void {
  const summary = describeAnimatedWebp(webp);
  if (
    summary.width !== options.width ||
    summary.height !== options.height ||
    summary.loopCount !== 0 ||
    summary.durationsMs.length !== options.frameCount ||
    summary.durationsMs.some((duration, index) => duration !== expectedDurations[index])
  ) {
    throw new Error("Animated WebP summary does not match the recorded frames.");
  }
}

function ffmpegArguments(
  options: AnimatedWebpEncoderOptions,
  outputPath: string,
): string[] {
  return [
    "-y",
    "-hide_banner",
    "-loglevel",
    "error",
    "-f",
    "rawvideo",
    "-pixel_format",
    "rgba",
    "-video_size",
    `${options.width}x${options.height}`,
    "-framerate",
    String(options.framesPerSecond),
    "-i",
    "pipe:0",
    "-frames:v",
    String(options.frameCount),
    "-an",
    "-c:v",
    "libwebp",
    "-quality",
    String(options.quality),
    "-preset",
    options.preset,
    "-loop",
    "0",
    "-pix_fmt",
    "yuv420p",
    "-map_metadata",
    "-1",
    "-fflags",
    "+bitexact",
    "-flags:v",
    "+bitexact",
    outputPath,
  ];
}

async function stopEncoder(
  child: ChildProcess,
  directory: string,
  markSettled: () => void,
  alreadySettled: boolean,
): Promise<void> {
  if (!alreadySettled) {
    markSettled();
    child.kill("SIGKILL");
  }
  await rm(directory, { recursive: true, force: true });
}

function collect(stream: NodeJS.ReadableStream): Promise<Buffer> {
  const chunks: Buffer[] = [];
  return new Promise((resolve, reject) => {
    stream.on("data", (chunk: Buffer | string) => {
      chunks.push(typeof chunk === "string" ? Buffer.from(chunk) : chunk);
    });
    stream.once("error", reject);
    stream.once("end", () => {
      resolve(Buffer.concat(chunks));
    });
  });
}

function waitForExit(child: ChildProcess): Promise<number> {
  return new Promise((resolve, reject) => {
    let settled = false;
    child.once("error", (error) => {
      if (settled) {
        return;
      }
      settled = true;
      if (isMissingExecutable(error)) {
        reject(new Error("ffmpeg is required to record README WebP previews."));
        return;
      }
      reject(error);
    });
    child.once("close", (code) => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(code ?? 1);
    });
  });
}

function writeAll(stream: Writable, chunk: Buffer): Promise<void> {
  if (stream.write(chunk)) {
    return Promise.resolve();
  }

  return new Promise((resolve, reject) => {
    let settled = false;
    const finish = (error?: Error) => {
      if (settled) {
        return;
      }
      settled = true;
      stream.off("drain", onDrain);
      stream.off("error", onError);
      if (error === undefined) {
        resolve();
        return;
      }
      reject(error);
    };
    const onDrain = () => {
      finish();
    };
    const onError = (error: Error) => {
      finish(error);
    };
    stream.once("drain", onDrain);
    stream.once("error", onError);
  });
}

function ffmpegSpawnError(error: Error): Error {
  if (isMissingExecutable(error)) {
    return new Error("ffmpeg is required to record README WebP previews.");
  }
  return error;
}

function isMissingExecutable(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}
