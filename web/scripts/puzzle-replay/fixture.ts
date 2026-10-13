/**
 * Puzzle replay fixtures (Phase 37 D-18, D-19, D-22; Phase 39 D-12).
 *
 * Mirrors `crates/liquidfun-wasm/src/puzzle/replay/fixture.rs` (grammar) and
 * `replay.rs` (contract) one to one: the same strict line grammar, the same
 * `fixture line N: ...` errors, and the same contract, so both runtimes
 * accept exactly the same text.
 */

import { PUZZLE_IDS } from "../../src/catalog/puzzles";

/** Recorded outcome tokens. */
export type FixtureOutcome = "won" | "lost";

/** One `placement <kind> <x> <y> <angle>` line. */
export interface FixturePlacement {
  readonly kind: "plank";
  readonly x: number;
  readonly y: number;
  readonly angle: number;
}

/**
 * The recorded `outcome`, `outcome_step`, `final_count`, `peak_held` and
 * `brim_step` lines.
 */
export interface FixtureExpectation {
  readonly outcome: FixtureOutcome;
  readonly outcomeStep: number;
  /** Frame goal count at `outcomeStep`. */
  readonly finalCount: number;
  /** Frame peak held count at the decision step (D-12). */
  readonly peakHeld: number;
  /** Frame Brim step; undefined for the `none` token (D-12). */
  readonly maybeBrimStep: number | undefined;
}

/** A parsed fixture; input-only fixtures have no expectation. */
export interface Fixture {
  readonly level: string;
  readonly placements: readonly FixturePlacement[];
  readonly maybeExpectation?: FixtureExpectation;
}

/** What one fixture run produced. */
export interface ReplayTrace {
  readonly outcome: FixtureOutcome;
  readonly outcomeStep: number;
  readonly finalCount: number;
  /** Frame peak held count at `decidedStep` (D-12). */
  readonly peakHeld: number;
  /** Frame Brim step at `decidedStep` (D-12). */
  readonly maybeBrimStep: number | undefined;
  /** The D-10 decision step, read from the frames. */
  readonly decidedStep: number;
  /**
   * `counts[i]` is the goal count after step `i + 1`, through the later of
   * `decidedStep` and `stepLimit - MARGIN_WIN_SLACK_STEPS`.
   */
  readonly counts: readonly number[];
}

/** Level numbers the contract needs, read from `levelInfo()`. */
export interface ReplayLevel {
  readonly targetCount: number;
  readonly fullCount: number;
  readonly brimCount: number;
  readonly settleSteps: number;
  readonly stepLimit: number;
}

/** Checked-in fixtures under `crates/liquidfun-wasm/src/puzzle/fixtures`. */
export const FIXTURE_FILES = [
  "first-pour-solution.txt",
  "first-pour-empty.txt",
] as const;

/** Tool tokens and their `kinds` lane codes. */
export const TOOL_CODES = { plank: 1 } as const;

/** Steps run after the latch and the margin window: two `advance(4)` calls. */
export const POST_LATCH_STEPS = 8;

/** Mirrors Rust `goal::MARGIN_WIN_SLACK_STEPS`. */
export const MARGIN_WIN_SLACK_STEPS = 120;

/**
 * Release-native vs wasm contract (D-03, D-22). The 37-01 spike recorded
 * `spike_contract: exact`, and Plan 08 confirmed it on first-pour: the wasm
 * RECORD lines (won 201 / 689, lost 720 / 0) equal the release-native ones.
 */
export const WASM_REPLAY_CONTRACT: "exact" | "fallback" = "exact";

/** Outcome-step tolerance, used only by the fallback contract. */
export const WASM_OUTCOME_STEP_TOLERANCE = 0;

const KNOWN_KEYS = [
  "level",
  "placement",
  "outcome",
  "outcome_step",
  "final_count",
  "peak_held",
  "brim_step",
] as const;

/** The `brim_step` token for a run that never reached Brim. */
export const NO_BRIM_TOKEN = "none";

const I32_MIN = -2_147_483_648;
const I32_MAX = 2_147_483_647;
const U32_MAX = 4_294_967_295;

interface Row {
  readonly number: number;
  readonly tokens: readonly [string, ...string[]];
}

