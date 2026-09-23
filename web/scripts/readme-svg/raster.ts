import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { access } from "node:fs/promises";
import type { Writable } from "node:stream";

import { Resvg } from "@resvg/resvg-js";

import { parseAnimatedSvg, renderParsedSvg, type ParsedAnimatedSvg } from "./smil";
import {
  describeAnimatedWebp,
  frameCountForDuration,
  frameDurationsMs,
  isWebpPreset,
  muxAnimatedWebp,
} from "./webp";

/** Pinned face so label text does not depend on the host font set. */
export const README_RASTER_FONT_FILE = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

const README_RASTER_FONT_FAMILY = "DejaVu Sans";

export type RasterizeAnimatedSvgOptions = {
  readonly framesPerSecond: number;
  readonly quality: number;
  readonly preset: string;
  readonly fontFile: string;
  readonly onFrame?: (completed: number, total: number) => void;
};

/**
 * Flattens one animated SVG to an opaque 60 fps-class WebP.
 *
 * Frames are the SMIL document evaluated at `index / framesPerSecond`, drawn
 * with resvg, encoded as still WebP images, then muxed with integer
 * millisecond durations that average the requested rate.
 */
export async function rasterizeAnimatedSvg(
  svg: string,
  options: RasterizeAnimatedSvgOptions,
): Promise<Buffer> {
  if (!Number.isInteger(options.quality) || options.quality < 0 || options.quality > 100) {
    throw new Error("WebP quality must be an integer from 0 through 100.");
  }
  if (!isWebpPreset(options.preset)) {
    throw new Error(`Unsupported WebP preset "${options.preset}".`);
  }
  await assertFontFile(options.fontFile);

  const parsed = parseAnimatedSvg(svg);
  if (parsed.durationSeconds <= 0) {
    throw new Error("Animated SVG has no SMIL duration to rasterize.");
  }
  if (parsed.width % 2 !== 0 || parsed.height % 2 !== 0) {
    throw new Error(
      `WebP raster size must be even because the encoder uses 4:2:0, got ${parsed.width}x${parsed.height}.`,
    );
  }

  const frameCount = frameCountForDuration(parsed.durationSeconds, options.framesPerSecond);
  const durationsMs = frameDurationsMs(frameCount, options.framesPerSecond);
  const stills = await encodeStillWebps(parsed, frameCount, options);
  const animated = muxAnimatedWebp(stills, durationsMs, parsed.width, parsed.height);
  const summary = describeAnimatedWebp(animated);
  if (
    summary.width !== parsed.width ||
    summary.height !== parsed.height ||
    summary.loopCount !== 0 ||
    summary.durationsMs.length !== frameCount ||
    summary.durationsMs.some((duration, index) => duration !== durationsMs[index])
  ) {
    throw new Error("Animated WebP summary does not match the SVG frame.");
  }
  return animated;
}

async function assertFontFile(fontFile: string): Promise<void> {
  try {
    await access(fontFile);
  } catch (error) {
    throw new Error(
      `README WebP raster font is missing at ${fontFile}. Install DejaVu Sans (fonts-dejavu-core).`,
      { cause: error },
    );
  }
}

