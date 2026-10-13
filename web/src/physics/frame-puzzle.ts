import type {
  PuzzleFrameStatus,
  PuzzleOutcome,
  RawProofFrame,
} from "./frame";
import {
  invalidFrame,
  parseNonNegativeInteger,
  parsePositiveInteger,
} from "./frame-scalars";

/** Goal fields parsed before the D-11 star fields are checked against them. */
type GoalFrameStatus = Omit<
  PuzzleFrameStatus,
  "peakHeldCount" | "brimCount" | "maybeBrimStep" | "maybeDecidedStep"
>;

/** D-11 star fields of a puzzle frame status. */
type StarFrameFields = Pick<
  PuzzleFrameStatus,
  "peakHeldCount" | "brimCount" | "maybeBrimStep" | "maybeDecidedStep"
>;

/** Raw optional puzzle getter results, read once per frame. */
interface MaybePuzzleScalars {
  readonly maybeGoalCount: number | undefined;
  readonly maybeGoalTarget: number | undefined;
  readonly maybeGoalFull: number | undefined;
  readonly maybeOutcome: number | undefined;
  readonly maybeOutcomeStep: number | undefined;
  readonly maybeStepLimit: number | undefined;
  readonly maybePeakHeldCount: number | undefined;
  readonly maybeBrimCount: number | undefined;
  readonly maybeBrimStep: number | undefined;
  readonly maybeDecidedStep: number | undefined;
}

const PUZZLE_OUTCOMES: readonly PuzzleOutcome[] = ["running", "won", "lost"];

function parseOutcomeStep(
  outcome: PuzzleOutcome,
  maybeOutcomeStep: number | undefined,
  stepIndex: number,
  stepLimit: number,
): number | undefined {
  if (outcome === "running") {
    return maybeOutcomeStep === undefined ? undefined : invalidFrame();
  }

  if (maybeOutcomeStep === undefined) {
    return invalidFrame();
  }

  const outcomeStep = parseNonNegativeInteger(maybeOutcomeStep);
  if (outcomeStep > stepIndex || outcomeStep > stepLimit) {
    return invalidFrame();
  }

  if (outcome === "lost" && outcomeStep !== stepLimit) {
    return invalidFrame();
  }

  return outcomeStep;
}

function readPuzzleScalars(rawFrame: RawProofFrame): MaybePuzzleScalars {
  return {
    maybeGoalCount: rawFrame.goalCount?.(),
    maybeGoalTarget: rawFrame.goalTarget?.(),
    maybeGoalFull: rawFrame.goalFull?.(),
    maybeOutcome: rawFrame.outcome?.(),
    maybeOutcomeStep: rawFrame.outcomeStep?.(),
    maybeStepLimit: rawFrame.stepLimit?.(),
    maybePeakHeldCount: rawFrame.peakHeldCount?.(),
    maybeBrimCount: rawFrame.brimCount?.(),
    maybeBrimStep: rawFrame.brimStep?.(),
    maybeDecidedStep: rawFrame.decidedStep?.(),
  };
}

function parseGoalStatus(
  scalars: MaybePuzzleScalars,
  stepIndex: number,
  particleCount: number,
): GoalFrameStatus {
  const {
    maybeGoalCount,
    maybeGoalTarget,
    maybeGoalFull,
    maybeOutcome,
    maybeStepLimit,
  } = scalars;
  if (
    maybeGoalCount === undefined ||
    maybeGoalTarget === undefined ||
    maybeGoalFull === undefined ||
    maybeOutcome === undefined ||
    maybeStepLimit === undefined
  ) {
    return invalidFrame();
  }

  const goalCount = parseNonNegativeInteger(maybeGoalCount);
  const goalTarget = parsePositiveInteger(maybeGoalTarget);
  const goalFull = parseNonNegativeInteger(maybeGoalFull);
  const stepLimit = parsePositiveInteger(maybeStepLimit);
  if (goalFull < goalTarget || goalCount > particleCount) {
    return invalidFrame();
  }

  const outcome = PUZZLE_OUTCOMES[parseNonNegativeInteger(maybeOutcome)];
  if (outcome === undefined) {
    return invalidFrame();
  }

  const outcomeStep = parseOutcomeStep(
    outcome,
    scalars.maybeOutcomeStep,
    stepIndex,
    stepLimit,
  );
  return {
    goalCount,
    goalTarget,
    goalFull,
    outcome,
    ...(outcomeStep === undefined ? {} : { maybeOutcomeStep: outcomeStep }),
    stepLimit,
  };
}

