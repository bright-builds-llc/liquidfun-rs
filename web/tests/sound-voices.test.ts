import { describe, expect, it } from "vitest";

import { NO_SOUND_SIGNALS, type PresentedSoundSignals } from "../src/audio/sound-signals";
import {
  CHIME_ROOT_HZ,
  IMPACT_FULL_IMPULSE,
  IMPACT_MIN_STRENGTH,
  PLINK_ROOT_HZ,
  SPLASH_FULL_CONTACTS_PER_STEP,
  SPLASH_MIN_CONTACTS_PER_STEP,
  SPLASH_MIN_SPEED_MPS,
  chimeNotes,
  clampUnit,
  impactParams,
  impactStrength,
  plinkFrequency,
  plinkParams,
  plinkStep,
  splashIntensity,
  splashParams,
  voiceDurationSeconds,
} from "../src/audio/voices";

const FAST_MPS = 2;

function splashSignals(steps: number, splashContacts: number): PresentedSoundSignals {
  return { ...NO_SOUND_SIGNALS, steps, splashContacts };
}

function impactSignals(impactCount: number, impactImpulse: number): PresentedSoundSignals {
  return { ...NO_SOUND_SIGNALS, steps: 1, impactCount, impactImpulse };
}

describe("clampUnit", () => {
  it("clamps to 0..1 and maps NaN to 0", () => {
    // Act
    const clamped = [-1, 0.5, 2, Number.NaN].map(clampUnit);

    // Assert
    expect(clamped).toEqual([0, 0.5, 1, 0]);
  });
});

describe("plinkStep", () => {
  it("maps an empty cup to the first scale step", () => {
    expect(plinkStep(0, 100)).toBe(0);
  });

  it("maps a full target to the last scale step", () => {
    expect(plinkStep(100, 100)).toBe(10);
  });

  it("clamps an overfilled cup to the last scale step", () => {
    expect(plinkStep(250, 100)).toBe(10);
  });

  it("maps a level without a target to the first scale step", () => {
    expect(plinkStep(5, 0)).toBe(0);
  });

  it("maps a NaN count to the first scale step", () => {
    expect(plinkStep(Number.NaN, 100)).toBe(0);
  });
});

describe("plinkFrequency", () => {
  it("starts at the C5 root", () => {
    expect(plinkFrequency(0, 100)).toBe(PLINK_ROOT_HZ);
  });

  it("ends two octaves above the root", () => {
    expect(plinkFrequency(100, 100)).toBeCloseTo(523.25 * 4, 9);
  });

  it("reaches one octave at half fill", () => {
    expect(plinkFrequency(50, 100)).toBeCloseTo(523.25 * 2 ** (12 / 12), 9);
  });

  it("never falls as the cup fills", () => {
    // Arrange
    const counts = Array.from({ length: 101 }, (_, count) => count);

    // Act
    const frequencies = counts.map((count) => plinkFrequency(count, 100));

    // Assert
    const falls = frequencies.filter((frequency, index) => index > 0 && frequency < (frequencies[index - 1] ?? 0));
    expect(falls).toEqual([]);
  });
});

describe("chimeNotes", () => {
  it("plays a major arpeggio from the chime root", () => {
    // Act
    const frequencies = chimeNotes().map((note) => note.frequency);

    // Assert
    const expected = [1, 2 ** (4 / 12), 2 ** (7 / 12)].map((ratio) => CHIME_ROOT_HZ * ratio);
    expect(frequencies).toHaveLength(3);
    frequencies.forEach((frequency, index) => expect(frequency).toBeCloseTo(expected[index] ?? 0, 9));
  });

  it("starts at offset 0 with strictly increasing offsets", () => {
    // Act
    const offsets = chimeNotes().map((note) => note.offsetSeconds);

    // Assert
    expect(offsets[0]).toBe(0);
    expect(offsets.every((offset, index) => index === 0 || offset > (offsets[index - 1] ?? 0))).toBe(true);
  });

  it("gives every note a positive duration and gain", () => {
    // Act
    const notes = chimeNotes();

    // Assert
    expect(notes.every((note) => note.durationSeconds > 0 && note.gain > 0)).toBe(true);
  });
});

