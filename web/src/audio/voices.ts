/**
 * Pure voice parameters for puzzle sound (Phase 42 D-06). Times are audio
 * clock seconds. The numbers are tunable discretion within D-09; the splash
 * and impact thresholds come from the Plan 01 sound probe
 * (`target/phase42/sound-probe.txt`, recorded in 42-02-SUMMARY.md).
 */

import type { PresentedSoundSignals } from "./sound-signals";

export type VoiceKind = "splash" | "impact" | "plink" | "chime";

/** One voice the trigger rules ask the engine to start. */
export type VoiceRequest =
  | { readonly kind: "splash"; readonly intensity: number }
  | { readonly kind: "impact"; readonly strength: number }
  | { readonly kind: "plink"; readonly frequency: number }
  | { readonly kind: "chime" };

/** Two octaves of major pentatonic: 11 steps from the root. */
export const PENTATONIC_SEMITONES = [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24] as const;
/** C5. */
export const PLINK_ROOT_HZ = 523.25;
/** E5; the win chime is an E major arpeggio. */
export const CHIME_ROOT_HZ = 659.25;

/** New splash contacts per step below this are resting-liquid flicker (rest p95 3, plus 1). */
export const SPLASH_MIN_CONTACTS_PER_STEP = 4;
/** New splash contacts per step at full intensity (p90 of audible steps, 29). */
export const SPLASH_FULL_CONTACTS_PER_STEP = 29;
/** Settled water stays silent even if its contacts flicker. */
export const SPLASH_MIN_SPEED_MPS = 0.3;
/** Impulse at full impact strength (N·s): the Float Up solution's impulse p90. */
export const IMPACT_FULL_IMPULSE = 0.062008;
/** Softest audible impact, so a counted impact is never inaudible. */
export const IMPACT_MIN_STRENGTH = 0.2;

const SEMITONES_PER_OCTAVE = 12;
const LAST_PLINK_STEP = PENTATONIC_SEMITONES.length - 1;
const MAJOR_TRIAD_SEMITONES = [0, 4, 7] as const;
const CHIME_NOTE_SPACING_SECONDS = 0.11;
const CHIME_NOTE_SECONDS = 0.7;
const CHIME_NOTE_GAIN = 0.2;

/** Clamps to 0..1; NaN becomes 0 so bad input is silent rather than loud. */
export function clampUnit(value: number): number {
  if (!(value > 0)) {
    return 0;
  }

  return Math.min(1, value);
}

function semitonesAbove(rootHz: number, semitones: number): number {
  return rootHz * 2 ** (semitones / SEMITONES_PER_OCTAVE);
}

/** Pentatonic scale step 0..10 for the cup fill `goalCount / goalTarget` (D-06). */
export function plinkStep(goalCount: number, goalTarget: number): number {
  const fill = goalTarget > 0 ? clampUnit(goalCount / goalTarget) : 0;
  return Math.round(fill * LAST_PLINK_STEP);
}

/** Plink pitch: rises along the pentatonic scale as the cup fills (D-06). */
export function plinkFrequency(goalCount: number, goalTarget: number): number {
  const semitones = PENTATONIC_SEMITONES[plinkStep(goalCount, goalTarget)] ?? 0;
  return semitonesAbove(PLINK_ROOT_HZ, semitones);
}

/**
 * Splash intensity in (0, 1], or undefined when the frame should be silent.
 * Contacts are averaged per folded step so fast-forward is not louder (D-09).
 */
export function splashIntensity(
  signals: PresentedSoundSignals,
  maxSpeed: number,
): number | undefined {
  if (signals.steps === 0) {
    return undefined;
  }

  const perStep = signals.splashContacts / signals.steps;
  if (perStep < SPLASH_MIN_CONTACTS_PER_STEP || !(maxSpeed >= SPLASH_MIN_SPEED_MPS)) {
    return undefined;
  }

  const span = SPLASH_FULL_CONTACTS_PER_STEP - SPLASH_MIN_CONTACTS_PER_STEP + 1;
  return clampUnit((perStep - SPLASH_MIN_CONTACTS_PER_STEP + 1) / span);
}

/** Rigid impact strength in [IMPACT_MIN_STRENGTH, 1], or undefined without an impact. */
export function impactStrength(signals: PresentedSoundSignals): number | undefined {
  if (signals.impactCount === 0) {
    return undefined;
  }

  return Math.max(IMPACT_MIN_STRENGTH, clampUnit(signals.impactImpulse / IMPACT_FULL_IMPULSE));
}

/** Bandpassed noise burst parameters. */
export type SplashParams = {
  readonly gain: number;
  readonly filterHz: number;
  readonly q: number;
  readonly attackSeconds: number;
  readonly durationSeconds: number;
};

/** Louder and brighter with intensity. */
export function splashParams(intensity: number): SplashParams {
  const amount = clampUnit(intensity);
  return {
    gain: 0.06 + 0.24 * amount,
    filterHz: 700 + 1800 * amount,
    q: 0.9,
    attackSeconds: 0.005,
    durationSeconds: 0.18,
  };
}

/** Triangle thump with a falling pitch. */
export type ImpactParams = {
  readonly gain: number;
  readonly startHz: number;
  readonly endHz: number;
  readonly durationSeconds: number;
};

/** Louder and slightly higher with strength; the pitch always falls. */
export function impactParams(strength: number): ImpactParams {
  const amount = clampUnit(strength);
  return {
    gain: 0.08 + 0.32 * amount,
    startHz: 160 + 60 * amount,
    endHz: 60,
    durationSeconds: 0.16,
  };
}

/** Short sine blip. */
export type PlinkParams = {
  readonly gain: number;
  readonly frequency: number;
  readonly durationSeconds: number;
};

/** Fixed gain and length; only the pitch carries the fill. */
export function plinkParams(frequency: number): PlinkParams {
  return { gain: 0.18, frequency, durationSeconds: 0.12 };
}

/** One sine partial of the win chime, offset from the chime start. */
export type ChimeNote = {
  readonly frequency: number;
  readonly offsetSeconds: number;
  readonly durationSeconds: number;
  readonly gain: number;
};

/** The three-note major arpeggio played once on a win (D-06). */
export function chimeNotes(): readonly ChimeNote[] {
  return MAJOR_TRIAD_SEMITONES.map((semitones, index) => ({
    frequency: semitonesAbove(CHIME_ROOT_HZ, semitones),
    offsetSeconds: index * CHIME_NOTE_SPACING_SECONDS,
    durationSeconds: CHIME_NOTE_SECONDS,
    gain: CHIME_NOTE_GAIN,
  }));
}

function chimeDurationSeconds(): number {
  return chimeNotes().reduce(
    (end, note) => Math.max(end, note.offsetSeconds + note.durationSeconds),
    0,
  );
}

/** How long a requested voice sounds, so the pool can prune it by time. */
export function voiceDurationSeconds(request: VoiceRequest): number {
  switch (request.kind) {
    case "splash":
      return splashParams(request.intensity).durationSeconds;
    case "impact":
      return impactParams(request.strength).durationSeconds;
    case "plink":
      return plinkParams(request.frequency).durationSeconds;
    case "chime":
      return chimeDurationSeconds();
  }
}