/** Brim latches only on a won run, between the win latch and the step limit. */
function parseBrimStep(
  maybeBrimStep: number | undefined,
  reachedBrim: boolean,
  goal: GoalFrameStatus,
  stepIndex: number,
): number | undefined {
  if (maybeBrimStep === undefined) {
    return reachedBrim ? invalidFrame() : undefined;
  }

  const brimStep = parseNonNegativeInteger(maybeBrimStep);
  const { maybeOutcomeStep } = goal;
  if (
    !reachedBrim ||
    goal.outcome !== "won" ||
    maybeOutcomeStep === undefined ||
    brimStep < maybeOutcomeStep ||
    brimStep > Math.min(stepIndex, goal.stepLimit)
  ) {
    return invalidFrame();
  }

  return brimStep;
}

/**
 * The D-10 decision step: a loss at its latch (the step limit), a win at its
 * Brim latch, or at the step limit for a win without Brim.
 */
function expectedDecidedStep(
  outcome: PuzzleOutcome,
  maybeBrimStep: number | undefined,
  stepIndex: number,
  stepLimit: number,
): number | undefined {
  if (outcome === "running") {
    return undefined;
  }

  if (outcome === "lost") {
    return stepLimit;
  }

  if (maybeBrimStep !== undefined) {
    return maybeBrimStep;
  }

  return stepIndex >= stepLimit ? stepLimit : undefined;
}

function parseStarFields(
  scalars: MaybePuzzleScalars,
  goal: GoalFrameStatus,
  stepIndex: number,
): StarFrameFields {
  const { maybePeakHeldCount, maybeBrimCount } = scalars;
  if (maybePeakHeldCount === undefined || maybeBrimCount === undefined) {
    return invalidFrame();
  }

  // The peak is not bounded by the live particle count: eviction can remove
  // particles after the peak was held.
  const peakHeldCount = parseNonNegativeInteger(maybePeakHeldCount);
  const brimCount = parsePositiveInteger(maybeBrimCount);
  if (
    brimCount <= goal.goalTarget ||
    brimCount > goal.goalFull ||
    (goal.outcome === "won") !== peakHeldCount >= goal.goalTarget
  ) {
    return invalidFrame();
  }

  const reachedBrim = peakHeldCount >= brimCount;
  const brimStep = parseBrimStep(
    scalars.maybeBrimStep,
    reachedBrim,
    goal,
    stepIndex,
  );
  const decidedStep = expectedDecidedStep(
    goal.outcome,
    brimStep,
    stepIndex,
    goal.stepLimit,
  );
  if (scalars.maybeDecidedStep !== decidedStep) {
    return invalidFrame();
  }

  return {
    peakHeldCount,
    brimCount,
    ...(brimStep === undefined ? {} : { maybeBrimStep: brimStep }),
    ...(decidedStep === undefined ? {} : { maybeDecidedStep: decidedStep }),
  };
}

/**
 * Parses the optional D-17 and D-11 puzzle scalars, failing closed on partial
 * or inconsistent values. Returns undefined for scene frames.
 */
export function parsePuzzleStatus(
  rawFrame: RawProofFrame,
  stepIndex: number,
  particleCount: number,
): PuzzleFrameStatus | undefined {
  const scalars = readPuzzleScalars(rawFrame);
  if (Object.values(scalars).every((value) => value === undefined)) {
    return undefined;
  }

  const goal = parseGoalStatus(scalars, stepIndex, particleCount);
  return { ...goal, ...parseStarFields(scalars, goal, stepIndex) };
}
