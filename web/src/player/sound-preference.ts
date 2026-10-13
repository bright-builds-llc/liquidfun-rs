/**
 * Opt-in puzzle sound preference (Phase 42 D-03). Kept separate from the
 * level-progress and first-hint keys so each can change on its own.
 */

export const SOUND_STORAGE_KEY = "liquidfun.puzzle-sound.v1";
const ON_TOKEN = "on";
const OFF_TOKEN = "off";

type SoundStorage = Pick<Storage, "getItem" | "setItem">;
export type SoundStorageProvider = () => SoundStorage;

/** The browser provider; each caller invokes it inside its own `try`. */
export const browserSoundStorage: SoundStorageProvider = () => window.localStorage;

/** Whether storage holds the opt-in; only the exact `"on"` token counts. */
export function loadSoundOn(provider: SoundStorageProvider): boolean {
  try {
    return provider().getItem(SOUND_STORAGE_KEY) === ON_TOKEN;
  } catch {
    // Unreadable storage means off (D-03): sound is never on without a choice.
    return false;
  }
}

/** Records the choice without turning storage failure into app failure. */
export function persistSoundOn(provider: SoundStorageProvider, on: boolean): void {
  try {
    provider().setItem(SOUND_STORAGE_KEY, on ? ON_TOKEN : OFF_TOKEN);
  } catch {
    // The in-memory choice still holds for this page session (D-03).
  }
}

/** Page-session sound choice backed by storage when it is available. */
export type SoundPreferenceMemory = {
  readonly on: () => boolean;
  readonly set: (on: boolean) => void;
};

/**
 * Reads storage lazily on the first `on()` call, so creating the memory never
 * touches storage. `set` updates memory first, so a failed write still holds.
 */
export function createSoundPreferenceMemory(
  provider: SoundStorageProvider,
): SoundPreferenceMemory {
  let maybeOn: boolean | undefined;
  return {
    on: () => {
      maybeOn ??= loadSoundOn(provider);
      return maybeOn;
    },
    set: (on) => {
      maybeOn = on;
      persistSoundOn(provider, on);
    },
  };
}
