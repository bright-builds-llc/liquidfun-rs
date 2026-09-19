import { describe, expect, it } from "vitest";

import type { FrameObservation } from "../src/player/observe";
import {
  maybeObservedFrame,
  playerStatus,
  type PlayerView,
} from "../src/player/view";

const OBSERVATION: FrameObservation = {
  particleCount: 2,
  rigidShapeCount: 1,
  stepIndex: 7,
  movedFrameCount: 3,
};

describe("player view", () => {
  it.each([
    [{ kind: "fallback" }, "loading"],
    [{ kind: "loading" }, "loading"],
    [{ kind: "playing", frame: OBSERVATION }, "playing"],
    [{ kind: "paused", frame: OBSERVATION }, "paused"],
    [
      {
        kind: "failure",
        maybeFrame: undefined,
        maybeDetails: undefined,
      },
      "failed",
    ],
  ] as const)("maps %o to %s status", (view, expected) => {
    // Act
    const status = playerStatus(view);

    // Assert
    expect(status).toBe(expected);
  });

  it("returns observations only from views that carry one", () => {
    // Arrange
    const views: readonly PlayerView[] = [
      { kind: "fallback" },
      { kind: "loading" },
      { kind: "playing", frame: OBSERVATION },
      { kind: "paused", frame: OBSERVATION },
      {
        kind: "failure",
        maybeFrame: OBSERVATION,
        maybeDetails: undefined,
      },
      {
        kind: "failure",
        maybeFrame: undefined,
        maybeDetails: undefined,
      },
    ];

    // Act
    const observations = views.map(maybeObservedFrame);

    // Assert
    expect(observations).toEqual([
      undefined,
      undefined,
      OBSERVATION,
      OBSERVATION,
      OBSERVATION,
      undefined,
    ]);
  });
});
