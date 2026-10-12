import init, {
  ProofSession,
} from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";
import type { PuzzleId } from "../catalog/puzzles";
import type { GeneratedPuzzleSession } from "./puzzle-session";

/** Initializes the generated package once and builds a puzzle level on the direct backend (D-04). */
export async function loadPuzzleSession(
  id: PuzzleId,
  lanes: { readonly kinds: Uint8Array; readonly poses: Int32Array },
): Promise<GeneratedPuzzleSession> {
  await init({ module_or_path: wasmUrl });
  return ProofSession.newPuzzle(id, lanes.kinds, lanes.poses);
}
