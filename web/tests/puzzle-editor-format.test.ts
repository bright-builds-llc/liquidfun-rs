import { describe, expect, it } from "vitest";

import {
  applyInspection,
  armKind,
  editorFromPlacements,
  moveTool,
  selectTool,
  type BuildingEditor,
} from "../src/puzzle/editor";
import {
  CLEARED_ANNOUNCEMENT,
  PLACEMENT_CANCELED_ANNOUNCEMENT,
  armedAnnouncement,
  buildingMessage,
  placedAnnouncement,
  removedAnnouncement,
  rotatedAnnouncement,
  selectedAnnouncement,
  statusLine,
  stoppedAnnouncement,
  toolKindLabel,
  trayItems,
  validityChangeAnnouncement,
} from "../src/puzzle/editor-format";
import type { PlacementStatusCode } from "../src/puzzle/placement-status";

const GOAL = "Fill the cup to 60% before time runs out.";
const FAKE_BUDGET = [
  { kind: "plank", count: 2 },
  { kind: "pin", count: 1 },
] as const;
const PLANK_BUDGET = [{ kind: "plank", count: 3 }] as const;

function editorWith(count: number): BuildingEditor {
  const placements = Array.from({ length: count }, (_, index) => ({
    kind: "plank" as const,
    x: index * 10,
    y: 64,
    angle: 0,
  }));
  return editorFromPlacements(placements, PLANK_BUDGET);
}

function committed(
  editor: BuildingEditor,
  statuses: readonly PlacementStatusCode[],
): BuildingEditor {
  const inspection = {
    revision: editor.revision,
    statuses,
    outlines: statuses.map(() => undefined),
  };
  return applyInspection(editor, inspection, "commit");
}

describe("toolKindLabel", () => {
  it("capitalizes the kind", () => {
    expect(toolKindLabel("plank")).toBe("Plank");
  });
});

describe("trayItems", () => {
  const items = trayItems(FAKE_BUDGET, ["plank", "pin"], "plank", true);

  it("keeps budget order", () => {
    expect(items.map((item) => item.kind)).toEqual(["plank", "pin"]);
  });

  it("describes a kind with tools left", () => {
    expect(items[0]).toEqual({
      kind: "plank",
      label: "Plank",
      remaining: 1,
      countText: "×1",
      accessibleName: "Plank, 1 left",
      title: "Place a plank (1 left)",
      armed: true,
      disabled: false,
    });
  });

  it("disables a kind with none left", () => {
    expect(items[1]).toEqual({
      kind: "pin",
      label: "Pin",
      remaining: 0,
      countText: "×0",
      accessibleName: "Pin, none left",
      title: "Place a pin (0 left)",
      armed: false,
      disabled: true,
    });
  });

  it("disables every item when the tray is not enabled", () => {
    // Act
    const disabled = trayItems(FAKE_BUDGET, [], undefined, false);

    // Assert
    expect(disabled.map((item) => item.disabled)).toEqual([true, true]);
  });
});

describe("statusLine", () => {
  it("names the reason for one invalid tool", () => {
    // Arrange
    const editor = committed(editorWith(2), [0, 8]);

    // Act / Assert
    expect(statusLine(editor)).toBe("Overlaps the cup. Move or remove that tool to play.");
  });

  it("leads with the selected tool's reason when several are invalid", () => {
    // Arrange
    const editor = selectTool(committed(editorWith(3), [7, 0, 9]), 3);

    // Act / Assert
    expect(statusLine(editor)).toBe(
      "Inside the cup. 2 tools need moving or removing before you can play.",
    );
  });

  it("asks the player to select one when several are invalid and the selected tool is valid", () => {
    // Arrange
    const editor = selectTool(committed(editorWith(3), [7, 0, 9]), 2);

    // Act / Assert
    expect(statusLine(editor)).toBe(
      "2 tools need moving or removing before you can play. Select one to see why.",
    );
  });

  it("asks the player to select one when several are invalid and nothing is selected", () => {
    // Arrange
    const editor = committed(editorWith(3), [7, 11, 11]);

    // Act / Assert
    expect(statusLine(editor)).toBe(
      "3 tools need moving or removing before you can play. Select one to see why.",
    );
  });

  it("is undefined when every committed status is valid", () => {
    expect(statusLine(committed(editorWith(2), [0, 0]))).toBeUndefined();
  });

  it("is undefined without a committed snapshot", () => {
    expect(statusLine(editorWith(1))).toBeUndefined();
  });

  it("is undefined when the committed snapshot length differs from the tools", () => {
    // Arrange
    const snapshot = committed(editorWith(1), [8]);
    const editor = { ...snapshot, tools: editorWith(2).tools };

    // Act / Assert
    expect(statusLine(editor)).toBeUndefined();
  });

  it("ignores a live inspection that marks a tool invalid (UI-D14)", () => {
    // Arrange
    const valid = committed(editorWith(1), [0]);
    const dragged = moveTool(valid, 1, 38, 44);
    const live = { revision: dragged.revision, statuses: [9] as const, outlines: [undefined] };
    const editor = applyInspection(dragged, live, "live");

    // Act / Assert
    expect([statusLine(editor), buildingMessage(editor, GOAL)]).toEqual([
      undefined,
      { tone: "goal", text: GOAL },
    ]);
  });
});

