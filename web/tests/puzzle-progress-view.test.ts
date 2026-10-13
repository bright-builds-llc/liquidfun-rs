import { describe, expect, it } from "vitest";

import { PUZZLE_CAPTIONS } from "../src/catalog/puzzle-levels";
import { PUZZLE_IDS, type PuzzleId } from "../src/catalog/puzzles";
import type { PuzzlePlacement } from "../src/puzzle/placements";
import { EMPTY_PROGRESS, withPlacements, withStars, type ProgressDocument } from "../src/puzzle/progress";
import {
  MAX_SAVED_PLACEMENTS,
  levelCardModels,
  restoreChoice,
  savedPlacementsFor,
  solvedCount,
  unlockedLevelIds,
} from "../src/puzzle/progress-view";
import type { LevelStars } from "../src/puzzle/stars";

const LIVE_HASH = "0123456789abcdef";
const OTHER_HASH = "fedcba9876543210";
const WIN: LevelStars = { win: true, brim: false, thrifty: false, bestPeakHeldPct: 61 };
const PLANK: PuzzlePlacement = { kind: "plank", x: 8, y: 64, angle: 23, openTick: 0 };
const VALVE: PuzzlePlacement = { kind: "valve", x: -16, y: 87, angle: 0, openTick: 4 };

function won(...ids: readonly PuzzleId[]): ProgressDocument {
  return ids.reduce((progress, id) => withStars(progress, id, WIN), EMPTY_PROGRESS);
}

function savedTools(tools: readonly unknown[], levelHash = LIVE_HASH): ProgressDocument {
  return withPlacements(EMPTY_PROGRESS, "first-pour", { levelHash, tools });
}

function card(progress: ProgressDocument, degraded: boolean, index: number) {
  const maybeCard = levelCardModels(progress, degraded)[index];
  if (maybeCard === undefined) {
    throw new Error(`No card at ${index}`);
  }
  return maybeCard;
}

describe("unlockedLevelIds", () => {
  it("unlocks only the first level on fresh progress", () => {
    // Act
    const unlocked = unlockedLevelIds(EMPTY_PROGRESS, false);

    // Assert
    expect([...unlocked]).toEqual(["first-pour"]);
  });

  it("unlocks the level after a solved one", () => {
    // Act
    const unlocked = unlockedLevelIds(won("first-pour"), false);

    // Assert
    expect([...unlocked]).toEqual(["first-pour", "thick-syrup"]);
  });

  it("unlocks after an out-of-order solve", () => {
    // Act
    const unlocked = unlockedLevelIds(won("jelly-pad"), false);

    // Assert
    expect([...unlocked]).toEqual(["first-pour", "jelly-pad", "float-up"]);
  });

  it("keeps a solved level unlocked while its unsolved predecessor stays locked", () => {
    // Arrange
    const progress = won("jelly-pad");

    // Act
    const unlocked = unlockedLevelIds(progress, false);

    // Assert
    expect(unlocked.has("jelly-pad")).toBe(true);
    expect(unlocked.has("thick-syrup")).toBe(false);
  });

  it("unlocks every level when degraded", () => {
    // Act
    const unlocked = unlockedLevelIds(won("timed-gate"), true);

    // Assert
    expect([...unlocked]).toEqual([...PUZZLE_IDS]);
  });
});

describe("solvedCount", () => {
  it("counts nothing on fresh progress", () => {
    // Act
    const count = solvedCount(EMPTY_PROGRESS);

    // Assert
    expect(count).toBe(0);
  });

  it("counts one win", () => {
    // Act
    const count = solvedCount(won("first-pour"));

    // Assert
    expect(count).toBe(1);
  });

  it("does not count saved placements without a win", () => {
    // Arrange
    const progress = withPlacements(won("first-pour"), "jelly-pad", { levelHash: LIVE_HASH, tools: [] });

    // Act
    const count = solvedCount(progress);

    // Assert
    expect(count).toBe(1);
  });
});

