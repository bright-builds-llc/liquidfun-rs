import { describe, expect, it } from "vitest";

import { applyEditEvent, type PuzzleEditEvent } from "../src/puzzle/edit-event";
import {
  applyInspection,
  editorFromPlacements,
  type BuildingEditor,
} from "../src/puzzle/editor";
import { editAnnouncement } from "../src/puzzle/editor-announce";
import type { PlacementStatusCode } from "../src/puzzle/placement-status";
import type { PuzzlePlacement } from "../src/puzzle/placements";

const BUDGET = [{ kind: "plank", count: 2 }] as const;
const QUAD = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 1 },
  { x: 0, y: 1 },
] as const;
const TWO_PLANKS: readonly PuzzlePlacement[] = [
  { kind: "plank", x: 8, y: 64, angle: 0 },
  { kind: "plank", x: 20, y: 64, angle: 0 },
];

/** Stores a committed inspection for the editor's current revision. */
function committed(
  editor: BuildingEditor,
  statuses: readonly PlacementStatusCode[],
): BuildingEditor {
  return applyInspection(
    editor,
    { revision: editor.revision, statuses, outlines: statuses.map(() => QUAD) },
    "commit",
  );
}

function editor(
  placements: readonly PuzzlePlacement[],
  statuses: readonly PlacementStatusCode[],
): BuildingEditor {
  return committed(editorFromPlacements(placements, BUDGET), statuses);
}

/** Applies `event`, then commits an inspection reporting `statuses` for the result. */
function announce(
  before: BuildingEditor,
  event: PuzzleEditEvent,
  statuses: readonly PlacementStatusCode[],
): string | undefined {
  const after = committed(applyEditEvent(before, event), statuses);
  return editAnnouncement(before, after, event);
}

describe("editAnnouncement arming", () => {
  it("announces an armed kind", () => {
    // Arrange
    const before = editor([], []);

    // Act
    const text = announce(before, { kind: "arm", toolKind: "plank" }, []);

    // Assert
    expect(text).toBe("Plank ready. Click or tap the build area.");
  });

  it("announces a cancel when the armed kind is toggled off", () => {
    // Arrange
    const before = applyEditEvent(editor([], []), { kind: "arm", toolKind: "plank" });

    // Act
    const text = announce(before, { kind: "arm", toolKind: "plank" }, []);

    // Assert
    expect(text).toBe("Placement canceled.");
  });

  it("announces a cancel on disarm", () => {
    // Arrange
    const before = applyEditEvent(editor([], []), { kind: "arm", toolKind: "plank" });

    // Act
    const text = announce(before, { kind: "disarm" }, []);

    // Assert
    expect(text).toBe("Placement canceled.");
  });
});

describe("editAnnouncement placing and selecting", () => {
  it("announces a valid placement", () => {
    // Arrange
    const before = editor([], []);

    // Act
    const text = announce(before, { kind: "place", toolKind: "plank", x: 10, y: 64 }, [0]);

    // Assert
    expect(text).toBe("Plank placed.");
  });

  it("adds the reason to an invalid placement", () => {
    // Arrange
    const before = editor(TWO_PLANKS.slice(0, 1), [0]);

    // Act
    const text = announce(before, { kind: "place", toolKind: "plank", x: 8, y: 64 }, [11, 11]);

    // Assert
    expect(text).toBe("Plank placed. Overlaps another tool.");
  });

  it("announces the creation-order ordinal of a selected tool", () => {
    // Arrange
    const before = editor(TWO_PLANKS, [0, 0]);

    // Act
    const text = announce(before, { kind: "select", maybeId: 1 }, [0, 0]);

    // Assert
    expect(text).toBe("Plank 1 of 2 selected.");
  });

  it("adds the reason when the selected tool is invalid", () => {
    // Arrange
    const before = editor(TWO_PLANKS, [0, 8]);

    // Act
    const text = announce(before, { kind: "select", maybeId: 2 }, [0, 8]);

    // Assert
    expect(text).toBe("Plank 2 of 2 selected. Overlaps the cup.");
  });

  it("stays silent on deselect", () => {
    // Arrange
    const before = applyEditEvent(editor(TWO_PLANKS, [0, 0]), { kind: "select", maybeId: 1 });

    // Act
    const text = announce(before, { kind: "select", maybeId: undefined }, [0, 0]);

    // Assert
    expect(text).toBeUndefined();
  });
});

describe("editAnnouncement moving and rotating", () => {
  const selected = (statuses: readonly PlacementStatusCode[]) =>
    applyEditEvent(editor(TWO_PLANKS, statuses), { kind: "select", maybeId: 1 });

  it("stays silent on a nudge that keeps validity", () => {
    // Act
    const text = announce(selected([0, 0]), { kind: "nudge", dx: 1, dy: 0 }, [0, 0]);

    // Assert
    expect(text).toBeUndefined();
  });

  it("announces a nudge that makes the tool invalid", () => {
    // Act
    const text = announce(selected([0, 0]), { kind: "nudge", dx: 1, dy: 0 }, [8, 0]);

    // Assert
    expect(text).toBe("Now overlaps the cup.");
  });

  it("announces a nudge that makes the tool valid again", () => {
    // Act
    const text = announce(selected([8, 0]), { kind: "nudge", dx: -1, dy: 0 }, [0, 0]);

    // Assert
    expect(text).toBe("This tool fits again.");
  });

  it("announces a committed move only when validity changes", () => {
    // Act
    const text = announce(selected([0, 0]), { kind: "move", id: 2, x: 40, y: 64 }, [0, 6]);

    // Assert
    expect(text).toBe("Now outside the build area.");
  });

  it("announces a clockwise rotation with a minus sign", () => {
    // Act
    const text = announce(selected([0, 0]), { kind: "rotate", direction: -1 }, [0, 0]);

    // Assert
    expect(text).toBe("Rotated to −15°.");
  });

  it("adds a validity change to a rotation", () => {
    // Act
    const text = announce(selected([0, 0]), { kind: "rotate", direction: 1 }, [8, 0]);

    // Assert
    expect(text).toBe("Rotated to 15°. Now overlaps the cup.");
  });
});

describe("editAnnouncement removing", () => {
  it("announces the removed tool", () => {
    // Arrange
    const before = applyEditEvent(editor(TWO_PLANKS, [0, 0]), { kind: "select", maybeId: 2 });

    // Act
    const text = announce(before, { kind: "remove" }, [0]);

    // Assert
    expect(text).toBe("Plank removed.");
  });

  it("announces Clear", () => {
    // Act
    const text = announce(editor(TWO_PLANKS, [0, 0]), { kind: "clear" }, []);

    // Assert
    expect(text).toBe("All tools cleared.");
  });

  it("stays silent on an inspection", () => {
    // Arrange
    const before = editor(TWO_PLANKS, [0, 0]);
    const event: PuzzleEditEvent = {
      kind: "inspected",
      inspection: { revision: before.revision, statuses: [0, 0], outlines: [QUAD, QUAD] },
      mode: "commit",
    };

    // Act
    const text = editAnnouncement(before, applyEditEvent(before, event), event);

    // Assert
    expect(text).toBeUndefined();
  });
});
