import { spawnSync } from "node:child_process";

export type WebpProbe = {
  readonly codecName: string;
  readonly width: number;
  readonly height: number;
  readonly averageFrameRate: string;
  readonly maybeFrameCount: number | null;
  readonly durationSeconds: number;
};

export async function probeWebp(path: string): Promise<WebpProbe> {
  const result = spawnSync("webpmux", ["-info", path], {
    encoding: "utf8",
  });
  if (result.error !== undefined) {
    throw new Error(
      `failed to spawn webpmux for ${path}: ${result.error.message}. Install WebP tools so \`webpmux\` is available on PATH before running demo-media generation.`,
    );
  }
  if (result.status !== 0) {
    const diagnostic = result.stderr?.trim() ?? "";
    throw new Error(
      `webpmux -info failed for ${path} (${result.status})${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
    );
  }
  return parseWebpMuxInfo(path, result.stdout);
}

export function parseWebpMuxInfo(
  path: string,
  infoText: string,
): WebpProbe {
  const canvasMatch = infoText.match(/^Canvas size:\s+(\d+)\s+x\s+(\d+)$/m);
  const frameCountMatch = infoText.match(/^Number of frames:\s+(\d+)$/m);
  if (canvasMatch === null) {
    throw new Error(`webpmux canvas size is invalid for ${path}`);
  }
  if (frameCountMatch === null) {
    throw new Error(`webpmux frame count is invalid for ${path}`);
  }

  const width = Number(canvasMatch[1]);
  const height = Number(canvasMatch[2]);
  const maybeFrameCount = Number(frameCountMatch[1]);
  if (!Number.isInteger(width) || width <= 0) {
    throw new Error(`webpmux width is invalid for ${path}`);
  }
  if (!Number.isInteger(height) || height <= 0) {
    throw new Error(`webpmux height is invalid for ${path}`);
  }
  if (!Number.isInteger(maybeFrameCount) || maybeFrameCount <= 0) {
    throw new Error(`webpmux frame count is invalid for ${path}`);
  }

  const frameLineMatches = [...infoText.matchAll(
    /^\s*\d+:\s+\d+\s+\d+\s+\S+\s+\d+\s+\d+\s+(\d+)\s+/gm,
  )];
  if (frameLineMatches.length === 0) {
    throw new Error(`webpmux frame durations are invalid for ${path}`);
  }

  const frameDurationsMilliseconds = frameLineMatches.map((match) =>
    Number(match[1]),
  );
  if (
    frameDurationsMilliseconds.some((durationMilliseconds) =>
      !Number.isInteger(durationMilliseconds) || durationMilliseconds <= 0
    )
  ) {
    throw new Error(`webpmux frame durations are invalid for ${path}`);
  }

  if (frameDurationsMilliseconds.length !== maybeFrameCount) {
    throw new Error(
      `webpmux frame table length ${frameDurationsMilliseconds.length} does not match declared frame count ${maybeFrameCount} for ${path}`,
    );
  }

  const observedDurationMilliseconds = frameDurationsMilliseconds.reduce(
    (sum, durationMilliseconds) => sum + durationMilliseconds,
    0,
  );
  const averageFrameRate = rationalFrameRate(
    maybeFrameCount * 1_000,
    observedDurationMilliseconds,
  );
  const durationSeconds = observedDurationMilliseconds / 1_000;

  return {
    codecName: "webp",
    width,
    height,
    averageFrameRate,
    maybeFrameCount,
    durationSeconds,
  };
}

function rationalFrameRate(
  numerator: number,
  denominator: number,
): string {
  if (
    !Number.isInteger(numerator) ||
    !Number.isInteger(denominator) ||
    numerator <= 0 ||
    denominator <= 0
  ) {
    throw new Error("frame rate rational inputs must be positive integers");
  }
  const divisor = greatestCommonDivisor(numerator, denominator);
  return `${numerator / divisor}/${denominator / divisor}`;
}

function greatestCommonDivisor(left: number, right: number): number {
  let currentLeft = left;
  let currentRight = right;
  while (currentRight !== 0) {
    const remainder = currentLeft % currentRight;
    currentLeft = currentRight;
    currentRight = remainder;
  }
  return currentLeft;
}
