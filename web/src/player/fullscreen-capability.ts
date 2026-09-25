/** True when `Element.requestFullscreen` can succeed in this document. */
export function fullscreenApiAvailable(fullscreenEnabled: boolean): boolean {
  return fullscreenEnabled;
}

export type ClientHints = {
  readonly mobile?: boolean;
  readonly platform?: string;
};

export type FormFactorSource = {
  readonly userAgent: string;
  readonly maybeClientHints: ClientHints | null;
};

/**
 * Android phones report Chromium client hints (`platform: "Android"`,
 * `mobile: true`). Other Android browsers, including Firefox, put `Mobile`
 * in the user agent and leave it off for tablets.
 */
export function androidPhone(source: FormFactorSource): boolean {
  const maybeHints = source.maybeClientHints;
  if (maybeHints === null) {
    return androidMobileUserAgent(source.userAgent);
  }

  const platform = maybeHints.platform;
  if (typeof platform !== "string" || platform.length === 0) {
    return androidMobileUserAgent(source.userAgent);
  }
  if (platform !== "Android") {
    return false;
  }
  if (typeof maybeHints.mobile === "boolean") {
    return maybeHints.mobile;
  }

  return androidMobileUserAgent(source.userAgent);
}

function androidMobileUserAgent(userAgent: string): boolean {
  return /Android/i.test(userAgent) && /Mobile/i.test(userAgent);
}

export type CanvasStageInput = {
  readonly fullscreenEnabled: boolean;
  readonly androidPhone: boolean;
};

/**
 * iPhone Safari has no element fullscreen. Android phones do, and still use
 * the full-canvas stage so the simulation is the page.
 */
export function canvasStageActive(input: CanvasStageInput): boolean {
  return !fullscreenApiAvailable(input.fullscreenEnabled) || input.androidPhone;
}

type NavigatorWithHints = Navigator & {
  userAgentData?: unknown;
};

/** Reads the live document and navigator into the canvas-stage decision. */
export function readCanvasStage(
  fullscreenEnabled: boolean,
  navigatorLike: Navigator,
): boolean {
  return canvasStageActive({
    fullscreenEnabled,
    androidPhone: androidPhone({
      userAgent: navigatorLike.userAgent,
      maybeClientHints: clientHintsFrom(navigatorLike),
    }),
  });
}

function clientHintsFrom(navigatorLike: Navigator): ClientHints | null {
  const maybeData = (navigatorLike as NavigatorWithHints).userAgentData;
  if (maybeData == null || typeof maybeData !== "object") {
    return null;
  }

  const hints = maybeData as { mobile?: unknown; platform?: unknown };
  const hintsRecord: { mobile?: boolean; platform?: string } = {};
  if (typeof hints.mobile === "boolean") {
    hintsRecord.mobile = hints.mobile;
  }
  if (typeof hints.platform === "string") {
    hintsRecord.platform = hints.platform;
  }
  if (hintsRecord.mobile === undefined && hintsRecord.platform === undefined) {
    return null;
  }

  return hintsRecord;
}
