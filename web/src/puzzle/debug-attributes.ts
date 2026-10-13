import { canonicalPlacements, type BuildingEditor } from "./editor";
import type { PuzzlePlacement } from "./placements";
import { canPlay, isDeciding, type PuzzleState } from "./reduce-puzzle";

/**
 * One placement token: `plank:x,y,angle` (the Phase 39 form, kept so its e2e
 * literals stay valid) or `valve:x,y,angle,openTick` (Phase 40 D-02).
 */
function placementToken(placement: PuzzlePlacement): string {
  const pose = `${placement.kind}:${placement.x},${placement.y},${placement.angle}`;
  return placement.kind === "valve" ? `${pose},${placement.openTick}` : pose;
}

/** Canonical placement tokens joined by `;` (D-27); empty for none. */
function placementsAttribute(editor: BuildingEditor): string {
  return canonicalPlacements(editor).map(placementToken).join(";");
}

/** Creation-order index of the selected tool, or empty. */
function selectedAttribute(editor: BuildingEditor): string {
  const index = editor.tools.findIndex((tool) => tool.id === editor.maybeSelectedId);
  return index < 0 ? "" : String(index);
}

/**
 * Deterministic `data-puzzle-*` attributes for `main` (UI-SPEC e2e
 * attributes, D-27, D-28). Values are integers, booleans, tool kinds and the
 * held-frame hash only; no user data.
 */
export function puzzleDebugAttributes(
  state: PuzzleState,
  maybeHeldHash: string | undefined,
): Readonly<Record<string, string>> {
  const { editor } = state;
  const attributes: Record<string, string> = {
    "data-puzzle-placements": placementsAttribute(editor),
    "data-puzzle-selected": selectedAttribute(editor),
    "data-puzzle-armed": editor.maybeArmedKind ?? "",
    "data-puzzle-speed": String(state.speed),
    "data-puzzle-play-enabled": String(canPlay(state)),
    "data-puzzle-deciding": String(isDeciding(state)),
  };
  if (state.kind === "ended") {
    attributes["data-puzzle-outcome-step"] = String(state.latch.outcomeStep);
    attributes["data-puzzle-decided-step"] = String(state.latch.decidedStep);
    attributes["data-puzzle-peak-held"] = String(state.latch.peakHeldCount);
  }
  if (maybeHeldHash !== undefined) {
    attributes["data-puzzle-held-hash"] = maybeHeldHash;
  }
  return attributes;
}
