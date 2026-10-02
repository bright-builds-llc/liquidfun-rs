import { describe, expect, it } from "vitest";
import {
  parseWorkerReply,
  parseWorkerRequest,
  frameTransferables,
} from "../src/physics/worker-messages";

const envelope = { version: 1, generation: 1, id: 1 };

describe("live Tesla worker boundary", () => {
  it("accepts only bounded step requests and finite ordered mutation envelopes", () => {
    // Arrange / Act / Assert
    expect(
      parseWorkerRequest({
        ...envelope,
        op: "advance",
        count: 4,
        budgeted: true,
      }).op,
    ).toBe("advance");
    expect(() =>
      parseWorkerRequest({
        ...envelope,
        op: "advance",
        count: 5,
        budgeted: false,
      }),
    ).toThrow();
    expect(() =>
      parseWorkerRequest({ ...envelope, generation: -1, op: "init" }),
    ).toThrow();
    expect(() =>
      parseWorkerRequest({ ...envelope, op: "gravity", x: Number.NaN, y: -10 }),
    ).toThrow();
    expect(() =>
      parseWorkerRequest({
        ...envelope,
        op: "pointer",
        kind: "invalid",
        x: 0,
        y: 0,
      }),
    ).toThrow();
  });

  it("validates transferred frame lanes before exposing an owned frame", () => {
    // Arrange
    const frame = {
      stepIndex: 1,
      particleCount: 1,
      rigidShapeCount: 0,
      maxSpeed: 2,
      stuckCandidateCount: 0,
      bodyContactCount: 0,
      particlePositions: new Float32Array([0, 1]),
      particleColors: new Uint8Array([77, 163, 255, 255]),
      particleRadii: new Float32Array([0.005]),
      rigidSegments: new Float32Array(),
      rigidCircles: new Float32Array(),
      circleLabels: [],
    };

    // Act
    const reply = parseWorkerReply({
      ...envelope,
      kind: "frame",
      frame,
      ran: 1,
      timing: { advanceMs: 1, captureMs: 0.2, parseMs: 0.1 },
    });

    // Assert
    expect(reply.kind).toBe("frame");
    expect(frameTransferables(frame)).toHaveLength(5);
    expect(() =>
      parseWorkerReply({
        ...envelope,
        kind: "frame",
        ran: 1,
        timing: { advanceMs: 1, captureMs: 0.2, parseMs: 0.1 },
        frame: {
          ...frame,
          particlePositions: new Float32Array([Number.NaN, 1]),
        },
      }),
    ).toThrow();
    const oversized = new Float32Array(new ArrayBuffer(1024 * 1024), 0, 2);
    expect(() =>
      parseWorkerReply({
        ...envelope,
        kind: "frame",
        ran: 1,
        timing: { advanceMs: 1, captureMs: 0.2, parseMs: 0.1 },
        frame: { ...frame, particlePositions: oversized },
      }),
    ).toThrow("bounded");
    expect(() =>
      frameTransferables({
        ...frame,
        particlePositions: new Float32Array(new ArrayBuffer(12), 4, 2),
      }),
    ).toThrow("bounded");
  });
});
