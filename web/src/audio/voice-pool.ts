/**
 * Voice cap and per-kind rate limits (Phase 42 D-09). Voices are pruned by
 * their scheduled end time rather than by end events, because those events
 * do not fire while the audio clock is suspended (Research Pattern 6).
 */

import type { VoiceKind } from "./voices";

/** Most voices that may sound at once. */
export const MAX_VOICES = 6;
/** Shortest gap between two starts of the same kind (audio clock seconds). */
export const MIN_INTERVAL_SECONDS: Readonly<
  Record<Exclude<VoiceKind, "chime">, number>
> = { splash: 0.08, impact: 0.06, plink: 0.05 };
/** The win chime takes one pool entry per note. */
export const CHIME_VOICE_COUNT = 3;

const INTERVAL_TOLERANCE_SECONDS = 1e-9;

/** A started voice and its scheduled lifetime. */
export type ActiveVoice = {
  readonly id: number;
  readonly kind: VoiceKind;
  readonly startedAt: number;
  readonly endsAt: number;
};

/** Immutable pool state threaded through the shell. */
export type VoicePool = {
  readonly voices: readonly ActiveVoice[];
  readonly nextId: number;
  readonly lastStartedAt: Readonly<Partial<Record<VoiceKind, number>>>;
};

export const EMPTY_VOICE_POOL: VoicePool = { voices: [], nextId: 0, lastStartedAt: {} };

/** The pool after a request, the voices to start and the voices to stop. */
export type Admission = {
  readonly pool: VoicePool;
  readonly admitted: readonly ActiveVoice[];
  readonly evicted: readonly ActiveVoice[];
};

/** Drops voices whose scheduled end is at or before `nowSeconds`. */
export function pruneVoices(pool: VoicePool, nowSeconds: number): VoicePool {
  const voices = pool.voices.filter((voice) => voice.endsAt > nowSeconds);
  return voices.length === pool.voices.length ? pool : { ...pool, voices };
}

function tooSoon(pool: VoicePool, kind: Exclude<VoiceKind, "chime">, nowSeconds: number): boolean {
  const maybeLast = pool.lastStartedAt[kind];
  if (maybeLast === undefined) {
    return false;
  }

  return nowSeconds - maybeLast < MIN_INTERVAL_SECONDS[kind] - INTERVAL_TOLERANCE_SECONDS;
}

function startVoices(
  pool: VoicePool,
  kind: VoiceKind,
  count: number,
  nowSeconds: number,
  durationSeconds: number,
): { readonly pool: VoicePool; readonly admitted: readonly ActiveVoice[] } {
  const admitted = Array.from({ length: count }, (_, index) => ({
    id: pool.nextId + index,
    kind,
    startedAt: nowSeconds,
    endsAt: nowSeconds + durationSeconds,
  }));
  return {
    pool: {
      voices: [...pool.voices, ...admitted],
      nextId: pool.nextId + count,
      lastStartedAt: { ...pool.lastStartedAt, [kind]: nowSeconds },
    },
    admitted,
  };
}

function oldestFirst(left: ActiveVoice, right: ActiveVoice): number {
  return left.startedAt - right.startedAt || left.id - right.id;
}

function admitChime(pool: VoicePool, nowSeconds: number, durationSeconds: number): Admission {
  const overflow = Math.max(0, pool.voices.length + CHIME_VOICE_COUNT - MAX_VOICES);
  const evicted = [...pool.voices].sort(oldestFirst).slice(0, overflow);
  const evictedIds = new Set(evicted.map((voice) => voice.id));
  const kept = { ...pool, voices: pool.voices.filter((voice) => !evictedIds.has(voice.id)) };
  return { ...startVoices(kept, "chime", CHIME_VOICE_COUNT, nowSeconds, durationSeconds), evicted };
}

/**
 * Admits one voice request. The chime always plays and evicts the oldest
 * voices to fit; other kinds are dropped when the pool is full or when they
 * start inside their kind's minimum interval. A dropped request leaves the
 * kind's last start time unchanged.
 */
export function admitVoice(
  pool: VoicePool,
  kind: VoiceKind,
  nowSeconds: number,
  durationSeconds: number,
): Admission {
  const live = pruneVoices(pool, nowSeconds);
  if (kind === "chime") {
    return admitChime(live, nowSeconds, durationSeconds);
  }

  if (live.voices.length >= MAX_VOICES || tooSoon(live, kind, nowSeconds)) {
    return { pool: live, admitted: [], evicted: [] };
  }

  return { ...startVoices(live, kind, 1, nowSeconds, durationSeconds), evicted: [] };
}
