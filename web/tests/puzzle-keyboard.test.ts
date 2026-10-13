import { describe, expect, it } from "vitest";

import {
  NUDGE_SHIFT_CELLS,
  keyTargetOf,
  puzzleKeyCommand,
  type PuzzleKeyInput,
} from "../src/puzzle/keyboard";

function key(name: string, overrides: Partial<PuzzleKeyInput> = {}): PuzzleKeyInput {
  return {
    key: name,
    shiftKey: false,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    target: "canvas",
    ...overrides,
  };
}

describe("puzzleKeyCommand nudges", () => {
  it("nudges left one cell for ArrowLeft", () => {
    expect(puzzleKeyCommand(key("ArrowLeft"))).toEqual({ kind: "nudge", dx: -1, dy: 0 });
  });

  it("nudges up one cell for ArrowUp because world y is up", () => {
    expect(puzzleKeyCommand(key("ArrowUp"))).toEqual({ kind: "nudge", dx: 0, dy: 1 });
  });

  it("nudges down one cell for ArrowDown", () => {
    expect(puzzleKeyCommand(key("ArrowDown"))).toEqual({ kind: "nudge", dx: 0, dy: -1 });
  });

  it("nudges ten cells for Shift+ArrowRight", () => {
    // Act
    const command = puzzleKeyCommand(key("ArrowRight", { shiftKey: true }));

    // Assert
    expect(command).toEqual({ kind: "nudge", dx: 10, dy: 0 });
  });

  it("uses a ten-cell shift nudge", () => {
    expect(NUDGE_SHIFT_CELLS).toBe(10);
  });

  it("nudges from a focused HUD button", () => {
    expect(puzzleKeyCommand(key("ArrowLeft", { target: "button" }))).toEqual({
      kind: "nudge",
      dx: -1,
      dy: 0,
    });
  });
});

describe("puzzleKeyCommand edits and transport", () => {
  it("rotates +1 for r", () => {
    expect(puzzleKeyCommand(key("r"))).toEqual({ kind: "rotate", direction: 1 });
  });

  it("rotates -1 for Shift+R", () => {
    expect(puzzleKeyCommand(key("R", { shiftKey: true }))).toEqual({
      kind: "rotate",
      direction: -1,
    });
  });

  it("rotates +1 for an unshifted R such as with Caps Lock", () => {
    expect(puzzleKeyCommand(key("R"))).toEqual({ kind: "rotate", direction: 1 });
  });

  it("removes for Delete", () => {
    expect(puzzleKeyCommand(key("Delete"))).toEqual({ kind: "remove" });
  });

  it("removes for Backspace", () => {
    expect(puzzleKeyCommand(key("Backspace"))).toEqual({ kind: "remove" });
  });

  it("toggles play for Space", () => {
    expect(puzzleKeyCommand(key(" "))).toEqual({ kind: "togglePlay" });
  });

  it("escapes for Escape", () => {
    expect(puzzleKeyCommand(key("Escape"))).toEqual({ kind: "escape" });
  });
});

describe("puzzleKeyCommand Tab cycling", () => {
  it("cycles forward for Tab on the canvas", () => {
    expect(puzzleKeyCommand(key("Tab"))).toEqual({ kind: "cycleSelection", direction: 1 });
  });

  it("cycles backward for Shift+Tab on the canvas", () => {
    expect(puzzleKeyCommand(key("Tab", { shiftKey: true }))).toEqual({
      kind: "cycleSelection",
      direction: -1,
    });
  });

  it("leaves Tab alone on a HUD control", () => {
    expect(puzzleKeyCommand(key("Tab", { target: "control" }))).toBeUndefined();
  });

  it("leaves Tab alone on a button", () => {
    expect(puzzleKeyCommand(key("Tab", { target: "button" }))).toBeUndefined();
  });
});

describe("puzzleKeyCommand ignored keys", () => {
  it("leaves Space to the native click on a button", () => {
    expect(puzzleKeyCommand(key(" ", { target: "button" }))).toBeUndefined();
  });

  it("leaves Enter to the native click on a button", () => {
    expect(puzzleKeyCommand(key("Enter", { target: "button" }))).toBeUndefined();
  });

  it("ignores every key in a form control", () => {
    // Arrange
    const keys = ["ArrowLeft", "r", "Delete", "Backspace", " ", "Escape", "Tab"];

    // Act
    const commands = keys.map((name) => puzzleKeyCommand(key(name, { target: "formControl" })));

    // Assert
    expect(commands.every((command) => command === undefined)).toBe(true);
  });

  it("ignores a key with Ctrl", () => {
    expect(puzzleKeyCommand(key("r", { ctrlKey: true }))).toBeUndefined();
  });

  it("ignores a key with Meta", () => {
    expect(puzzleKeyCommand(key("Backspace", { metaKey: true }))).toBeUndefined();
  });

  it("ignores a key with Alt", () => {
    expect(puzzleKeyCommand(key("ArrowLeft", { altKey: true }))).toBeUndefined();
  });

  it("ignores an unmapped key", () => {
    expect(puzzleKeyCommand(key("q"))).toBeUndefined();
  });

  it("ignores Enter on the canvas", () => {
    expect(puzzleKeyCommand(key("Enter"))).toBeUndefined();
  });
});

describe("keyTargetOf", () => {
  it("classifies a null target as other", () => {
    expect(keyTargetOf(null)).toBe("other");
  });

  it("classifies a non-element target as other without a DOM", () => {
    expect(keyTargetOf(new EventTarget())).toBe("other");
  });
});
