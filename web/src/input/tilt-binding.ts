import { finishOperation, type PlayerSession } from "../physics/live-session";
import {
  describeUnknownError,
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
  session: () => PlayerSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setDebug: (debug: TiltDebug) => void,
  maybeOnFailure?: (error: unknown) => void,
): void {
  binding.maybeLive = live;
  const maybeOwner = session();
  const request = binding.request;
  const scaled = worldGravityFromTilt(live.measured, maybeSliderMagnitude());
  const current = () =>
    session() === maybeOwner &&
    request === binding.request &&
    binding.stop !== undefined &&
    binding.maybeLive === live;
  const fail = (error: unknown) => {
    if (current()) reportFailure(error, setDebug, maybeOnFailure);
  };
  try {
    finishOperation(
      maybeOwner?.setGravity(scaled.gravity.x, scaled.gravity.y),
      current,
      () => {
        setDebug({
          kind: "live",
          sample: live.sample,
          gravity: scaled.gravity,
          screenAngleDegrees: live.screenAngleDegrees,
          fullLengthMagnitude: scaled.fullLengthMagnitude,
        });
      },
      fail,
    );
  } catch (error) {
    fail(error);
  }
}

function reportFailure(
  error: unknown,
  setDebug: (debug: TiltDebug) => void,
  maybeOnFailure: ((error: unknown) => void) | undefined,
): void {
  if (maybeOnFailure !== undefined) {
    maybeOnFailure(error);
    return;
  }
  setDebug({ kind: "problem", detail: describeUnknownError(error) });
}

/**
 * Reapplies the latest accelerometer sample after the world gravity is rebuilt.
 *
 * Scene recreation restores the authored downward vector. While tilt is live,
 * the stored sample is scaled by the current gravity slider and written back.
 */
export function reapplyStoredTiltGravity(
  binding: TiltBinding,
  session: () => PlayerSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setDebug: (debug: TiltDebug) => void,
  maybeOnFailure?: (error: unknown) => void,
): void {
  const live = binding.maybeLive;
  if (live === undefined || binding.stop === undefined) {
    return;
  }

  publishLiveTilt(
    binding,
    live,
    session,
    maybeSliderMagnitude,
    setDebug,
    maybeOnFailure,
  );
}

/** Turns phone motion into live gravity, or records why that failed. */
export async function changeTiltGravity(
  binding: TiltBinding,
  enabled: boolean,
  session: () => PlayerSession | undefined,
  maybeSliderMagnitude: () => number | undefined,
  setEnabled: (enabled: boolean) => void,
  setDebug: (debug: TiltDebug) => void,
  maybeOnFailure?: (error: unknown) => void,
): Promise<void> {
  const request = binding.request + 1;
  binding.request = request;
  binding.stop?.();
  binding.stop = undefined;
  binding.maybeLive = undefined;
  if (!enabled) {
    setEnabled(false);
    setDebug({ kind: "idle" });
    const maybeOwner = session();
    const current = () =>
      session() === maybeOwner && request === binding.request;
    const fail = (error: unknown) => {
      if (current()) reportFailure(error, setDebug, maybeOnFailure);
    };
    try {
      finishOperation(
        maybeOwner?.restoreAuthoredGravity(),
        current,
        () => {},
        fail,
      );
    } catch (error) {
      fail(error);
    }
    return;
  }

  setEnabled(true);
  setDebug({ kind: "waiting" });
  try {
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
      if (request !== binding.request || binding.stop === undefined) return;
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
          maybeOnFailure,
        );
        return;
      }
      setDebug({
        kind: "problem",
        detail: report.detail,
        maybeSample: report.sample,
      });
    });
  } catch (error) {
    if (request !== binding.request) return;
    setEnabled(false);
    reportFailure(error, setDebug, maybeOnFailure);
  }
}
