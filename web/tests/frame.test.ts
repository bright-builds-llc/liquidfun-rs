import { describe, expect, it } from "vitest";

import {
  parseRenderFrame,
  type RawProofFrame,
} from "../src/physics/frame";

type FrameValues = {
  stepIndex: number;
  particleCount: number;
  rigidShapeCount: number;
  maxSpeed: number;
  stuckCandidateCount: number;
  bodyContactCount: number;
  particlePositions: Float32Array;
  particleColors: Uint8Array;
  particleRadii: Float32Array;
  rigidSegments: Float32Array;
  rigidCircles: Float32Array;
  circleLabels?: readonly string[];
};

function validFrameValues(): FrameValues {
  return {
    stepIndex: 7,
    particleCount: 1,
    rigidShapeCount: 2,
    maxSpeed: 1.25,
    stuckCandidateCount: 0,
    bodyContactCount: 3,
    particlePositions: new Float32Array([1, 2]),
    particleColors: new Uint8Array([32, 96, 192, 255]),
    particleRadii: new Float32Array([0.25]),
    rigidSegments: new Float32Array([-1, 0, 1, 0]),
    rigidCircles: new Float32Array([0, 1, 0.5]),
  };
}

class FakeRawProofFrame implements RawProofFrame {
  readonly getterCalls = {
    particlePositions: 0,
    particleColors: 0,
    particleRadii: 0,
    rigidSegments: 0,
    rigidCircles: 0,
  };
  freeCalls = 0;
  private readonly values: FrameValues;

  constructor(overrides: Partial<FrameValues> = {}) {
    this.values = { ...validFrameValues(), ...overrides };
  }

  stepIndex(): number {
    return this.values.stepIndex;
  }

  particleCount(): number {
    return this.values.particleCount;
  }

  rigidShapeCount(): number {
    return this.values.rigidShapeCount;
  }

  maxSpeed(): number {
    return this.values.maxSpeed;
  }

  stuckCandidateCount(): number {
    return this.values.stuckCandidateCount;
  }

  bodyContactCount(): number {
    return this.values.bodyContactCount;
  }

  particlePositions(): Float32Array {
    this.getterCalls.particlePositions += 1;
    return this.values.particlePositions;
  }

  particleColors(): Uint8Array {
    this.getterCalls.particleColors += 1;
    return this.values.particleColors;
  }

  particleRadii(): Float32Array {
    this.getterCalls.particleRadii += 1;
    return this.values.particleRadii;
  }

  rigidSegments(): Float32Array {
    this.getterCalls.rigidSegments += 1;
    return this.values.rigidSegments;
  }

  rigidCircles(): Float32Array {
    this.getterCalls.rigidCircles += 1;
    return this.values.rigidCircles;
  }

  circleLabels(): readonly string[] {
    if (this.values.circleLabels !== undefined) {
      return this.values.circleLabels;
    }

    const count = this.values.rigidCircles.length / 3;
    if (!Number.isInteger(count)) {
      return [];
    }

    return Array.from({ length: count }, () => "");
  }

  free(): void {
    this.freeCalls += 1;
  }
}

function wrongFloatLane(values: number[]): Float32Array {
  return new Float64Array(values) as unknown as Float32Array;
}

function wrongColorLane(values: number[]): Uint8Array {
  return new Uint16Array(values) as unknown as Uint8Array;
}

describe("parseRenderFrame", () => {
  it("accepts a bounded frame and calls each bulk getter once", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.stepIndex).toBe(7);
    expect(frame.particleCount).toBe(1);
    expect(frame.rigidShapeCount).toBe(2);
    expect(frame.maxSpeed).toBe(1.25);
    expect(frame.stuckCandidateCount).toBe(0);
    expect(frame.bodyContactCount).toBe(3);
    expect(rawFrame.getterCalls).toEqual({
      particlePositions: 1,
      particleColors: 1,
      particleRadii: 1,
      rigidSegments: 1,
      rigidCircles: 1,
    });
  });

  it("retains JavaScript-owned lanes after the raw frame is freed", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame();

    // Act
    const frame = parseRenderFrame(rawFrame);
    rawFrame.free();

    // Assert
    expect([...frame.particlePositions]).toEqual([1, 2]);
    expect([...frame.particleColors]).toEqual([32, 96, 192, 255]);
    expect(rawFrame.freeCalls).toBe(1);
  });

  it("rejects a non-integer step index", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ stepIndex: 0.5 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a negative particle count", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ particleCount: -1 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects particle counts above 16384", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ particleCount: 16385 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("accepts a well-formed 16384-particle fake frame", () => {
    // Arrange
    const particleCount = 16384;
    const rawFrame = new FakeRawProofFrame({
      particleCount,
      particlePositions: new Float32Array(particleCount * 2).fill(1),
      particleColors: new Uint8Array(particleCount * 4).fill(255),
      particleRadii: new Float32Array(particleCount).fill(0.2),
    });

    // Act
    const frame = parseRenderFrame(rawFrame);

    // Assert
    expect(frame.particleCount).toBe(16384);
    expect(frame.particlePositions.length).toBe(16384 * 2);
    expect(frame.particleColors.length).toBe(16384 * 4);
    expect(frame.particleRadii.length).toBe(16384);
  });

  it("rejects segment counts above 64", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidShapeCount: 65,
      rigidSegments: new Float32Array(65 * 4),
      rigidCircles: new Float32Array(),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects circle counts above 48", () => {
    // Arrange
    const circles = new Float32Array(49 * 3);
    circles.fill(1);
    const rawFrame = new FakeRawProofFrame({
      rigidShapeCount: 49,
      rigidSegments: new Float32Array(),
      rigidCircles: circles,
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-Float32Array particle position lane", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particlePositions: wrongFloatLane([1, 2]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-Uint8Array particle color lane", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particleColors: wrongColorLane([32, 96, 192, 255]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-Float32Array particle radius lane", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particleRadii: wrongFloatLane([0.25]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-Float32Array rigid segment lane", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidSegments: wrongFloatLane([-1, 0, 1, 0]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-Float32Array rigid circle lane", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidCircles: wrongFloatLane([0, 1, 0.5]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a particle position length outside stride 2", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particlePositions: new Float32Array([1]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a particle color length outside stride 4", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particleColors: new Uint8Array([32, 96, 192]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a particle radius length outside stride 1 count equality", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particleRadii: new Float32Array(),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a rigid segment length outside stride 4", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidSegments: new Float32Array([-1, 0, 1]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a rigid circle length outside stride 3", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidCircles: new Float32Array([0, 1]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects NaN float geometry", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particlePositions: new Float32Array([Number.NaN, 2]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects infinite float geometry", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidSegments: new Float32Array([-1, 0, Number.POSITIVE_INFINITY, 0]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-positive particle radius", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      particleRadii: new Float32Array([0]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a non-positive rigid circle radius", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({
      rigidCircles: new Float32Array([0, 1, -0.5]),
    });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects an inconsistent rigid shape count", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ rigidShapeCount: 1 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a negative max speed", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ maxSpeed: -0.1 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });

  it("rejects a fractional stuck-candidate count", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame({ stuckCandidateCount: 1.5 });

    // Act
    const parse = () => parseRenderFrame(rawFrame);

    // Assert
    expect(parse).toThrow();
  });
});
