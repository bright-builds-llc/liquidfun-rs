import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

import {
  checkReplay,
  FIXTURE_FILES,
  type Fixture,
  type FixtureOutcome,
  fillMarginCount,
  fixtureLanes,
  MARGIN_WIN_SLACK_STEPS,
  parseFixture,
  type ReplayLevel,
  type ReplayTrace,
} from "../scripts/puzzle-replay/fixture";
import {
  fnv1a64Bytes,
  fnv1a64Lanes,
  hex16,
} from "../scripts/puzzle-replay/hash";
import {
  compareSpikes,
  parseSpikeLines,
  spikeLine,
} from "../scripts/puzzle-replay/spike";
import { repoRoot } from "../scripts/puzzle-replay/wasm";

const HASH_A = "0000000000000001";
const HASH_B = "0000000000000002";

describe("puzzle-replay frame hash", () => {
  it("fnv1a64Bytes of no bytes is the offset basis", () => {
    // Arrange
    const bytes = new Uint8Array(0);

    // Act
    const hash = fnv1a64Bytes(bytes);

    // Assert
    expect(hex16(hash)).toBe("cbf29ce484222325");
  });

  it('fnv1a64Bytes of "a" matches the published vector', () => {
    // Arrange
    const bytes = new TextEncoder().encode("a");

    // Act
    const hash = fnv1a64Bytes(bytes);

    // Assert
    expect(hex16(hash)).toBe("af63dc4c8601ec8c");
  });

  it("fnv1a64Lanes matches the shared Rust lane vector", () => {
    // Arrange
    const lanes = [
      Float32Array.of(1, -0),
      new Float32Array(0),
      Float32Array.of(0.5),
    ];

    // Act
    const hash = fnv1a64Lanes(lanes);

    // Assert
    expect(hex16(hash)).toBe("99e01cab84d730f5");
  });
});

