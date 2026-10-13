import { describe, expect, it } from "vitest";

import { starListLabel, starRows, type StarThresholds } from "../src/puzzle/star-format";
import type { LevelStars, RunStars } from "../src/puzzle/stars";

const FIRST_POUR: StarThresholds = { targetPct: 60, brimPct: 90, parTools: 1 };
const NO_RUN: RunStars = { win: false, brim: false, thrifty: false };

function best(overrides: Partial<LevelStars> = {}): LevelStars {
  return { win: false, brim: false, thrifty: false, bestPeakHeldPct: 0, ...overrides };
}

describe("starRows labels", () => {
  it("shows the threshold labels in Win, Brim, Thrifty order", () => {
    // Act
    const rows = starRows(FIRST_POUR, undefined, undefined);

    // Assert
    expect(rows.map((row) => row.label)).toEqual(["Win 60%", "Brim 90%", "Thrifty ≤ 1 tool"]);
  });

  it("tags each row with its criterion", () => {
    // Act
    const rows = starRows(FIRST_POUR, undefined, undefined);

    // Assert
    expect(rows.map((row) => row.criterion)).toEqual(["win", "brim", "thrifty"]);
  });

  it("pluralizes the Thrifty label for a par above one", () => {
    // Act
    const rows = starRows({ ...FIRST_POUR, parTools: 2 }, undefined, undefined);

    // Assert
    expect(rows[2]?.label).toBe("Thrifty ≤ 2 tools");
  });

  it("pluralizes the Thrifty hidden text for a par above one", () => {
    // Act
    const rows = starRows({ ...FIRST_POUR, parTools: 2 }, undefined, undefined);

    // Assert
    expect(rows[2]?.hiddenText).toBe("Thrifty: win using at most 2 tools. Not earned yet.");
  });
});

describe("starRows while building", () => {
  it("shows every glyph as none with no best", () => {
    // Act
    const rows = starRows(FIRST_POUR, undefined, undefined);

    // Assert
    expect(rows.map((row) => row.glyph)).toEqual(["none", "none", "none"]);
  });

  it("explains each criterion with a not-earned-yet phrase", () => {
    // Act
    const rows = starRows(FIRST_POUR, undefined, undefined);

    // Assert
    expect(rows.map((row) => row.hiddenText)).toEqual([
      "Win: fill the cup to 60%. Not earned yet.",
      "Brim: fill the cup to 90%. Not earned yet.",
      "Thrifty: win using at most 1 tool. Not earned yet.",
    ]);
  });

  it("marks a best-of star as earned", () => {
    // Act
    const rows = starRows(FIRST_POUR, best({ win: true }), undefined);

    // Assert
    expect(rows[0]?.glyph).toBe("earned");
    expect(rows[0]?.hiddenText).toBe("Win: fill the cup to 60%. Earned.");
  });

  it("leaves stars missing from the best-of as none", () => {
    // Act
    const rows = starRows(FIRST_POUR, best({ win: true }), undefined);

    // Assert
    expect(rows[1]?.glyph).toBe("none");
  });
});

describe("starRows during and after a run", () => {
  it("marks a star latched this run as earned", () => {
    // Arrange
    const run: RunStars = { win: true, brim: false, thrifty: false };

    // Act
    const rows = starRows(FIRST_POUR, undefined, run);

    // Assert
    expect(rows[0]?.glyph).toBe("earned");
    expect(rows[0]?.hiddenText).toBe("Win: fill the cup to 60%. Earned this run.");
  });

  it("marks a best-of star not latched this run as best", () => {
    // Act
    const rows = starRows(FIRST_POUR, best({ brim: true }), NO_RUN);

    // Assert
    expect(rows[1]?.glyph).toBe("best");
    expect(rows[1]?.hiddenText).toBe("Brim: fill the cup to 90%. Earned before.");
  });

  it("marks a star in neither as none", () => {
    // Act
    const rows = starRows(FIRST_POUR, best({ brim: true }), NO_RUN);

    // Assert
    expect(rows[2]?.glyph).toBe("none");
    expect(rows[2]?.hiddenText).toBe("Thrifty: win using at most 1 tool. Not earned.");
  });

  it("prefers this run over the best-of", () => {
    // Arrange
    const run: RunStars = { win: true, brim: true, thrifty: true };

    // Act
    const rows = starRows(FIRST_POUR, best({ win: true, brim: true, thrifty: true }), run);

    // Assert
    expect(rows.map((row) => row.glyph)).toEqual(["earned", "earned", "earned"]);
  });
});

describe("starListLabel", () => {
  it("is Stars before the run ends", () => {
    expect(starListLabel(false)).toBe("Stars");
  });

  it("is Stars this run once ended", () => {
    expect(starListLabel(true)).toBe("Stars this run");
  });
});
