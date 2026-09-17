import {
  parseRenderFrame,
  type RawProofFrame,
  type RenderFrame,
} from "./frame";

const DISPOSED_MESSAGE = "Rust/WASM session is disposed";
const FAILED_MESSAGE = "Rust/WASM session failed";

/** Generated session methods owned by the browser lifecycle adapter. */
export interface GeneratedProofSession {
  advance(stepCount: number): void;
  captureFrame(): RawProofFrame;
  free(): void;
}

/** Browser-facing owner of one opaque Rust/WASM proof session. */
export interface SceneSession {
  nextFrame(): RenderFrame;
  dispose(): void;
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

  function nextFrame(): RenderFrame {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }

    try {
      generatedSession.advance(1);
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

  function dispose(): void {
    if (disposed) {
      return;
    }

    disposed = true;
    generatedSession.free();
  }

  return { nextFrame, dispose };
}
