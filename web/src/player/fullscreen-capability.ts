/** True when `Element.requestFullscreen` can succeed in this document. */
export function fullscreenApiAvailable(fullscreenEnabled: boolean): boolean {
  return fullscreenEnabled;
}

/**
 * Phones without element fullscreen, including iPhone Safari, use a
 * full-canvas stage instead of a full-screen button.
 */
export function canvasStageActive(fullscreenEnabled: boolean): boolean {
  return !fullscreenApiAvailable(fullscreenEnabled);
}
