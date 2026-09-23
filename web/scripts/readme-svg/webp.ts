/**
 * Animated WebP muxing for README previews.
 *
 * WebP stores each frame duration as a whole number of milliseconds. A steady
 * 60 fps clock is 16.666... ms, so frame boundaries are rounded onto the
 * millisecond timeline. Over a 10 second, 600-frame loop the durations are a
 * mix of 16 ms and 17 ms and the total is exactly 10000 ms.
 */

/** Playback rate of the README rasters. */
export const README_WEBP_FPS = 60;

/**
 * libwebp quality for the README rasters.
 *
 * 100 is the encoder maximum. The fountain spray is the largest clip: at
 * quality 60 the full 1280 by 960 file stays under 30MB, while a quality 80
 * sample of that spray extrapolated past GitHub's 50MB file warning.
 */
export const README_WEBP_QUALITY = 60;

/** GitHub warns on files at or above 50MiB. README rasters stay under that. */
export const README_WEBP_BYTE_LIMIT = 50 * 1024 * 1024;

/** libwebp preset suited to the thin wireframe strokes in the scene SVGs. */
export const README_WEBP_PRESET = "drawing";

const WEBP_PRESETS = ["none", "default", "picture", "photo", "drawing", "icon", "text"] as const;

/** Dispose to the background and draw this frame opaquely over the canvas. */
const ANMF_REPLACE_FRAME = 0b11;

/** VP8X feature bit that marks an animated canvas. */
const VP8X_ANIMATION_FLAG = 0b10;

/** Byte length of the ANMF frame header before the image chunk. */
const ANMF_HEADER_BYTES = 16;

type WebpChunk = {
  readonly tag: string;
  readonly payload: Uint8Array;
};

export type AnimatedWebpSummary = {
  readonly width: number;
  readonly height: number;
  readonly loopCount: number;
  readonly durationsMs: readonly number[];
};

/** Millisecond frame durations whose average rate is `framesPerSecond`. */
export function frameDurationsMs(
  frameCount: number,
  framesPerSecond: number,
): readonly number[] {
  assertPositiveInteger(frameCount, "frame count");
  assertPositiveInteger(framesPerSecond, "frame rate");

  const durations: number[] = [];
  let previous = 0;
  for (let index = 1; index <= frameCount; index += 1) {
    const boundary = Math.round((index * 1000) / framesPerSecond);
    const duration = boundary - previous;
    if (duration <= 0 || duration > 0xffffff) {
      throw new Error(`Frame ${index} duration ${duration} ms is outside the WebP range.`);
    }
    durations.push(duration);
    previous = boundary;
  }
  return durations;
}

/** Whole frames covering `durationSeconds` at a constant rate. */
export function frameCountForDuration(durationSeconds: number, framesPerSecond: number): number {
  assertPositiveInteger(framesPerSecond, "frame rate");
  if (!Number.isFinite(durationSeconds) || durationSeconds <= 0) {
    throw new Error("WebP duration must be a positive finite number of seconds.");
  }

  const frameCount = Math.round(durationSeconds * framesPerSecond);
  assertPositiveInteger(frameCount, "frame count");
  return frameCount;
}

/** Rejects a raster that would trip GitHub's large-file warning. */
export function assertReadmeWebpByteLimit(sceneId: string, byteLength: number): void {
  if (byteLength < README_WEBP_BYTE_LIMIT) {
    return;
  }

  throw new Error(
    `${sceneId} WebP is ${byteLength} bytes, which reaches GitHub's 50MB file warning.`,
  );
}

/** True when `preset` is a libwebp preset name ffmpeg accepts. */
export function isWebpPreset(preset: string): preset is (typeof WEBP_PRESETS)[number] {
  return WEBP_PRESETS.some((candidate) => candidate === preset);
}

/**
 * Wraps independent still WebP files into one looping animation.
 *
 * Each still must be a single VP8 or VP8L image, which is what ffmpeg writes
 * for a lossy WebP frame. Every frame covers the full canvas.
 */
