import { describe, expect, it } from "vitest";

import { NO_SOUND_SIGNALS, foldSoundSignals } from "../src/audio/sound-signals";

describe("foldSoundSignals", () => {
  it("folds one step into the empty signals", () => {
    // Act
    const folded = foldSoundSignals(NO_SOUND_SIGNALS, {
      splashContacts: 3,
      impactCount: 1,
      impactImpulse: 0.2,
    });

    // Assert
    expect(folded).toEqual({ steps: 1, splashContacts: 3, impactCount: 1, impactImpulse: 0.2 });
  });

  it("sums counts and keeps the largest impulse over two steps", () => {
    // Arrange
    const first = foldSoundSignals(NO_SOUND_SIGNALS, {
      splashContacts: 3,
      impactCount: 1,
      impactImpulse: 0.2,
    });

    // Act
    const folded = foldSoundSignals(first, {
      splashContacts: 5,
      impactCount: 2,
      impactImpulse: 0.1,
    });

    // Assert
    expect(folded).toEqual({ steps: 2, splashContacts: 8, impactCount: 3, impactImpulse: 0.2 });
  });

  it("counts a step without sound data as a silent step", () => {
    // Act
    const folded = foldSoundSignals(NO_SOUND_SIGNALS, undefined);

    // Assert
    expect(folded).toEqual({ steps: 1, splashContacts: 0, impactCount: 0, impactImpulse: 0 });
  });

  it("does not change the empty signals constant", () => {
    // Act
    foldSoundSignals(NO_SOUND_SIGNALS, { splashContacts: 1, impactCount: 0, impactImpulse: 0 });

    // Assert
    expect(NO_SOUND_SIGNALS).toEqual({ steps: 0, splashContacts: 0, impactCount: 0, impactImpulse: 0 });
  });
});
