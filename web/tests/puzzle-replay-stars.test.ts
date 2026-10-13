/**
 * Phase 39 D-12 replay fixture lines: `peak_held` and `brim_step`.
 *
 * The rejection messages mirror
 * `crates/liquidfun-wasm/src/puzzle/replay/fixture/tests.rs` byte for byte,
 * and the contract cases mirror `crates/liquidfun-wasm/src/puzzle/replay/tests.rs`.
 */

import { describe, expect, it } from "vitest";

import {
  checkReplay,
  type Fixture,
  type FixtureExpectation,
  fillMarginCount,
  fixtureInputText,
  MARGIN_WIN_SLACK_STEPS,
  parseFixture,
  type ReplayLevel,
  type ReplayTrace,
} from "../scripts/puzzle-replay/fixture";

const EXPECTATION_PREFIX =
  "level first-pour\nplacement plank 8 64 23\noutcome won\noutcome_step 412\nfinal_count 640\n";
const FULL_WON = `${EXPECTATION_PREFIX}peak_held 811\nbrim_step 262\n`;

/** first-pour level numbers, as `levelInfo()` reports them. */
const FIRST_POUR: ReplayLevel = {
  targetCount: 540,
  fullCount: 900,
  brimCount: 810,
  settleSteps: 30,
  stepLimit: 720,
};

const EXACT_TOLERANCE = 0;
const FALLBACK_TOLERANCE = 5;
const RAISED_BRIM = FIRST_POUR.brimCount + fillMarginCount(FIRST_POUR.fullCount);

/**
 * A won run holding `held` from the start of its settle window through the
 * margin window end, which reaches Brim and is decided at `brimStep`.
 */
function brimTrace(
  outcomeStep: number,
  held: number,
  maybeBrimStep: number | undefined,
): ReplayTrace {
  const fromStep = outcomeStep + 1 - FIRST_POUR.settleSteps;
  const windowEnd = FIRST_POUR.stepLimit - MARGIN_WIN_SLACK_STEPS;
  return {
    outcome: "won",
    outcomeStep,
    finalCount: held,
    peakHeld: held,
    maybeBrimStep,
    decidedStep: maybeBrimStep ?? FIRST_POUR.stepLimit,
    counts: Array.from({ length: windowEnd }, (_, index) =>
      index + 1 < fromStep ? 0 : held,
    ),
  };
}

/** The expectation a trace would record, so tests can perturb one field. */
function expectationOf(trace: ReplayTrace): FixtureExpectation {
  return {
    outcome: trace.outcome,
    outcomeStep: trace.outcomeStep,
    finalCount: trace.finalCount,
    peakHeld: trace.peakHeld,
    maybeBrimStep: trace.maybeBrimStep,
  };
}

function fixtureExpecting(expectation: FixtureExpectation): Fixture {
  return { level: "first-pour", placements: [], maybeExpectation: expectation };
}

describe("puzzle-replay D-12 fixture lines", () => {
  it("parseFixture reads peak_held and a numeric brim_step", () => {
    // Arrange
    const text = FULL_WON;

    // Act
    const fixture = parseFixture(text);

    // Assert
    expect(fixture.maybeExpectation).toEqual({
      outcome: "won",
      outcomeStep: 412,
      finalCount: 640,
      peakHeld: 811,
      maybeBrimStep: 262,
    });
  });

  it("parseFixture reads brim_step none as no Brim", () => {
    // Arrange
    const text =
      "level first-pour\noutcome lost\noutcome_step 720\nfinal_count 0\npeak_held 0\nbrim_step none\n";

    // Act
    const expectation = parseFixture(text).maybeExpectation;

    // Assert
    expect([expectation?.peakHeld, expectation?.maybeBrimStep]).toEqual([
      0,
      undefined,
    ]);
  });

  it.each([
    [
      "a missing peak_held after final_count",
      EXPECTATION_PREFIX,
      "fixture line 6: missing `peak_held` line",
    ],
    [
      "a missing brim_step after peak_held",
      `${EXPECTATION_PREFIX}peak_held 811\n`,
      "fixture line 7: missing `brim_step` line",
    ],
    [
      "brim_step before peak_held",
      `${EXPECTATION_PREFIX}brim_step 262\npeak_held 811\n`,
      "fixture line 6: expected `peak_held`, found `brim_step`",
    ],
    [
      "brim_step -1",
      `${EXPECTATION_PREFIX}peak_held 811\nbrim_step -1\n`,
      "fixture line 7: `-1` is not a non-negative integer",
    ],
    [
      "brim_step nope",
      `${EXPECTATION_PREFIX}peak_held 811\nbrim_step nope\n`,
      "fixture line 7: `nope` is not a non-negative integer",
    ],
    [
      "peak_held none",
      `${EXPECTATION_PREFIX}peak_held none\nbrim_step 262\n`,
      "fixture line 6: `none` is not a non-negative integer",
    ],
    [
      "a line after brim_step",
      `${FULL_WON}peak_held 811\n`,
      "fixture line 8: unexpected line after brim_step",
    ],
  ])("parseFixture rejects %s", (_name, text, message) => {
    // Arrange
    const parse = () => parseFixture(text);

    // Act and Assert
    expect(parse).toThrow(new Error(message));
  });

  it("fixtureInputText keeps only the level and placement lines", () => {
    // Arrange
    const text = FULL_WON;

    // Act
    const inputs = fixtureInputText(text);

    // Assert
    expect(inputs).toBe("level first-pour\nplacement plank 8 64 23\n");
  });
});

