import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import {
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { basename, relative, resolve, sep } from "node:path";

import type { MediaFileRecord } from "./model";

const CAPTURE_INPUT_PATHS = [
  "Cargo.lock",
  "Cargo.toml",
  "rust-toolchain.toml",
  "crates/liquidfun/src",
  "crates/liquidfun-wasm",
  "web/bun.lock",
  "web/package.json",
  "web/src",
  "web/vite.config.ts",
  "web/scripts/demo-media.ts",
  "web/scripts/demo-media",
] as const;

const README_FILE_PATTERN = /^README(?:\..+)?$/i;

type FfprobeStream = {
  readonly codec_name?: unknown;
  readonly width?: unknown;
  readonly height?: unknown;
  readonly avg_frame_rate?: unknown;
  readonly nb_frames?: unknown;
};

type FfprobeFormat = {
  readonly duration?: unknown;
};

type FfprobeResponse = {
  readonly streams?: readonly FfprobeStream[];
  readonly format?: FfprobeFormat;
};

export type MediaProbe = {
  readonly codecName: string;
  readonly width: number;
  readonly height: number;
  readonly averageFrameRate: string;
  readonly maybeFrameCount: number | null;
  readonly durationSeconds: number;
};
export type MediaDimensions = {
  readonly width: number;
  readonly height: number;
};

export type RenamePath = (from: string, to: string) => Promise<void>;
export type RemoveDirectory = (path: string) => Promise<void>;
export type ReplaceOutputDirectoryDependencies = {
  readonly renamePath?: RenamePath;
  readonly removeDirectory?: RemoveDirectory;
};

export async function captureInputSha256(repoRoot: string): Promise<string> {
  const trackedPaths = trackedCaptureInputPaths(repoRoot);
  const hash = createHash("sha256");

  for (const relativePath of trackedPaths) {
    const bytes = await readFile(resolve(repoRoot, relativePath));
    hash.update(relativePath);
    hash.update("\0");
    hash.update(String(bytes.length));
    hash.update("\0");
    hash.update(bytes);
  }

  return hash.digest("hex");
}

export function mp4Arguments(
  framesDirectory: string,
  outputPath: string,
): readonly string[] {
  return [
    "-hide_banner",
    "-loglevel",
    "error",
    "-framerate",
    "30",
    "-i",
    resolve(framesDirectory, "frame-%04d.png"),
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
    "-vf",
    "pad=ceil(iw/2)*2:ceil(ih/2)*2",
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
  ];
}

export function padToEvenDimensions(
  dimensions: MediaDimensions,
): MediaDimensions {
  return {
    width: padDimensionToEven(dimensions.width),
    height: padDimensionToEven(dimensions.height),
  };
}

export function webpArguments(
  framesDirectory: string,
  outputPath: string,
): readonly string[] {
  return [
    "-hide_banner",
    "-loglevel",
    "error",
    "-framerate",
    "30",
    "-i",
    resolve(framesDirectory, "frame-%04d.png"),
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
  ];
}

export async function compareOutputDirectories(
  expected: string,
  actual: string,
): Promise<readonly string[]> {
  const expectedFiles = await listRelativeFiles(expected);
  const actualFiles = await listRelativeFiles(actual);
  const diagnostics: string[] = [];
  const relativePaths = [...new Set([...expectedFiles, ...actualFiles])].sort();

  for (const relativePath of relativePaths) {
    const expectedPath = resolve(expected, relativePath);
    const actualPath = resolve(actual, relativePath);
    const expectedExists = expectedFiles.includes(relativePath);
    const actualExists = actualFiles.includes(relativePath);

    if (expectedExists && !actualExists) {
      diagnostics.push(`missing: ${relativePath}`);
      continue;
    }
    if (!expectedExists && actualExists) {
      diagnostics.push(`extra: ${relativePath}`);
      continue;
    }

    const [expectedBytes, actualBytes] = await Promise.all([
      readFile(expectedPath),
      readFile(actualPath),
    ]);
    if (
      expectedBytes.length !== actualBytes.length ||
      sha256(expectedBytes) !== sha256(actualBytes)
    ) {
      diagnostics.push(`changed: ${relativePath}`);
    }
  }

  return diagnostics;
}

export async function replaceOutputDirectory(
  next: string,
  target: string,
  renamePathOrDependencies: RenamePath | ReplaceOutputDirectoryDependencies = {},
): Promise<readonly string[]> {
  const dependencies =
    typeof renamePathOrDependencies === "function"
      ? { renamePath: renamePathOrDependencies }
      : renamePathOrDependencies;
  const renamePath = dependencies.renamePath ?? rename;
  const removeDirectory = dependencies.removeDirectory ?? removeDirectoryTree;
  const targetExists = await pathExists(target);
  if (!targetExists) {
    await renamePath(next, target);
    return [];
  }

  const backupPath = `${target}.backup-${process.pid}`;
  await renamePath(target, backupPath);

  try {
    await renamePath(next, target);
  } catch (error) {
    await renamePath(backupPath, target);
    throw error;
  }

  try {
    await removeDirectory(backupPath);
    return [];
  } catch (error) {
    return [
      `Installed ${target} but could not remove backup ${backupPath}: ${errorMessage(error)}`,
    ];
  }
}

export function commandVersionLine(
  command: string,
  installHint?: string,
): string {
  const result = spawnSync(command, ["-version"], {
    encoding: "utf8",
  });
  if (result.error !== undefined) {
    const suffix = installHint === undefined ? "" : ` ${installHint}`;
    throw new Error(
      `${command} is unavailable or unusable: ${result.error.message}.${suffix}`.trim(),
    );
  }
  if (result.status !== 0) {
    const diagnostic = result.stderr?.trim() ?? "";
    throw new Error(
      `${command} -version failed (${result.status})${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
    );
  }

  const [firstLine] = (result.stdout ?? "").split("\n");
  if (firstLine === undefined || firstLine.trim().length === 0) {
    throw new Error(`${command} -version did not report a version line`);
  }
  return firstLine.trim();
}

export async function ensureDirectory(path: string): Promise<void> {
  await mkdir(path, { recursive: true });
}

export async function ensureRegularFile(path: string, label: string): Promise<void> {
  const fileStatus = await stat(path).catch((error: unknown) => {
    if (isMissingPathError(error)) {
      throw new Error(`${label} is missing: ${path}`);
    }
    throw error;
  });
  if (!fileStatus.isFile() || fileStatus.size === 0) {
    throw new Error(`${label} is missing or empty: ${path}`);
  }
}

export async function captureMediaFileRecord(
  repoRoot: string,
  path: string,
): Promise<MediaFileRecord> {
  const bytes = await readFile(path);
  return {
    path: normalizeRelativePath(repoRoot, path),
    sha256: sha256(bytes),
    bytes: bytes.length,
  };
}

export async function probeMedia(path: string): Promise<MediaProbe> {
  const result = spawnSync(
    "ffprobe",
    [
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=codec_name,width,height,avg_frame_rate,nb_frames",
      "-show_entries",
      "format=duration",
      "-of",
      "json",
      path,
    ],
    {
      encoding: "utf8",
    },
  );
  if (result.error !== undefined) {
    throw new Error(
      `failed to spawn ffprobe for ${path}: ${result.error.message}`,
    );
  }
  if (result.status !== 0) {
    const diagnostic = result.stderr?.trim() ?? "";
    throw new Error(
      `ffprobe failed for ${path} (${result.status})${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
    );
  }
  return parseMediaProbeJson(path, result.stdout);
}

export function parseMediaProbeJson(
  path: string,
  jsonText: string,
): MediaProbe {
  const response = JSON.parse(jsonText) as FfprobeResponse;
  const maybeStream = response.streams?.[0];
  if (maybeStream === undefined) {
    throw new Error(`ffprobe did not report a video stream for ${path}`);
  }

  const codecName = requireString(maybeStream.codec_name, "codec_name", path);
  const width = requirePositiveInteger(maybeStream.width, "width", path);
  const height = requirePositiveInteger(maybeStream.height, "height", path);
  const averageFrameRate = requireString(
    maybeStream.avg_frame_rate,
    "avg_frame_rate",
    path,
  );
  parseAverageFrameRate(averageFrameRate, path);
  const maybeFrameCount = parseMaybeFrameCount(maybeStream.nb_frames, path);
  const durationSeconds = requirePositiveNumber(
    response.format?.duration,
    "duration",
    path,
  );

  return {
    codecName,
    width,
    height,
    averageFrameRate,
    maybeFrameCount,
    durationSeconds,
  };
}

export function validateMp4Probe(path: string, probe: MediaProbe): void {
  validateCommonProbe(path, probe);
  if (probe.codecName !== "h264") {
    throw new Error(`Expected H.264 MP4 output, found ${probe.codecName} for ${path}`);
  }
}

export function validateWebpProbe(path: string, probe: MediaProbe): void {
  validateCommonProbe(path, probe);
  if (probe.codecName !== "webp") {
    throw new Error(`Expected WebP output, found ${probe.codecName} for ${path}`);
  }
  if (probe.width !== 640) {
    throw new Error(`Expected 640px WebP width, found ${probe.width} for ${path}`);
  }
}

export async function writeTextAtomically(
  path: string,
  contents: string,
): Promise<void> {
  const temporaryPath = `${path}.tmp-${process.pid}`;
  await writeFile(temporaryPath, contents, { flag: "wx" });
  await rename(temporaryPath, path);
}

function trackedCaptureInputPaths(repoRoot: string): readonly string[] {
  const result = spawnSync(
    "git",
    ["ls-files", "-z", "--", ...CAPTURE_INPUT_PATHS],
    {
      cwd: repoRoot,
      encoding: "utf8",
    },
  );
  if (result.status !== 0) {
    const diagnostic = result.stderr?.trim() ?? "";
    throw new Error(
      `git ls-files failed (${result.status})${diagnostic.length === 0 ? "" : `: ${diagnostic}`}`,
    );
  }

  return result.stdout
    .split("\0")
    .filter((path) => path.length > 0)
    .map(normalizePathSeparators)
    .filter((path) => !isExcludedCaptureInput(path))
    .sort();
}

async function listRelativeFiles(root: string): Promise<readonly string[]> {
  const filePaths: string[] = [];
  await collectRelativeFiles(root, root, filePaths);
  filePaths.sort();
  return filePaths;
}

async function collectRelativeFiles(
  root: string,
  currentDirectory: string,
  output: string[],
): Promise<void> {
  let entries;
  try {
    entries = await readdir(currentDirectory, { withFileTypes: true });
  } catch (error) {
    if (currentDirectory === root && isMissingPathError(error)) {
      return;
    }
    throw error;
  }

  for (const entry of [...entries].sort((left, right) => left.name.localeCompare(right.name))) {
    const absolutePath = resolve(currentDirectory, entry.name);
    if (entry.isDirectory()) {
      await collectRelativeFiles(root, absolutePath, output);
      continue;
    }
    if (entry.isFile()) {
      output.push(normalizeRelativePath(root, absolutePath));
    }
  }
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

function isExcludedCaptureInput(path: string): boolean {
  if (path.startsWith("web/src/generated/")) {
    return true;
  }
  if (path.startsWith("docs/assets/demos/")) {
    return true;
  }
  if (path.startsWith("target/")) {
    return true;
  }
  return README_FILE_PATTERN.test(basename(path));
}

function normalizeRelativePath(root: string, path: string): string {
  return normalizePathSeparators(relative(root, path));
}

function normalizePathSeparators(path: string): string {
  return path.split(sep).join("/");
}

async function pathExists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch (error) {
    if (isMissingPathError(error)) {
      return false;
    }
    throw error;
  }
}

function isMissingPathError(error: unknown): boolean {
  return (
    error instanceof Error &&
    "code" in error &&
    error.code === "ENOENT"
  );
}

async function removeDirectoryTree(path: string): Promise<void> {
  await rm(path, { recursive: true, force: true });
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function requireString(value: unknown, label: string, path: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`ffprobe ${label} is invalid for ${path}`);
  }
  return value;
}

function requirePositiveInteger(
  value: unknown,
  label: string,
  path: string,
): number {
  if (typeof value !== "number" || !Number.isInteger(value) || value <= 0) {
    throw new Error(`ffprobe ${label} is invalid for ${path}`);
  }
  return value;
}

function requirePositiveNumber(
  value: unknown,
  label: string,
  path: string,
): number {
  const parsed =
    typeof value === "number"
      ? value
      : typeof value === "string"
        ? Number(value)
        : Number.NaN;
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`ffprobe ${label} is invalid for ${path}`);
  }
  return parsed;
}

