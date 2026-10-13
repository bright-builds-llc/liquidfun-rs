import { describe, expect, it } from "vitest";

import {
  EMPTY_EDITOR,
  allPlacementsValid,
  angleDegrees,
  applyInspection,
  armKind,
  buildAreaCenterCells,
  canonicalPlacements,
  clearInteraction,
  clearTools,
  committedPlacementsValid,
  cycleSelection,
  disarm,
  editorFromPlacements,
  isInspectionFresh,
  moveTool,
  nudgeSelected,
  placeTool,
  placementsInOrder,
  remainingCount,
  removeSelected,
  rotateSelected,
  selectTool,
  snapWorldToGrid,
  withBudget,
  type BuildingEditor,
  type EditorInspection,
} from "../src/puzzle/editor";
import type { PlacementStatusCode } from "../src/puzzle/placement-status";

const TWO_PLANKS = [{ kind: "plank", count: 2 }] as const;

function budgetEditor(): BuildingEditor {
  return withBudget(EMPTY_EDITOR, TWO_PLANKS);
}

function editorWithAngle(angle: number): BuildingEditor {
  return editorFromPlacements([{ kind: "plank", x: 10, y: 64, angle }], TWO_PLANKS);
}

function selectedFirst(editor: BuildingEditor): BuildingEditor {
  const first = editor.tools[0];
  if (first === undefined) {
    throw new Error("Test editor has no tools");
  }
  return selectTool(editor, first.id);
}

function inspection(
  editor: BuildingEditor,
  statuses: readonly PlacementStatusCode[],
): EditorInspection {
  return {
    revision: editor.revision,
    statuses,
    outlines: statuses.map(() => undefined),
  };
}

describe("remainingCount", () => {
  it("subtracts placed tools of the kind from the budget", () => {
    // Arrange
    const editor = placeTool(budgetEditor(), "plank", 10, 64);

    // Act
    const remaining = remainingCount(editor, "plank");

    // Assert
    expect(remaining).toBe(1);
  });

  it("is zero for a kind missing from the budget", () => {
    expect(remainingCount(EMPTY_EDITOR, "plank")).toBe(0);
  });
});

describe("armKind", () => {
  it("arms a kind with tools remaining", () => {
    // Act
    const editor = armKind(budgetEditor(), "plank");

    // Assert
    expect(editor.maybeArmedKind).toBe("plank");
  });

  it("disarms when the armed kind is armed again", () => {
    // Arrange
    const armed = armKind(budgetEditor(), "plank");

    // Act
    const editor = armKind(armed, "plank");

    // Assert
    expect(editor.maybeArmedKind).toBeUndefined();
  });

  it("returns the same editor for a kind with none left", () => {
    // Arrange
    const full = placeTool(placeTool(budgetEditor(), "plank", 10, 64), "plank", 30, 84);

    // Act
    const editor = armKind(full, "plank");

    // Assert
    expect(editor).toBe(full);
  });

  it("disarm returns the same editor when nothing is armed", () => {
    // Arrange
    const editor = budgetEditor();

    // Act / Assert
    expect(disarm(editor)).toBe(editor);
  });
});

describe("placeTool", () => {
  it("appends a snapped plank at angle 0 with the next id", () => {
    // Arrange
    const before = budgetEditor();

    // Act
    const editor = placeTool(before, "plank", 10, 64);

    // Assert
    expect(editor.tools).toEqual([
      { id: before.nextId, placement: { kind: "plank", x: 10, y: 64, angle: 0 } },
    ]);
  });

  it("selects the placed tool, disarms and bumps the revision", () => {
    // Arrange
    const before = armKind(budgetEditor(), "plank");

    // Act
    const editor = placeTool(before, "plank", 10, 64);

    // Assert
    expect([editor.maybeSelectedId, editor.maybeArmedKind, editor.revision]).toEqual([
      before.nextId,
      undefined,
      before.revision + 1,
    ]);
  });

  it("returns the same editor when the budget is spent", () => {
    // Arrange
    const full = placeTool(placeTool(budgetEditor(), "plank", 10, 64), "plank", 30, 84);

    // Act
    const editor = placeTool(full, "plank", -10, 50);

    // Assert
    expect(editor).toBe(full);
  });

  it("throws for a non-integer position", () => {
    // Act
    const act = () => placeTool(budgetEditor(), "plank", 10.5, 64);

    // Assert
    expect(act).toThrow("Invalid editor move");
  });
});

