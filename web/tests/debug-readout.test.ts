import { describe, expect, it } from "vitest";

import { debugRows } from "../src/components/debug-readout";
import type { RenderFrame } from "../src/physics/frame";

const FRAME: RenderFrame = {
  stepIndex: 120,
  particleCount: 1920,
  rigidShapeCount: 4,
  maxSpeed: 1.5,
  stuckCandidateCount: 2,
  bodyContactCount: 8,
  particlePositions: new Float32Array(),
  particleColors: new Uint8Array(),
  particleRadii: new Float32Array(),
  rigidSegments: new Float32Array(),
  rigidCircles: new Float32Array(),
  circleLabels: [],
};

describe("debugRows", () => {
  it("formats the agreed simulation readout", () => {
    // Arrange / Act
    const rows = debugRows(FRAME, 2);

    // Assert
    expect(rows).toEqual([
      { label: "Step", value: "120" },
      { label: "Time", value: "2.00 s" },
      { label: "Particles", value: "1920" },
      { label: "Rigid shapes", value: "4" },
      { label: "Steps this frame", value: "2" },
      { label: "Max speed", value: "1.500 m/s" },
      { label: "Stuck", value: "2" },
      { label: "Body contacts", value: "8" },
    ]);
  });
});