function lineError(number: number, reason: string): Error {
  return new Error(`fixture line ${number}: ${reason}`);
}

function splitRows(text: string): Row[] {
  const body = text.endsWith("\n") ? text.slice(0, -1) : text;
  return body.split("\n").map((line, index) => {
    const number = index + 1;
    if (line.includes("\r")) {
      throw lineError(number, "carriage return");
    }
    if (line === "") {
      throw lineError(number, "blank line");
    }
    const [first = "", ...rest] = line.split(" ");
    return { number, tokens: [first, ...rest] };
  });
}

function checkKey(row: Row, key: string): void {
  const found = row.tokens[0];
  if (!(KNOWN_KEYS as readonly string[]).includes(found)) {
    throw lineError(row.number, `unknown key \`${found}\``);
  }
  if (found !== key) {
    throw lineError(row.number, `expected \`${key}\`, found \`${found}\``);
  }
}

function checkArity(row: Row, arity: number): void {
  if (row.tokens.length !== arity) {
    throw lineError(row.number, `expected ${arity} space-separated tokens`);
  }
}

function token(row: Row, index: number): string {
  const value = row.tokens[index];
  if (value === undefined) {
    throw lineError(row.number, `missing token ${index + 1}`);
  }
  return value;
}

/** Accepts exactly `-?[0-9]+` within `i32`. */
function parseI32(number: number, raw: string): number {
  if (!/^-?[0-9]+$/.test(raw)) {
    throw lineError(number, `\`${raw}\` is not an integer`);
  }
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value < I32_MIN || value > I32_MAX) {
    throw lineError(number, `\`${raw}\` is out of range`);
  }
  return value === 0 ? 0 : value;
}

/** Accepts exactly `[0-9]+` within `u32`. */
function parseU32(number: number, raw: string): number {
  if (!/^[0-9]+$/.test(raw)) {
    throw lineError(number, `\`${raw}\` is not a non-negative integer`);
  }
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value > U32_MAX) {
    throw lineError(number, `\`${raw}\` is out of range`);
  }
  return value;
}

function parseLevelLine(row: Row): string {
  checkKey(row, "level");
  checkArity(row, 2);
  const level = token(row, 1);
  if (!(PUZZLE_IDS as readonly string[]).includes(level)) {
    throw lineError(row.number, "unknown level");
  }
  return level;
}

function isToolToken(raw: string): raw is keyof typeof TOOL_CODES {
  return Object.hasOwn(TOOL_CODES, raw);
}

function parsePlacementLine(row: Row): FixturePlacement {
  checkArity(row, 5);
  const kind = token(row, 1);
  if (!isToolToken(kind)) {
    throw lineError(row.number, "unknown tool");
  }
  return {
    kind,
    x: parseI32(row.number, token(row, 2)),
    y: parseI32(row.number, token(row, 3)),
    angle: parseI32(row.number, token(row, 4)),
  };
}

function expectField(
  maybeRow: Row | undefined,
  key: string,
  missingNumber: number,
): { readonly number: number; readonly value: string } {
  if (maybeRow === undefined) {
    throw lineError(missingNumber, `missing \`${key}\` line`);
  }
  checkKey(maybeRow, key);
  checkArity(maybeRow, 2);
  return { number: maybeRow.number, value: token(maybeRow, 1) };
}

function parseExpectation(
  rows: readonly Row[],
  missingNumber: number,
): FixtureExpectation {
  const outcomeField = expectField(rows[0], "outcome", missingNumber);
  const outcome = outcomeField.value;
  if (outcome !== "won" && outcome !== "lost") {
    throw lineError(outcomeField.number, "outcome must be `won` or `lost`");
  }
  const stepField = expectField(rows[1], "outcome_step", missingNumber);
  const outcomeStep = parseU32(stepField.number, stepField.value);
  const countField = expectField(rows[2], "final_count", missingNumber);
  const finalCount = parseU32(countField.number, countField.value);
  const peakField = expectField(rows[3], "peak_held", missingNumber);
  const peakHeld = parseU32(peakField.number, peakField.value);
  const brimField = expectField(rows[4], "brim_step", missingNumber);
  const maybeBrimStep =
    brimField.value === NO_BRIM_TOKEN
      ? undefined
      : parseU32(brimField.number, brimField.value);
  const extra = rows[5];
  if (extra !== undefined) {
    throw lineError(extra.number, "unexpected line after brim_step");
  }
  return { outcome, outcomeStep, finalCount, peakHeld, maybeBrimStep };
}

