/** Largest gravity magnitude accepted from the phone accelerometer, in m/s². */
export const TILT_GRAVITY_LIMIT = 20;

export type TiltGravity = {
  readonly x: number;
  readonly y: number;
};

export type AccelerationSample = {
  readonly x: number | null;
  readonly y: number | null;
  readonly z: number | null;
};

export type TiltDebug =
  | { readonly kind: "idle" }
  | { readonly kind: "waiting" }
  | {
      readonly kind: "problem";
      readonly detail: string;
      readonly maybeSample?: AccelerationSample;
    }
  | {
      readonly kind: "live";
      readonly sample: AccelerationSample;
      readonly gravity: TiltGravity;
    };

export type TiltSampleResult =
  | {
      readonly kind: "live";
      readonly sample: AccelerationSample;
      readonly gravity: TiltGravity;
    }
  | {
      readonly kind: "problem";
      readonly detail: string;
      readonly sample: AccelerationSample;
    };

export type MotionPermissionResult =
  | { readonly ok: true }
  | { readonly ok: false; readonly detail: string };

/**
 * How a browser reports `accelerationIncludingGravity` while the phone is still.
 *
 * `support-force` is the upward push (Chrome and most non-iOS browsers).
 * `gravity-direction` is the direction things fall (iOS).
 */
export type AccelerationConvention = "support-force" | "gravity-direction";

/**
 * Maps device acceleration, including gravity, into world gravity.
 *
 * Device +x points to the right of the phone and +y points toward the top.
 * World +y is up, so water falls toward negative world y when the phone is upright.
 */
export function maybeWorldGravityFromAcceleration(
  accelerationX: number,
  accelerationY: number,
  convention: AccelerationConvention = "support-force",
): TiltGravity | undefined {
  if (!Number.isFinite(accelerationX) || !Number.isFinite(accelerationY)) {
    return undefined;
  }

  const sign = convention === "gravity-direction" ? 1 : -1;
  let x = sign * accelerationX;
  let y = sign * accelerationY;
  const magnitude = Math.hypot(x, y);
  if (magnitude > TILT_GRAVITY_LIMIT) {
    const scale = TILT_GRAVITY_LIMIT / magnitude;
    x *= scale;
    y *= scale;
  }
  return { x, y };
}

/** Turns an unknown thrown value into a name and message for the readout. */
export function describeUnknownError(error: unknown): string {
  if (error instanceof Error) {
    const name = error.name.length > 0 ? error.name : "Error";
    return error.message.length > 0 ? `${name}: ${error.message}` : name;
  }
  if (typeof error === "string" && error.length > 0) {
    return error;
  }
  try {
    const encoded = JSON.stringify(error);
    return encoded === undefined ? "Unknown error" : encoded;
  } catch (stringifyError) {
    return describeUnknownError(stringifyError);
  }
}

type MotionPermissionTarget = {
  requestPermission?: () => Promise<PermissionState>;
};

/** Asks for motion access on browsers that require a user gesture. */
export async function requestMotionPermission(): Promise<MotionPermissionResult> {
  if (!globalThis.isSecureContext) {
    return {
      ok: false,
      detail: "Device motion requires a secure context (HTTPS or localhost).",
    };
  }

  const motion = DeviceMotionEvent as unknown as MotionPermissionTarget;
  if (typeof motion.requestPermission !== "function") {
    return { ok: true };
  }

  try {
    const result = await motion.requestPermission();
    if (result === "granted") {
      return { ok: true };
    }
    return {
      ok: false,
      detail: `DeviceMotionEvent.requestPermission returned "${result}"`,
    };
  } catch (error) {
    return { ok: false, detail: describeUnknownError(error) };
  }
}

const EMPTY_SAMPLE: AccelerationSample = { x: null, y: null, z: null };

