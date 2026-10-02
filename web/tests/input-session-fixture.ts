import type { LiveSceneSession } from "../src/physics/live-session";
import type { SceneSession } from "../src/physics/session";

export function inputSession(
  options: {
    readonly pointer?: () => Promise<void>;
    readonly gravity?: (x: number, y: number) => Promise<void>;
    readonly restore?: () => Promise<void>;
  } = {},
): LiveSceneSession {
  return {
    backend: "worker",
    maybeLastTiming: undefined,
    nextFrame: async () => {
      throw new Error("Unexpected frame request");
    },
    advanceBudgeted: async () => {
      throw new Error("Unexpected advance");
    },
    advanceOnly: async () => {
      throw new Error("Unexpected advance");
    },
    captureFrame: async () => {
      throw new Error("Unexpected capture");
    },
    applyControl: async () => {
      throw new Error("Unexpected control");
    },
    applyAction: async () => {
      throw new Error("Unexpected action");
    },
    pointerAction: options.pointer ?? (async () => {}),
    setGravity: options.gravity ?? (async () => {}),
    restoreAuthoredGravity: options.restore ?? (async () => {}),
    dispose() {},
  };
}

export function directInputSession(
  options: {
    readonly pointer?: () => void;
    readonly gravity?: (x: number, y: number) => void;
    readonly restore?: () => void;
  } = {},
): SceneSession {
  return {
    nextFrame() {
      throw new Error("Unexpected frame");
    },
    applyControl() {
      throw new Error("Unexpected control");
    },
    applyAction() {
      throw new Error("Unexpected action");
    },
    pointerAction: options.pointer ?? (() => {}),
    setGravity: options.gravity ?? (() => {}),
    restoreAuthoredGravity: options.restore ?? (() => {}),
    dispose() {},
  };
}