/**
 * Parses the strict D-18 line format with the D-12 expectation lines; throws
 * `fixture line N: ...`.
 */
export function parseFixture(text: string): Fixture {
  const rows = splitRows(text);
  const [levelRow, ...rest] = rows;
  if (levelRow === undefined) {
    throw lineError(1, "missing level line");
  }
  const level = parseLevelLine(levelRow);
  const placements: FixturePlacement[] = [];
  let index = 0;
  for (; rest[index]?.tokens[0] === "placement"; index += 1) {
    const row = rest[index];
    if (row !== undefined) {
      placements.push(parsePlacementLine(row));
    }
  }
  const expectationRows = rest.slice(index);
  if (expectationRows.length === 0) {
    return { level, placements };
  }
  return {
    level,
    placements,
    maybeExpectation: parseExpectation(expectationRows, rows.length + 1),
  };
}

/** Flattens placements into the `newPuzzle` lanes in fixture order. */
export function fixtureLanes(fixture: Fixture): {
  readonly kinds: Uint8Array;
  readonly poses: Int32Array;
} {
  return {
    kinds: Uint8Array.from(
      fixture.placements.map((placement) => TOOL_CODES[placement.kind]),
    ),
    poses: Int32Array.from(
      fixture.placements.flatMap((placement) => [
        placement.x,
        placement.y,
        placement.angle,
      ]),
    ),
  };
}

/** The +5% fill margin as a particle count, rounded up (`div_ceil(20)`). */
export function fillMarginCount(fullCount: number): number {
  return Math.floor((fullCount + 19) / 20);
}

/**
 * Mirrors Rust `goal::maybe_margin_win_step`: replays `counts` through a
 * `GoalTracker` at `count + fillMarginCount(fullCount)` with the same
 * `settleSteps` and a deadline `MARGIN_WIN_SLACK_STEPS` before `stepLimit`.
 * `count` is the target for the win margin and the Brim count for the Brim
 * margin. Returns the win step, or undefined when that tracker never wins.
 */
export function marginWinStep(
  counts: readonly number[],
  count: number,
  level: ReplayLevel,
): number | undefined {
  const raisedTarget = count + fillMarginCount(level.fullCount);
  const deadline = level.stepLimit - MARGIN_WIN_SLACK_STEPS;
  if (
    deadline < 0 ||
    raisedTarget === 0 ||
    level.settleSteps === 0 ||
    level.settleSteps > deadline
  ) {
    return undefined;
  }
  let held = 0;
  for (const [index, count] of counts.entries()) {
    const step = index + 1;
    held = count >= raisedTarget ? held + 1 : 0;
    if (held >= level.settleSteps && step <= deadline) {
      return step;
    }
    if (step >= deadline) {
      return undefined;
    }
  }
  return undefined;
}

/** `count` raised by the fill margin must hold by the margin window end. */
function raisedHoldFailures(
  label: string,
  trace: ReplayTrace,
  count: number,
  level: ReplayLevel,
): string[] {
  return marginWinStep(trace.counts, count, level) === undefined
    ? [
        `${label} does not hold at the raised count ${count + fillMarginCount(level.fullCount)} by step ${Math.max(0, level.stepLimit - MARGIN_WIN_SLACK_STEPS)}`,
      ]
    : [];
}

function marginFailures(trace: ReplayTrace, level: ReplayLevel): string[] {
  const margin = fillMarginCount(level.fullCount);
  if (trace.outcome === "won") {
    return raisedHoldFailures("win", trace, level.targetCount, level);
  }
  if (trace.outcomeStep !== level.stepLimit) {
    return [
      `loss latched at step ${trace.outcomeStep} instead of the step limit ${level.stepLimit}`,
    ];
  }
  const ceiling = Math.max(0, level.targetCount - margin);
  const index = trace.counts.findIndex((count) => count > ceiling);
  return index === -1
    ? []
    : [
        `count ${trace.counts[index]} after step ${index + 1} exceeds the loss ceiling ${ceiling}`,
      ];
}

