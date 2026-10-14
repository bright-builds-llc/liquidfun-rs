import { describe, expect, it } from "vitest";
import {
  parseWorkerReply,
  parseWorkerRequest,
  frameTransferables,
  validateTransferredFrame,
} from "../src/physics/worker-messages";

const envelope = { version: 1, generation: 1, id: 1 };
const timing = { advanceMs: 1, captureMs: 0.2, parseMs: 0.1 };

/** A one-particle transferred frame with tight, owned lane buffers. */
function transferredFrame() {
  return {
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
}

describe("live Tesla worker boundary", () => {
  it("accepts only bounded step requests and finite ordered mutation envelopes", () => {
    // Arrange / Act / Assert
    expect(
      parseWorkerRequest({
        ...envelope,
        op: "advance",
        count: 4,
        budgeted: true,
        speeds: false,
      }).op,
    ).toBe("advance");
    expect(() =>
      parseWorkerRequest({
        ...envelope,
        op: "advance",
        count: 5,
        budgeted: false,
        speeds: false,
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

describe("live worker speed lane flag", () => {
  it("accepts an advance request carrying a boolean speed flag", () => {
    // Arrange
    const message = { ...envelope, op: "advance", count: 1, budgeted: false, speeds: true };

    // Act
    const request = parseWorkerRequest(message);

    // Assert
    expect(request).toEqual(message);
  });

  it("accepts a capture request carrying a boolean speed flag", () => {
    // Arrange
    const message = { ...envelope, op: "capture", speeds: false };

    // Act
    const request = parseWorkerRequest(message);

    // Assert
    expect(request).toEqual(message);
  });

  it.each([undefined, "true", 1, null])(
    "rejects an advance request whose speed flag is %s",
    (speeds) => {
      // Arrange
      const message = { ...envelope, op: "advance", count: 1, budgeted: false, speeds };

      // Act
      const act = () => parseWorkerRequest(message);

      // Assert
      expect(act).toThrow("Invalid speed lane flag");
    },
  );

  it.each([undefined, "false", 0, null])(
    "rejects a capture request whose speed flag is %s",
    (speeds) => {
      // Arrange
      const message = { ...envelope, op: "capture", speeds };

      // Act
      const act = () => parseWorkerRequest(message);

      // Assert
      expect(act).toThrow("Invalid speed lane flag");
    },
  );
});

describe("live worker transferred speed lane", () => {
  it("keeps the lane values of a transferred frame", () => {
    // Arrange
    const frame = { ...transferredFrame(), maybeParticleSpeeds: new Float32Array([1.25]) };

    // Act
    const parsed = validateTransferredFrame(frame);

    // Assert
    expect(parsed.maybeParticleSpeeds).toEqual(new Float32Array([1.25]));
  });

  it("omits the lane key when the transferred frame has no lane", () => {
    // Arrange
    const frame = transferredFrame();

    // Act
    const parsed = validateTransferredFrame(frame);

    // Assert
    expect("maybeParticleSpeeds" in parsed).toBe(false);
  });

  it("rejects a transferred lane whose length is not the particle count", () => {
    // Arrange
    const frame = { ...transferredFrame(), maybeParticleSpeeds: new Float32Array([1, 2]) };

    // Act
    const act = () => parseWorkerReply({ ...envelope, kind: "frame", frame, ran: 1, timing });

    // Assert
    expect(act).toThrow();
  });

  it("transfers six unique buffers when the lane is present", () => {
    // Arrange
    const frame = validateTransferredFrame({
      ...transferredFrame(),
      maybeParticleSpeeds: new Float32Array([0.5]),
    });

    // Act
    const buffers = frameTransferables(frame);

    // Assert
    expect(buffers).toHaveLength(6);
  });

  it("transfers the original five buffers when the lane is absent", () => {
    // Arrange
    const frame = validateTransferredFrame(transferredFrame());

    // Act
    const buffers = frameTransferables(frame);

    // Assert
    expect(buffers).toHaveLength(5);
  });

  it("rejects a lane that is an offset view into a shared buffer", () => {
    // Arrange
    const lane = new Float32Array(new ArrayBuffer(8), 4, 1);
    const frame = { ...validateTransferredFrame(transferredFrame()), maybeParticleSpeeds: lane };

    // Act
    const act = () => frameTransferables(frame);

    // Assert
    expect(act).toThrow("bounded");
  });

  it("rejects a lane whose buffer is larger than the lane", () => {
    // Arrange
    const lane = new Float32Array(new ArrayBuffer(1024), 0, 1);
    const frame = { ...validateTransferredFrame(transferredFrame()), maybeParticleSpeeds: lane };

    // Act
    const act = () => frameTransferables(frame);

    // Assert
    expect(act).toThrow("bounded");
  });
});
