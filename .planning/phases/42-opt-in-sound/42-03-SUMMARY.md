---
phase: 42-opt-in-sound
plan: "03"
subsystem: web-audio-shell
tags: [typescript, sound, puzzle, web-audio, solid, vitest, tdd]
requires:
  - "42-02 pure core: decideVoices/trackWithoutClock, admitVoice, voice params, foldSoundSignals, sound-preference memory"
  - "42-01 RenderFrame.maybeSound per fixed step"
provides:
  - "web/src/audio/sound-engine.ts: createSoundEngine, browserContextFactory, SoundContext, SoundContextFactory, SoundEngine, SoundEngineOptions"
  - "web/src/player/puzzle-sound.ts: createPuzzleSound, createDisabledPuzzleSound, createBrowserPuzzleSound, PuzzleSound, PuzzleSoundDeps, SoundEventTarget, SOUND_CONTROL_SELECTOR"
  - "PuzzleLoopDeps.publish 4th argument: PresentedSoundSignals folded over every fixed step"
  - "PuzzleRuntime.sound; PuzzleController soundEnabled/soundAvailable/soundFailed/setSoundEnabled"
  - "web/tests/sound-fake-context.ts: recording fake audio context shared by sound tests"
affects:
  - "42-04 HUD toggle and drawer section read the controller's sound accessors and call setSoundEnabled from their handlers"
  - "42-05 phase gate greps sound-engine.ts and puzzle-sound.ts and runs the fake-AudioContext e2e"
tech-stack:
  added: []
  patterns:
    - "Factory-injected Web Audio shell; the only file naming the browser constructors"
    - "One page-session context: stop-before-suspend, never closed, resume in-flight guard"
    - "Guarded glue: every engine call routes errors to one disableAfterFailure (log, off in memory, failed)"
key-files:
  created:
    - web/src/audio/sound-engine.ts
    - web/src/player/puzzle-sound.ts
    - web/tests/sound-engine.test.ts
    - web/tests/sound-fake-context.ts
    - web/tests/puzzle-sound.test.ts
    - web/tests/puzzle-sound-wiring.test.ts
  modified:
    - web/src/player/puzzle-loop.ts
    - web/src/player/puzzle-runtime.ts
    - web/src/player/puzzle-host.ts
    - web/src/player/puzzle-run.ts
    - web/src/player/puzzle-lifecycle.ts
    - web/src/puzzle/controller.ts
    - web/src/player/playground-session.ts
    - web/tests/puzzle-loop.test.ts
    - web/tests/puzzle-host-fixture.ts
decisions:
  - "42-03: pointerup counts as activation only for non-mouse pointers (touch, pen), matching the HTML activation-triggering list; mouse unlocks on pointerdown"
  - "42-03: engine suspend() clears the resume in-flight guard so a resume left pending before a suspend never blocks the next resume"
  - "42-03: construction failure is logged once through the glue's logError (engine option maybeLogError, default console.error) and makes sound unavailable for the session; runtime failures set failed and suspend once"
  - "42-03: each chime note takes its own pool entry, oscillator and gain; non-chime voices map one entry to one source"
metrics:
  duration: 8min
  completed: 2026-10-13
  tasks: 3
  files: 15
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: "2026-10-13T19:39:53.515Z"
---

# Phase 42 Plan 03: Web Audio shell and puzzle sound wiring Summary

Puzzle sound now works end to end behind the controller. `sound-engine.ts` is a factory-injected Web Audio shell. It creates one page-session context lazily, routes it through a 0.5 master gain into a compressor, and synthesizes the four voices behind the Plan 02 pool. `puzzle-sound.ts` holds the shared on, available and failed state. It also handles the first-gesture unlock, visibility and route lifecycle, and calls `present` once per presented frame. The loop folds every fixed step's sound signals into `publish`. Sound is off by default and nothing builds an audio context until the player opts in, so the existing puzzle suite passes unchanged.

## Performance

- **Duration:** about 8 min
- **Started:** 2026-10-13T19:32:32Z
- **Completed:** 2026-10-13T19:40Z
- **Tasks:** 3 (5 commits: RED and GREEN for Tasks 1 and 2, one commit for Task 3)
- **Files:** 15 (6 created, 9 modified)