describe("puzzle-replay spike comparison", () => {
  it("compareSpikes reports identical scenes", () => {
    // Arrange
    const text = [
      spikeLine("dam-break", 1, 1n),
      spikeLine("dam-break", 2, 2n),
    ].join("\n");

    // Act
    const rows = compareSpikes(parseSpikeLines(text), parseSpikeLines(text));

    // Assert
    expect(rows).toEqual([{ scene: "dam-break", identical: true, steps: 2 }]);
  });

  it("compareSpikes reports the first diverging step", () => {
    // Arrange
    const left = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 ${HASH_A}\nSPIKE dam-break 3 ${HASH_A}\n`;
    const right = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 ${HASH_B}\nSPIKE dam-break 3 ${HASH_B}\n`;

    // Act
    const rows = compareSpikes(parseSpikeLines(left), parseSpikeLines(right));

    // Assert
    expect(rows).toEqual([
      {
        scene: "dam-break",
        identical: false,
        steps: 3,
        maybeFirstDivergingStep: 2,
      },
    ]);
  });

  it("parseSpikeLines rejects a malformed line", () => {
    // Arrange
    const text = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 2 not-a-hash\n`;

    // Act
    const parse = () => parseSpikeLines(text);

    // Assert
    expect(parse).toThrow("Malformed spike line 2");
  });

  it("parseSpikeLines rejects a non-contiguous step", () => {
    // Arrange
    const text = `SPIKE dam-break 1 ${HASH_A}\nSPIKE dam-break 3 ${HASH_A}\n`;

    // Act
    const parse = () => parseSpikeLines(text);

    // Assert
    expect(parse).toThrow("Malformed spike line 2");
  });
});

const FULL_WON =
  "level first-pour\nplacement plank 8 64 23\noutcome won\noutcome_step 412\nfinal_count 640\n";

/** first-pour level numbers, as `levelInfo()` reports them. */
const FIRST_POUR: ReplayLevel = {
  targetCount: 540,
  fullCount: 900,
  settleSteps: 30,
  stepLimit: 720,
};

const EXACT_TOLERANCE = 0;
const FALLBACK_TOLERANCE = 5;

function fixtureExpecting(
  outcome: FixtureOutcome,
  outcomeStep: number,
  finalCount: number,
): Fixture {
  return {
    level: "first-pour",
    placements: [],
    maybeExpectation: { outcome, outcomeStep, finalCount },
  };
}

/** Counts of 0 before `fromStep`, then `held` through the margin window end. */
function heldCounts(fromStep: number, held: number): number[] {
  const windowEnd = FIRST_POUR.stepLimit - MARGIN_WIN_SLACK_STEPS;
  return Array.from({ length: windowEnd }, (_, index) =>
    index + 1 < fromStep ? 0 : held,
  );
}

function wonTrace(outcomeStep: number, held: number): ReplayTrace {
  return {
    outcome: "won",
    outcomeStep,
    finalCount: held,
    counts: heldCounts(outcomeStep + 1 - FIRST_POUR.settleSteps, held),
  };
}

function lostTrace(counts: readonly number[]): ReplayTrace {
  return {
    outcome: "lost",
    outcomeStep: FIRST_POUR.stepLimit,
    finalCount: counts.at(-1) ?? 0,
    counts,
  };
}

describe("puzzle-replay fixture parser", () => {
  it("parseFixture reads a full won fixture", () => {
    // Arrange
    const text = FULL_WON;

    // Act
    const fixture = parseFixture(text);

    // Assert
    expect(fixture).toEqual({
      level: "first-pour",
      placements: [{ kind: "plank", x: 8, y: 64, angle: 23 }],
      maybeExpectation: { outcome: "won", outcomeStep: 412, finalCount: 640 },
    });
  });

  it("parseFixture reads an input-only fixture", () => {
    // Arrange
    const text = "level first-pour\n";

    // Act
    const fixture = parseFixture(text);

    // Assert
    expect(fixture).toEqual({ level: "first-pour", placements: [] });
  });

  it("parseFixture rejects an unknown key", () => {
    // Arrange
    const text = "level first-pour\ncolor red\n";

    // Act
    const parse = () => parseFixture(text);

    // Assert
    expect(parse).toThrow("fixture line 2");
  });

  it("parseFixture rejects expectation lines out of order", () => {
    // Arrange
    const text =
      "level first-pour\noutcome_step 412\noutcome won\nfinal_count 640\n";

    // Act
    const parse = () => parseFixture(text);

    // Assert
    expect(parse).toThrow("fixture line");
  });

  it("parseFixture rejects an unknown level or tool", () => {
    // Arrange
    const texts = [
      "level dam-break\n",
      "level first-pour\nplacement valve 1 2 3\n",
    ];

    // Act
    const parses = texts.map((text) => () => parseFixture(text));

    // Assert
    for (const parse of parses) {
      expect(parse).toThrow("fixture line");
    }
  });

  it("parseFixture rejects malformed integers", () => {
    // Arrange
    const texts = [
      "level first-pour\nplacement plank +8 64 23\n",
      "level first-pour\nplacement plank 8.0 64 23\n",
      "level first-pour\nplacement plank x 64 23\n",
      "level first-pour\nplacement plank  64 23\n",
      "level first-pour\noutcome won\noutcome_step -1\nfinal_count 640\n",
    ];

    // Act
    const parses = texts.map((text) => () => parseFixture(text));

    // Assert
    for (const parse of parses) {
      expect(parse).toThrow("fixture line");
    }
  });

  it("parseFixture rejects blank lines and carriage returns", () => {
    // Arrange
    const texts = [
      "level first-pour\n\n",
      "level first-pour\n\nplacement plank 8 64 23\n",
      "level first-pour\r\n",
      "",
    ];

    // Act
    const parses = texts.map((text) => () => parseFixture(text));

    // Assert
    for (const parse of parses) {
      expect(parse).toThrow("fixture line");
    }
  });

  it("fixtureLanes flattens placements in fixture order", () => {
    // Arrange
    const fixture = parseFixture(
      "level first-pour\nplacement plank 8 64 23\nplacement plank -3 50 0\n",
    );

    // Act
    const lanes = fixtureLanes(fixture);

    // Assert
    expect(Array.from(lanes.kinds)).toEqual([1, 1]);
    expect(Array.from(lanes.poses)).toEqual([8, 64, 23, -3, 50, 0]);
  });

  it("checked-in fixtures parse", () => {
    // Arrange
    const directory = resolve(repoRoot, "crates/liquidfun-wasm/src/puzzle/fixtures");

    // Act
    const fixtures = FIXTURE_FILES.map((file) =>
      parseFixture(readFileSync(resolve(directory, file), "utf8")),
    );

    // Assert
    expect(fixtures.map((fixture) => fixture.maybeExpectation?.outcome)).toEqual(
      ["won", "lost"],
    );
  });
});

describe("puzzle-replay contract", () => {
  it("checkReplay exact contract requires equal step and count", () => {
    // Arrange
    const raised = FIRST_POUR.targetCount + fillMarginCount(FIRST_POUR.fullCount);
    const fixture = fixtureExpecting("won", 200, raised);
    const trace = wonTrace(201, raised);

    // Act
    const exact = checkReplay(fixture, trace, FIRST_POUR, true, EXACT_TOLERANCE);
    const fallback = checkReplay(fixture, trace, FIRST_POUR, false, FALLBACK_TOLERANCE);

    // Assert
    expect(exact).not.toEqual([]);
    expect(fallback).toEqual([]);
  });

  it("checkReplay requires a win at the raised target", () => {
    // Arrange
    const raised = FIRST_POUR.targetCount + fillMarginCount(FIRST_POUR.fullCount);
    const short = fixtureExpecting("won", 200, raised - 1);
    const enough = fixtureExpecting("won", 200, raised);

    // Act
    const shortFailures = checkReplay(
      short,
      wonTrace(200, raised - 1),
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );
    const enoughFailures = checkReplay(
      enough,
      wonTrace(200, raised),
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );

    // Assert
    expect(shortFailures).not.toEqual([]);
    expect(enoughFailures).toEqual([]);
  });

  it("checkReplay lost contract requires counts at or below target minus margin", () => {
    // Arrange
    const ceiling = FIRST_POUR.targetCount - fillMarginCount(FIRST_POUR.fullCount);
    const fixture = fixtureExpecting("lost", FIRST_POUR.stepLimit, ceiling);
    const atCeiling = Array.from({ length: FIRST_POUR.stepLimit }, () => ceiling);
    const overCeiling = atCeiling.map((count, index) =>
      index === FIRST_POUR.stepLimit / 2 ? count + 1 : count,
    );

    // Act
    const atFailures = checkReplay(
      fixture,
      lostTrace(atCeiling),
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );
    const overFailures = checkReplay(
      fixture,
      lostTrace(overCeiling),
      FIRST_POUR,
      true,
      EXACT_TOLERANCE,
    );

    // Assert
    expect(atFailures).toEqual([]);
    expect(overFailures).not.toEqual([]);
  });

  it("checkReplay fails on an outcome mismatch", () => {
    // Arrange
    const fixture = fixtureExpecting("won", FIRST_POUR.stepLimit, 0);
    const trace = lostTrace(Array.from({ length: 10 }, () => 0));

    // Act
    const exact = checkReplay(fixture, trace, FIRST_POUR, true, EXACT_TOLERANCE);
    const fallback = checkReplay(fixture, trace, FIRST_POUR, false, FALLBACK_TOLERANCE);

    // Assert
    expect(exact).not.toEqual([]);
    expect(fallback).not.toEqual([]);
  });
});