describe("buildingMessage", () => {
  it("shows the status line first", () => {
    // Arrange
    const editor = armKind(committed(editorWith(1), [8]), "plank");

    // Act / Assert
    expect(buildingMessage(editor, GOAL)).toEqual({
      tone: "status",
      text: "Overlaps the cup. Move or remove that tool to play.",
    });
  });

  it("shows the armed hint when nothing is invalid", () => {
    // Arrange
    const editor = armKind(editorWith(0), "plank");

    // Act / Assert
    expect(buildingMessage(editor, GOAL)).toEqual({
      tone: "hint",
      text: "Click or tap where the plank should go.",
    });
  });

  it("shows the empty hint with no tools", () => {
    expect(buildingMessage(editorWith(0), GOAL)).toEqual({
      tone: "hint",
      text: `${GOAL} Pick a tool from the tray to start.`,
    });
  });

  it("shows the goal otherwise", () => {
    expect(buildingMessage(editorWith(1), GOAL)).toEqual({ tone: "goal", text: GOAL });
  });
});

describe("announcements", () => {
  it("announces an armed kind", () => {
    expect(armedAnnouncement("Plank")).toBe("Plank ready. Click or tap the build area.");
  });

  it("announces a canceled placement", () => {
    expect(PLACEMENT_CANCELED_ANNOUNCEMENT).toBe("Placement canceled.");
  });

  it.each([
    [undefined, "Plank placed."],
    ["Overlaps the cup", "Plank placed. Overlaps the cup."],
  ])("announces a placed tool with reason %s", (maybeReason, text) => {
    expect(placedAnnouncement("Plank", maybeReason)).toBe(text);
  });

  it.each([
    [undefined, "Plank 1 of 2 selected."],
    ["Overlaps the cup", "Plank 1 of 2 selected. Overlaps the cup."],
  ])("announces a selected tool with reason %s", (maybeReason, text) => {
    expect(selectedAnnouncement("Plank", 1, 2, maybeReason)).toBe(text);
  });

  it.each([
    [23, "Rotated to −15°."],
    [1, "Rotated to 15°."],
    [0, "Rotated to 0°."],
  ])("announces angle index %i", (angleIndex, text) => {
    expect(rotatedAnnouncement(angleIndex)).toBe(text);
  });

  it("announces a tool that became invalid", () => {
    expect(validityChangeAnnouncement(true, false, "Overlaps the cup")).toBe(
      "Now overlaps the cup.",
    );
  });

  it("announces a tool that fits again", () => {
    expect(validityChangeAnnouncement(false, true, undefined)).toBe("This tool fits again.");
  });

  it("stays silent when validity is unchanged", () => {
    expect(validityChangeAnnouncement(false, false, "Overlaps the cup")).toBeUndefined();
  });

  it("announces a removed tool", () => {
    expect(removedAnnouncement("Plank")).toBe("Plank removed.");
  });

  it("announces cleared tools", () => {
    expect(CLEARED_ANNOUNCEMENT).toBe("All tools cleared.");
  });

  it("lists kept stars when stopped", () => {
    expect(stoppedAnnouncement(["Win", "Thrifty"])).toBe("Stopped. Stars kept: Win, Thrifty.");
  });

  it("says the tools are kept when stopped without stars", () => {
    expect(stoppedAnnouncement([])).toBe("Stopped. Your tools are kept.");
  });
});