## Engine API as shipped (`web/src/audio/sound-engine.ts`)

- `SoundContext` is the plan's `Pick<AudioContext, …>`. `SoundContextFactory = () => SoundContext`.
- `browserContextFactory()` returns `window.AudioContext ?? webkitAudioContext` wrapped as a factory, or `undefined`. It never constructs.
- `createSoundEngine({ createContext, onFailure, maybeLogError? })` returns `{ hasContext, isRunning, ensureRunning, maybeNowSeconds, play, stopAll, suspend }`.
  - `ensureRunning` constructs on first use and builds the master chain. It calls `resume()` only when the state is not `"running"` and no resume is in flight. A throwing construction is logged once, latches as failed and returns `false`. It never throws.
  - `play` runs only when the state is `"running"`, so `"suspended"` and `"interrupted"` create no nodes. It feeds each request through `admitVoice`, stops evicted voices, starts the admitted ones and registers each voice only after `start()`.
  - Splash: one 0.25 s mono noise buffer per context, created on the first splash, through a `bandpass` biquad. Impact: a `triangle` oscillator whose pitch ramps from `startHz` to `endHz`. Plink: a `sine` oscillator. Chime: three `sine` notes at `now + offsetSeconds`, one pool entry each. Envelopes go from 0.0001 to the peak and back to 0.0001, and none ramps to 0.
  - `stopAll` cancels gain automation, sets the gain to 0, calls `stop(now)` on each source and `disconnect()` on its gain. It tries every voice and rethrows the first error. It is safe with no context and safe to call twice.
  - `suspend` runs `stopAll`, then in a `finally` clears the resume guard and calls `context.suspend()`. It never calls `close()`. Async resume and suspend rejections reach `onFailure`.

## Gesture rules (`web/src/player/puzzle-sound.ts`)

- Listeners: `pointerdown`, `pointerup` and `keydown` with `{ capture: true, passive: true }`, plus `visibilitychange`. They are added once by `attach()` and removed by `leave()`.
- A gesture counts only if all of these hold: `event.isTrusted`; it is not a `keydown` with `Escape`; it is a `pointerdown` from a mouse or a `pointerup` from a non-mouse pointer; and its target does not match `SOUND_CONTROL_SELECTOR` (`.puzzle-sound-toggle, .puzzle-sound-section input`, checked with a duck-typed `closest`).
- An unlock runs only when sound is enabled and available, the tab is visible and the engine is not running.
- `setEnabled(true)` clears `failed` and calls `ensureRunning` inside the handler. On success it stores `"on"`. A failed construction makes sound unavailable and writes nothing. `setEnabled(false)` stores `"off"`, then stops voices and suspends.
- `present` uses `decideVoices` when the audio clock exists and `trackWithoutClock` when it does not. It never invents a time. It calls `play` only when sound is enabled, visible and running.

## D-10 stop and suspend hooks

| Event | Hook | Effect |
| --- | --- | --- |
| Play, Stop, Edit/Retry build, route entry, level change | `incrementGeneration` in `puzzle-host.ts` | `sound.stopVoices()`: voices stopped, trigger memory reset |
| Puzzle route entry | `beginPuzzleRoute` after `setMaybeLevelId(id)` | `sound.attach()` |
| Failure view | `releaseFailedPuzzle` | `sound.pause()`: stop, then suspend; listeners kept |
| Leave the puzzle route | first line of `leavePuzzle` | `sound.leave()`: listeners removed, stop, suspend; context kept |
| Tab hidden / visible | `visibilitychange` listener (puzzle routes only) | hidden: stop, then suspend. Visible: resume only when sound is on and a context exists |
| Each presented frame | `startLoop` publish in `puzzle-run.ts`, after `publishState`, `publishHeldHash` and `setView` | `sound.present(frame, signals, state)` |

## Verification

