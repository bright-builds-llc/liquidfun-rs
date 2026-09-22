import type { SceneSession } from "../physics/session";
import {
  listenForTiltGravity,
  requestMotionPermission,
  type TiltDebug,
} from "./tilt-gravity";

export type TiltBinding = {
  request: number;
  stop: (() => void) | undefined;
};

export function createTiltBinding(): TiltBinding {
  return { request: 0, stop: undefined };
}

/** Turns phone motion into live gravity, or records why that failed. */
export async function changeTiltGravity(
  binding: TiltBinding,
  enabled: boolean,
  session: () => SceneSession | undefined,
  setEnabled: (enabled: boolean) => void,
  setDebug: (debug: TiltDebug) => void,
): Promise<void> {
  const request = binding.request + 1;
  binding.request = request;
  binding.stop?.();
  binding.stop = undefined;
  if (!enabled) {
    setEnabled(false);
    setDebug({ kind: "idle" });
    session()?.restoreAuthoredGravity();
    return;
  }

  setEnabled(true);
  setDebug({ kind: "waiting" });
  const permission = await requestMotionPermission();
  if (request !== binding.request) {
    return;
  }
  if (!permission.ok) {
    setEnabled(false);
    setDebug({ kind: "problem", detail: permission.detail });
    return;
  }

  setDebug({ kind: "waiting" });
  binding.stop = listenForTiltGravity((report) => {
    if (report.kind === "live") {
      session()?.setGravity(report.gravity.x, report.gravity.y);
      setDebug({
        kind: "live",
        sample: report.sample,
        gravity: report.gravity,
      });
      return;
    }
    setDebug({
      kind: "problem",
      detail: report.detail,
      maybeSample: report.sample,
    });
  });
}
