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
      readonly screenAngleDegrees: number;
    };

export type TiltSampleResult =
  | {
      readonly kind: "live";
      readonly sample: AccelerationSample;
      readonly gravity: TiltGravity;
      readonly screenAngleDegrees: number;
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

const FULL_TURN_DEGREES = 360;
const QUARTER_TURNS_PER_TURN = 4;

export type ScreenAngleEnvironment = {
  readonly hasRequestPermission: boolean;
  readonly userAgent: string;
  readonly maybeScreenOrientationAngle: number | null;
  readonly maybeWindowOrientation: number | null;
};

/**
 * Turns device-frame acceleration into the current screen axes.
 *
 * `screenAngleDegrees` is clockwise from the device's natural orientation.
 * Browsers report that angle in quarter turns, including `-90` for `270`.
 */
export function rotateDeviceAccelerationToScreen(
  accelerationX: number,
  accelerationY: number,
  screenAngleDegrees: number,
): { readonly x: number; readonly y: number } {
  if (!Number.isFinite(screenAngleDegrees)) {
    return { x: accelerationX, y: accelerationY };
  }

  const quarterTurns = Math.round(
    (screenAngleDegrees / FULL_TURN_DEGREES) * QUARTER_TURNS_PER_TURN,
  );
  const quadrant =
    ((quarterTurns % QUARTER_TURNS_PER_TURN) + QUARTER_TURNS_PER_TURN) %
    QUARTER_TURNS_PER_TURN;
  if (quadrant === 1) {
    return { x: -accelerationY, y: accelerationX };
  }
  if (quadrant === 2) {
    return { x: -accelerationX, y: -accelerationY };
  }
  if (quadrant === 3) {
    return { x: accelerationY, y: -accelerationX };
  }
  return { x: accelerationX, y: accelerationY };
}

/**
 * Maps device acceleration, including gravity, into world gravity.
 *
 * Device +x points right and +y points toward the top in the phone's natural
 * orientation. Chrome, Firefox, and WebKit keep that frame when the page
 * rotates, so `screenAngleDegrees` maps it onto the current screen. World +y
 * is up, so water falls toward negative world y when the bottom of the screen
 * points at the ground.
 */
export function maybeWorldGravityFromAcceleration(
  accelerationX: number,
  accelerationY: number,
  convention: AccelerationConvention = "support-force",
  screenAngleDegrees = 0,
): TiltGravity | undefined {
  if (!Number.isFinite(accelerationX) || !Number.isFinite(accelerationY)) {
    return undefined;
  }

  const screenAcceleration = rotateDeviceAccelerationToScreen(
    accelerationX,
    accelerationY,
    screenAngleDegrees,
  );
  const sign = convention === "gravity-direction" ? 1 : -1;
  let x = sign * screenAcceleration.x;
  let y = sign * screenAcceleration.y;
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

/**
 * WebKit motion, including Chrome and Firefox on iPhone and iPad.
 *
 * `requestPermission` covers iPadOS desktop mode, whose user agent says Macintosh.
 */
function usesWebKitMotion(environment: {
  readonly hasRequestPermission: boolean;
  readonly userAgent: string;
}): boolean {
  if (environment.hasRequestPermission) {
    return true;
  }
  return /iPhone|iPad|iPod/.test(environment.userAgent);
}

function maybeFiniteDegrees(value: number | null): number | undefined {
  if (value === null || !Number.isFinite(value)) {
    return undefined;
  }
  return value;
}

/**
 * Clockwise degrees from the device's natural orientation.
 *
 * Chrome, Firefox, Samsung Internet, and Safari 17 and later agree on
 * `screen.orientation.angle`. Safari before iOS 17 reported the opposite
 * angle, while `window.orientation` stayed clockwise, so WebKit prefers that
 * legacy value when both exist. Other browsers prefer the standard angle
 * because Firefox removed `window.orientation`.
 */
export function screenAngleDegreesFromEnvironment(
  environment: ScreenAngleEnvironment,
): number {
  const webKit = usesWebKitMotion(environment);
  const maybeScreen = maybeFiniteDegrees(environment.maybeScreenOrientationAngle);
  const maybeWindow = maybeFiniteDegrees(environment.maybeWindowOrientation);
  const maybePreferred = webKit ? maybeWindow : maybeScreen;
  const maybeFallback = webKit ? maybeScreen : maybeWindow;
  return maybePreferred ?? maybeFallback ?? 0;
}

/** iOS reports the fall direction. Other browsers report the upward support force. */
export function accelerationConventionFromEnvironment(environment: {
  readonly hasRequestPermission: boolean;
  readonly userAgent: string;
}): AccelerationConvention {
  if (usesWebKitMotion(environment)) {
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
  screenAngleDegrees = 0,
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

  const appliedScreenAngle = Number.isFinite(screenAngleDegrees)
    ? screenAngleDegrees
    : 0;
  const maybeGravity = maybeWorldGravityFromAcceleration(
    sample.x,
    sample.y,
    convention,
    appliedScreenAngle,
  );
  if (maybeGravity === undefined) {
    return {
      kind: "problem",
      detail: "Could not map accelerationIncludingGravity to world gravity",
      sample,
    };
  }
  return {
    kind: "live",
    sample,
    gravity: maybeGravity,
    screenAngleDegrees: appliedScreenAngle,
  };
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
  const sample = formatSample(debug.sample);
  const gravity = `gravity ${formatAxis(debug.gravity.x)}, ${formatAxis(debug.gravity.y)}`;
  const screenAngle = `screen ${formatDegrees(debug.screenAngleDegrees)}°`;
  return `${sample}\n${gravity}\n${screenAngle}`;
}

function formatDegrees(degrees: number): string {
  if (!Number.isFinite(degrees)) {
    return String(degrees);
  }
  if (Number.isInteger(degrees)) {
    return String(degrees);
  }
  return degrees.toFixed(1);
}

type LegacyWindowOrientation = {
  readonly orientation?: number;
};

/** Reads the screen angle used to map device acceleration onto the page. */
export function currentScreenAngleDegrees(): number {
  const motion = DeviceMotionEvent as unknown as MotionPermissionTarget;
  const host = window as Window & LegacyWindowOrientation;
  const maybeWindowOrientation =
    typeof host.orientation === "number" ? host.orientation : null;
  const reportedAngle = screen.orientation?.angle;
  const maybeScreenOrientationAngle =
    typeof reportedAngle === "number" ? reportedAngle : null;
  return screenAngleDegreesFromEnvironment({
    hasRequestPermission: typeof motion.requestPermission === "function",
    userAgent: navigator.userAgent,
    maybeScreenOrientationAngle,
    maybeWindowOrientation,
  });
}

/** Listens for accelerometer samples and reports world gravity or a fault. */
export function listenForTiltGravity(
  onReport: (report: TiltSampleResult) => void,
): () => void {
  const convention = accelerationConvention();
  const onMotion = (event: DeviceMotionEvent) => {
    const screenAngleDegrees = currentScreenAngleDegrees();
    const acceleration = event.accelerationIncludingGravity;
    if (acceleration === null) {
      onReport(interpretAcceleration(null, convention, screenAngleDegrees));
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
        screenAngleDegrees,
      ),
    );
  };

  window.addEventListener("devicemotion", onMotion);
  return () => window.removeEventListener("devicemotion", onMotion);
}
