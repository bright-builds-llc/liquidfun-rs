import { describe, expect, it } from "vitest";

import { startAnimatedWebpEncoder } from "../scripts/readme-svg/encode";
import {
  assertReadmeWebpByteLimit,
  describeAnimatedWebp,
  frameCountForDuration,
  frameDurationsMs,
  muxAnimatedWebp,
  prepareRecordedWebp,
  README_WEBP_BYTE_LIMIT,
  README_WEBP_FPS,
} from "../scripts/readme-svg/webp";

describe("frameDurationsMs", () => {
  it("averages 60 fps across the 10 second README loop", () => {
    // Arrange
    const frameCount = frameCountForDuration(10, README_WEBP_FPS);

    // Act
    const durations = frameDurationsMs(frameCount, README_WEBP_FPS);

    // Assert
    expect(frameCount).toBe(600);
    expect(durations[0]).toBe(17);
    expect(durations[1]).toBe(16);
    expect(durations.every((duration) => duration === 16 || duration === 17)).toBe(true);
    expect(durations.reduce((sum, duration) => sum + duration, 0)).toBe(10_000);
  });
});

describe("assertReadmeWebpByteLimit", () => {
  it("allows a raster under GitHub's 50MB file warning", () => {
    // Arrange
    const byteLength = README_WEBP_BYTE_LIMIT - 1;

    // Act
    const check = () => assertReadmeWebpByteLimit("fountain", byteLength);

    // Assert
    expect(check).not.toThrow();
  });

  it("rejects a raster that reaches GitHub's 50MB file warning", () => {
    // Arrange
    const byteLength = README_WEBP_BYTE_LIMIT;

    // Act
    const check = () => assertReadmeWebpByteLimit("fountain", byteLength);

    // Assert
    expect(check).toThrow("50MB file warning");
  });
});

describe("muxAnimatedWebp", () => {
  it("stores full-canvas frames with the requested millisecond durations", () => {
    // Arrange
    const frames = [stillWebp(Buffer.from([1, 2, 3, 4])), stillWebp(Buffer.from([5]))];
    const durationsMs = [17, 16];

    // Act
    const animated = muxAnimatedWebp(frames, durationsMs, 16, 8);
    const summary = describeAnimatedWebp(animated);

    // Assert
    expect(summary).toEqual({
      width: 16,
      height: 8,
      loopCount: 0,
      durationsMs: [17, 16],
    });
  });

  it("rejects a still that is not a single VP8 image", () => {
    // Arrange
    const frames = [Buffer.from("not a webp")];

    // Act
    const mux = () => muxAnimatedWebp(frames, [17], 16, 8);

    // Assert
    expect(mux).toThrow("still WebP");
  });
});

describe("startAnimatedWebpEncoder", () => {
  it("records RGBA frames as one looping 60 fps WebP", async () => {
    // Arrange
    const width = 16;
    const height = 8;
    const frameCount = 6;
    const encoder = await startAnimatedWebpEncoder({
      width,
      height,
      frameCount,
      framesPerSecond: 60,
      quality: 60,
      preset: "drawing",
    });
    const pixels = Buffer.alloc(width * height * 4, 255);

    // Act
    for (let index = 0; index < frameCount; index += 1) {
      pixels[0] = index;
      await encoder.writeFrame(pixels);
    }
    const webp = await encoder.finish();
    const summary = describeAnimatedWebp(webp);

    // Assert
    expect(summary).toEqual({
      width,
      height,
      loopCount: 0,
      durationsMs: [...frameDurationsMs(frameCount, 60)],
    });
    expect(replaceFrameFlags(webp)).toEqual({
      canvasFlags: 0x02,
      background: [0x18, 0x10, 0x07, 0xff],
      frameFlags: [0b11, 0b11, 0b11, 0b11, 0b11, 0b11],
    });
  });
});

describe("prepareRecordedWebp", () => {
  it("turns blended ffmpeg frames into opaque canvas replacements", () => {
    // Arrange
    const animated = muxAnimatedWebp(
      [stillWebp(Buffer.from([1, 2, 3, 4])), stillWebp(Buffer.from([5]))],
      [17, 16],
      16,
      8,
    );
    const blended = Buffer.from(animated);
    blended[20] = 0x12;
    blended[38] = 0xff;
    blended[39] = 0xff;
    blended[40] = 0xff;
    blended[41] = 0xff;
    blended[67] = 0;

    // Act
    const prepared = prepareRecordedWebp(blended);

    // Assert
    expect(prepared).toEqual(animated);
  });
});

function replaceFrameFlags(bytes: Buffer): {
  readonly canvasFlags: number;
  readonly background: readonly number[];
  readonly frameFlags: readonly number[];
} {
  let offset = 12;
  let canvasFlags = 0;
  const background: number[] = [];
  const frameFlags: number[] = [];
  while (offset + 8 <= bytes.length) {
    const tag = bytes.toString("ascii", offset, offset + 4);
    const size = bytes.readUInt32LE(offset + 4);
    const payload = offset + 8;
    if (tag === "VP8X") {
      canvasFlags = bytes[payload] ?? 0;
    }
    if (tag === "ANIM") {
      background.push(...bytes.subarray(payload, payload + 4));
    }
    if (tag === "ANMF") {
      frameFlags.push(bytes[payload + 15] ?? 0);
    }
    offset = payload + size + (size % 2);
  }
  return { canvasFlags, background, frameFlags };
}

function stillWebp(payload: Buffer): Buffer {
  const chunk = Buffer.concat([
    Buffer.from("VP8 "),
    uint32(payload.length),
    payload,
    payload.length % 2 === 0 ? Buffer.alloc(0) : Buffer.from([0]),
  ]);
  const body = Buffer.concat([Buffer.from("WEBP"), chunk]);
  return Buffer.concat([Buffer.from("RIFF"), uint32(body.length), body]);
}

function uint32(value: number): Buffer {
  const bytes = Buffer.alloc(4);
  bytes.writeUInt32LE(value);
  return bytes;
}