/** The fixture token for an optional Brim step: digits or `none`. */
export function brimToken(maybeBrimStep: number | undefined): string {
  return maybeBrimStep === undefined ? NO_BRIM_TOKEN : String(maybeBrimStep);
}

function exactFailures(
  trace: ReplayTrace,
  expected: FixtureExpectation,
): string[] {
  if (trace.outcomeStep !== expected.outcomeStep) {
    return [
      `outcome step ${trace.outcomeStep} differs from recorded ${expected.outcomeStep}`,
    ];
  }
  if (trace.finalCount !== expected.finalCount) {
    return [
      `final count ${trace.finalCount} differs from recorded ${expected.finalCount}`,
    ];
  }
  if (trace.peakHeld !== expected.peakHeld) {
    return [
      `peak held ${trace.peakHeld} differs from recorded ${expected.peakHeld}`,
    ];
  }
  return trace.maybeBrimStep === expected.maybeBrimStep
    ? []
    : [
        `Brim step ${brimToken(trace.maybeBrimStep)} differs from recorded ${brimToken(expected.maybeBrimStep)}`,
      ];
}

function toleranceFailures(
  label: string,
  step: number,
  recorded: number,
  tolerance: number,
): string[] {
  const spread = Math.abs(step - recorded);
  return spread <= tolerance
    ? []
    : [
        `${label} ${step} is ${spread} steps from recorded ${recorded} (tolerance ${tolerance})`,
      ];
}

function fallbackFailures(
  trace: ReplayTrace,
  expected: FixtureExpectation,
  tolerance: number,
): string[] {
  const outcome = toleranceFailures(
    "outcome step",
    trace.outcomeStep,
    expected.outcomeStep,
    tolerance,
  );
  if (outcome.length > 0) {
    return outcome;
  }
  const { maybeBrimStep } = trace;
  const maybeRecorded = expected.maybeBrimStep;
  if (maybeBrimStep !== undefined && maybeRecorded !== undefined) {
    return toleranceFailures("Brim step", maybeBrimStep, maybeRecorded, tolerance);
  }
  return maybeBrimStep === maybeRecorded
    ? []
    : [
        `Brim step ${brimToken(maybeBrimStep)} differs from recorded ${brimToken(maybeRecorded)}`,
      ];
}

/**
 * Mirrors Rust `check_contract`; returns failures (`[]` = pass). Both forms
 * require the recorded outcome and the margin, and a recorded Brim step must
 * still be reached with the Brim count raised by the margin (D-12). `exact`
 * also requires the recorded step, final count, peak held count and Brim
 * step; otherwise the outcome and Brim steps may differ by `tolerance` and
 * Brim must be reached exactly when recorded.
 */
export function checkReplay(
  fixture: Fixture,
  trace: ReplayTrace,
  level: ReplayLevel,
  exact: boolean,
  tolerance: number,
): string[] {
  const expected = fixture.maybeExpectation;
  if (expected === undefined) {
    return [
      "fixture has no recorded outcome; record with `bun scripts/puzzle-replay.ts replay --record`",
    ];
  }
  if (trace.outcome !== expected.outcome) {
    return [
      `outcome ${trace.outcome} differs from recorded ${expected.outcome}`,
    ];
  }
  const margin = marginFailures(trace, level);
  if (margin.length > 0) {
    return margin;
  }
  if (expected.maybeBrimStep !== undefined) {
    const brim = raisedHoldFailures("Brim", trace, level.brimCount, level);
    if (brim.length > 0) {
      return brim;
    }
  }
  return exact
    ? exactFailures(trace, expected)
    : fallbackFailures(trace, expected, tolerance);
}

/**
 * The level and placement lines of a fixture. Record mode regenerates the
 * expectation, so it must not depend on the recorded lines' grammar.
 */
export function fixtureInputText(text: string): string {
  const lines = text.endsWith("\n") ? text.slice(0, -1).split("\n") : text.split("\n");
  const end = lines.findIndex((line) => line.startsWith("outcome "));
  return `${(end === -1 ? lines : lines.slice(0, end)).join("\n")}\n`;
}