/** iOS reports the fall direction. Other browsers report the upward support force. */
export function accelerationConventionFromEnvironment(environment: {
  readonly hasRequestPermission: boolean;
  readonly userAgent: string;
}): AccelerationConvention {
  if (
    environment.hasRequestPermission ||
    /iPhone|iPad|iPod/.test(environment.userAgent)
  ) {
    return "gravity-direction";
  }
  return "support-force";
}

/** Reads the current browser's accelerometer sign convention. */
export function accelerationConvention(): AccelerationConvention {
  const motion = DeviceMotionEvent as unknown as MotionPermissionTarget;
  return accelerationConventionFromEnvironment({
    hasRequestPermission: typeof motion.requestPermission === "function",
    userAgent: navigator.userAgent,
  });
}

/** Classifies one accelerometer sample for gravity and the debug readout. */
export function interpretAcceleration(
  maybeAcceleration: AccelerationSample | null,
  convention: AccelerationConvention = "support-force",
): TiltSampleResult {
  if (maybeAcceleration === null) {
    return {
      kind: "problem",
      detail: "accelerationIncludingGravity is null",
      sample: EMPTY_SAMPLE,
    };
  }

  const sample: AccelerationSample = {
    x: maybeAcceleration.x,
    y: maybeAcceleration.y,
    z: maybeAcceleration.z,
  };
  if (sample.x === null || sample.y === null) {
    const missing = sample.x === null ? "x" : "y";
    return {
      kind: "problem",
      detail: `accelerationIncludingGravity.${missing} is null`,
      sample,
    };
  }
  if (!Number.isFinite(sample.x) || !Number.isFinite(sample.y)) {
    return {
      kind: "problem",
      detail: `Non-finite accelerationIncludingGravity: x=${String(sample.x)} y=${String(sample.y)} z=${String(sample.z)}`,
      sample,
    };
  }

  const maybeGravity = maybeWorldGravityFromAcceleration(
    sample.x,
    sample.y,
    convention,
  );
  if (maybeGravity === undefined) {
    return {
      kind: "problem",
      detail: "Could not map accelerationIncludingGravity to world gravity",
      sample,
    };
  }
  return { kind: "live", sample, gravity: maybeGravity };
}

function formatAxis(value: number | null): string {
  if (value === null) {
    return "null";
  }
  if (!Number.isFinite(value)) {
    return String(value);
  }
  return value.toFixed(3);
}

function formatSample(sample: AccelerationSample): string {
  return `ax ${formatAxis(sample.x)}  ay ${formatAxis(sample.y)}  az ${formatAxis(sample.z)}`;
}

/** Renders the tilt readout, including the idle explanation. */
export function formatTiltDebug(debug: TiltDebug): string {
  if (debug.kind === "idle") {
    return "Off. The scene keeps its own gravity.";
  }
  if (debug.kind === "waiting") {
    return "Waiting for a devicemotion event";
  }
  if (debug.kind === "problem") {
    if (debug.maybeSample === undefined) {
      return debug.detail;
    }
    return `${formatSample(debug.maybeSample)}\n${debug.detail}`;
  }
  return `${formatSample(debug.sample)}\ngravity ${formatAxis(debug.gravity.x)}, ${formatAxis(debug.gravity.y)}`;
}

/** Listens for accelerometer samples and reports world gravity or a fault. */
export function listenForTiltGravity(
  onReport: (report: TiltSampleResult) => void,
): () => void {
  const convention = accelerationConvention();
  const onMotion = (event: DeviceMotionEvent) => {
    const acceleration = event.accelerationIncludingGravity;
    if (acceleration === null) {
      onReport(interpretAcceleration(null, convention));
      return;
    }
    onReport(
      interpretAcceleration(
        {
          x: acceleration.x,
          y: acceleration.y,
          z: acceleration.z,
        },
        convention,
      ),
    );
  };

  window.addEventListener("devicemotion", onMotion);
  return () => window.removeEventListener("devicemotion", onMotion);
}
