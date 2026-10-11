/** Whether the on-canvas HUD slider card is shown. */
export type HudSlidersVisibility = "shown" | "hidden";

export const HUD_SLIDERS_STORAGE_KEY = "liquidfun.hud-sliders.v1";
export const DEFAULT_HUD_SLIDERS_VISIBILITY: HudSlidersVisibility = "shown";

type HudSlidersStorage = Pick<Storage, "getItem" | "setItem">;
export type HudSlidersStorageProvider = () => HudSlidersStorage;

/** Parses one persisted visibility token. */
export function maybeParseHudSlidersVisibility(
  maybeValue: string | null,
): HudSlidersVisibility | undefined {
  if (maybeValue === "shown" || maybeValue === "hidden") {
    return maybeValue;
  }

  return undefined;
}

/** Loads the slider preference; unavailable storage or unknown values read as shown. */
export function loadHudSlidersVisibility(
  storageProvider: HudSlidersStorageProvider,
): HudSlidersVisibility {
  try {
    return (
      maybeParseHudSlidersVisibility(
        storageProvider().getItem(HUD_SLIDERS_STORAGE_KEY),
      ) ?? DEFAULT_HUD_SLIDERS_VISIBILITY
    );
  } catch {
    return DEFAULT_HUD_SLIDERS_VISIBILITY;
  }
}

/** Persists the slider preference without turning storage failure into app failure. */
export function persistHudSlidersVisibility(
  storageProvider: HudSlidersStorageProvider,
  visibility: HudSlidersVisibility,
): void {
  try {
    storageProvider().setItem(HUD_SLIDERS_STORAGE_KEY, visibility);
  } catch {
    // The in-memory preference remains valid when storage is unavailable.
  }
}

/** Accessible label for the toggle, naming the action a click performs. */
export function hudSlidersToggleLabel(visibility: HudSlidersVisibility): string {
  return visibility === "shown" ? "Hide sliders" : "Show sliders";
}
