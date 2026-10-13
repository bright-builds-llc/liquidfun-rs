/**
 * First-level placement hint dismissal (LEVEL-03, D-20). Kept small and
 * separate so the level-progress store can absorb it later.
 */

export const FIRST_HINT_STORAGE_KEY = "liquidfun.puzzle-first-hint.v1";
const DISMISSED_TOKEN = "dismissed";

type HintStorage = Pick<Storage, "getItem" | "setItem">;
export type HintStorageProvider = () => HintStorage;

/** The browser provider; each caller invokes it inside its own `try`. */
export const browserHintStorage: HintStorageProvider = () => window.localStorage;

/** Whether storage holds the dismissal; only the exact token counts. */
export function loadHintDismissed(provider: HintStorageProvider): boolean {
  try {
    return provider().getItem(FIRST_HINT_STORAGE_KEY) === DISMISSED_TOKEN;
  } catch {
    // Unreadable storage shows the hint; the worst case is seeing it again.
    return false;
  }
}

/** Records the dismissal without turning storage failure into app failure. */
export function persistHintDismissed(provider: HintStorageProvider): void {
  try {
    provider().setItem(FIRST_HINT_STORAGE_KEY, DISMISSED_TOKEN);
  } catch {
    // The in-memory dismissal still holds for this page session (D-20).
  }
}

/** Page-session dismissal state backed by storage when it is available. */
export type FirstHintMemory = {
  readonly dismissed: () => boolean;
  readonly dismiss: () => void;
};

/**
 * Reads storage lazily on the first `dismissed()` call, so creating the
 * memory never touches storage. Once dismissed, it stays dismissed.
 */
export function createFirstHintMemory(provider: HintStorageProvider): FirstHintMemory {
  let maybeDismissed: boolean | undefined;
  return {
    dismissed: () => {
      maybeDismissed ??= loadHintDismissed(provider);
      return maybeDismissed;
    },
    dismiss: () => {
      maybeDismissed = true;
      persistHintDismissed(provider);
    },
  };
}

/** Page-session instance shared by every puzzle route mount. */
export const firstHintMemory: FirstHintMemory = createFirstHintMemory(browserHintStorage);