describe("splashIntensity", () => {
  it("is silent below the per-step contact minimum", () => {
    expect(splashIntensity(splashSignals(1, SPLASH_MIN_CONTACTS_PER_STEP - 1), FAST_MPS)).toBeUndefined();
  });

  it("is audible above zero at the per-step contact minimum", () => {
    // Act
    const maybeIntensity = splashIntensity(splashSignals(1, SPLASH_MIN_CONTACTS_PER_STEP), FAST_MPS);

    // Assert
    expect(maybeIntensity).toBeGreaterThan(0);
    expect(maybeIntensity).toBeLessThanOrEqual(1);
  });

  it("is full at and above the full contact count", () => {
    // Act
    const atFull = splashIntensity(splashSignals(1, SPLASH_FULL_CONTACTS_PER_STEP), FAST_MPS);
    const aboveFull = splashIntensity(splashSignals(1, SPLASH_FULL_CONTACTS_PER_STEP * 3), FAST_MPS);

    // Assert
    expect([atFull, aboveFull]).toEqual([1, 1]);
  });

  it("is silent when the liquid is slower than the minimum speed", () => {
    // Arrange
    const signals = splashSignals(1, SPLASH_FULL_CONTACTS_PER_STEP);

    // Act
    const maybeIntensity = splashIntensity(signals, SPLASH_MIN_SPEED_MPS - 0.01);

    // Assert
    expect(maybeIntensity).toBeUndefined();
  });

  it("is silent when no step was folded", () => {
    expect(splashIntensity(splashSignals(0, 50), FAST_MPS)).toBeUndefined();
  });

  it("normalizes by folded steps so fast-forward is not louder", () => {
    // Arrange
    const contacts = SPLASH_MIN_CONTACTS_PER_STEP + 5;

    // Act
    const fourSteps = splashIntensity(splashSignals(4, 4 * contacts), FAST_MPS);
    const oneStep = splashIntensity(splashSignals(1, contacts), FAST_MPS);

    // Assert
    expect(fourSteps).toBe(oneStep);
  });
});

describe("impactStrength", () => {
  it("is silent without a counted impact", () => {
    expect(impactStrength(impactSignals(0, 0))).toBeUndefined();
  });

  it("scales by the full impulse with a floor and a cap", () => {
    // Arrange
    const impulses = [0, IMPACT_FULL_IMPULSE / 2, IMPACT_FULL_IMPULSE * 5];

    // Act
    const strengths = impulses.map((impulse) => impactStrength(impactSignals(1, impulse)));

    // Assert
    const expected = impulses.map((impulse) =>
      Math.max(IMPACT_MIN_STRENGTH, Math.min(1, impulse / IMPACT_FULL_IMPULSE)),
    );
    expect(strengths).toEqual(expected);
  });
});

describe("voice parameters", () => {
  it("clamps splash and impact inputs to 0..1", () => {
    // Act
    const clamped = [
      splashParams(Number.NaN).gain === splashParams(0).gain,
      splashParams(5).gain === splashParams(1).gain,
      impactParams(Number.NaN).gain === impactParams(0).gain,
      impactParams(-3).gain === impactParams(0).gain,
    ];

    // Assert
    expect(clamped).toEqual([true, true, true, true]);
  });

  it("raises splash gain and filter frequency with intensity", () => {
    // Act
    const quiet = splashParams(0);
    const loud = splashParams(1);

    // Assert
    expect(loud.gain).toBeGreaterThan(quiet.gain);
    expect(loud.filterHz).toBeGreaterThan(quiet.filterHz);
  });

  it("raises impact gain with strength and drops its pitch", () => {
    // Act
    const soft = impactParams(0.2);
    const hard = impactParams(0.9);

    // Assert
    expect(hard.gain).toBeGreaterThan(soft.gain);
    expect(hard.startHz).toBeGreaterThan(hard.endHz);
  });

  it("keeps the plink frequency it is given", () => {
    expect(plinkParams(880).frequency).toBe(880);
  });

  it("lasts the chime until its last note ends", () => {
    // Arrange
    const notes = chimeNotes();
    const last = notes[notes.length - 1];

    // Act
    const duration = voiceDurationSeconds({ kind: "chime" });

    // Assert
    expect(duration).toBeCloseTo((last?.offsetSeconds ?? 0) + (last?.durationSeconds ?? 0), 9);
  });

  it("gives each one-shot voice its parameter duration", () => {
    // Act
    const durations = [
      voiceDurationSeconds({ kind: "splash", intensity: 0.5 }),
      voiceDurationSeconds({ kind: "impact", strength: 0.5 }),
      voiceDurationSeconds({ kind: "plink", frequency: 600 }),
    ];

    // Assert
    expect(durations).toEqual([
      splashParams(0.5).durationSeconds,
      impactParams(0.5).durationSeconds,
      plinkParams(600).durationSeconds,
    ]);
  });
});
