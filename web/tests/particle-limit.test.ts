import { describe, expect, it } from "vitest";

import {
  DEFAULT_RENDERED_PARTICLE_LIMIT,
  maybeParseRenderedParticleLimit,
} from "../src/render/particle-limit";

describe("maybeParseRenderedParticleLimit", () => {
  it("accepts the default and the frame ceiling", () => {
    // Arrange / Act / Assert
    expect(maybeParseRenderedParticleLimit("4000")).toBe(
      DEFAULT_RENDERED_PARTICLE_LIMIT,
    );
    expect(maybeParseRenderedParticleLimit("0")).toBe(0);
    expect(maybeParseRenderedParticleLimit("10240")).toBe(10240);
  });

  it("rejects partial, empty, and out-of-range text", () => {
    // Arrange
    const rejected = ["", " ", "4.5", "-1", "10241", "nope", "4000a"];

    // Act / Assert
    for (const raw of rejected) {
      expect(maybeParseRenderedParticleLimit(raw)).toBeUndefined();
    }
  });
});