describe("puzzle-replay D-12 contract", () => {
  it("checkReplay exact contract requires the recorded peak held", () => {
    // Arrange
    const trace = brimTrace(201, RAISED_BRIM, 262);
    const fixture = fixtureExpecting({
      ...expectationOf(trace),
      peakHeld: trace.peakHeld + 1,
    });

    // Act
    const exact = checkReplay(fixture, trace, FIRST_POUR, true, EXACT_TOLERANCE);
    const fallback = checkReplay(fixture, trace, FIRST_POUR, false, FALLBACK_TOLERANCE);

    // Assert
    expect(exact).toEqual([
      `peak held ${trace.peakHeld} differs from recorded ${trace.peakHeld + 1}`,
    ]);
    expect(fallback).toEqual([]);
  });

  it("checkReplay exact contract requires the recorded Brim step", () => {
    // Arrange
    const trace = brimTrace(201, RAISED_BRIM, 262);
    const fixture = fixtureExpecting({ ...expectationOf(trace), maybeBrimStep: 263 });

    // Act
    const failures = checkReplay(fixture, trace, FIRST_POUR, true, EXACT_TOLERANCE);

    // Assert
    expect(failures).toEqual(["Brim step 262 differs from recorded 263"]);
  });

  it("checkReplay fallback accepts a Brim step within the tolerance", () => {
    // Arrange
    const trace = brimTrace(201, RAISED_BRIM, 262);
    const within = fixtureExpecting({
      ...expectationOf(trace),
      maybeBrimStep: 262 + FALLBACK_TOLERANCE,
    });
    const beyond = fixtureExpecting({
      ...expectationOf(trace),
      maybeBrimStep: 262 + FALLBACK_TOLERANCE + 1,
    });

    // Act
    const withinFailures = checkReplay(within, trace, FIRST_POUR, false, FALLBACK_TOLERANCE);
    const beyondFailures = checkReplay(beyond, trace, FIRST_POUR, false, FALLBACK_TOLERANCE);

    // Assert
    expect(withinFailures).toEqual([]);
    expect(beyondFailures).not.toEqual([]);
  });

  it("checkReplay fallback requires Brim exactly when recorded", () => {
    // Arrange
    const withBrim = brimTrace(201, RAISED_BRIM, 262);
    const withoutBrim: ReplayTrace = { ...withBrim, maybeBrimStep: undefined };

    // Act
    const missing = checkReplay(
      fixtureExpecting(expectationOf(withBrim)),
      withoutBrim,
      FIRST_POUR,
      false,
      FALLBACK_TOLERANCE,
    );
    const unexpected = checkReplay(
      fixtureExpecting(expectationOf(withoutBrim)),
      withBrim,
      FIRST_POUR,
      false,
      FALLBACK_TOLERANCE,
    );

    // Assert
    expect(missing).toEqual(["Brim step none differs from recorded 262"]);
    expect(unexpected).toEqual(["Brim step 262 differs from recorded none"]);
  });

  it("checkReplay requires a recorded Brim to hold at the raised Brim count", () => {
    // Arrange
    const short = brimTrace(201, RAISED_BRIM - 1, 262);
    const enough = brimTrace(201, RAISED_BRIM, 262);

    // Act
    const shortFailures = checkReplay(
      fixtureExpecting(expectationOf(short)),
      short,
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );
    const enoughFailures = checkReplay(
      fixtureExpecting(expectationOf(enough)),
      enough,
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );

    // Assert
    expect(shortFailures).toEqual([
      `Brim does not hold at the raised count ${RAISED_BRIM} by step 600`,
    ]);
    expect(enoughFailures).toEqual([]);
  });
});
