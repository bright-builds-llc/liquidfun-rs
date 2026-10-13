import { describe, expect, it } from "vitest";

import {
  FIRST_PUZZLE_ID,
  maybeNextPuzzleId,
  PUZZLE_TITLES,
  puzzleHash,
  puzzleTitleForId,
} from "../src/catalog/puzzle-levels";
import { PUZZLE_IDS } from "../src/catalog/puzzles";
import {
  maybePlayableTarget,
  type PlayableTarget,
  playableRouteIdentity,
  samePlayableTarget,
  titleForPlayableRoute,
} from "../src/player/playable-target";
import { DEFAULT_TITLE, titleForRoute } from "../src/player/runtime";
import type { SceneRoute } from "../src/routing/hash";
import type { PlayableRoute } from "../src/routing/playable-route";

const DAM_BREAK: PlayableTarget = { kind: "scene", id: "dam-break" };
const FIRST_POUR: PlayableTarget = { kind: "puzzle", id: "first-pour" };

describe("maybePlayableTarget", () => {
  it("maps a ready scene route to a scene target", () => {
    // Arrange
    const route: PlayableRoute = { kind: "scene", id: "dam-break" };

    // Act
    const maybeTarget = maybePlayableTarget(route);

    // Assert
    expect(maybeTarget).toEqual(DAM_BREAK);
  });

  it("maps a puzzle route to a puzzle target", () => {
    // Arrange
    const route: PlayableRoute = { kind: "puzzle", id: "first-pour" };

    // Act
    const maybeTarget = maybePlayableTarget(route);

    // Assert
    expect(maybeTarget).toEqual(FIRST_POUR);
  });

  it("has no target for empty, unknown and unknown-puzzle routes", () => {
    // Arrange
    const routes: PlayableRoute[] = [
      { kind: "empty" },
      { kind: "unknown", maybeRaw: "x" },
      { kind: "unknown-puzzle", maybeRaw: "nope" },
    ];

    // Act
    const targets = routes.map((route) => maybePlayableTarget(route));

    // Assert
    expect(targets).toEqual([undefined, undefined, undefined]);
  });
});

describe("samePlayableTarget", () => {
  it("is true for equal kind and id", () => {
    // Arrange
    const other: PlayableTarget = { kind: "puzzle", id: "first-pour" };

    // Act
    const same = samePlayableTarget(FIRST_POUR, other);

    // Assert
    expect(same).toBe(true);
  });

  it("is false when either side is undefined", () => {
    // Arrange
    const pairs: [PlayableTarget | undefined, PlayableTarget | undefined][] = [
      [undefined, DAM_BREAK],
      [DAM_BREAK, undefined],
      [undefined, undefined],
    ];

    // Act
    const results = pairs.map(([maybeA, maybeB]) => samePlayableTarget(maybeA, maybeB));

    // Assert
    expect(results).toEqual([false, false, false]);
  });

  it("is false for a scene against a puzzle", () => {
    // Arrange
    const scene = DAM_BREAK;

    // Act
    const same = samePlayableTarget(scene, FIRST_POUR);

    // Assert
    expect(same).toBe(false);
  });

  it("is false for two different scenes", () => {
    // Arrange
    const other: PlayableTarget = { kind: "scene", id: "wave-machine" };

    // Act
    const same = samePlayableTarget(DAM_BREAK, other);

    // Assert
    expect(same).toBe(false);
  });
});

describe("titleForPlayableRoute", () => {
  it("titles a puzzle route with the level name", () => {
    // Arrange
    const route: PlayableRoute = { kind: "puzzle", id: "first-pour" };

    // Act
    const title = titleForPlayableRoute(route);

    // Assert
    expect(title).toBe("First Pour · puzzle · liquidfun-rs playground");
  });

  it("uses the default title for an unknown puzzle", () => {
    // Arrange
    const route: PlayableRoute = { kind: "unknown-puzzle", maybeRaw: "nope" };

    // Act
    const title = titleForPlayableRoute(route);

    // Assert
    expect(title).toBe(DEFAULT_TITLE);
  });

  it("keeps the scene title for every scene route kind", () => {
    // Arrange
    const routes: SceneRoute[] = [
      { kind: "scene", id: "dam-break" },
      { kind: "empty" },
      { kind: "unknown", maybeRaw: "x" },
    ];

    // Act
    const titles = routes.map((route) => titleForPlayableRoute(route));

    // Assert
    expect(titles).toEqual(routes.map((route) => titleForRoute(route)));
  });
});

describe("playableRouteIdentity", () => {
  it("keys every route kind distinctly", () => {
    // Arrange
    const routes: PlayableRoute[] = [
      { kind: "scene", id: "dam-break" },
      { kind: "unknown", maybeRaw: "x" },
      { kind: "empty" },
      { kind: "puzzle", id: "first-pour" },
      { kind: "unknown-puzzle", maybeRaw: "nope" },
    ];

    // Act
    const identities = routes.map((route) => playableRouteIdentity(route));

    // Assert
    expect(identities).toEqual([
      "scene:dam-break",
      "unknown:x",
      "empty",
      "puzzle:first-pour",
      "unknown-puzzle:nope",
    ]);
  });
});

describe("puzzle level titles", () => {
  it("has exactly one title per registry id", () => {
    // Arrange
    const registry = [...PUZZLE_IDS].sort();

    // Act
    const titled = Object.keys(PUZZLE_TITLES).sort();

    // Assert
    expect(titled).toEqual(registry);
  });

  it("titles first-pour as First Pour", () => {
    // Arrange
    const id = "first-pour";

    // Act
    const title = puzzleTitleForId(id);

    // Assert
    expect(title).toBe("First Pour");
  });

  it("links first-pour to the following registry level", () => {
    // Arrange
    const id = "first-pour";

    // Act
    const maybeNext = maybeNextPuzzleId(id);

    // Assert
    expect(maybeNext).toBe("thick-syrup");
  });

  it("has no next level after the last level", () => {
    // Arrange
    const maybeLastId = PUZZLE_IDS.at(-1);

    // Act
    const maybeNext = maybeLastId && maybeNextPuzzleId(maybeLastId);

    // Assert
    expect(maybeLastId).toBeDefined();
    expect(maybeNext).toBeUndefined();
  });

  it("builds the canonical level hash", () => {
    // Arrange
    const id = "first-pour";

    // Act
    const hash = puzzleHash(id);

    // Assert
    expect(hash).toBe("#/puzzle/first-pour");
  });

  it("starts at the first registry id", () => {
    // Arrange
    const expected = PUZZLE_IDS[0];

    // Act
    const first = FIRST_PUZZLE_ID;

    // Assert
    expect(first).toBe(expected);
  });
});
