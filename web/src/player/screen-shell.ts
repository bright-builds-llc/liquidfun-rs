/** Playground chrome. `canvas` is the full-viewport shell with the HUD. */
export type ScreenShell = "canvas" | "form-factor";

export const SCREEN_SHELL_PARAM = "shell";
export const CANVAS_SCREEN_SHELL = "canvas";
/** Restores the older page layouts that depend on viewport and form factor. */
export const FORM_FACTOR_SCREEN_SHELL = "form-factor";

/**
 * Canvas shell is the default at every screen size.
 * `?shell=form-factor` restores the older modes: the scrolling page when
 * element fullscreen is available, and the canvas shell on iPhone and
 * Android phones.
 */
export function screenShellFromSearch(search: string): ScreenShell {
  const value = new URLSearchParams(search).get(SCREEN_SHELL_PARAM);
  if (value === FORM_FACTOR_SCREEN_SHELL) {
    return "form-factor";
  }

  return "canvas";
}

/** Inserts the form-factor shell query without dropping a hash route. */
export function formFactorShellHref(path: string): string {
  const hashIndex = path.indexOf("#");
  const beforeHash = hashIndex === -1 ? path : path.slice(0, hashIndex);
  const hash = hashIndex === -1 ? "" : path.slice(hashIndex);
  const separator = beforeHash.includes("?") ? "&" : "?";
  return `${beforeHash}${separator}${SCREEN_SHELL_PARAM}=${FORM_FACTOR_SCREEN_SHELL}${hash}`;
}
