import { createSignal, type Accessor } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import type { PuzzlePlacement } from "../puzzle/placements";
import {
  EMPTY_PROGRESS,
  PROGRESS_STORAGE_KEY,
  maybeLevelStars,
  mergeProgress,
  parseProgress,
  serializeProgress,
  withPlacements,
  withStars,
  type ProgressDocument,
} from "../puzzle/progress";
import { restoreChoice, savedPlacementsFor } from "../puzzle/progress-view";
import type { LevelStars } from "../puzzle/stars";

/** Separate key for the availability probe, so the progress key is never touched by it (Pitfall 7). */
export const PROGRESS_PROBE_KEY = "liquidfun.puzzle-progress.probe";

/** The storage methods the shell uses. */
export type ProgressStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

/** Returns the browser store, or null when the browser exposes none; may throw. */
export type ProgressStorageProvider = () => ProgressStorage | null;

/** Provider for node tests and any context without storage: always degraded. */
export const UNAVAILABLE_PROGRESS_STORAGE: ProgressStorageProvider = () => null;

/** Per-browser puzzle progress (D-08). All methods are safe to call in degraded mode. */
export type PuzzleProgress = {
  /** True when storage is unavailable; every level unlocks and nothing is written (D-14). */
  readonly degraded: Accessor<boolean>;
  /** Persisted progress plus this session's changes. */
  readonly current: Accessor<ProgressDocument>;
  /** Merges a won level's best; writes through unless degraded; returns the merged best (D-09). */
  recordStars(id: PuzzleId, best: LevelStars): LevelStars;
  /** Saves the editor's placements in order with the live level hash (D-16). */
  savePlacements(id: PuzzleId, levelHash: string, placements: readonly PuzzlePlacement[]): void;
  /** Restorable placements, or undefined; discards a mismatched or malformed record (D-18). */
  maybeRestore(id: PuzzleId, liveHash: string): readonly PuzzlePlacement[] | undefined;
};

type OpenedStorage =
  | { readonly kind: "ready"; readonly storage: ProgressStorage; readonly progress: ProgressDocument }
  | { readonly kind: "degraded" };

const DEGRADED: OpenedStorage = { kind: "degraded" };

function isStorage(value: unknown): value is ProgressStorage {
  return typeof value === "object" && value !== null;
}

/** Provider, probe and first read; any failure or unusable value means degraded (D-14). */
function openStorage(provider: ProgressStorageProvider): OpenedStorage {
  let maybeStorage: unknown;
  try {
    maybeStorage = provider();
  } catch {
    // Degraded mode (D-14): the session continues in memory and never writes.
    return DEGRADED;
  }
  if (!isStorage(maybeStorage)) {
    return DEGRADED;
  }

  const storage = maybeStorage;
  try {
    storage.setItem(PROGRESS_PROBE_KEY, "1");
    storage.removeItem(PROGRESS_PROBE_KEY);
  } catch {
    // Degraded mode (D-14): a blocked probe keeps the session in memory and never writes.
    return DEGRADED;
  }

  const maybeProgress = maybeReread(storage);
  if (maybeProgress === undefined) {
    return DEGRADED;
  }
  return { kind: "ready", storage, progress: maybeProgress };
}

/** Re-reads the stored document; undefined means storage failed or holds an unusable value. */
function maybeReread(storage: ProgressStorage): ProgressDocument | undefined {
  let raw: string | null;
  try {
    raw = storage.getItem(PROGRESS_STORAGE_KEY);
  } catch {
    // Degraded mode (D-14): the caller switches to memory and never writes again.
    return undefined;
  }

  const parsed = parseProgress(raw);
  return parsed.kind === "ok" ? parsed.progress : undefined;
}

function tryWrite(storage: ProgressStorage, progress: ProgressDocument): boolean {
  try {
    storage.setItem(PROGRESS_STORAGE_KEY, serializeProgress(progress));
    return true;
  } catch {
    // Degraded mode (D-14): a quota or policy failure keeps this session in memory.
    return false;
  }
}

/** Opens progress storage once; call inside a Solid root so the signals have an owner. */
export function createPuzzleProgress(provider: ProgressStorageProvider): PuzzleProgress {
  const opened = openStorage(provider);
  let maybeStorage = opened.kind === "ready" ? opened.storage : undefined;
  const [degraded, setDegraded] = createSignal(opened.kind === "degraded");
  const [current, setCurrent] = createSignal<ProgressDocument>(
    opened.kind === "ready" ? opened.progress : EMPTY_PROGRESS,
  );

  function degrade(local: ProgressDocument): void {
    maybeStorage = undefined;
    setDegraded(true);
    setCurrent(local);
  }

  /** Applies a change in memory; unless degraded, re-reads, merges, reapplies and writes (D-09). */
  function commit(change: (progress: ProgressDocument) => ProgressDocument): void {
    const local = change(current());
    if (maybeStorage === undefined) {
      setCurrent(local);
      return;
    }

    const maybeStored = maybeReread(maybeStorage);
    if (maybeStored === undefined) {
      degrade(local);
      return;
    }

    const merged = change(mergeProgress(maybeStored, local));
    if (!tryWrite(maybeStorage, merged)) {
      degrade(local);
      return;
    }
    setCurrent(merged);
  }

  return {
    degraded,
    current,
    recordStars(id, best) {
      commit((progress) => withStars(progress, id, best));
      return maybeLevelStars(current(), id) ?? best;
    },
    savePlacements(id, levelHash, placements) {
      const saved = savedPlacementsFor(levelHash, placements);
      commit((progress) => withPlacements(progress, id, saved));
    },
    maybeRestore(id, liveHash) {
      const choice = restoreChoice(current(), id, liveHash);
      switch (choice.kind) {
        case "none":
          return undefined;
        case "restore":
          return choice.placements;
        case "discard":
          commit((progress) => withPlacements(progress, id, undefined));
          return undefined;
      }
    },
  };
}
