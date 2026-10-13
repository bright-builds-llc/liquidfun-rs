import { describe, expect, it, vi } from "vitest";

import type { PuzzleFrameStatus } from "../src/physics/frame";
import {
  attachPuzzleEditorInput,
  dragTargetCells,
  TOUCH_LIFT_CSS_PX,
  type PuzzleEditorInputDeps,
} from "../src/player/puzzle-editor-input";
import type { PuzzleEditEvent } from "../src/puzzle/edit-event";
import {
  applyInspection,
  armKind,
  editorFromPlacements,
  type BuildingEditor,
} from "../src/puzzle/editor";
import type { Quad } from "../src/puzzle/placement-status";
import type { PuzzleState } from "../src/puzzle/reduce-puzzle";
import type { Camera } from "../src/render/camera";

const GRID_METERS = 0.05;
/** 100 CSS px per meter over a 4 m square: world (x, y) is CSS (100x, 400 - 100y). */
const CAMERA: Camera = {
  viewport: { width: 400, height: 400 },
  bounds: { minX: 0, minY: 0, maxX: 4, maxY: 4 },
  scale: 100,
  offsetX: 0,
  offsetY: 0,
};
/** Plank centred at cells (8, 64) = world (0.4, 3.2), 0.5 m long and 0.1 m thick. */
const PLANK_QUAD: Quad = [
  { x: 0.15, y: 3.15 },
  { x: 0.65, y: 3.15 },
  { x: 0.65, y: 3.25 },
  { x: 0.15, y: 3.25 },
];
const PLANK_CSS = { x: 40, y: 80 } as const;
const EMPTY_CSS = { x: 300, y: 300 } as const;

type Listener = (event: PointerEvent) => void;

function status(): PuzzleFrameStatus {
  return {
    goalCount: 0,
    goalTarget: 540,
    goalFull: 900,
    outcome: "running",
    stepLimit: 720,
    peakHeldCount: 0,
    brimCount: 810,
  };
}

function plankEditor(): BuildingEditor {
  const editor = editorFromPlacements([{ kind: "plank", x: 8, y: 64, angle: 0 }], [
    { kind: "plank", count: 2 },
  ]);
  return applyInspection(
    editor,
    { revision: editor.revision, statuses: [0], outlines: [PLANK_QUAD] },
    "commit",
  );
}

function building(editor: BuildingEditor): PuzzleState {
  return { kind: "building", status: status(), editor, speed: 1 };
}

function fakeCanvas() {
  const listeners = new Map<string, Listener[]>();
  const classes = new Set<string>();
  const captured = new Set<number>();
  const canvas = {
    addEventListener: vi.fn((name: string, listener: Listener) => {
      listeners.set(name, [...(listeners.get(name) ?? []), listener]);
    }),
    removeEventListener: vi.fn((name: string, listener: Listener) => {
      listeners.set(
        name,
        (listeners.get(name) ?? []).filter((candidate) => candidate !== listener),
      );
    }),
    getBoundingClientRect: () => ({ left: 0, top: 0, width: 400, height: 400 }),
    setPointerCapture: vi.fn((id: number) => captured.add(id)),
    hasPointerCapture: (id: number) => captured.has(id),
    releasePointerCapture: vi.fn((id: number) => captured.delete(id)),
    classList: {
      toggle: (name: string, force?: boolean) => {
        const on = force ?? !classes.has(name);
        if (on) classes.add(name);
        else classes.delete(name);
        return on;
      },
      add: (...names: string[]) => names.forEach((name) => classes.add(name)),
      remove: (...names: string[]) => names.forEach((name) => classes.delete(name)),
      contains: (name: string) => classes.has(name),
    },
  };
  function fire(
    type: string,
    point: { readonly x: number; readonly y: number },
    pointerType = "mouse",
    pointerId = 1,
  ): void {
    const event = { type, pointerId, pointerType, clientX: point.x, clientY: point.y };
    for (const listener of listeners.get(type) ?? []) {
      listener(event as unknown as PointerEvent);
    }
  }
  return { canvas: canvas as unknown as HTMLCanvasElement, classes, listeners, fire };
}