describe("selectTool", () => {
  it("returns the same editor for an unknown id", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(selectTool(editor, 99)).toBe(editor);
  });

  it("deselects with undefined", () => {
    // Arrange
    const selected = selectedFirst(editorWithAngle(0));

    // Act
    const editor = selectTool(selected, undefined);

    // Assert
    expect(editor.maybeSelectedId).toBeUndefined();
  });
});

describe("moveTool", () => {
  it("moves a tool and bumps the revision", () => {
    // Arrange
    const before = editorWithAngle(5);

    // Act
    const editor = moveTool(before, 1, 12, 60);

    // Assert
    expect([editor.tools[0]?.placement, editor.revision]).toEqual([
      { kind: "plank", x: 12, y: 60, angle: 5 },
      before.revision + 1,
    ]);
  });

  it("returns the same editor when the position is unchanged", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(moveTool(editor, 1, 10, 64)).toBe(editor);
  });

  it("throws for a non-integer position", () => {
    // Act
    const act = () => moveTool(editorWithAngle(0), 1, 10, Number.NaN);

    // Assert
    expect(act).toThrow("Invalid editor move");
  });
});

describe("rotateSelected", () => {
  it("wraps +1 from angle 23 to 0", () => {
    // Arrange
    const before = selectedFirst(editorWithAngle(23));

    // Act
    const editor = rotateSelected(before, 1);

    // Assert
    expect(editor.tools[0]?.placement.angle).toBe(0);
  });

  it("wraps -1 from angle 0 to 23", () => {
    // Arrange
    const before = selectedFirst(editorWithAngle(0));

    // Act
    const editor = rotateSelected(before, -1);

    // Assert
    expect(editor.tools[0]?.placement.angle).toBe(23);
  });

  it("returns the same editor with nothing selected", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(rotateSelected(editor, 1)).toBe(editor);
  });
});

describe("nudgeSelected", () => {
  it("adds cell deltas to the selected tool", () => {
    // Arrange
    const before = selectedFirst(editorWithAngle(0));

    // Act
    const editor = nudgeSelected(before, -5, 1);

    // Assert
    expect(editor.tools[0]?.placement).toEqual({ kind: "plank", x: 5, y: 65, angle: 0 });
  });

  it("returns the same editor with nothing selected", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(nudgeSelected(editor, 1, 0)).toBe(editor);
  });
});

describe("removeSelected", () => {
  it("removes the selected tool and clears the selection", () => {
    // Arrange
    const before = selectedFirst(editorWithAngle(0));

    // Act
    const editor = removeSelected(before);

    // Assert
    expect([editor.tools, editor.maybeSelectedId]).toEqual([[], undefined]);
  });

  it("returns the same editor with nothing selected", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(removeSelected(editor)).toBe(editor);
  });
});

describe("clearTools", () => {
  it("empties tools, disarms, deselects and bumps the revision", () => {
    // Arrange
    const before = armKind(selectedFirst(editorWithAngle(0)), "plank");

    // Act
    const editor = clearTools(before);

    // Assert
    expect([
      editor.tools,
      editor.maybeArmedKind,
      editor.maybeSelectedId,
      editor.revision,
    ]).toEqual([[], undefined, undefined, before.revision + 1]);
  });

  it("returns the same editor when it is already empty", () => {
    // Arrange
    const editor = budgetEditor();

    // Act / Assert
    expect(clearTools(editor)).toBe(editor);
  });
});

describe("clearInteraction", () => {
  it("deselects and disarms", () => {
    // Arrange
    const before = armKind(selectedFirst(editorWithAngle(0)), "plank");

    // Act
    const editor = clearInteraction(before);

    // Assert
    expect([editor.maybeSelectedId, editor.maybeArmedKind]).toEqual([undefined, undefined]);
  });
});

