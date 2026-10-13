import { describe, expect, it } from "vitest";

import {
  NUDGE_SHIFT_CELLS,
  classifyKeyTarget,
  isRepeatedPlayToggle,
  keyTargetOf,
  puzzleKeyCommand,
  type KeyTargetElement,
  type PuzzleKeyInput,
} from "../src/puzzle/keyboard";

function element(
  tagName: string,
  attributes: Readonly<Record<string, string>> = {},
  isContentEditable = false,
): KeyTargetElement {
  return { tagName, isContentEditable, getAttribute: (name) => attributes[name] ?? null };
}

function key(name: string, overrides: Partial<PuzzleKeyInput> = {}): PuzzleKeyInput {
  return {
    key: name,
    shiftKey: false,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    altGraph: false,
    target: "canvas",
    ...overrides,
  };
}

describe("puzzleKeyCommand valve time", () => {
  it("opens the valve earlier for [", () => {
    expect(puzzleKeyCommand(key("["))).toEqual({ kind: "valveTime", direction: -1 });
  });

  it("opens the valve later for ]", () => {
    expect(puzzleKeyCommand(key("]"))).toEqual({ kind: "valveTime", direction: 1 });
  });

  it.each([{ ctrlKey: true }, { metaKey: true }, { altKey: true, metaKey: true }])(
    "ignores a bracket with a shortcut modifier %o",
    (modifier) => {
      expect(puzzleKeyCommand(key("]", modifier))).toBeUndefined();
    },
  );

  it("steps the valve for a bracket typed with Option on a macOS German layout", () => {
    // Act
    const command = puzzleKeyCommand(key("[", { altKey: true }));

    // Assert
    expect(command).toEqual({ kind: "valveTime", direction: -1 });
  });

  it("steps the valve for a bracket typed with AltGr on a Windows German layout", () => {
    // Arrange
    const altGr = { ctrlKey: true, altKey: true, altGraph: true };

    // Act
    const command = puzzleKeyCommand(key("]", altGr));

    // Assert
    expect(command).toEqual({ kind: "valveTime", direction: 1 });
  });

  it("ignores a bracket with Ctrl+Alt when AltGraph is not active", () => {
    expect(puzzleKeyCommand(key("]", { ctrlKey: true, altKey: true }))).toBeUndefined();
  });

  it("ignores a bracket in a form control", () => {
    expect(puzzleKeyCommand(key("[", { target: "formControl" }))).toBeUndefined();
  });
});

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

describe("classifyKeyTarget", () => {
  it.each([
    ["a <summary> disclosure", element("SUMMARY")],
    ["a role=button element", element("DIV", { role: "button" })],
    ["a role=switch element", element("SPAN", { role: "switch" })],
    ["a lowercase button tag", element("button")],
  ] as const)("classifies %s as a button", (_label, target) => {
    // Act
    const kind = classifyKeyTarget(target);

    // Assert
    expect(kind).toBe("button");
  });

  it("classifies editable text as a form control", () => {
    // Act
    const kind = classifyKeyTarget(element("DIV", {}, true));

    // Assert
    expect(kind).toBe("formControl");
  });

  it("keeps the canvas a canvas despite its application role", () => {
    // Act
    const kind = classifyKeyTarget(element("CANVAS", { role: "application" }));

    // Assert
    expect(kind).toBe("canvas");
  });

  it("classifies a plain element as other", () => {
    // Act
    const kind = classifyKeyTarget(element("DIV"));

    // Assert
    expect(kind).toBe("other");
  });

  it("leaves Space on the failure details disclosure to its native toggle", () => {
    // Arrange
    const target = classifyKeyTarget(element("SUMMARY"));

    // Act
    const command = puzzleKeyCommand(key(" ", { target }));

    // Assert
    expect(command).toBeUndefined();
  });
});

describe("isRepeatedPlayToggle", () => {
  it("swallows an auto-repeated Space toggle", () => {
    // Act
    const swallowed = isRepeatedPlayToggle({ kind: "togglePlay" }, true);

    // Assert
    expect(swallowed).toBe(true);
  });

  it("lets the first Space press toggle", () => {
    // Act
    const swallowed = isRepeatedPlayToggle({ kind: "togglePlay" }, false);

    // Assert
    expect(swallowed).toBe(false);
  });

  it("keeps auto-repeated arrow nudges", () => {
    // Act
    const swallowed = isRepeatedPlayToggle({ kind: "nudge", dx: 1, dy: 0 }, true);

    // Assert
    expect(swallowed).toBe(false);
  });
});
