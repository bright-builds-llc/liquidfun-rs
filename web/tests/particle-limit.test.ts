import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDERED_PARTICLE_LIMIT,
  initialRenderedParticleLimit,
  maybeParseRenderedParticleLimit,
  particleDrawStride,
} from "../src/render/particle-limit";

describe("maybeParseRenderedParticleLimit", () => {
  it("accepts the default and the frame ceiling", () => {
    // Arrange / Act / Assert
    expect(maybeParseRenderedParticleLimit("4000")).toBe(
      DEFAULT_RENDERED_PARTICLE_LIMIT,
    );
    expect(maybeParseRenderedParticleLimit("0")).toBe(0);
    expect(maybeParseRenderedParticleLimit("16384")).toBe(16384);
    expect(initialRenderedParticleLimit("dam-break")).toBe(
      DEFAULT_RENDERED_PARTICLE_LIMIT,
    );
    expect(initialRenderedParticleLimit("liquid-tumbler")).toBe(16384);
  });

  it("rejects partial, empty, and out-of-range text", () => {
    // Arrange
    const rejected = ["", " ", "4.5", "-1", "16385", "nope", "4000a"];

    // Act / Assert
    for (const raw of rejected) {
      expect(maybeParseRenderedParticleLimit(raw)).toBeUndefined();
    }
  });

  it("steps across a full liquid when the draw cap is lower than the fill", () => {
    // Arrange / Act / Assert
    expect(particleDrawStride(100, 100)).toBe(1);
    expect(particleDrawStride(100, 4000)).toBe(1);
    expect(particleDrawStride(12513, 4000)).toBe(4);
    expect(particleDrawStride(10, 0)).toBe(1);
  });
});