- `bunx vitest run tests/sound-engine.test.ts`: 21 passed. `tests/puzzle-sound.test.ts`: 24 passed. `tests/puzzle-loop.test.ts`: 12 passed, including the new fold test. `tests/puzzle-sound-wiring.test.ts`: 6 passed.
- `bun run typecheck` passes. `bun run test:unit`: 113 files and 1708 tests passed (before this plan: 110 and 1656).
- `bun scripts/bright-builds-check.ts all`: 0 findings.
- `bun run build:app`, then `CI=true bun run test:puzzle`: 81 passed.
- Grep gates:
  - `git grep -lE "window\.AudioContext|webkitAudioContext" -- web/src` prints only `web/src/audio/sound-engine.ts`.
  - No forbidden identifiers (`localStorage`, `Date.now`, `performance.now`, `setTimeout`, `setInterval`, `AudioContext`, `innerHTML`) appear in `puzzle-sound.ts`. `sound-engine.ts` has no wall-clock words, no `.close(` and no `exponentialRampToValueAtTime(0,`.
  - `audioSession` appears nowhere in `web/src`.

## Task Commits

1. **Task 1 RED:** `0d17dd85a` (test). **GREEN:** `750ce4374` (feat)
1. **Task 2 RED:** `acc1f7040` (test). **GREEN:** `c89748d6f` (feat)
1. **Task 3:** `2bc73dfc2` (feat)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] A pending resume could block every later resume**

- **Found during:** Task 2 (the visibility resume test)
- **Issue:** The A3 in-flight guard stayed set while a `resume()` promise was pending. If the context was suspended in that window (tab hidden, sound off, route leave), the next `ensureRunning` skipped `resume()`.
- **Fix:** `suspend()` clears the guard in its `finally`. Added the engine test "lets the next ensureRunning resume after suspending a pending resume".
- **Files modified:** `web/src/audio/sound-engine.ts`, `web/tests/sound-engine.test.ts`
- **Commit:** `c89748d6f`

### Interpretations

**2. Engine option `maybeLogError`**

- The plan's engine options are `{ createContext, onFailure }`, and construction failures must be logged. An optional `maybeLogError` (default `console.error`) lets the glue send that log through its own `logError`. This avoids logging the failure twice, and the glue test can observe it. It does not route through `onFailure`, because a missing constructor or a throwing construction makes sound unavailable (D-05). It is not a runtime failure (D-16).

**3. Mouse `pointerup` does not unlock**

- The plan lists `pointerup` as a gesture. The HTML activation-triggering list counts `pointerup` only for non-mouse pointers, so a mouse `pointerup` is ignored. Mouse unlocks on `pointerdown`. This avoids creating the context outside an activation, which would leave a resume pending. Every tested behavior (touch `pointerup`, mouse `pointerdown`, keydown) is as planned.

**4. A runtime failure also suspends**

- `disableAfterFailure` calls `engine.suspend()`, which stops voices first, instead of a bare `stopAll`. This matches the "Runtime audio failure" row of the UI-SPEC state table. It runs only once per failure, so a rejected suspend cannot loop.

## Known Stubs

None. `createDisabledPuzzleSound()` is the intended default for runtimes built outside the browser session (Research Pitfall 9). `playground-session.ts` passes `createBrowserPuzzleSound()`.

## Threat Flags

None. The new listeners are passive and only read `isTrusted`, `pointerType`, `key` and `target.closest`. T-42-08 through T-42-11 are covered as planned: trusted activation only and no `audioSession`; every engine call guarded; the fold is read-only with publish order and budget unchanged; stops on every generation bump and on leave.

## Next Phase Readiness

- Plan 04 can render the HUD toggle and drawer checkbox from `controller.soundEnabled()`, `soundAvailable()` and `soundFailed()`, and call `controller.setSoundEnabled(on)` from the click or change handler. The controls need the classes `puzzle-sound-toggle` and `puzzle-sound-section` (with its `input`) so the gesture listener skips them (UI-D35).

## Self-Check: PASSED

- All 6 created files exist on disk.
- Commits `0d17dd85a`, `750ce4374`, `acc1f7040`, `c89748d6f` and `2bc73dfc2` are in `git log`.