export function muxAnimatedWebp(
  frames: readonly Uint8Array[],
  durationsMs: readonly number[],
  width: number,
  height: number,
): Buffer {
  if (frames.length === 0) {
    throw new Error("Animated WebP needs at least one frame.");
  }
  if (frames.length !== durationsMs.length) {
    throw new Error(
      `Animated WebP has ${frames.length} frames and ${durationsMs.length} durations.`,
    );
  }
  assertCanvasSize(width, height);

  const parts: Buffer[] = [Buffer.from("WEBP")];
  parts.push(riffChunk("VP8X", vp8xPayload(width, height)));
  parts.push(riffChunk("ANIM", animPayload()));
  for (let index = 0; index < frames.length; index += 1) {
    const frame = frames[index];
    const durationMs = durationsMs[index];
    if (frame === undefined || durationMs === undefined) {
      throw new Error(`Animated WebP is missing frame ${index}.`);
    }
    if (!Number.isInteger(durationMs) || durationMs <= 0 || durationMs > 0xffffff) {
      throw new Error(`Frame ${index} duration ${durationMs} ms is outside the WebP range.`);
    }
    const payload = Buffer.concat([anmfHeader(width, height, durationMs), imageChunks(frame)]);
    parts.push(riffChunk("ANMF", payload));
  }

  const body = Buffer.concat(parts);
  return Buffer.concat([Buffer.from("RIFF"), u32(body.length), body]);
}

/** Reads the canvas size and per-frame durations back out of an animated WebP. */
export function describeAnimatedWebp(bytes: Uint8Array): AnimatedWebpSummary {
  if (ascii(bytes, 0, 4) !== "RIFF" || ascii(bytes, 8, 4) !== "WEBP") {
    throw new Error("Animated WebP is missing the RIFF/WEBP header.");
  }
  const declared = readU32(bytes, 4);
  if (declared + 8 !== bytes.length) {
    throw new Error("Animated WebP RIFF size does not match the buffer.");
  }

  const chunks = readChunks(bytes, 12, bytes.length);
  const canvas = chunks[0];
  const animation = chunks[1];
  if (canvas === undefined || canvas.tag !== "VP8X") {
    throw new Error("Animated WebP is missing the VP8X canvas.");
  }
  if (animation === undefined || animation.tag !== "ANIM") {
    throw new Error("Animated WebP is missing the ANIM chunk.");
  }

  const { width, height } = readCanvas(canvas.payload);
  const loopCount = readLoopCount(animation.payload);
  const durationsMs = chunks.slice(2).map((chunk, index) => readFrameDuration(chunk, index, width, height));
  if (durationsMs.length === 0) {
    throw new Error("Animated WebP has no frames.");
  }
  return { width, height, loopCount, durationsMs };
}

function readCanvas(payload: Uint8Array): { readonly width: number; readonly height: number } {
  if (payload.length !== 10) {
    throw new Error("VP8X canvas chunk has an unexpected size.");
  }
  const flags = payload[0] ?? 0;
  if ((flags & VP8X_ANIMATION_FLAG) === 0) {
    throw new Error("VP8X canvas is not marked as an animation.");
  }
  const width = readU24(payload, 4) + 1;
  const height = readU24(payload, 7) + 1;
  assertCanvasSize(width, height);
  return { width, height };
}

function readLoopCount(payload: Uint8Array): number {
  if (payload.length !== 6) {
    throw new Error("ANIM chunk has an unexpected size.");
  }
  return readU16(payload, 4);
}

function readFrameDuration(
  chunk: WebpChunk,
  index: number,
  width: number,
  height: number,
): number {
  if (chunk.tag !== "ANMF") {
    throw new Error(`Animated WebP frame ${index} is a ${chunk.tag} chunk.`);
  }
  if (chunk.payload.length < ANMF_HEADER_BYTES) {
    throw new Error(`Animated WebP frame ${index} header is truncated.`);
  }
  if (readU24(chunk.payload, 0) !== 0 || readU24(chunk.payload, 3) !== 0) {
    throw new Error(`Animated WebP frame ${index} is not anchored at the origin.`);
  }
  if (readU24(chunk.payload, 6) + 1 !== width || readU24(chunk.payload, 9) + 1 !== height) {
    throw new Error(`Animated WebP frame ${index} does not cover the canvas.`);
  }
  const inner = readChunks(chunk.payload, ANMF_HEADER_BYTES, chunk.payload.length);
  const image = inner[0];
  if (image === undefined || (image.tag !== "VP8 " && image.tag !== "VP8L")) {
    throw new Error(`Animated WebP frame ${index} is missing a VP8 image.`);
  }
  return readU24(chunk.payload, 12);
}

