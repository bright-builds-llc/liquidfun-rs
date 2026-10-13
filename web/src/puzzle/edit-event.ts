import {
  applyInspection,
  armKind,
  clearTools,
  disarm,
  moveTool,
  nudgeSelected,
  placeTool,
  removeSelected,
  rotateSelected,
  selectTool,
  type BuildingEditor,
  type EditorInspection,
  type InspectionMode,
} from "./editor";
import type { PuzzleToolKind } from "./placements";

/** Building-only edits; the phase machine ignores them in every other phase (D-25). */
export type PuzzleEditEvent =
  | { readonly kind: "arm"; readonly toolKind: PuzzleToolKind }
  | { readonly kind: "disarm" }
  | {
      readonly kind: "place";
      readonly toolKind: PuzzleToolKind;
      readonly x: number;
      readonly y: number;
    }
  | { readonly kind: "select"; readonly maybeId: number | undefined }
  | { readonly kind: "move"; readonly id: number; readonly x: number; readonly y: number }
  | { readonly kind: "rotate"; readonly direction: 1 | -1 }
  | { readonly kind: "nudge"; readonly dx: number; readonly dy: number }
  | { readonly kind: "remove" }
  | { readonly kind: "clear" }
  | {
      readonly kind: "inspected";
      readonly inspection: EditorInspection;
      readonly mode: InspectionMode;
    };

/** Applies one edit with the pure editor operations; unchanged edits return the same editor. */
export function applyEditEvent(editor: BuildingEditor, event: PuzzleEditEvent): BuildingEditor {
  switch (event.kind) {
    case "arm":
      return armKind(editor, event.toolKind);
    case "disarm":
      return disarm(editor);
    case "place":
      return placeTool(editor, event.toolKind, event.x, event.y);
    case "select":
      return selectTool(editor, event.maybeId);
    case "move":
      return moveTool(editor, event.id, event.x, event.y);
    case "rotate":
      return rotateSelected(editor, event.direction);
    case "nudge":
      return nudgeSelected(editor, event.dx, event.dy);
    case "remove":
      return removeSelected(editor);
    case "clear":
      return clearTools(editor);
    case "inspected":
      return applyInspection(editor, event.inspection, event.mode);
    default: {
      const unreachable: never = event;
      return unreachable;
    }
  }
}
