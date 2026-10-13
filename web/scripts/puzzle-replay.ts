/**
 * Phase 37 puzzle replay entry point (run from `web/`).
 *
 * - `spike`: prints per-step wasm frame hashes for the spike scenes.
 * - `compare <left> <right>`: compares two spike hash files per scene.
 * - `replay [--record]`: replays the recorded puzzle fixtures through the
 *   generated wasm `newPuzzle` (D-19), checking the outcome, the peak held
 *   count and the Brim step (Phase 39 D-12). `--record` replays only the
 *   level and placement lines, prints `RECORD` lines and never writes a
 *   fixture.
 */

import { appendFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

import {
  parseRenderFrame,
  type PuzzleFrameStatus,
  type RawProofFrame,
  type RenderFrame,
} from "../src/physics/frame";
import type { GeneratedProofSession } from "../src/physics/session";

import { fnv1a64Lanes } from "./puzzle-replay/hash";
import {
  compareSpikes,
  parseSpikeLines,
  SPIKE_SCENES,
  SPIKE_STEPS,
  spikeLine,
} from "./puzzle-replay/spike";
import {
  brimToken,
  checkReplay,
  FIXTURE_FILES,
  fixtureInputText,
  type FixtureOutcome,
  fixtureLanes,
  MARGIN_WIN_SLACK_STEPS,
  parseFixture,
  POST_LATCH_STEPS,
  type ReplayLevel,
  type ReplayTrace,
  WASM_OUTCOME_STEP_TOLERANCE,
  WASM_REPLAY_CONTRACT,
} from "./puzzle-replay/fixture";
import {
  type GeneratedPuzzleSession,
  loadGeneratedWasm,
  type RawPuzzleLevelInfo,
  repoRoot,
} from "./puzzle-replay/wasm";

const USAGE =
  "usage: bun scripts/puzzle-replay.ts spike | compare <leftPath> <rightPath> | replay [--record]";

const FIXTURE_DIRECTORY = resolve(
  repoRoot,
  "crates/liquidfun-wasm/src/puzzle/fixtures",
);
const REPLAY_LOG_DIRECTORY = resolve(repoRoot, "target/puzzle-replay");
const REPLAY_LOG = resolve(REPLAY_LOG_DIRECTORY, "replay.log");
const POST_LATCH_ADVANCE = 4;

async function runSpike(): Promise<void> {
  const generated = await loadGeneratedWasm();
  for (const scene of SPIKE_SCENES) {
    const session = new generated.ProofSession(scene);
    try {
      for (let step = 1; step <= SPIKE_STEPS; step += 1) {
        session.advance(1);
        const frame = session.captureFrame();
        try {
          const hash = fnv1a64Lanes([
            frame.particlePositions(),
            frame.rigidSegments(),
            frame.rigidCircles(),
          ]);
          console.log(spikeLine(scene, step, hash));
        } finally {
          frame.free();
        }
      }
    } finally {
      session.free();
    }
  }
}

async function runCompare(leftPath: string, rightPath: string): Promise<void> {
  const [left, right] = await Promise.all([
    readFile(leftPath, "utf8"),
    readFile(rightPath, "utf8"),
  ]);
  const rows = compareSpikes(parseSpikeLines(left), parseSpikeLines(right));
  for (const row of rows) {
    console.log(
      row.maybeFirstDivergingStep === undefined
        ? `COMPARE ${row.scene} identical ${row.steps}`
        : `COMPARE ${row.scene} diverges-at ${row.maybeFirstDivergingStep} of ${row.steps}`,
    );
  }
}

function readLevel(session: GeneratedPuzzleSession): ReplayLevel {
  const info: RawPuzzleLevelInfo = session.levelInfo();
  try {
    return {
      targetCount: info.targetCount(),
      fullCount: info.fullCount(),
      brimCount: info.brimCount(),
      settleSteps: info.settleSteps(),
      stepLimit: info.stepLimit(),
    };
  } finally {
    info.free();
  }
}

function parseAndFree(rawFrame: RawProofFrame): RenderFrame {
  try {
    return parseRenderFrame(rawFrame);
  } finally {
    rawFrame.free();
  }
}

/** Captures, validates and frees one frame; returns its puzzle readout. */
function capturePuzzle(
  session: GeneratedProofSession,
  level: ReplayLevel,
): { readonly stepIndex: number; readonly puzzle: PuzzleFrameStatus } {
  const frame = parseAndFree(session.captureFrame());
  const puzzle = frame.maybePuzzle;
  if (puzzle === undefined) {
    throw new Error(`puzzle frame ${frame.stepIndex} has no maybePuzzle`);
  }
  if (
    puzzle.goalTarget !== level.targetCount ||
    puzzle.goalFull !== level.fullCount ||
    puzzle.brimCount !== level.brimCount ||
    puzzle.stepLimit !== level.stepLimit
  ) {
    throw new Error(
      `puzzle frame ${frame.stepIndex} does not match levelInfo()`,
    );
  }
  return { stepIndex: frame.stepIndex, puzzle };
}

function requireLatch(
  stepIndex: number,
  puzzle: PuzzleFrameStatus,
  outcome: FixtureOutcome,
  outcomeStep: number,
): void {
  if (puzzle.outcome !== outcome || puzzle.maybeOutcomeStep !== outcomeStep) {
    throw new Error(`latched outcome changed at step ${stepIndex}`);
  }
}

/** Peak held count and Brim step read from the frame at the decision step. */
interface Decision {
  readonly step: number;
  readonly peakHeld: number;
  readonly maybeBrimStep: number | undefined;
}

/**
 * Mirrors Rust `observe_decision`: records the decision from the frame whose
 * step is the decided step, and requires it never to change once present.
 */
function observeDecision(
  maybeDecision: Decision | undefined,
  stepIndex: number,
  puzzle: PuzzleFrameStatus,
): Decision | undefined {
  if (maybeDecision !== undefined) {
    if (puzzle.maybeDecidedStep !== maybeDecision.step) {
      throw new Error(`decided step changed at step ${stepIndex}`);
    }
    return maybeDecision;
  }
  const decided = puzzle.maybeDecidedStep;
  if (decided === undefined) {
    return undefined;
  }
  if (decided !== stepIndex) {
    throw new Error(
      `decided step ${decided} first appeared on frame ${stepIndex}`,
    );
  }
  return {
    step: stepIndex,
    peakHeld: puzzle.peakHeldCount,
    maybeBrimStep: puzzle.maybeBrimStep,
  };
}

/** Steps a puzzle session like Rust `replay::replay`, on one session. */
function replaySession(
  session: GeneratedPuzzleSession,
  level: ReplayLevel,
): ReplayTrace {
  const counts: number[] = [];
  let latch: { outcome: FixtureOutcome; outcomeStep: number; finalCount: number } | undefined;
  let maybeDecision: Decision | undefined;
  let stepIndex = 0;
  while (latch === undefined) {
    session.advance(1);
    const captured = capturePuzzle(session, level);
    stepIndex = captured.stepIndex;
    counts.push(captured.puzzle.goalCount);
    maybeDecision = observeDecision(maybeDecision, stepIndex, captured.puzzle);
    if (captured.puzzle.outcome !== "running") {
      const outcomeStep = captured.puzzle.maybeOutcomeStep;
      if (outcomeStep === undefined) {
        throw new Error(`latched frame ${stepIndex} has no outcome step`);
      }
      latch = {
        outcome: captured.puzzle.outcome,
        outcomeStep,
        finalCount: captured.puzzle.goalCount,
      };
    } else if (stepIndex > level.stepLimit) {
      throw new Error(
        `replay ran past step limit ${level.stepLimit} without latching`,
      );
    }
  }
  const windowEnd = level.stepLimit - MARGIN_WIN_SLACK_STEPS;
  while (maybeDecision === undefined || stepIndex < windowEnd) {
    if (maybeDecision === undefined && stepIndex >= level.stepLimit) {
      throw new Error(
        `replay reached step limit ${level.stepLimit} without a decision`,
      );
    }
    session.advance(1);
    const captured = capturePuzzle(session, level);
    stepIndex = captured.stepIndex;
    requireLatch(stepIndex, captured.puzzle, latch.outcome, latch.outcomeStep);
    counts.push(captured.puzzle.goalCount);
    maybeDecision = observeDecision(maybeDecision, stepIndex, captured.puzzle);
  }
  const decision = maybeDecision;
  for (let done = 0; done < POST_LATCH_STEPS; done += POST_LATCH_ADVANCE) {
    session.advance(POST_LATCH_ADVANCE);
    const captured = capturePuzzle(session, level);
    requireLatch(
      captured.stepIndex,
      captured.puzzle,
      latch.outcome,
      latch.outcomeStep,
    );
    observeDecision(decision, captured.stepIndex, captured.puzzle);
  }
  return {
    ...latch,
    peakHeld: decision.peakHeld,
    maybeBrimStep: decision.maybeBrimStep,
    decidedStep: decision.step,
    counts,
  };
}

async function runReplay(record: boolean): Promise<void> {
  await mkdir(REPLAY_LOG_DIRECTORY, { recursive: true });
  await writeFile(REPLAY_LOG, "");
  const report = async (line: string): Promise<void> => {
    console.log(line);
    await appendFile(REPLAY_LOG, `${line}\n`);
  };
  const generated = await loadGeneratedWasm();
  let failed = false;
  for (const file of FIXTURE_FILES) {
    const text = await readFile(resolve(FIXTURE_DIRECTORY, file), "utf8");
    const fixture = parseFixture(record ? fixtureInputText(text) : text);
    const { kinds, poses } = fixtureLanes(fixture);
    const session = generated.ProofSession.newPuzzle(fixture.level, kinds, poses);
    try {
      const level = readLevel(session);
      const trace = replaySession(session, level);
      if (record) {
        await report(
          `RECORD ${file} outcome ${trace.outcome} outcome_step ${trace.outcomeStep} final_count ${trace.finalCount} peak_held ${trace.peakHeld} brim_step ${brimToken(trace.maybeBrimStep)} profile wasm`,
        );
        continue;
      }
      const failures = checkReplay(
        fixture,
        trace,
        level,
        WASM_REPLAY_CONTRACT === "exact",
        WASM_OUTCOME_STEP_TOLERANCE,
      );
      if (failures.length === 0) {
        await report(
          `REPLAY ${file} pass (outcome ${trace.outcome} step ${trace.outcomeStep} final_count ${trace.finalCount} peak_held ${trace.peakHeld} brim_step ${brimToken(trace.maybeBrimStep)})`,
        );
        continue;
      }
      failed = true;
      for (const failure of failures) {
        await report(`REPLAY ${file} fail: ${failure}`);
      }
    } finally {
      session.free();
    }
  }
  if (failed) {
    process.exit(1);
  }
}

async function main(args: readonly string[]): Promise<void> {
  const [mode, leftPath, rightPath] = args;
  if (mode === "replay" && args.length <= 2) {
    const flag = args[1];
    if (flag === undefined || flag === "--record") {
      await runReplay(flag === "--record");
      return;
    }
  }
  if (mode === "spike" && args.length === 1) {
    await runSpike();
    return;
  }
  if (
    mode === "compare" &&
    args.length === 3 &&
    leftPath !== undefined &&
    rightPath !== undefined
  ) {
    await runCompare(leftPath, rightPath);
    return;
  }
  console.error(USAGE);
  process.exit(2);
}

main(process.argv.slice(2)).catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