describe("applyInspection", () => {
  it("returns the same editor for a stale revision", () => {
    // Arrange
    const editor = editorWithAngle(0);
    const stale = { ...inspection(editor, [0]), revision: editor.revision - 1 };

    // Act / Assert
    expect(applyInspection(editor, stale, "commit")).toBe(editor);
  });

  it("returns the same editor when the status count differs from the tool count", () => {
    // Arrange
    const editor = editorWithAngle(0);

    // Act / Assert
    expect(applyInspection(editor, inspection(editor, [0, 0]), "commit")).toBe(editor);
  });

  it("live mode replaces only the live inspection", () => {
    // Arrange
    const committed = applyInspection(editorWithAngle(0), inspection(editorWithAngle(0), [0]), "commit");
    const live = inspection(committed, [9]);

    // Act
    const editor = applyInspection(committed, live, "live");

    // Assert
    expect([editor.maybeInspection, editor.maybeCommittedInspection]).toEqual([
      live,
      committed.maybeCommittedInspection,
    ]);
    expect(editor.maybeCommittedInspection).toBe(committed.maybeCommittedInspection);
  });

  it("commit mode replaces both inspections", () => {
    // Arrange
    const before = editorWithAngle(0);
    const answer = inspection(before, [8]);

    // Act
    const editor = applyInspection(before, answer, "commit");

    // Assert
    expect([editor.maybeInspection, editor.maybeCommittedInspection]).toEqual([answer, answer]);
  });
});

describe("placement validity", () => {
  it("keeps the committed snapshot valid during a live drag while the fresh gate closes", () => {
    // Arrange
    const start = editorWithAngle(0);
    const committed = applyInspection(start, inspection(start, [0]), "commit");
    const dragged = moveTool(committed, 1, 38, 44);
    const editor = applyInspection(dragged, inspection(dragged, [9]), "live");

    // Act
    const validity = [committedPlacementsValid(editor), allPlacementsValid(editor)];

    // Assert
    expect(validity).toEqual([true, false]);
  });

  it("treats a fresh all-valid commit inspection as playable", () => {
    // Arrange
    const start = editorWithAngle(0);
    const editor = applyInspection(start, inspection(start, [0]), "commit");

    // Act
    const validity = [isInspectionFresh(editor), allPlacementsValid(editor)];

    // Assert
    expect(validity).toEqual([true, true]);
  });

  it("is not fresh without a committed inspection", () => {
    expect(isInspectionFresh(editorWithAngle(0))).toBe(false);
  });
});

describe("canonicalPlacements", () => {
  it("sorts by kind code, x, y and angle without ids", () => {
    // Arrange
    const editor = editorFromPlacements(
      [
        { kind: "plank", x: 30, y: 84, angle: 0 },
        { kind: "plank", x: 8, y: 64, angle: 23 },
      ],
      TWO_PLANKS,
    );

    // Act
    const placements = canonicalPlacements(editor);

    // Assert
    expect(placements).toEqual([
      { kind: "plank", x: 8, y: 64, angle: 23 },
      { kind: "plank", x: 30, y: 84, angle: 0 },
    ]);
  });

  it("placementsInOrder keeps creation order", () => {
    // Arrange
    const first = { kind: "plank", x: 30, y: 84, angle: 0 } as const;
    const second = { kind: "plank", x: 8, y: 64, angle: 23 } as const;
    const editor = editorFromPlacements([first, second], TWO_PLANKS);

    // Act / Assert
    expect(placementsInOrder(editor)).toEqual([first, second]);
  });
});

describe("cycleSelection", () => {
  const editor = editorFromPlacements(
    [
      { kind: "plank", x: 30, y: 84, angle: 0 },
      { kind: "plank", x: 8, y: 64, angle: 23 },
    ],
    TWO_PLANKS,
  );

  it("forward from no selection picks the first tool", () => {
    expect(cycleSelection(editor, 1)).toBe(1);
  });

  it("forward from the last tool leaves the tools", () => {
    expect(cycleSelection(selectTool(editor, 2), 1)).toBeUndefined();
  });

  it("backward from no selection picks the last tool", () => {
    expect(cycleSelection(editor, -1)).toBe(2);
  });

  it("backward from the first tool leaves the tools", () => {
    expect(cycleSelection(selectTool(editor, 1), -1)).toBeUndefined();
  });
});

describe("grid helpers", () => {
  it("snaps world meters to 0.05 m cells", () => {
    expect(snapWorldToGrid({ x: 0.4, y: 3.2 }, 0.05)).toEqual({ x: 8, y: 64 });
  });

  it("finds the build-area centre in cells", () => {
    expect(buildAreaCenterCells([-20, 44, 40, 84])).toEqual({ x: 10, y: 64 });
  });

  it.each([
    [0, 0],
    [1, 15],
    [12, 180],
    [23, -15],
  ])("angle index %i is %i degrees", (index, degrees) => {
    expect(angleDegrees(index)).toBe(degrees);
  });
});
