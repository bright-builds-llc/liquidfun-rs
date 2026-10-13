import type { Point } from "../render/camera";
import type { PlacementInspection } from "./placement-status";
import { PUZZLE_TOOL_CODES, type PuzzlePlacement, type PuzzleToolKind } from "./placements";

/** One placed tool; ids are editor-local and never reach wasm. */
export type EditorTool = { readonly id: number; readonly placement: PuzzlePlacement };

/** How many tools of one kind a level allows. */
export type ToolAllowance = { readonly kind: PuzzleToolKind; readonly count: number };

/** Inspector answer tagged with the editor revision it describes. */
export type EditorInspection = PlacementInspection & { readonly revision: number };

/** Pure building-phase editor state (D-17, D-19, D-23, D-25). */
export type BuildingEditor = {
  /** Creation order; inspector statuses map to these by index. */
  readonly tools: readonly EditorTool[];
  readonly nextId: number;
  readonly maybeSelectedId: number | undefined;
  readonly maybeArmedKind: PuzzleToolKind | undefined;
  readonly budget: readonly ToolAllowance[];
  /** Bumps on every placement change. */
  readonly revision: number;
  /** Latest inspection (live or commit): canvas tint, outlines and hit-testing. */
  readonly maybeInspection: EditorInspection | undefined;
  /** Last commit-mode inspection: copy, announcements and Play (UI-D14). */
  readonly maybeCommittedInspection: EditorInspection | undefined;
};

/** `live` answers drag frames; `commit` answers committed edits. */
export type InspectionMode = "commit" | "live";

/** No tools, no budget, nothing selected or armed. */
export const EMPTY_EDITOR: BuildingEditor = {
  tools: [],
  nextId: 1,
  maybeSelectedId: undefined,
  maybeArmedKind: undefined,
  budget: [],
  revision: 0,
  maybeInspection: undefined,
  maybeCommittedInspection: undefined,
};

const ANGLE_STEPS = 24;
const DEGREES_PER_STEP = 15;
const INVALID_MOVE_MESSAGE = "Invalid editor move";

function checkCells(...values: readonly number[]): void {
  if (!values.every(Number.isSafeInteger)) {
    throw new Error(INVALID_MOVE_MESSAGE);
  }
}

/** Replaces tools and bumps the revision; every placement change goes through here. */
function withTools(editor: BuildingEditor, tools: readonly EditorTool[]): BuildingEditor {
  return { ...editor, tools, revision: editor.revision + 1 };
}

function updateTool(
  editor: BuildingEditor,
  id: number,
  change: (placement: PuzzlePlacement) => PuzzlePlacement,
): BuildingEditor {
  const maybeTool = editor.tools.find((tool) => tool.id === id);
  if (maybeTool === undefined) {
    return editor;
  }

  const placement = change(maybeTool.placement);
  const unchanged =
    placement.x === maybeTool.placement.x &&
    placement.y === maybeTool.placement.y &&
    placement.angle === maybeTool.placement.angle;
  if (unchanged) {
    return editor;
  }

  return withTools(
    editor,
    editor.tools.map((tool) => (tool.id === id ? { id, placement } : tool)),
  );
}

/** Editor seeded from existing placements (ids 1..n in the given order). */
export function editorFromPlacements(
  placements: readonly PuzzlePlacement[],
  budget: readonly ToolAllowance[] = [],
): BuildingEditor {
  placements.forEach((placement) => checkCells(placement.x, placement.y, placement.angle));
  const tools = placements.map((placement, index) => ({ id: index + 1, placement }));
  return { ...EMPTY_EDITOR, tools, nextId: tools.length + 1, budget };
}

/** Replaces the per-kind budget. */
export function withBudget(
  editor: BuildingEditor,
  budget: readonly ToolAllowance[],
): BuildingEditor {
  return editor.budget === budget ? editor : { ...editor, budget };
}

