/**
 * Building-only canvas editor pointer (D-17, D-18, D-20).
 *
 * A separate handle from the scene pointer (Research Pitfall 6): it never
 * forwards to the scene session, never touches the scene pointer counter and
 * never sets the scene crosshair class. Tapping with an armed kind places a
 * snapped tool; pressing a tool selects it and dragging past the threshold
 * moves it with the grab offset kept; tapping empty canvas deselects.
 */

import { cssPointFromClient } from "../input/pointer";
import type { PuzzleEditEvent } from "../puzzle/edit-event";
import { snapWorldToGrid, type BuildingEditor } from "../puzzle/editor";
import { pickTool, type HitCandidate } from "../puzzle/hit-test";
import type { PuzzleToolKind } from "../puzzle/placements";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { unprojectPoint, type Camera, type Point } from "../render/camera";

/** What the binding reads and commands; implemented by the scene surface. */
export type PuzzleEditorInputDeps = {
  readonly state: () => PuzzleState;
  readonly camera: () => Camera | undefined;
  readonly gridMeters: () => number | undefined;
  readonly edit: (event: PuzzleEditEvent, mode: "commit" | "live") => void;
  /** Commits an inspection after a drag (inspectNow). */
  readonly settle: () => void;
};

/** Handle returned by `attachPuzzleEditorInput`. */
export type PuzzleEditorInput = { cancel(): void; detach(): void };

/** Touch drags draw and snap the tool this far above the finger (UI-SPEC spacing 2xl). */
export const TOUCH_LIFT_CSS_PX = 48;
/** A mouse press becomes a drag after moving past this distance. */
export const MOUSE_DRAG_THRESHOLD_CSS_PX = 4;
/** A touch press becomes a drag after moving past this distance. */
export const TOUCH_DRAG_THRESHOLD_CSS_PX = 8;

const DRAGGING_CLASS = "puzzle-canvas--dragging";
const OVER_TOOL_CLASS = "puzzle-canvas--over-tool";

const POINTER_EVENT_NAMES = [
  "pointerdown",
  "pointermove",
  "pointerup",
  "pointercancel",
  "lostpointercapture",
] as const;

type Cells = { readonly x: number; readonly y: number };

/** Grid cells under a dragged tool: pointer plus grab offset, lifted for touch, then snapped. */
export function dragTargetCells(
  input: { readonly grabOffset: Point; readonly pointerWorld: Point; readonly liftMeters: number },
  gridMeters: number,
): Cells {
  return snapWorldToGrid(
    {
      x: input.pointerWorld.x + input.grabOffset.x,
      y: input.pointerWorld.y + input.grabOffset.y + input.liftMeters,
    },
    gridMeters,
  );
}

/** What the current press will do: place an armed kind, move a tool, or deselect on a tap. */
type PressTarget =
  | { readonly kind: "place"; readonly toolKind: PuzzleToolKind }
  | { readonly kind: "tool"; readonly id: number; readonly grabOffset: Point; cells: Cells }
  | { readonly kind: "empty" };

type Press = {
  readonly pointerId: number;
  readonly touch: boolean;
  readonly startCss: Point;
  readonly startWorld: Point;
  readonly target: PressTarget;
  /** Moved past the drag threshold: no longer a tap. */
  moved: boolean;
};

type PointerSample = { readonly css: Point; readonly world: Point; readonly camera: Camera };

function hitCandidates(editor: BuildingEditor): readonly HitCandidate[] {
  const maybeInspection = editor.maybeInspection;
  const fresh = maybeInspection?.outlines.length === editor.tools.length;
  return editor.tools.map((tool, index) => ({
    id: tool.id,
    maybeQuad: fresh ? maybeInspection?.outlines[index] : undefined,
  }));
}

function maybeToolAt(editor: BuildingEditor, world: Point, camera: Camera): number | undefined {
  return pickTool(world, hitCandidates(editor), camera.scale, editor.maybeSelectedId);
}

function pressTarget(
  editor: BuildingEditor,
  sample: PointerSample,
  gridMeters: number,
): PressTarget {
  if (editor.maybeArmedKind !== undefined) {
    return { kind: "place", toolKind: editor.maybeArmedKind };
  }

  const maybeId = maybeToolAt(editor, sample.world, sample.camera);
  const maybeTool = editor.tools.find((tool) => tool.id === maybeId);
  if (maybeTool === undefined) {
    return { kind: "empty" };
  }

  const { x, y } = maybeTool.placement;
  const grabOffset = { x: x * gridMeters - sample.world.x, y: y * gridMeters - sample.world.y };
  return { kind: "tool", id: maybeTool.id, grabOffset, cells: { x, y } };
}

function pastThreshold(press: Press, css: Point): boolean {
  const threshold = press.touch ? TOUCH_DRAG_THRESHOLD_CSS_PX : MOUSE_DRAG_THRESHOLD_CSS_PX;
  return Math.hypot(css.x - press.startCss.x, css.y - press.startCss.y) > threshold;
}

/**
 * Only the primary pointer's main button edits (D-17, D-18). A right or
 * middle click keeps its context menu or auto-scroll and never places,
 * moves, selects or deselects a tool.
 */
function isEditingPress(event: PointerEvent): boolean {
  return event.isPrimary && event.button === 0;
}

