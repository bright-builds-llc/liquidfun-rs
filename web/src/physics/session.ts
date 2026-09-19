import { parsePointerKind } from "../input/pointer";
import {
  parseRenderFrame,
  type RawProofFrame,
  type RenderFrame,
} from "./frame";

const DISPOSED_MESSAGE = "Rust/WASM session is disposed";
const FAILED_MESSAGE = "Rust/WASM session failed";
const MIN_STEPS_PER_FRAME = 1;
const MAX_STEPS_PER_FRAME = 4;

/** Generated session methods owned by the browser lifecycle adapter. */
export interface GeneratedProofSession {
  advance(stepCount: number): void;
  captureFrame(): RawProofFrame;
  applyControl(name: string, value: string): boolean;
  applyAction(name: string): void;
  pointerAction(kind: string, worldX: number, worldY: number): void;
  free(): void;
}

/** Browser-facing owner of one opaque Rust/WASM proof session. */
export interface SceneSession {
  nextFrame(stepCount?: number): RenderFrame;
  applyControl(name: string, value: string): boolean;
  applyAction(name: string): void;
  pointerAction(kind: string, worldX: number, worldY: number): void;
  dispose(): void;
}

function isAcceptedStepCount(stepCount: number): boolean {
  return (
    Number.isSafeInteger(stepCount) &&
    stepCount >= MIN_STEPS_PER_FRAME &&
    stepCount <= MAX_STEPS_PER_FRAME
  );
}

/** Creates an exactly-once owner around one generated Rust session. */
export function createSceneSession(
  generatedSession: GeneratedProofSession,
): SceneSession {
  let disposed = false;

  function disposeAfterFailure(): void {
    if (disposed) {
      return;
    }

    disposed = true;
    try {
      generatedSession.free();
    } catch {
      // The owner is still poisoned; generated cleanup must not expose details.
    }
  }

  function nextFrame(stepCount = 1): RenderFrame {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }

    if (!isAcceptedStepCount(stepCount)) {
      disposeAfterFailure();
      throw new Error(FAILED_MESSAGE);
    }

    try {
      generatedSession.advance(stepCount);
      const rawFrame = generatedSession.captureFrame();
      try {
        return parseRenderFrame(rawFrame);
      } finally {
        rawFrame.free();
      }
    } catch {
      disposeAfterFailure();
      throw new Error(FAILED_MESSAGE);
    }
  }

  function applyControl(name: string, value: string): boolean {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }

    try {
      return generatedSession.applyControl(name, value);
    } catch {
      disposeAfterFailure();
      throw new Error(FAILED_MESSAGE);
    }
  }

  function applyAction(name: string): void {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }

    try {
      generatedSession.applyAction(name);
    } catch {
      disposeAfterFailure();
      throw new Error(FAILED_MESSAGE);
    }
  }

  function pointerAction(kind: string, worldX: number, worldY: number): void {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }

    if (
      parsePointerKind(kind) === undefined ||
      !Number.isFinite(worldX) ||
      !Number.isFinite(worldY)
    ) {
      return;
    }

    try {
      generatedSession.pointerAction(kind, worldX, worldY);
    } catch {
      disposeAfterFailure();
      throw new Error(FAILED_MESSAGE);
    }
  }

  function dispose(): void {
    if (disposed) {
      return;
    }

    disposed = true;
    generatedSession.free();
  }

  return { nextFrame, applyControl, applyAction, pointerAction, dispose };
}