/** Budget count minus placed tools of that kind, never below 0; 0 when the kind is not in the budget. */
export function remainingCount(editor: BuildingEditor, kind: PuzzleToolKind): number {
  const allowed = editor.budget
    .filter((allowance) => allowance.kind === kind)
    .reduce((sum, allowance) => sum + allowance.count, 0);
  const placed = editor.tools.filter((tool) => tool.placement.kind === kind).length;
  return Math.max(0, allowed - placed);
}

/** Arms a kind for placement; arming the armed kind disarms it (D-17). */
export function armKind(editor: BuildingEditor, kind: PuzzleToolKind): BuildingEditor {
  if (editor.maybeArmedKind === kind) {
    return disarm(editor);
  }
  if (remainingCount(editor, kind) === 0) {
    return editor;
  }

  return { ...editor, maybeArmedKind: kind };
}

/** Clears the armed kind. */
export function disarm(editor: BuildingEditor): BuildingEditor {
  return editor.maybeArmedKind === undefined ? editor : { ...editor, maybeArmedKind: undefined };
}

/** Places one tool at angle 0, selects it and disarms; ignored when the budget is spent (D-23). */
export function placeTool(
  editor: BuildingEditor,
  kind: PuzzleToolKind,
  x: number,
  y: number,
): BuildingEditor {
  checkCells(x, y);
  if (remainingCount(editor, kind) === 0) {
    return editor;
  }

  const id = editor.nextId;
  const tool: EditorTool = { id, placement: { kind, x, y, angle: 0 } };
  return {
    ...withTools(editor, [...editor.tools, tool]),
    nextId: id + 1,
    maybeSelectedId: id,
    maybeArmedKind: undefined,
  };
}

/** Selects a tool by id, or deselects with undefined; unknown ids are ignored. */
export function selectTool(editor: BuildingEditor, maybeId: number | undefined): BuildingEditor {
  if (editor.maybeSelectedId === maybeId) {
    return editor;
  }
  if (maybeId !== undefined && !editor.tools.some((tool) => tool.id === maybeId)) {
    return editor;
  }

  return { ...editor, maybeSelectedId: maybeId };
}

/** Moves a tool to integer cells. */
export function moveTool(editor: BuildingEditor, id: number, x: number, y: number): BuildingEditor {
  checkCells(x, y);
  return updateTool(editor, id, (placement) => ({ ...placement, x, y }));
}

/** Rotates the selected tool one τ/24 step; +1 is counterclockwise (D-19). */
export function rotateSelected(editor: BuildingEditor, direction: 1 | -1): BuildingEditor {
  if (editor.maybeSelectedId === undefined) {
    return editor;
  }

  return updateTool(editor, editor.maybeSelectedId, (placement) => ({
    ...placement,
    angle: (placement.angle + direction + ANGLE_STEPS) % ANGLE_STEPS,
  }));
}

/** Moves the selected tool by whole cells (D-25). */
export function nudgeSelected(editor: BuildingEditor, dx: number, dy: number): BuildingEditor {
  checkCells(dx, dy);
  if (editor.maybeSelectedId === undefined) {
    return editor;
  }

  return updateTool(editor, editor.maybeSelectedId, (placement) => ({
    ...placement,
    x: placement.x + dx,
    y: placement.y + dy,
  }));
}

/** Removes the selected tool and clears the selection. */
export function removeSelected(editor: BuildingEditor): BuildingEditor {
  const maybeId = editor.maybeSelectedId;
  if (maybeId === undefined) {
    return editor;
  }

  const tools = editor.tools.filter((tool) => tool.id !== maybeId);
  return { ...withTools(editor, tools), maybeSelectedId: undefined };
}

/** Removes every tool, disarms and deselects (D-21). */
export function clearTools(editor: BuildingEditor): BuildingEditor {
  if (editor.tools.length === 0) {
    return clearInteraction(editor);
  }

  return { ...withTools(editor, []), maybeSelectedId: undefined, maybeArmedKind: undefined };
}

/** Deselects and disarms. */
export function clearInteraction(editor: BuildingEditor): BuildingEditor {
  if (editor.maybeSelectedId === undefined && editor.maybeArmedKind === undefined) {
    return editor;
  }

  return { ...editor, maybeSelectedId: undefined, maybeArmedKind: undefined };
}