describe("levelCardModels", () => {
  it("lists six cards in registry order", () => {
    // Act
    const cards = levelCardModels(EMPTY_PROGRESS, false);

    // Assert
    expect(cards.map((model) => model.id)).toEqual([...PUZZLE_IDS]);
  });

  it("models the first card on fresh progress", () => {
    // Act
    const first = card(EMPTY_PROGRESS, false, 0);

    // Assert
    expect(first).toEqual({
      id: "first-pour",
      number: 1,
      title: "First Pour",
      caption: PUZZLE_CAPTIONS["first-pour"],
      stars: { win: false, brim: false, thrifty: false },
      locked: false,
      solved: false,
      isNextUp: true,
      footer: "Not solved yet",
      lockedLabel: "Level 1, First Pour, locked",
      href: "#/puzzle/first-pour",
    });
  });

  it("models the second card as locked behind the first", () => {
    // Act
    const second = card(EMPTY_PROGRESS, false, 1);

    // Assert
    expect(second).toMatchObject({
      number: 2,
      locked: true,
      isNextUp: false,
      footer: "Solve First Pour to unlock",
      lockedLabel: "Level 2, Thick Syrup, locked",
    });
  });

  it("shows a solved card's stars and floored best fill", () => {
    // Arrange
    const progress = withStars(EMPTY_PROGRESS, "first-pour", {
      win: true,
      brim: true,
      thrifty: false,
      bestPeakHeldPct: 90.9,
    });

    // Act
    const first = card(progress, false, 0);

    // Assert
    expect(first).toMatchObject({
      solved: true,
      isNextUp: false,
      footer: "Best fill 90%",
      stars: { win: true, brim: true, thrifty: false },
    });
  });

  it("moves next-up to the card after a solved one", () => {
    // Act
    const second = card(won("first-pour"), false, 1);

    // Assert
    expect(second).toMatchObject({ locked: false, isNextUp: true, footer: "Not solved yet" });
  });

  it("unlocks an out-of-order solved card with its stars and best fill", () => {
    // Act
    const third = card(won("jelly-pad"), false, 2);

    // Assert
    expect(third).toMatchObject({
      locked: false,
      solved: true,
      isNextUp: false,
      stars: { win: true, brim: false, thrifty: false },
      footer: "Best fill 61%",
    });
  });

  it("keeps the unsolved predecessor of an out-of-order solve locked", () => {
    // Act
    const second = card(won("jelly-pad"), false, 1);

    // Assert
    expect(second).toMatchObject({ locked: true, solved: false, footer: "Solve First Pour to unlock" });
  });

  it("has no next-up card when every level is solved", () => {
    // Act
    const cards = levelCardModels(won(...PUZZLE_IDS), false);

    // Assert
    expect(cards.some((model) => model.isNextUp)).toBe(false);
  });

  it("unlocks every card when degraded and keeps the first next-up", () => {
    // Act
    const cards = levelCardModels(EMPTY_PROGRESS, true);

    // Assert
    expect(cards.map((model) => [model.locked, model.isNextUp])).toEqual([
      [false, true],
      [false, false],
      [false, false],
      [false, false],
      [false, false],
      [false, false],
    ]);
  });
});

describe("restoreChoice", () => {
  it("has nothing to restore without a record", () => {
    // Act
    const choice = restoreChoice(EMPTY_PROGRESS, "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "none" });
  });

  it("has nothing to restore without saved placements", () => {
    // Act
    const choice = restoreChoice(won("first-pour"), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "none" });
  });

  it("restores valid placements in saved order", () => {
    // Act
    const choice = restoreChoice(savedTools([VALVE, PLANK]), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "restore", placements: [VALVE, PLANK] });
  });

  it("discards placements saved for another level hash", () => {
    // Act
    const choice = restoreChoice(savedTools([PLANK], OTHER_HASH), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "discard" });
  });

  it("discards the list when one entry is malformed", () => {
    // Act
    const choice = restoreChoice(savedTools([PLANK, { ...PLANK, x: 1.5 }]), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "discard" });
  });

  it("discards more placements than the Rust lane cap", () => {
    // Arrange
    const tools = Array.from({ length: MAX_SAVED_PLACEMENTS + 1 }, () => PLANK);

    // Act
    const choice = restoreChoice(savedTools(tools), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "discard" });
  });

  it("restores exactly the lane cap", () => {
    // Arrange
    const tools = Array.from({ length: MAX_SAVED_PLACEMENTS }, () => PLANK);

    // Act
    const choice = restoreChoice(savedTools(tools), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "restore", placements: tools });
  });

  it("restores an empty saved list", () => {
    // Act
    const choice = restoreChoice(savedTools([]), "first-pour", LIVE_HASH);

    // Assert
    expect(choice).toEqual({ kind: "restore", placements: [] });
  });
});

describe("savedPlacementsFor", () => {
  it("copies placements into fresh records under the level hash", () => {
    // Arrange
    const placements = [PLANK, VALVE];

    // Act
    const saved = savedPlacementsFor(LIVE_HASH, placements);

    // Assert
    expect(saved).toEqual({ levelHash: LIVE_HASH, tools: [PLANK, VALVE] });
    expect(saved.tools[0]).not.toBe(PLANK);
  });
});
