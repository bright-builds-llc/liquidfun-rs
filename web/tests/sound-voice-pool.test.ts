import { describe, expect, it } from "vitest";

import {
  CHIME_VOICE_COUNT,
  EMPTY_VOICE_POOL,
  MAX_VOICES,
  admitVoice,
  pruneVoices,
  type VoicePool,
} from "../src/audio/voice-pool";

const LONG_SECONDS = 10;

function fullPlinkPool(): VoicePool {
  let pool = EMPTY_VOICE_POOL;
  for (let index = 0; index < MAX_VOICES; index += 1) {
    pool = admitVoice(pool, "plink", index * 0.05, LONG_SECONDS).pool;
  }
  return pool;
}

describe("admitVoice cap", () => {
  it("fills the pool with six sounding voices", () => {
    // Act
    const pool = fullPlinkPool();

    // Assert
    expect(pool.voices).toHaveLength(MAX_VOICES);
  });

  it("rejects a seventh non-chime voice without evicting", () => {
    // Arrange
    const pool = fullPlinkPool();

    // Act
    const admission = admitVoice(pool, "splash", 1, 0.2);

    // Assert
    expect(admission.admitted).toEqual([]);
    expect(admission.evicted).toEqual([]);
    expect(admission.pool.voices).toEqual(pool.voices);
  });
});

describe("admitVoice minimum intervals", () => {
  it.each([
    ["splash", 0.08],
    ["impact", 0.06],
    ["plink", 0.05],
  ] as const)("rejects a %s started just inside its interval", (kind, interval) => {
    // Arrange
    const first = admitVoice(EMPTY_VOICE_POOL, kind, 0, 0.01).pool;

    // Act
    const admission = admitVoice(first, kind, interval - 0.001, 0.01);

    // Assert
    expect(admission.admitted).toEqual([]);
  });

  it.each([
    ["splash", 0.08],
    ["impact", 0.06],
    ["plink", 0.05],
  ] as const)("admits a %s started at its interval", (kind, interval) => {
    // Arrange
    const first = admitVoice(EMPTY_VOICE_POOL, kind, 0, 0.01).pool;

    // Act
    const admission = admitVoice(first, kind, interval, 0.01);

    // Assert
    expect(admission.admitted).toHaveLength(1);
  });

  it("limits each kind separately", () => {
    // Arrange
    const afterSplash = admitVoice(EMPTY_VOICE_POOL, "splash", 0, 0.2).pool;

    // Act
    const admission = admitVoice(afterSplash, "plink", 0.001, 0.1);

    // Assert
    expect(admission.admitted.map((voice) => voice.kind)).toEqual(["plink"]);
  });

  it("does not move the last start time when a request is rejected", () => {
    // Arrange
    const first = admitVoice(EMPTY_VOICE_POOL, "splash", 0, 0.01).pool;
    const rejected = admitVoice(first, "splash", 0.05, 0.01).pool;

    // Act
    const admission = admitVoice(rejected, "splash", 0.08, 0.01);

    // Assert
    expect(rejected.lastStartedAt.splash).toBe(0);
    expect(admission.admitted).toHaveLength(1);
  });
});

describe("pruneVoices", () => {
  it("drops voices that have ended", () => {
    // Arrange
    const pool = admitVoice(EMPTY_VOICE_POOL, "plink", 0, 0.12).pool;

    // Act
    const pruned = pruneVoices(pool, 0.12);

    // Assert
    expect(pruned.voices).toEqual([]);
  });

  it("lets a full pool admit again once its voices end", () => {
    // Arrange
    const pool = fullPlinkPool();

    // Act
    const admission = admitVoice(pool, "splash", LONG_SECONDS + 1, 0.2);

    // Assert
    expect(admission.admitted).toHaveLength(1);
    expect(admission.pool.voices).toHaveLength(1);
  });
});

describe("admitVoice chime", () => {
  it("evicts exactly the three oldest voices from a full pool", () => {
    // Arrange
    const pool = fullPlinkPool();
    const oldestIds = pool.voices.slice(0, CHIME_VOICE_COUNT).map((voice) => voice.id);

    // Act
    const admission = admitVoice(pool, "chime", 1, 0.92);

    // Assert
    expect(admission.evicted.map((voice) => voice.id)).toEqual(oldestIds);
    expect(admission.admitted).toHaveLength(CHIME_VOICE_COUNT);
    expect(admission.pool.voices).toHaveLength(MAX_VOICES);
  });

  it("breaks start-time ties by the lower id", () => {
    // Arrange
    const voiceAt = (id: number, startedAt: number) =>
      ({ id, kind: "splash", startedAt, endsAt: LONG_SECONDS }) as const;
    const pool: VoicePool = {
      voices: [voiceAt(5, 0), voiceAt(2, 0), voiceAt(7, 0.5), voiceAt(1, 0), voiceAt(3, 0.5), voiceAt(4, 0.5)],
      nextId: 8,
      lastStartedAt: { splash: 0.5 },
    };

    // Act
    const admission = admitVoice(pool, "chime", 1, 0.92);

    // Assert
    expect(admission.evicted.map((voice) => voice.id)).toEqual([1, 2, 5]);
  });

  it("ignores the minimum interval", () => {
    // Arrange
    const afterChime = admitVoice(EMPTY_VOICE_POOL, "chime", 0, 0.92).pool;

    // Act
    const admission = admitVoice(afterChime, "chime", 0.001, 0.92);

    // Assert
    expect(admission.admitted).toHaveLength(CHIME_VOICE_COUNT);
  });

  it("evicts nothing when the pool has room", () => {
    // Arrange
    const pool = admitVoice(EMPTY_VOICE_POOL, "splash", 0, LONG_SECONDS).pool;

    // Act
    const admission = admitVoice(pool, "chime", 0.5, 0.92);

    // Assert
    expect(admission.evicted).toEqual([]);
    expect(admission.pool.voices).toHaveLength(1 + CHIME_VOICE_COUNT);
  });
});