/** Stores an inspector answer; stale revisions and length mismatches are dropped (T-39-07). */
export function applyInspection(
  editor: BuildingEditor,
  inspection: EditorInspection,
  mode: InspectionMode,
): BuildingEditor {
  const matches =
    inspection.revision === editor.revision &&
    inspection.statuses.length === editor.tools.length &&
    inspection.outlines.length === editor.tools.length;
  if (!matches) {
    return editor;
  }
  if (mode === "live") {
    return { ...editor, maybeInspection: inspection };
  }

  return { ...editor, maybeInspection: inspection, maybeCommittedInspection: inspection };
}

/** Committed snapshot describes the current revision (Play gate). */
export function isInspectionFresh(editor: BuildingEditor): boolean {
  return editor.maybeCommittedInspection?.revision === editor.revision;
}

/** Fresh committed snapshot with every placement valid; gates Play. */
export function allPlacementsValid(editor: BuildingEditor): boolean {
  return isInspectionFresh(editor) && committedPlacementsValid(editor);
}

/** Display-only validity from the committed snapshot; ignores revision so live drags do not flicker Play. */
export function committedPlacementsValid(editor: BuildingEditor): boolean {
  const maybeCommitted = editor.maybeCommittedInspection;
  if (maybeCommitted === undefined || maybeCommitted.statuses.length !== editor.tools.length) {
    return false;
  }

  return maybeCommitted.statuses.every((status) => status === 0);
}

/** Next tool id in creation order, or undefined when cycling past either end. */
export function cycleSelection(editor: BuildingEditor, direction: 1 | -1): number | undefined {
  const ids = editor.tools.map((tool) => tool.id);
  const maybeSelected = editor.maybeSelectedId;
  if (maybeSelected === undefined) {
    return direction === 1 ? ids[0] : ids[ids.length - 1];
  }

  const index = ids.indexOf(maybeSelected);
  return index < 0 ? undefined : ids[index + direction];
}

/** Placements in creation order (inspector input; statuses map by index). */
export function placementsInOrder(editor: BuildingEditor): readonly PuzzlePlacement[] {
  return editor.tools.map((tool) => tool.placement);
}

function comparePlacements(a: PuzzlePlacement, b: PuzzlePlacement): number {
  return (
    PUZZLE_TOOL_CODES[a.kind] - PUZZLE_TOOL_CODES[b.kind] ||
    a.x - b.x ||
    a.y - b.y ||
    a.angle - b.angle
  );
}

/** Placements sorted by (kind code, x, y, angle), matching Rust `Placement: Ord` (D-27). */
export function canonicalPlacements(editor: BuildingEditor): readonly PuzzlePlacement[] {
  return [...placementsInOrder(editor)].sort(comparePlacements);
}

/** World meters to the nearest grid cell; `+ 0` folds -0 into 0. */
export function snapWorldToGrid(
  point: Point,
  gridMeters: number,
): { readonly x: number; readonly y: number } {
  const cellsPerMeter = Math.round(1 / gridMeters);
  if (!Number.isSafeInteger(cellsPerMeter) || cellsPerMeter <= 0) {
    throw new Error("Grid size must be a positive fraction of a meter");
  }

  return {
    x: Math.round(point.x * cellsPerMeter) + 0,
    y: Math.round(point.y * cellsPerMeter) + 0,
  };
}

/** Centre cell of a `[minX, minY, maxX, maxY]` build area. */
export function buildAreaCenterCells(
  cells: readonly [number, number, number, number],
): { readonly x: number; readonly y: number } {
  const [minX, minY, maxX, maxY] = cells;
  return { x: Math.round((minX + maxX) / 2), y: Math.round((minY + maxY) / 2) };
}

/** Signed degrees for an angle index: 0..12 counterclockwise, 13..23 as negative clockwise. */
export function angleDegrees(angleIndex: number): number {
  const half = ANGLE_STEPS / 2;
  return angleIndex <= half
    ? angleIndex * DEGREES_PER_STEP
    : (angleIndex - ANGLE_STEPS) * DEGREES_PER_STEP;
}
