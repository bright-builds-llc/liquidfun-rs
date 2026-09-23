import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { parseAnimatedSvg, sampleAnimatedSvg } from "../scripts/readme-svg/smil";
import {
  assertReadmeWebpByteLimit,
  describeAnimatedWebp,
  frameCountForDuration,
  frameDurationsMs,
  muxAnimatedWebp,
  README_WEBP_BYTE_LIMIT,
  README_WEBP_FPS,
} from "../scripts/readme-svg/webp";

const SVG = [
  '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20">',
  "<title>Drop</title>",
  '<rect width="100%" height="100%" fill="#071018"/>',
  '<circle cx="0" cy="1" r="0.5" fill="none" stroke="rgba(0,0,0,1)" stroke-width="0.3">',
  '<animate attributeName="cx" values="0;10" dur="10s" repeatCount="indefinite" calcMode="linear"/>',
  '<animate attributeName="stroke" values="rgba(0,0,0,1);rgba(10,20,30,0)" dur="10s" repeatCount="indefinite" calcMode="linear"/>',
  '<animate attributeName="opacity" values="0;1" dur="10s" repeatCount="indefinite" calcMode="linear"/>',
  "</circle>",
  '<text x="1" y="1" font-family="sans-serif">A&amp;B</text>',
  "</svg>",
].join("");

describe("sampleAnimatedSvg", () => {
  it("returns the first keyframe at the start of the loop", () => {
    // Arrange
    const timeSeconds = 0;

    // Act
    const sampled = sampleAnimatedSvg(SVG, timeSeconds);

    // Assert
    expect(sampled).toBe(
      [
        '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 20 20">',
        "<title>Drop</title>",
        '<rect width="100%" height="100%" fill="#071018"/>',
        '<circle cx="0" cy="1" r="0.5" fill="none" stroke="rgba(0,0,0,1)" stroke-width="0.3" opacity="0"/>',
        '<text x="1" y="1" font-family="sans-serif">A&amp;B</text>',
        "</svg>",
      ].join(""),
    );
  });

  it("interpolates numbers and colors halfway through a two-value animation", () => {
    // Arrange
    const timeSeconds = 5;

    // Act
    const sampled = sampleAnimatedSvg(SVG, timeSeconds);

    // Assert
    expect(sampled).toContain('cx="5"');
    expect(sampled).toContain('stroke="rgba(5,10,15,0.5)"');
    expect(sampled).toContain('opacity="0.5"');
    expect(sampled).not.toContain("<animate");
  });

  it("restarts at the first keyframe when the duration elapses", () => {
    // Arrange
    const start = sampleAnimatedSvg(SVG, 0);

    // Act
    const looped = sampleAnimatedSvg(SVG, 10);

    // Assert
    expect(looped).toBe(start);
  });

  it("keeps an exact later keyframe token", () => {
    // Arrange
    const source = SVG.replace('values="0;10"', 'values="0;1.5;3"');

    // Act
    const sampled = sampleAnimatedSvg(source, 5);

    // Assert
    expect(sampled).toContain('cx="1.5"');
  });

  it("rewrites sans-serif labels to the pinned raster font when asked", () => {
    // Arrange
    const fontFamily = "DejaVu Sans";

    // Act
    const sampled = sampleAnimatedSvg(SVG, 0, { fontFamily });

    // Assert
    expect(sampled).toContain('font-family="DejaVu Sans"');
  });

  it("rejects a blend from none to a paint color", () => {
    // Arrange
    const source = SVG.replace(
      'values="rgba(0,0,0,1);rgba(10,20,30,0)"',
      'values="none;rgba(10,20,30,0)"',
    );

    // Act
    const sample = () => sampleAnimatedSvg(source, 5);

    // Assert
    expect(sample).toThrow("cannot blend none");
  });

  it("omits circles that sit fully outside the viewport", () => {
    // Arrange
    const source = [
      '<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">',
      '<circle cx="4" cy="4" r="1" fill="none"/>',
      '<circle cx="80" cy="4" r="1" fill="none"/>',
      '<line x1="-5" y1="-5" x2="30" y2="20" stroke="#fff"/>',
      '<line x1="40" y1="40" x2="50" y2="50" stroke="#fff"/>',
      "</svg>",
    ].join("");

    // Act
    const sampled = sampleAnimatedSvg(source, 0);

    // Assert
    expect(sampled).toContain('cx="4"');
    expect(sampled).not.toContain('cx="80"');
    expect(sampled).toContain('x1="-5"');
    expect(sampled).not.toContain('x1="40"');
  });

  it("rejects a non-linear SMIL mode", () => {
    // Arrange
    const source = SVG.replaceAll('calcMode="linear"', 'calcMode="discrete"');

    // Act
    const sample = () => sampleAnimatedSvg(source, 0);

    // Assert
    expect(sample).toThrow('calcMode "discrete"');
  });

  it("samples the committed Dam Break SVG as a 10 second 1280 by 960 loop", async () => {
    // Arrange
    const path = resolve(
      dirname(fileURLToPath(import.meta.url)),
      "../../docs/assets/readme/dam-break-10s.svg",
    );
    const source = await readFile(path, "utf8");

    // Act
    const parsed = parseAnimatedSvg(source);
    const atStart = sampleAnimatedSvg(source, 0);
    const atEnd = sampleAnimatedSvg(source, parsed.durationSeconds);

    // Assert
    expect(parsed).toMatchObject({ durationSeconds: 10, width: 1280, height: 960 });
    expect(atStart).not.toContain("<animate");
    expect(atStart.startsWith("<svg ")).toBe(true);
    expect(atEnd).toBe(atStart);
  });
});

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
