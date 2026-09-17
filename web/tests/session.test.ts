import { describe, expect, it } from "vitest";

import type { RawProofFrame } from "../src/physics/frame";
import {
  createSceneSession,
  type GeneratedProofSession,
} from "../src/physics/session";

class FakeRawProofFrame implements RawProofFrame {
  freeCalls = 0;

  constructor(
    private readonly positions = new Float32Array([1, 2]),
    private readonly radii = new Float32Array([0.25]),
  ) {}

  stepIndex(): number {
    return 1;
  }

  particleCount(): number {
    return 1;
  }

  rigidShapeCount(): number {
    return 2;
  }

  particlePositions(): Float32Array {
    return this.positions;
  }

  particleColors(): Uint8Array {
    return new Uint8Array([32, 96, 192, 255]);
  }

  particleRadii(): Float32Array {
    return this.radii;
  }

  rigidSegments(): Float32Array {
    return new Float32Array([-1, 0, 1, 0]);
  }

  rigidCircles(): Float32Array {
    return new Float32Array([0, 1, 0.5]);
  }

  free(): void {
    this.freeCalls += 1;
  }
}

class FakeGeneratedProofSession implements GeneratedProofSession {
  readonly advanceCalls: number[] = [];
  captureCalls = 0;
  freeCalls = 0;
  maybeAdvanceError: Error | undefined;
  maybeCaptureError: Error | undefined;

  constructor(readonly frame: RawProofFrame = new FakeRawProofFrame()) {}

  advance(stepCount: number): void {
    this.advanceCalls.push(stepCount);
    if (this.maybeAdvanceError !== undefined) {
      throw this.maybeAdvanceError;
    }
  }

  captureFrame(): RawProofFrame {
    this.captureCalls += 1;
    if (this.maybeCaptureError !== undefined) {
      throw this.maybeCaptureError;
    }

    return this.frame;
  }

  free(): void {
    this.freeCalls += 1;
  }
}

describe("createSceneSession", () => {
  it("advances one step, captures once, and frees the temporary frame", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame();
    const generatedSession = new FakeGeneratedProofSession(rawFrame);
    const session = createSceneSession(generatedSession);

    // Act
    const frame = session.nextFrame();

    // Assert
    expect(frame.stepIndex).toBe(1);
    expect(generatedSession.advanceCalls).toEqual([1]);
    expect(generatedSession.captureCalls).toBe(1);
    expect(rawFrame.freeCalls).toBe(1);
  });

  it("returns copied arrays that remain readable after frame free", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame();
    const generatedSession = new FakeGeneratedProofSession(rawFrame);
    const session = createSceneSession(generatedSession);

    // Act
    const frame = session.nextFrame();

    // Assert
    expect(rawFrame.freeCalls).toBe(1);
    expect([...frame.particlePositions]).toEqual([1, 2]);
    expect([...frame.particleRadii]).toEqual([0.25]);
  });

  it("frees the frame and poisons the session after parse failure", () => {
    // Arrange
    const rawFrame = new FakeRawProofFrame(
      new Float32Array([Number.NaN, 2]),
    );
    const generatedSession = new FakeGeneratedProofSession(rawFrame);
    const session = createSceneSession(generatedSession);

    // Act
    const nextFrame = () => session.nextFrame();

    // Assert
    expect(nextFrame).toThrow("Rust/WASM session failed");
    expect(rawFrame.freeCalls).toBe(1);
    expect(generatedSession.freeCalls).toBe(1);
    expect(nextFrame).toThrow("Rust/WASM session is disposed");
    expect(generatedSession.freeCalls).toBe(1);
  });

  it("poisons and frees the session after advance failure", () => {
    // Arrange
    const generatedSession = new FakeGeneratedProofSession();
    generatedSession.maybeAdvanceError = new Error("unbounded generated detail");
    const session = createSceneSession(generatedSession);

    // Act
    const nextFrame = () => session.nextFrame();

    // Assert
    expect(nextFrame).toThrow("Rust/WASM session failed");
    expect(generatedSession.advanceCalls).toEqual([1]);
    expect(generatedSession.captureCalls).toBe(0);
    expect(generatedSession.freeCalls).toBe(1);
    expect(nextFrame).toThrow("Rust/WASM session is disposed");
  });

  it("poisons and frees the session after capture failure", () => {
    // Arrange
    const generatedSession = new FakeGeneratedProofSession();
    generatedSession.maybeCaptureError = new Error("unbounded generated detail");
    const session = createSceneSession(generatedSession);

    // Act
    const nextFrame = () => session.nextFrame();

    // Assert
    expect(nextFrame).toThrow("Rust/WASM session failed");
    expect(generatedSession.advanceCalls).toEqual([1]);
    expect(generatedSession.captureCalls).toBe(1);
    expect(generatedSession.freeCalls).toBe(1);
    expect(nextFrame).toThrow("Rust/WASM session is disposed");
  });

  it("disposes the generated session exactly once", () => {
    // Arrange
    const generatedSession = new FakeGeneratedProofSession();
    const session = createSceneSession(generatedSession);

    // Act
    session.dispose();
    session.dispose();
    session.dispose();

    // Assert
    expect(generatedSession.freeCalls).toBe(1);
  });

  it("rejects frame requests after disposal with the fixed message", () => {
    // Arrange
    const generatedSession = new FakeGeneratedProofSession();
    const session = createSceneSession(generatedSession);
    session.dispose();

    // Act
    const nextFrame = () => session.nextFrame();

    // Assert
    expect(nextFrame).toThrow("Rust/WASM session is disposed");
    expect(generatedSession.advanceCalls).toEqual([]);
    expect(generatedSession.captureCalls).toBe(0);
    expect(generatedSession.freeCalls).toBe(1);
  });
});
