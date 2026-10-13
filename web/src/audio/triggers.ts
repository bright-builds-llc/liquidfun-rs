/**
 * Pure trigger rules for puzzle sound (Phase 42 D-06, D-09, D-15). They run
 * once per presented frame, never per fixed step, so fast-forward cannot
 * multiply voices. Time is the shell's audio clock in seconds.
 */

import type { RenderFrame } from "../physics/frame";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import type { PresentedSoundSignals } from "./sound-signals";
import {
  IMPACT_MIN_STRENGTH,
  impactStrength,
  plinkFrequency,
  splashIntensity,
  type VoiceRequest,
} from "./voices";

/** A splash after this much audio clock time without one is a burst onset (D-15). */
export const BURST_QUIET_SECONDS = 0.4;

const QUIET_TOLERANCE_SECONDS = 1e-9;

/** What the rules remember between presented frames of one run. */
export type TriggerMemory = {
  readonly maybeLastStepIndex: number | undefined;
  readonly lastGoalCount: number;
  readonly wasWon: boolean;
  readonly maybeLastSplashAt: number | undefined;
};

/** Memory before a run's first presented frame. */
export const INITIAL_TRIGGER_MEMORY: TriggerMemory = {
  maybeLastStepIndex: undefined,
  lastGoalCount: 0,
  wasWon: false,
  maybeLastSplashAt: undefined,
};

/** One presented frame, its folded signals and the audio clock time. */
export type TriggerInput = {
  readonly state: PuzzleState;
  readonly frame: RenderFrame;
  readonly signals: PresentedSoundSignals;
  readonly nowSeconds: number;
};

/** Updated memory and the voices to request, ordered chime, impact, splash, plink. */
export type TriggerDecision = {
  readonly memory: TriggerMemory;
  readonly requests: readonly VoiceRequest[];
};

type RunState = Extract<PuzzleState, { readonly kind: "running" | "ended" }>;

/** A run frame after resets: the memory it started from and the memory it leaves. */
type Observation = {
  readonly state: RunState;
  readonly previous: TriggerMemory;
  readonly next: TriggerMemory;
  readonly won: boolean;
};

function isWon(state: RunState): boolean {
  if (state.kind === "running") {
    return state.status.outcome === "won";
  }

  return state.latch.outcome === "won";
}

/**
 * Rules shared by both entry points: no run outside running/ended, a step
 * index that does not increase starts a new run, and the frame's step, goal
 * count and win are always recorded. The splash time carries over unchanged.
 */
function observe(
  memory: TriggerMemory,
  state: PuzzleState,
  frame: RenderFrame,
): Observation | undefined {
  if (state.kind === "loading" || state.kind === "building") {
    return undefined;
  }

  const maybeLast = memory.maybeLastStepIndex;
  const previous =
    maybeLast !== undefined && frame.stepIndex <= maybeLast ? INITIAL_TRIGGER_MEMORY : memory;
  const won = isWon(state);
  const next: TriggerMemory = {
    ...previous,
    maybeLastStepIndex: frame.stepIndex,
    lastGoalCount: frame.maybePuzzle?.goalCount ?? previous.lastGoalCount,
    wasWon: previous.wasWon || won,
  };
  return { state, previous, next, won };
}

function isBurstOnset(previous: TriggerMemory, nowSeconds: number): boolean {
  const maybeLastSplashAt = previous.maybeLastSplashAt;
  if (maybeLastSplashAt === undefined) {
    return true;
  }

  return nowSeconds - maybeLastSplashAt >= BURST_QUIET_SECONDS - QUIET_TOLERANCE_SECONDS;
}

/** One impact request: the stronger of a rigid impact and a splash burst onset. */
function maybeImpactStrength(
  maybeRigid: number | undefined,
  maybeBurst: number | undefined,
): number | undefined {
  if (maybeRigid === undefined) {
    return maybeBurst;
  }

  return maybeBurst === undefined ? maybeRigid : Math.max(maybeRigid, maybeBurst);
}

function maybePlink(previous: TriggerMemory, frame: RenderFrame): VoiceRequest | undefined {
  const maybePuzzle = frame.maybePuzzle;
  if (maybePuzzle === undefined || !(maybePuzzle.goalCount > previous.lastGoalCount)) {
    return undefined;
  }

  return { kind: "plink", frequency: plinkFrequency(maybePuzzle.goalCount, maybePuzzle.goalTarget) };
}

/** Splash, impact and plink voices for an active, not-won frame. */
function runVoices(observation: Observation, input: TriggerInput): TriggerDecision {
  const { previous, next } = observation;
  const maybeSplash = splashIntensity(input.signals, input.frame.maxSpeed);
  const maybeBurst =
    maybeSplash !== undefined && isBurstOnset(previous, input.nowSeconds)
      ? Math.max(IMPACT_MIN_STRENGTH, maybeSplash)
      : undefined;
  const maybeImpact = maybeImpactStrength(impactStrength(input.signals), maybeBurst);
  const requests: VoiceRequest[] = [];
  if (maybeImpact !== undefined) {
    requests.push({ kind: "impact", strength: maybeImpact });
  }
  if (maybeSplash !== undefined) {
    requests.push({ kind: "splash", intensity: maybeSplash });
  }
  const maybePlinkRequest = maybePlink(previous, input.frame);
  if (maybePlinkRequest !== undefined) {
    requests.push(maybePlinkRequest);
  }

  const memory =
    maybeSplash === undefined ? next : { ...next, maybeLastSplashAt: input.nowSeconds };
  return { memory, requests };
}

/**
 * Decides which voices one presented frame requests. Building and loading
 * are silent, a held end is silent, the first won frame requests only the
 * chime, and nothing else starts after a win.
 */
export function decideVoices(memory: TriggerMemory, input: TriggerInput): TriggerDecision {
  const maybeObservation = observe(memory, input.state, input.frame);
  if (maybeObservation === undefined) {
    return { memory: INITIAL_TRIGGER_MEMORY, requests: [] };
  }

  const { state, previous, next, won } = maybeObservation;
  if (state.kind === "ended" && state.held) {
    return { memory: next, requests: [] };
  }
  if (won) {
    return { memory: next, requests: previous.wasWon ? [] : [{ kind: "chime" }] };
  }

  return runVoices(maybeObservation, input);
}

/**
 * Memory update for a presented frame when no audio clock exists yet (no
 * context): tracks step, goal count and win like decideVoices but never
 * touches the splash time and requests nothing (D-15).
 */
export function trackWithoutClock(
  memory: TriggerMemory,
  input: Omit<TriggerInput, "signals" | "nowSeconds">,
): TriggerMemory {
  return observe(memory, input.state, input.frame)?.next ?? INITIAL_TRIGGER_MEMORY;
}