function attach(initial: PuzzleState) {
  let state = initial;
  const edits: { event: PuzzleEditEvent; mode: "commit" | "live" }[] = [];
  const settle = vi.fn();
  const deps: PuzzleEditorInputDeps = {
    state: () => state,
    camera: () => CAMERA,
    gridMeters: () => GRID_METERS,
    edit: (event, mode) => edits.push({ event, mode }),
    settle,
  };
  const fake = fakeCanvas();
  const input = attachPuzzleEditorInput(fake.canvas, deps);
  return {
    ...fake,
    input,
    edits,
    settle,
    setState: (next: PuzzleState) => {
      state = next;
    },
  };
}

describe("dragTargetCells", () => {
  it("adds the grab offset to the pointer and snaps to the grid", () => {
    // Arrange
    const input = { grabOffset: { x: 0.1, y: 0 }, pointerWorld: { x: 0.3, y: 3.2 }, liftMeters: 0 };

    // Act
    const cells = dragTargetCells(input, GRID_METERS);

    // Assert
    expect(cells).toEqual({ x: 8, y: 64 });
  });

  it("lifts a touch drag by the lift distance before snapping", () => {
    // Arrange
    const liftMeters = TOUCH_LIFT_CSS_PX / 29.3;
    const input = { grabOffset: { x: 0, y: 0 }, pointerWorld: { x: 0.4, y: 3.2 }, liftMeters };

    // Act
    const cells = dragTargetCells(input, GRID_METERS);

    // Assert
    expect(cells).toEqual({ x: 8, y: Math.round((3.2 + liftMeters) / GRID_METERS) });
  });
});

describe("armed placement", () => {
  it("places a snapped plank at the tap point without a lift", () => {
    // Arrange
    const harness = attach(building(armKind(plankEditor(), "plank")));

    // Act
    harness.fire("pointerdown", { x: 40, y: 80 }, "touch");
    harness.fire("pointerup", { x: 41, y: 81 }, "touch");

    // Assert
    expect(harness.edits).toEqual([
      { event: { kind: "place", toolKind: "plank", x: 8, y: 64 }, mode: "commit" },
    ]);
  });

  it("does not place when the press moves past the threshold", () => {
    // Arrange
    const harness = attach(building(armKind(plankEditor(), "plank")));

    // Act
    harness.fire("pointerdown", { x: 200, y: 200 });
    harness.fire("pointermove", { x: 260, y: 200 });
    harness.fire("pointerup", { x: 260, y: 200 });

    // Assert
    expect(harness.edits).toEqual([]);
  });
});

