import { describe, expect, it } from "vitest";

import {
  fnv1a64Bytes,
  fnv1a64Lanes,
  hex16,
} from "../scripts/puzzle-replay/hash";
import {
  compareSpikes,
  parseSpikeLines,
  spikeLine,
} from "../scripts/puzzle-replay/spike";

const HASH_A = "0000000000000001";
const HASH_B = "0000000000000002";

describe("puzzle-replay frame hash", () => {
  it("fnv1a64Bytes of no bytes is the offset basis", () => {
    // Arrange
    const bytes = new Uint8Array(0);

    // Act
    const hash = fnv1a64Bytes(bytes);

    // Assert
    expect(hex16(hash)).toBe("cbf29ce484222325");
  });

  it('fnv1a64Bytes of "a" matches the published vector', () => {
    // Arrange
    const bytes = new TextEncoder().encode("a");

    // Act
    const hash = fnv1a64Bytes(bytes);

    // Assert
    expect(hex16(hash)).toBe("af63dc4c8601ec8c");
  });

  it("fnv1a64Lanes matches the shared Rust lane vector", () => {
    // Arrange
    const lanes = [
      Float32Array.of(1, -0),
      new Float32Array(0),
      Float32Array.of(0.5),
    ];

    // Act
    const hash = fnv1a64Lanes(lanes);

    // Assert
    expect(hex16(hash)).toBe("99e01cab84d730f5");
  });
});

describe("puzzle-replay spike comparison", () => {
  it("compareSpikes reports identical scenes", () => {
    // Arrange
    const text = [
      spikeLine("dam-break", 1, 1n),
      spikeLine("dam-break", 2, 2n),
    ].join("\n");

    // Act
    const rows = compareSpikes(parseSpikeLines(text), parseSpikeLines(text));

    // Assert
    expect(rows).toEqual([{ scene: "dam-break", identical: true, steps: 2 }]);
  });

  it("compareSpikes reports the first diverging step", () => {
    // Arrange
    const left = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 ${HASH_A}\nSPIKE dam-break 3 ${HASH_A}\n`;
    const right = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 ${HASH_B}\nSPIKE dam-break 3 ${HASH_B}\n`;

    // Act
    const rows = compareSpikes(parseSpikeLines(left), parseSpikeLines(right));

    // Assert
    expect(rows).toEqual([
      {
        scene: "dam-break",
        identical: false,
        steps: 3,
        maybeFirstDivergingStep: 2,
      },
    ]);
  });

  it("parseSpikeLines rejects a malformed line", () => {
    // Arrange
    const text = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 not-a-hash\n`;

    // Act
    const parse = () => parseSpikeLines(text);

    // Assert
    expect(parse).toThrow("Malformed spike line 2");
  });

  it("parseSpikeLines rejects a non-contiguous step", () => {
    // Arrange
    const text = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 3 ${HASH_A}\n`;

    // Act
    const parse = () => parseSpikeLines(text);

    // Assert
    expect(parse).toThrow("Malformed spike line 2");
  });
});