/** Attaches the editor pointer to a puzzle canvas; every handler ignores events outside building. */
export function attachPuzzleEditorInput(
  canvas: HTMLCanvasElement,
  deps: PuzzleEditorInputDeps,
): PuzzleEditorInput {
  let maybePress: Press | undefined;

  function maybeSample(event: PointerEvent): PointerSample | undefined {
    const camera = deps.camera();
    const rect = canvas.getBoundingClientRect();
    const maybeCss = cssPointFromClient(event.clientX, event.clientY, rect);
    if (camera === undefined || maybeCss === undefined) {
      return undefined;
    }

    const world = unprojectPoint(camera, maybeCss);
    if (!Number.isFinite(world.x) || !Number.isFinite(world.y)) {
      return undefined;
    }
    return { css: maybeCss, world, camera };
  }

  function maybeBuildingEditor(): BuildingEditor | undefined {
    const state = deps.state();
    return state.kind === "building" ? state.editor : undefined;
  }

  function release(pointerId: number): void {
    if (canvas.hasPointerCapture(pointerId)) {
      canvas.releasePointerCapture(pointerId);
    }
  }

  /** Ends the press without editing. */
  function abandonPress(press: Press): void {
    maybePress = undefined;
    release(press.pointerId);
    canvas.classList.toggle(DRAGGING_CLASS, false);
  }

  /** Ends the press; a tool drag always settles with a committed inspection (D-20: no snap back). */
  function finishPress(press: Press): void {
    abandonPress(press);
    if (press.moved && press.target.kind === "tool" && maybeBuildingEditor() !== undefined) {
      deps.settle();
    }
  }

  function onDown(event: PointerEvent): void {
    // One pointer edits at a time; a second finger never steals the press.
    if (maybePress !== undefined || !isEditingPress(event)) {
      return;
    }

    const maybeEditor = maybeBuildingEditor();
    const maybeGrid = deps.gridMeters();
    const maybePointer = maybeSample(event);
    if (maybeEditor === undefined || maybeGrid === undefined || maybePointer === undefined) {
      return;
    }

    const target = pressTarget(maybeEditor, maybePointer, maybeGrid);
    maybePress = {
      pointerId: event.pointerId,
      touch: event.pointerType === "touch",
      startCss: maybePointer.css,
      startWorld: maybePointer.world,
      target,
      moved: false,
    };
    canvas.setPointerCapture(event.pointerId);
    if (target.kind === "tool") {
      deps.edit({ kind: "select", maybeId: target.id }, "commit");
    }
  }

  function dragTo(
    press: Press,
    target: Extract<PressTarget, { kind: "tool" }>,
    sample: PointerSample,
  ): void {
    const maybeGrid = deps.gridMeters();
    if (maybeGrid === undefined) {
      return;
    }

    const liftMeters = press.touch ? TOUCH_LIFT_CSS_PX / sample.camera.scale : 0;
    const cells = dragTargetCells(
      { grabOffset: target.grabOffset, pointerWorld: sample.world, liftMeters },
      maybeGrid,
    );
    if (cells.x === target.cells.x && cells.y === target.cells.y) {
      return;
    }

    target.cells = cells;
    deps.edit({ kind: "move", id: target.id, x: cells.x, y: cells.y }, "live");
  }

  function onMove(event: PointerEvent): void {
    const maybeEditor = maybeBuildingEditor();
    const maybePointer = maybeSample(event);
    if (maybeEditor === undefined) {
      canvas.classList.toggle(OVER_TOOL_CLASS, false);
      return;
    }
    if (maybePointer === undefined) {
      return;
    }

    const press = maybePress;
    if (press === undefined) {
      const hovering =
        event.pointerType === "mouse" &&
        maybeToolAt(maybeEditor, maybePointer.world, maybePointer.camera) !== undefined;
      canvas.classList.toggle(OVER_TOOL_CLASS, hovering);
      return;
    }
    if (press.pointerId !== event.pointerId) {
      return;
    }
    if (!press.moved && !pastThreshold(press, maybePointer.css)) {
      return;
    }

    press.moved = true;
    if (press.target.kind !== "tool") {
      return;
    }
    canvas.classList.toggle(DRAGGING_CLASS, true);
    dragTo(press, press.target, maybePointer);
  }

  function onUp(event: PointerEvent): void {
    const press = maybePress;
    if (press === undefined || press.pointerId !== event.pointerId) {
      return;
    }

    finishPress(press);
    if (!press.moved && maybeBuildingEditor() !== undefined) {
      tap(press);
    }
  }

  /** Tap-to-place uses the press point and never lifts (Research Pitfall 9). */
  function tap(press: Press): void {
    const { target } = press;
    if (target.kind === "empty") {
      deps.edit({ kind: "select", maybeId: undefined }, "commit");
      return;
    }
    if (target.kind !== "place") {
      return;
    }

    const maybeGrid = deps.gridMeters();
    if (maybeGrid === undefined) {
      return;
    }
    const { x, y } = snapWorldToGrid(press.startWorld, maybeGrid);
    deps.edit({ kind: "place", toolKind: target.toolKind, x, y }, "commit");
  }

  function onCancel(event: PointerEvent): void {
    const press = maybePress;
    if (press === undefined || press.pointerId !== event.pointerId) {
      return;
    }

    finishPress(press);
  }

  function onPointerEvent(event: PointerEvent): void {
    switch (event.type) {
      case "pointerdown":
        onDown(event);
        return;
      case "pointermove":
        onMove(event);
        return;
      case "pointerup":
        onUp(event);
        return;
      default:
        onCancel(event);
    }
  }

  function cancel(): void {
    canvas.classList.toggle(OVER_TOOL_CLASS, false);
    if (maybePress !== undefined) {
      finishPress(maybePress);
    }
  }

  function detach(): void {
    cancel();
    for (const name of POINTER_EVENT_NAMES) {
      canvas.removeEventListener(name, onPointerEvent);
    }
  }

  for (const name of POINTER_EVENT_NAMES) {
    canvas.addEventListener(name, onPointerEvent);
  }

  return { cancel, detach };
}
