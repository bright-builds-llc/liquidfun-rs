---
phase: 42-opt-in-sound
plan: "02"
subsystem: web-audio-core
tags: [typescript, sound, puzzle, pure-core, vitest, tdd]
requires:
  - "42-01 RenderFrame.maybeSound (splashContacts, impactCount, impactImpulse) and target/phase42/sound-probe.txt"
provides:
  - "web/src/player/sound-preference.ts: SOUND_STORAGE_KEY, loadSoundOn, persistSoundOn, createSoundPreferenceMemory, browserSoundStorage"
  - "web/src/audio/sound-signals.ts: PresentedSoundSignals, NO_SOUND_SIGNALS, foldSoundSignals"
  - "web/src/audio/voices.ts: VoiceRequest, plinkStep/plinkFrequency, splashIntensity, impactStrength, splash/impact/plink params, chimeNotes, voiceDurationSeconds, probe-tuned thresholds"
  - "web/src/audio/voice-pool.ts: MAX_VOICES 6, MIN_INTERVAL_SECONDS, CHIME_VOICE_COUNT 3, pruneVoices, admitVoice"
  - "web/src/audio/triggers.ts: TriggerMemory, INITIAL_TRIGGER_MEMORY, BURST_QUIET_SECONDS, decideVoices, trackWithoutClock"
affects:
  - "42-03 sound engine shell and puzzle-sound glue consume decideVoices/admitVoice/voice params and create the browser preference instance"
  - "42-05 phase gate greps web/src/audio and sound-preference.ts"
tech-stack:
  added: []
  patterns:
    - "Pure audio core: time passed in as nowSeconds (audio clock), no DOM or Web Audio"
    - "Immutable VoicePool threaded through admitVoice; time-based pruning by scheduled endsAt"
    - "One shared observe() helper for decideVoices and trackWithoutClock resets"
key-files:
  created:
    - web/src/player/sound-preference.ts
    - web/src/audio/sound-signals.ts
    - web/src/audio/voices.ts
    - web/src/audio/voice-pool.ts
    - web/src/audio/triggers.ts
    - web/tests/sound-preference.test.ts
    - web/tests/sound-signals.test.ts
    - web/tests/sound-voices.test.ts
    - web/tests/sound-voice-pool.test.ts
    - web/tests/sound-triggers.test.ts
  modified: []
decisions:
  - "42-02: splash thresholds from the probe: REST p95 3 (n 360, last 60 steps of 6 empty fixtures) -> SPLASH_MIN_CONTACTS_PER_STEP 4; p90 of 5,036 steps >= 4 is 29 -> SPLASH_FULL_CONTACTS_PER_STEP 29; IMPACT_FULL_IMPULSE 0.062008 (float-up-solution impulse_p90); first-pour solution audible on 302/321 steps, so no lowering"
  - "42-02: chime takes 3 pool entries (one per note) and evicts the oldest by startedAt then id; non-chime requests are dropped when full or inside their 80/60/50 ms interval, and a drop leaves lastStartedAt unchanged"
  - "42-02: a burst onset and a rigid impact merge into one impact request at the larger strength; maybeLastSplashAt records the requested splash time, not pool admission"
  - "42-02: a non-increasing stepIndex resets memory and then records the frame (step, goal count, win), in both decideVoices and trackWithoutClock"
metrics:
  duration: 6min
  completed: 2026-10-13
  tasks: 3
  files: 10
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: "2026-10-13T19:29:16.576Z"
---

# Phase 42 Plan 02: Pure sound core Summary

The sound decisions are now pure, unit-tested TypeScript: an exact-token `liquidfun.puzzle-sound.v1` preference, a per-presented-frame signal fold, a C5 pentatonic fill-to-pitch mapping with probe-tuned splash and impact thresholds, a 6-voice pool with per-kind rate limits and chime stealing, and trigger rules that play only during a run, chime once and fall silent after a win. None of it touches the DOM, Web Audio or a wall clock.

## Performance

- **Duration:** about 6 min
- **Started:** 2026-10-13T19:23:47Z
- **Completed:** 2026-10-13T19:29Z
- **Tasks:** 3 (6 commits, RED then GREEN per task)
- **Files:** 10 created

## Accomplishments

- **Task 1:** `sound-preference.ts` copies the first-hint shape. Only `"on"` reads as on; `set` updates memory before it persists, and creating the memory does not read storage. `foldSoundSignals` sums the counts, keeps the largest impulse, counts steps and treats `undefined` as a silent step.
- **Task 2:** `voices.ts` holds the pentatonic scale, the E5 major chime (offsets 0, 0.11 and 0.22 s; 0.7 s notes; gain 0.2), splash intensity averaged per folded step, impact strength with a 0.2 floor, and the synthesis parameters from the plan. `voice-pool.ts` holds the cap, the intervals with a 1e-9 tolerance, pruning by time, and chime eviction.
- **Task 3:** `triggers.ts` applies rules (a) through (g) through one `observe()` helper. Run voices are ordered impact, splash, plink, and `BURST_QUIET_SECONDS` is 0.4.