function imageChunks(still: Uint8Array): Buffer {
  if (ascii(still, 0, 4) !== "RIFF" || ascii(still, 8, 4) !== "WEBP") {
    throw new Error("Expected a still WebP RIFF document.");
  }
  const declared = readU32(still, 4);
  if (declared + 8 !== still.length) {
    throw new Error("Still WebP size does not match its RIFF length.");
  }
  const tag = ascii(still, 12, 4);
  if (tag !== "VP8 " && tag !== "VP8L") {
    throw new Error(`Still WebP must contain one VP8 image, found ${tag}.`);
  }
  return Buffer.from(still.subarray(12));
}

function vp8xPayload(width: number, height: number): Buffer {
  const payload = Buffer.alloc(10);
  payload[0] = VP8X_ANIMATION_FLAG;
  writeU24(payload, 4, width - 1);
  writeU24(payload, 7, height - 1);
  return payload;
}

function animPayload(): Buffer {
  // BGRA background matches the SVG fill #071018. Loop count 0 repeats forever.
  return Buffer.from([0x18, 0x10, 0x07, 0xff, 0x00, 0x00]);
}

function anmfHeader(width: number, height: number, durationMs: number): Buffer {
  const header = Buffer.alloc(ANMF_HEADER_BYTES);
  writeU24(header, 6, width - 1);
  writeU24(header, 9, height - 1);
  writeU24(header, 12, durationMs);
  header[15] = ANMF_REPLACE_FRAME;
  return header;
}

function riffChunk(tag: string, payload: Buffer): Buffer {
  const header = Buffer.concat([Buffer.from(tag), u32(payload.length)]);
  if (payload.length % 2 === 0) {
    return Buffer.concat([header, payload]);
  }
  return Buffer.concat([header, payload, Buffer.from([0])]);
}

function readChunks(bytes: Uint8Array, start: number, end: number): WebpChunk[] {
  const chunks: WebpChunk[] = [];
  let offset = start;
  while (offset + 8 <= end) {
    const tag = ascii(bytes, offset, 4);
    const size = readU32(bytes, offset + 4);
    const payloadEnd = offset + 8 + size;
    if (payloadEnd > end) {
      throw new Error(`WebP chunk ${tag} exceeds its parent.`);
    }
    chunks.push({ tag, payload: bytes.subarray(offset + 8, payloadEnd) });
    offset = payloadEnd + (size % 2);
  }
  if (offset !== end) {
    throw new Error("WebP chunk stream did not end on a chunk boundary.");
  }
  return chunks;
}

function assertCanvasSize(width: number, height: number): void {
  assertPositiveInteger(width, "canvas width");
  assertPositiveInteger(height, "canvas height");
  if (width > 0xffffff || height > 0xffffff) {
    throw new Error(`Canvas ${width}x${height} exceeds the WebP size limit.`);
  }
}

function assertPositiveInteger(value: number, label: string): void {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`WebP ${label} must be a positive integer.`);
  }
}

function ascii(bytes: Uint8Array, offset: number, length: number): string {
  return Buffer.from(bytes.subarray(offset, offset + length)).toString("ascii");
}

function u32(value: number): Buffer {
  const bytes = Buffer.alloc(4);
  bytes.writeUInt32LE(value);
  return bytes;
}

function readU32(bytes: Uint8Array, offset: number): number {
  return Buffer.from(bytes.subarray(offset, offset + 4)).readUInt32LE(0);
}

function readU24(bytes: Uint8Array, offset: number): number {
  return (bytes[offset] ?? 0) | ((bytes[offset + 1] ?? 0) << 8) | ((bytes[offset + 2] ?? 0) << 16);
}

function readU16(bytes: Uint8Array, offset: number): number {
  return (bytes[offset] ?? 0) | ((bytes[offset + 1] ?? 0) << 8);
}

function writeU24(bytes: Buffer, offset: number, value: number): void {
  bytes[offset] = value & 0xff;
  bytes[offset + 1] = (value >> 8) & 0xff;
  bytes[offset + 2] = (value >> 16) & 0xff;
}