function parseMaybeFrameCount(value: unknown, path: string): number | null {
  if (value === undefined || value === "N/A") {
    return null;
  }
  if (typeof value === "string" && /^(0|[1-9]\d*)$/.test(value)) {
    return Number(value);
  }
  if (typeof value === "number" && Number.isInteger(value) && value >= 0) {
    return value;
  }
  throw new Error(`ffprobe nb_frames is invalid for ${path}`);
}

function validateCommonProbe(path: string, probe: MediaProbe): void {
  const averageFrameRate = parseAverageFrameRate(probe.averageFrameRate, path);
  if (Math.abs(averageFrameRate - 30) > Number.EPSILON) {
    throw new Error(`Expected 30 fps output, found ${probe.averageFrameRate} for ${path}`);
  }
  if (probe.maybeFrameCount !== null && probe.maybeFrameCount !== 240) {
    throw new Error(
      `Expected 240 frames when reported, found ${probe.maybeFrameCount} for ${path}`,
    );
  }
  const frameToleranceSeconds = 1 / 30;
  if (Math.abs(probe.durationSeconds - 8) > frameToleranceSeconds) {
    throw new Error(
      `Expected ~8 second duration, found ${probe.durationSeconds} for ${path}`,
    );
  }
}

function padDimensionToEven(value: number): number {
  if (!Number.isInteger(value) || value <= 0) {
    throw new Error(`Expected a positive integer media dimension, got ${value}`);
  }
  return value % 2 === 0 ? value : value + 1;
}

function parseAverageFrameRate(value: string, path: string): number {
  const [numeratorText, denominatorText] = value.split("/");
  if (numeratorText === undefined || denominatorText === undefined) {
    throw new Error(`ffprobe avg_frame_rate is invalid for ${path}`);
  }
  const numerator = Number(numeratorText);
  const denominator = Number(denominatorText);
  if (!Number.isFinite(numerator) || !Number.isFinite(denominator) || denominator === 0) {
    throw new Error(`ffprobe avg_frame_rate is invalid for ${path}`);
  }
  return numerator / denominator;
}
