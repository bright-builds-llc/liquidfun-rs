import { describe, expect, it } from "vitest";

import {
  FIRST_LEVEL_HINT,
  PUZZLE_CAPTIONS,
  puzzleCaptionForId,
} from "../src/catalog/puzzle-levels";
import { PUZZLE_IDS } from "../src/catalog/puzzles";
import { captionModel } from "../src/puzzle/caption";

const MAX_CAPTION_LENGTH = 80;

describe("captionModel", () => {
  it("shows the placement hint with Dismiss on first-pour until dismissed", () => {
    // Act
    const model = captionModel("first-pour", false);

    // Assert
    expect(model).toEqual({ text: FIRST_LEVEL_HINT, showDismiss: true });
  });

  it("shows the first-pour caption without Dismiss once dismissed", () => {
    // Act
    const model = captionModel("first-pour", true);

    // Assert
    expect(model).toEqual({ text: PUZZLE_CAPTIONS["first-pour"], showDismiss: false });
  });
});

describe("PUZZLE_CAPTIONS", () => {
  it("gives every level a non-empty one-line caption of at most 80 characters", () => {
    // Act
    const offenders = PUZZLE_IDS.filter((id) => {
      const caption = puzzleCaptionForId(id);
      return caption.trim() === "" || /[\r\n]/.test(caption) || caption.length > MAX_CAPTION_LENGTH;
    });

    // Assert
    expect(offenders).toEqual([]);
  });

  it("keeps the first-level hint on one line within 80 characters", () => {
    // Act
    const length = FIRST_LEVEL_HINT.length;

    // Assert
    expect(FIRST_LEVEL_HINT).not.toMatch(/[\r\n]/);
    expect(length).toBeLessThanOrEqual(MAX_CAPTION_LENGTH);
  });
});