## Tuning data (from `target/phase42/sound-probe.txt`)

The throwaway scratchpad script (not committed) uses the same nearest-rank percentiles as the Plan 01 probe.

| Input | Value |
| --- | --- |
| REST: splash in the last 60 printed steps of the 6 `-empty.txt` sections | n 360, p95 **3**, max 10 (float-up-empty tail p95 5, others 1-2) |
| `SPLASH_MIN_CONTACTS_PER_STEP = max(2, p95 + 1)` | **4** |
| p90 of all splash values >= 4 across all 14 fixtures | n 5,036, p90 **29** |
| `SPLASH_FULL_CONTACTS_PER_STEP = max(4 + 8, 29)` | **29** |
| `SPLASH_MIN_SPEED_MPS` | 0.3 (discretion) |
| `IMPACT_FULL_IMPULSE` = `## float-up-solution.txt` impulse_p90 | **0.062008** N·s |
| `IMPACT_MIN_STRENGTH` | 0.2 |
| First-pour solution check | 302 of 321 steps reach 4 contacts at >= 0.3 m/s, so the stream is audible and the minimum stays 4 |

The speed gate barely affects the empty-fixture tails. Their `max_speed` stays near 9.2-9.6 m/s because lost particles keep falling, so the contact minimum is what quiets them. With both gates, 0 of the last 60 steps are audible in five of the six empty fixtures (float-up-empty: 14 of 60, grand-finale-empty: 1 of 60). Some solution tails still splash while the cup fills (for example first-pour-solution, 60 of 60). The pool's 80 ms splash interval limits how often those play.

Synthesis constants: splash gain `0.06 + 0.24 i`, filter `700 + 1800 i` Hz, Q 0.9, attack 0.005 s, length 0.18 s. Impact gain `0.08 + 0.32 s`, pitch from `160 + 60 s` Hz down to 60 Hz, length 0.16 s, triangle wave. Plink gain 0.18, length 0.12 s. Chime root 659.25 Hz with triad semitones [0, 4, 7]. Plink root 523.25 Hz.

## Verification

- `bunx vitest run` on the five new files: preference 20, signals 4, voices 27, pool 16, triggers 24 (91 total). Every plan threshold is met (>= 12, >= 18, >= 14).
- `bun run typecheck` passes. `bun run test:unit`: 110 files, 1656 tests passed (before this plan: 105 and 1565).
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- Grep gates: `Date.now|performance.now|setTimeout|setInterval|innerHTML|AudioContext|window` finds nothing in `web/src/audio/*.ts`. `sound-preference.ts` contains exactly one `window.localStorage`, in `browserSoundStorage`.

## Task Commits

1. **Task 1 RED:** `752425195` (test). **GREEN:** `b48774442` (feat)
1. **Task 2 RED:** `bc50e5f99` (test). **GREEN:** `2dc57c6f3` (feat)
1. **Task 3 RED:** `b5d086aa6` (test). **GREEN:** `fe6f35cc3` (feat)

## Deviations from Plan

### Interpretations

**1. Reset on a non-increasing step still records the frame**

- The plan says `trackWithoutClock` "returns `INITIAL_TRIGGER_MEMORY`" for resets. For `building` and `loading` it returns exactly that. When `stepIndex` does not increase, both entry points reset first and then apply rule (g): they record the frame's step, goal count and win. This keeps `decideVoices` and `trackWithoutClock` on one shared helper, as the plan requires. The test asserts `{ ...INITIAL_TRIGGER_MEMORY, maybeLastStepIndex: 40 }` for that case.
- `puzzle-loop.ts` publishes only after at least one step has run, so a non-increasing `stepIndex` really does mark a new run.

**2. Chime pool duration**

- `voiceDurationSeconds({ kind: "chime" })` is 0.92 s (0.22 + 0.7). The pool gives all three chime entries that end time. Plan 03 may schedule each note's own end. Pruning by the chime's last end only holds capacity a little longer.

No Rule 1-4 fixes were needed.

## Known Stubs

None. `browserSoundStorage` is exported but not instantiated here, as the plan intends; Plan 03 creates the browser instance.

## Threat Flags

None. T-42-05 is covered by the exact `=== "on"` comparison and its tests. T-42-06 is covered by the 6-voice cap, the per-kind intervals, at most one plink and one impact per frame, and pruning by time, all tested.

## Next Phase Readiness

- Plan 03 can call `decideVoices` from the publish path (folding `maybeSound` per step in `presentSteps`). It feeds each request through `admitVoice(pool, request.kind, currentTime, voiceDurationSeconds(request))`, stops the `evicted` voices and starts the `admitted` ones. Before a context exists, it calls `trackWithoutClock`.

## Self-Check: PASSED

- All 10 created files exist on disk.
- Commits `752425195`, `b48774442`, `bc50e5f99`, `2dc57c6f3`, `b5d086aa6` and `fe6f35cc3` are in `git log`.
