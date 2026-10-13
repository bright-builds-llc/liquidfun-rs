import { inspectPlacements } from "../physics/puzzle-inspector";
import { inspectPlacementLanes } from "../physics/puzzle-loader";
import type { PuzzleEditEvent } from "../puzzle/edit-event";
import {
  buildAreaCenterCells,
  cycleSelection,
  placementsInOrder,
  type BuildingEditor,
  type InspectionMode,
} from "../puzzle/editor";
import { editAnnouncement } from "../puzzle/editor-announce";
import type { PuzzleKeyCommand } from "../puzzle/keyboard";
import type { PuzzleToolKind } from "../puzzle/placements";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { dispatch, publishState, type PuzzleHost } from "./puzzle-host";

/** `commit` inspects now and announces; `live` coalesces to one inspection per animation frame (D-02). */
export type EditMode = "commit" | "live";

/** Key commands that edit the building editor; Space's Play/Stop toggle is a transport command. */
export type EditorKeyCommand = Exclude<PuzzleKeyCommand, { kind: "togglePlay" }>;

/** Cancels a pending coalesced live inspection. */
export function cancelPendingInspection(host: PuzzleHost): void {
  const maybeFrame = host.puzzle.maybeInspectionFrame;
  host.puzzle.maybeInspectionFrame = undefined;
  if (maybeFrame !== undefined) {
    cancelAnimationFrame(maybeFrame);
  }
}

/** Runs the stateless Rust inspector on the editor in creation order and folds the answer in. */
function dispatchInspection(host: PuzzleHost, mode: InspectionMode): PuzzleState {
  const maybeId = host.puzzle.maybeLevelId();
  if (maybeId === undefined) {
    throw new Error("Puzzle level is unavailable");
  }

  const { editor } = host.puzzle.current;
  const inspection = inspectPlacements(inspectPlacementLanes, maybeId, placementsInOrder(editor));
  return dispatch(host, {
    kind: "inspected",
    inspection: { ...inspection, revision: editor.revision },
    mode,
  });
}

/**
 * Commits an inspection without drawing or publishing, for callers that
 * present the building state themselves (level load, Stop). Throws on
 * inspector failure; callers route it to `failScene`.
 */
export function commitInspection(host: PuzzleHost): PuzzleState {
  cancelPendingInspection(host);
  return host.puzzle.current.kind === "building"
    ? dispatchInspection(host, "commit")
    : host.puzzle.current;
}

/** Redraws the last presented frame so the overlay shows the current editor (building only). */
export function redrawBuilding(host: PuzzleHost): void {
  const context = host.maybeContext;
  const maybeFrame = host.clock.maybePreviousFrame;
  const maybeCamera = host.clock.maybeCamera;
  if (host.puzzle.current.kind !== "building") {
    return;
  }
  if (context === undefined || maybeFrame === undefined || maybeCamera === undefined) {
    return;
  }

  host.drawSceneFrame(context, maybeFrame, maybeCamera);
}

function commitNow(host: PuzzleHost): PuzzleState {
  const state = commitInspection(host);
  redrawBuilding(host);
  publishState(host, state);
  return state;
}

/** Commits an inspection now (pointer-up after a drag), then redraws and publishes. */
export function inspectNow(host: PuzzleHost): void {
  if (host.puzzle.current.kind !== "building") {
    return;
  }

  try {
    commitNow(host);
  } catch (error) {
    host.failScene(error);
  }
}

/** Animation-frame callback: live inspection updates only the tint and outlines, never copy (UI-D14). */
function inspectLive(host: PuzzleHost): void {
  host.puzzle.maybeInspectionFrame = undefined;
  if (host.puzzle.current.kind !== "building") {
    return;
  }

  try {
    const state = dispatchInspection(host, "live");
    redrawBuilding(host);
    publishState(host, state);
  } catch (error) {
    host.failScene(error);
  }
}

function scheduleLiveInspection(host: PuzzleHost): void {
  if (host.puzzle.maybeInspectionFrame !== undefined) {
    return;
  }

  host.puzzle.maybeInspectionFrame = requestAnimationFrame(() => inspectLive(host));
}

function announce(
  host: PuzzleHost,
  before: BuildingEditor,
  after: BuildingEditor,
  event: PuzzleEditEvent,
): void {
  const maybeText = editAnnouncement(before, after, event);
  if (maybeText !== undefined) {
    host.puzzle.setAnnouncement(maybeText);
  }
}

/**
 * Applies one edit in building (D-25); every other phase ignores it. A
 * commit inspects now, redraws, publishes and announces; a live edit
 * publishes now and coalesces its inspection to one animation frame (D-02).
 */
export function applyPuzzleEdit(host: PuzzleHost, event: PuzzleEditEvent, mode: EditMode): void {
  const before = host.puzzle.current;
  if (before.kind !== "building") {
    return;
  }

  try {
    const edited = dispatch(host, event);
    if (edited === before) {
      return;
    }
    if (mode === "live") {
      publishState(host, edited);
      scheduleLiveInspection(host);
      return;
    }

    const after = commitNow(host);
    announce(host, before.editor, after.editor, event);
  } catch (error) {
    host.failScene(error);
  }
}

/** Tray press: arms the kind, or disarms it when already armed (D-17). */
export function toggleArm(host: PuzzleHost, kind: PuzzleToolKind): void {
  applyPuzzleEdit(host, { kind: "arm", toolKind: kind }, "commit");
}

/** Tray Enter: places one tool at the build-area centre cell, angle 0, and selects it (D-23). */
export function placeAtCenter(host: PuzzleHost, kind: PuzzleToolKind): void {
  const maybeInfo = host.puzzle.maybeLevelInfo();
  if (maybeInfo === undefined) {
    return;
  }

  const { x, y } = buildAreaCenterCells(maybeInfo.buildAreaCells);
  applyPuzzleEdit(host, { kind: "place", toolKind: kind, x, y }, "commit");
}

function editSelected(host: PuzzleHost, editor: BuildingEditor, event: PuzzleEditEvent): boolean {
  if (editor.maybeSelectedId === undefined) {
    return false;
  }

  applyPuzzleEdit(host, event, "commit");
  return true;
}

/**
 * Applies a D-23 editor key in building. Returns true when handled, so the
 * caller calls `preventDefault`; Tab past the last tool is unhandled so focus
 * can leave the canvas (UI-D18).
 */
export function applyKeyCommand(host: PuzzleHost, command: EditorKeyCommand): boolean {
  const current = host.puzzle.current;
  if (current.kind !== "building") {
    return false;
  }

  const { editor } = current;
  switch (command.kind) {
    case "nudge":
      return editSelected(host, editor, { kind: "nudge", dx: command.dx, dy: command.dy });
    case "rotate":
      return editSelected(host, editor, { kind: "rotate", direction: command.direction });
    case "remove":
      return editSelected(host, editor, { kind: "remove" });
    case "escape":
      applyPuzzleEdit(host, { kind: "disarm" }, "commit");
      applyPuzzleEdit(host, { kind: "select", maybeId: undefined }, "commit");
      return true;
    case "cycleSelection": {
      const maybeNext = cycleSelection(editor, command.direction);
      if (maybeNext === undefined) {
        return false;
      }
      applyPuzzleEdit(host, { kind: "select", maybeId: maybeNext }, "commit");
      return true;
    }
    default: {
      const unreachable: never = command;
      return unreachable;
    }
  }
}
