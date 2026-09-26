import type { SceneSession } from "../physics/session";
import {
  listenForTiltGravity,
  requestMotionPermission,
  worldGravityFromTilt,
  type AccelerationSample,
  type TiltDebug,
  type TiltGravity,
} from "./tilt-gravity";

type LiveTiltSample = {
  readonly sample: AccelerationSample;
  readonly measured: TiltGravity;
  readonly screenAngleDegrees: number;
};

export type TiltBinding = {
  request: number;
  stop: (() => void) | undefined;
  maybeLive: LiveTiltSample | undefined;
};

export function createTiltBinding(): TiltBinding {
  return { request: 0, stop: undefined, maybeLive: undefined };
}

function publishLiveTilt(
  binding: TiltBinding,
  live: LiveTiltSample,
  session: () => SceneSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setDebug: (debug: TiltDebug) => void,
): void {
  binding.maybeLive = live;
  const scaled = worldGravityFromTilt(live.measured, maybeSliderMagnitude());
  session()?.setGravity(scaled.gravity.x, scaled.gravity.y);
  setDebug({
    kind: "live",
    sample: live.sample,
    gravity: scaled.gravity,
    screenAngleDegrees: live.screenAngleDegrees,
    fullLengthMagnitude: scaled.fullLengthMagnitude,
  });
}

/**
 * Reapplies the latest accelerometer sample after the world gravity is rebuilt.
 *
 * Scene recreation restores the authored downward vector. While tilt is live,
 * the stored sample is scaled by the current gravity slider and written back.
 */
export function reapplyStoredTiltGravity(
  binding: TiltBinding,
  session: () => SceneSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setDebug: (debug: TiltDebug) => void,
): void {
  const live = binding.maybeLive;
  if (live === undefined || binding.stop === undefined) {
    return;
  }

  publishLiveTilt(binding, live, session, maybeSliderMagnitude, setDebug);
}

/** Turns phone motion into live gravity, or records why that failed. */
export async function changeTiltGravity(
  binding: TiltBinding,
  enabled: boolean,
  session: () => SceneSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setEnabled: (enabled: boolean) => void,
  setDebug: (debug: TiltDebug) => void,
): Promise<void> {
  const request = binding.request + 1;
  binding.request = request;
  binding.stop?.();
  binding.stop = undefined;
  binding.maybeLive = undefined;
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
      publishLiveTilt(
        binding,
        {
          sample: report.sample,
          measured: report.gravity,
          screenAngleDegrees: report.screenAngleDegrees,
        },
        session,
        maybeSliderMagnitude,
        setDebug,
      );
      return;
    }
    setDebug({
      kind: "problem",
      detail: report.detail,
      maybeSample: report.sample,
    });
  });
}
