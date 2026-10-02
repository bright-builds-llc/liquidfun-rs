import type { RenderFrame } from "./frame";
import type { SceneSession } from "./session";
import type { WorkerComputeTiming } from "./worker-messages";

export type WorkerTiming = WorkerComputeTiming & {
  readonly roundTripMs: number;
};
export type WorkerAdvance = {
  readonly frame: RenderFrame;
  readonly ran: number;
  readonly timing: WorkerTiming;
};

/** Async ownership used only for Tesla live playback; synchronous exporters stay direct. */
export interface LiveSceneSession {
  readonly backend: "worker";
  readonly maybeLastTiming: WorkerTiming | undefined;
  nextFrame(stepCount?: number): Promise<RenderFrame>;
  advanceBudgeted(stepCount: number): Promise<WorkerAdvance>;
  advanceOnly(stepCount: number): Promise<void>;
  captureFrame(): Promise<RenderFrame>;
  applyControl(name: string, value: string): Promise<boolean>;
  applyAction(name: string): Promise<void>;
  pointerAction(kind: string, worldX: number, worldY: number): Promise<void>;
  setGravity(x: number, y: number): Promise<void>;
  restoreAuthoredGravity(): Promise<void>;
  dispose(): void;
}

export type PlayerSession = SceneSession | LiveSceneSession;

export function isWorkerSession(
  session: PlayerSession,
): session is LiveSceneSession {
  return "backend" in session && session.backend === "worker";
}

/** Preserves synchronous direct callbacks; worker completion is observed, never silently dropped. */
export function finishOperation<T>(
  operation: T | Promise<T>,
  current: () => boolean,
  finish: (value: T) => void,
  fail: (error: unknown) => void,
): void {
  if (operation instanceof Promise) {
    void operation
      .then((value) => {
        if (current()) finish(value);
      })
      .catch((error: unknown) => {
        if (current()) fail(error);
      });
    return;
  }
  if (current()) finish(operation);
}