async function encodeStillWebps(
  parsed: ParsedAnimatedSvg,
  frameCount: number,
  options: RasterizeAnimatedSvgOptions,
): Promise<Buffer[]> {
  const ffmpeg = spawn(
    "ffmpeg",
    [
      "-hide_banner",
      "-loglevel",
      "error",
      "-f",
      "rawvideo",
      "-pixel_format",
      "rgba",
      "-video_size",
      `${parsed.width}x${parsed.height}`,
      "-framerate",
      String(options.framesPerSecond),
      "-i",
      "pipe:0",
      "-frames:v",
      String(frameCount),
      "-an",
      "-c:v",
      "libwebp",
      "-quality",
      String(options.quality),
      "-preset",
      options.preset,
      "-pix_fmt",
      "yuv420p",
      "-f",
      "image2pipe",
      "pipe:1",
    ],
    { stdio: ["pipe", "pipe", "pipe"] },
  );
  const stdout = collect(ffmpeg.stdout);
  const stderr = collect(ffmpeg.stderr);
  const exitCode = waitForExit(ffmpeg);
  void exitCode.catch(() => {
    // A failed write reports that error and kills ffmpeg. This catches the
    // resulting close so it does not surface as an unhandled rejection.
  });

  try {
    for (let index = 0; index < frameCount; index += 1) {
      const pixels = renderFrame(parsed, index / options.framesPerSecond, options.fontFile);
      if (pixels.length !== parsed.width * parsed.height * 4) {
        throw new Error(
          `Frame ${index} raster is ${pixels.length} bytes, expected ${parsed.width * parsed.height * 4}.`,
        );
      }
      await writeAll(ffmpeg.stdin, pixels);
      const completed = index + 1;
      if (completed % 30 === 0) {
        releaseRasterMemory();
      }
      if (completed === 1 || completed === frameCount || completed % 60 === 0) {
        options.onFrame?.(completed, frameCount);
      }
    }
    ffmpeg.stdin.end();
  } catch (error) {
    ffmpeg.kill("SIGKILL");
    throw error;
  }

  const code = await exitCode;
  if (code !== 0) {
    const detail = (await stderr).toString("utf8").trim().slice(-2000);
    throw new Error(`ffmpeg WebP encode failed (${code}): ${detail}`);
  }

  return splitStillWebps(await stdout, frameCount);
}

function renderFrame(parsed: ParsedAnimatedSvg, timeSeconds: number, fontFile: string): Buffer {
  const svg = renderParsedSvg(parsed, timeSeconds, { fontFamily: README_RASTER_FONT_FAMILY });
  const resvg = new Resvg(svg, {
    font: {
      loadSystemFonts: false,
      fontFiles: [fontFile],
      defaultFontFamily: README_RASTER_FONT_FAMILY,
      sansSerifFamily: README_RASTER_FONT_FAMILY,
    },
    shapeRendering: 2,
    textRendering: 2,
    logLevel: "off",
  });
  const image = resvg.render();
  if (image.width !== parsed.width || image.height !== parsed.height) {
    throw new Error(
      `Raster size ${image.width}x${image.height} does not match SVG ${parsed.width}x${parsed.height}.`,
    );
  }
  return image.pixels;
}

function splitStillWebps(bytes: Buffer, frameCount: number): Buffer[] {
  const stills: Buffer[] = [];
  let offset = 0;
  while (offset + 12 <= bytes.length) {
    if (bytes.toString("ascii", offset, offset + 4) !== "RIFF") {
      throw new Error(`ffmpeg WebP stream is not a RIFF document at byte ${offset}.`);
    }
    const size = bytes.readUInt32LE(offset + 4);
    const end = offset + 8 + size;
    if (end > bytes.length) {
      throw new Error("ffmpeg WebP stream ended inside a frame.");
    }
    stills.push(bytes.subarray(offset, end));
    offset = end;
  }
  if (offset !== bytes.length) {
    throw new Error("ffmpeg WebP stream has trailing bytes.");
  }
  if (stills.length !== frameCount) {
    throw new Error(`ffmpeg wrote ${stills.length} WebP frames, expected ${frameCount}.`);
  }
  return stills;
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

function waitForExit(child: ChildProcessWithoutNullStreams): Promise<number> {
  return new Promise((resolve, reject) => {
    let settled = false;
    child.once("error", (error) => {
      if (settled) {
        return;
      }
      settled = true;
      if (isMissingExecutable(error)) {
        reject(new Error("ffmpeg is required to rasterize README WebP previews."));
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

function releaseRasterMemory(): void {
  const maybeRuntime = globalThis as { Bun?: { gc?: (force?: boolean) => void } };
  maybeRuntime.Bun?.gc?.(true);
}

function isMissingExecutable(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}