describe("selection and dragging", () => {
  it("selects a pressed tool on press", () => {
    // Arrange
    const harness = attach(building(plankEditor()));

    // Act
    harness.fire("pointerdown", PLANK_CSS);

    // Assert
    expect(harness.edits).toEqual([{ event: { kind: "select", maybeId: 1 }, mode: "commit" }]);
  });

  it("drags past the mouse threshold with the grab offset kept", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", { x: 30, y: 80 });

    // Act
    harness.fire("pointermove", { x: 35, y: 80 });

    // Assert
    expect(harness.edits.at(-1)).toEqual({
      event: { kind: "move", id: 1, x: 9, y: 64 },
      mode: "live",
    });
  });

  it("dispatches a move only when the snapped cell changes", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS);
    harness.fire("pointermove", { x: 45, y: 80 });

    // Act
    harness.fire("pointermove", { x: 46, y: 80 });

    // Assert
    const moves = harness.edits.filter((edit) => edit.event.kind === "move");
    expect(moves).toEqual([{ event: { kind: "move", id: 1, x: 9, y: 64 }, mode: "live" }]);
  });

  it("settles with an inspection on release after a drag", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS);
    harness.fire("pointermove", { x: 50, y: 80 });

    // Act
    harness.fire("pointerup", { x: 50, y: 80 });

    // Assert
    expect(harness.settle).toHaveBeenCalledTimes(1);
  });

  it("does not drag a mouse press that moves 3 px", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS);

    // Act
    harness.fire("pointermove", { x: 43, y: 80 });
    harness.fire("pointerup", { x: 43, y: 80 });

    // Assert
    expect(harness.edits.map((edit) => edit.event.kind)).toEqual(["select"]);
    expect(harness.settle).not.toHaveBeenCalled();
  });

  it("does not drag a touch press that moves 7 px", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS, "touch");

    // Act
    harness.fire("pointermove", { x: 47, y: 80 }, "touch");

    // Assert
    expect(harness.edits.map((edit) => edit.event.kind)).toEqual(["select"]);
  });

  it("lifts a touch drag 48 CSS px above the finger", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS, "touch");

    // Act
    harness.fire("pointermove", { x: 50, y: 80 }, "touch");

    // Assert
    expect(harness.edits.at(-1)).toEqual({
      event: { kind: "move", id: 1, x: 10, y: Math.round((3.2 + 0.48) / GRID_METERS) },
      mode: "live",
    });
  });

  it("deselects on a tap on empty canvas", () => {
    // Arrange
    const harness = attach(building(plankEditor()));

    // Act
    harness.fire("pointerdown", EMPTY_CSS);
    harness.fire("pointerup", EMPTY_CSS);

    // Assert
    expect(harness.edits).toEqual([
      { event: { kind: "select", maybeId: undefined }, mode: "commit" },
    ]);
  });

  it("settles when the pointer is canceled during a drag", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS);
    harness.fire("pointermove", { x: 50, y: 80 });

    // Act
    harness.fire("pointercancel", { x: 50, y: 80 });

    // Assert
    expect(harness.settle).toHaveBeenCalledTimes(1);
    expect(harness.classes.has("puzzle-canvas--dragging")).toBe(false);
  });
});

describe("phase guard and classes", () => {
  it("ignores every pointer event outside building", () => {
    // Arrange
    const running: PuzzleState = {
      kind: "running",
      stepIndex: 10,
      status: status(),
      editor: plankEditor(),
      speed: 1,
    };
    const harness = attach(running);

    // Act
    harness.fire("pointerdown", PLANK_CSS);
    harness.fire("pointermove", { x: 60, y: 80 });
    harness.fire("pointerup", { x: 60, y: 80 });

    // Assert
    expect(harness.edits).toEqual([]);
    expect(harness.settle).not.toHaveBeenCalled();
    expect(harness.canvas.setPointerCapture).not.toHaveBeenCalled();
  });

  it("marks the canvas as dragging during a drag and never interactive", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointerdown", PLANK_CSS);

    // Act
    harness.fire("pointermove", { x: 50, y: 80 });

    // Assert
    expect(harness.classes.has("puzzle-canvas--dragging")).toBe(true);
    expect(harness.classes.has("canvas-interactive")).toBe(false);
  });

  it("marks mouse hover over a tool in building", () => {
    // Arrange
    const harness = attach(building(plankEditor()));

    // Act
    harness.fire("pointermove", PLANK_CSS);

    // Assert
    expect(harness.classes.has("puzzle-canvas--over-tool")).toBe(true);
  });

  it("clears the hover mark away from tools", () => {
    // Arrange
    const harness = attach(building(plankEditor()));
    harness.fire("pointermove", PLANK_CSS);

    // Act
    harness.fire("pointermove", EMPTY_CSS);

    // Assert
    expect(harness.classes.has("puzzle-canvas--over-tool")).toBe(false);
  });

  it("removes its listeners on detach", () => {
    // Arrange
    const harness = attach(building(plankEditor()));

    // Act
    harness.input.detach();
    harness.fire("pointerdown", PLANK_CSS);

    // Assert
    expect(harness.edits).toEqual([]);
  });
});
