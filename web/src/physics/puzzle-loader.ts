import init, {
  inspectPuzzlePlacements,
  ProofSession,
} from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";
import type { PuzzleId } from "../catalog/puzzles";
import type { RawInspectFn } from "./puzzle-inspector";
import type { GeneratedPuzzleSession } from "./puzzle-session";

const NOT_INITIALIZED_MESSAGE = "Puzzle wasm is not initialized";

type PuzzleLanes = { readonly kinds: Uint8Array; readonly poses: Int32Array };

// The generated wasm module is a page-wide singleton, so its initialization state is module state too.
let initialized = false;
let maybeInitializing: Promise<void> | undefined;

async function ensureInitialized(): Promise<void> {
  if (initialized) {
    return;
  }

  maybeInitializing ??= init({ module_or_path: wasmUrl }).then(
    () => {
      initialized = true;
    },
    (error: unknown) => {
      maybeInitializing = undefined;
      throw error;
    },
  );
  await maybeInitializing;
}

function requireInitialized(): void {
  if (!initialized) {
    throw new Error(NOT_INITIALIZED_MESSAGE);
  }
}

/** Initializes the generated package once and builds a puzzle level on the direct backend (D-04). */
export async function loadPuzzleSession(
  id: PuzzleId,
  lanes: PuzzleLanes,
): Promise<GeneratedPuzzleSession> {
  await ensureInitialized();
  return buildPuzzleSessionSync(id, lanes);
}

/** Builds a puzzle session synchronously; throws "Puzzle wasm is not initialized" before the first load. */
export function buildPuzzleSessionSync(id: PuzzleId, lanes: PuzzleLanes): GeneratedPuzzleSession {
  requireInitialized();
  return ProofSession.newPuzzle(id, lanes.kinds, lanes.poses);
}

/** Generated inspector for `inspectPlacements`; throws before init. */
export const inspectPlacementLanes: RawInspectFn = (levelId, kinds, poses) => {
  requireInitialized();
  return inspectPuzzlePlacements(levelId, kinds, poses);
};
