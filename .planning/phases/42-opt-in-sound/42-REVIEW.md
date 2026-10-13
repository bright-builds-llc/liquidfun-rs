---
phase: 42-opt-in-sound
reviewed: 2026-10-13T20:14:21Z
depth: standard
files_reviewed: 46
files_reviewed_list:
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/frame/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle.rs
  - crates/liquidfun-wasm/src/puzzle/sound.rs
  - crates/liquidfun-wasm/src/puzzle/sound/tests.rs
  - crates/liquidfun-wasm/src/session.rs
  - web/e2e/puzzle-sound.spec.ts
  - web/e2e/sound-helpers.ts
  - web/package.json
  - web/src/audio/sound-engine.ts
  - web/src/audio/sound-signals.ts
  - web/src/audio/triggers.ts
  - web/src/audio/voice-pool.ts
  - web/src/audio/voices.ts
  - web/src/components/PuzzleHud.tsx
  - web/src/components/PuzzleIcons.tsx
  - web/src/components/PuzzlePanel.tsx
  - web/src/components/PuzzleSoundToggle.tsx
  - web/src/physics/frame-sound.ts
  - web/src/physics/frame.ts
  - web/src/player/playground-session.ts
  - web/src/player/puzzle-host.ts
  - web/src/player/puzzle-lifecycle.ts
  - web/src/player/puzzle-loop.ts
  - web/src/player/puzzle-run.ts
  - web/src/player/puzzle-runtime.ts
  - web/src/player/puzzle-sound.ts
  - web/src/player/sound-preference.ts
  - web/src/puzzle/controller.ts
  - web/src/styles/puzzle-sound.css
  - web/src/styles/puzzle.css
  - web/tests/frame-puzzle.test.ts
  - web/tests/frame-sound.test.ts
  - web/tests/puzzle-chrome.test.ts
  - web/tests/puzzle-host-fixture.ts
  - web/tests/puzzle-loop.test.ts
  - web/tests/puzzle-session.test.ts
  - web/tests/puzzle-sound-wiring.test.ts
  - web/tests/puzzle-sound.test.ts
  - web/tests/sound-engine.test.ts
  - web/tests/sound-fake-context.ts
  - web/tests/sound-preference.test.ts
  - web/tests/sound-signals.test.ts
  - web/tests/sound-triggers.test.ts
  - web/tests/sound-voice-pool.test.ts
  - web/tests/sound-voices.test.ts
findings:
  critical: 0
  warning: 2
  info: 5
  total: 7
status: issues_found
---

# Phase 42: Code Review Report

**Reviewed:** 2026-10-13T20:14:21Z
**Depth:** standard
**Files Reviewed:** 46
**Status:** issues_found

## Summary

Reviewed the Phase 42 diff (`bd685b08f..HEAD`, excluding `.planning/`): the Rust read-only sound tally and frame getters, the TS frame parse, the `web/src/audio/*` pure core and Web Audio shell, the puzzle sound glue, its lifecycle wiring, the HUD and drawer controls, and the unit and e2e tests. The ten sound-related Vitest suites pass locally (176 tests).

What holds up:

- **Simulation isolation.** `SoundTally::observe` only borrows `&World` and `&StepReport`. The `HashSet` is used only for membership and is never iterated into output. `ParticleId` is a stable identity, not a compacting index. `removing_the_tally_leaves_positions_and_status_bit_identical` checks the read-only claim. Scene sessions get no tally, so scene frames keep their shape.
- **Frame-loop safety.** `present` wraps everything in try/catch. Every engine call from the glue goes through `guarded`. `publish` is only reached after at least one step, so the `stepIndex <= last` reset in `observe` only fires on a new session, not on stalled frames.
- **Context creation.** The engine calls its factory only from `ensureRunning`. That runs only from the toggle and checkbox handlers or a trusted activation gesture while sound is on.
- **Teardown.** Listeners are added once per route entry and removed in `leavePuzzle`. `incrementGeneration` stops voices on every Play, Stop, Retry, route entry and level change. No timers are used, and voices scheduled for later are silenced by `stopAll`.
- **Voice cap.** The cap and chime eviction in `voice-pool.ts` are correct.
- **Preference storage.** Storage follows D-03.
- **HUD box.** The obstacle slot is a fixed 96 px box (44 + 8 + 44), and the caption and title offsets were adjusted to match.

The two warnings are both in the resume and failure state machine: a resume that never settles can block later resumes, and a resume that throws synchronously leaves the on/failed state contradictory.

## Warnings

### WR-01: A resume that never settles blocks every later resume, including resumes from real gestures

**File:** `web/src/audio/sound-engine.ts:263-280` (with `web/src/player/puzzle-sound.ts:202-226`)
**Issue:** `resume()` returns early while `resumeInFlight` is true. Only settling the promise or calling `suspend()` clears the flag. A resume can stay pending forever: per the Web Audio spec and Chrome's autoplay policy, `AudioContext.resume()` called without user activation is queued until the context is allowed to start. It is not rejected, and a new `resume()` call is needed after a real activation. Two paths in this phase call `ensureRunning()` without a guaranteed activation:

1. `onVisibilityChange` when the tab becomes visible.
1. `onGesture` for any trusted `keydown` other than Escape. Some keys, such as modifier-only presses or UA-reserved shortcuts, do not grant activation.

If either leaves a resume pending, every later trusted `pointerdown`, `pointerup` or `keydown` hits the `resumeInFlight` early return. `context.resume()` is never called again inside a real activation, so sound stays silent while the toggle shows on. Only toggling off and on again recovers (`suspend()` clears the flag). The unit test "does not stack resume calls while one is in flight" locks in this behavior.
**Fix:** Let calls made inside a gesture always reach `context.resume()`. Repeated calls are cheap and idempotent. Keep the in-flight dedupe only for calls made without a gesture:

```ts
function resume(context: SoundContext, fromGesture: boolean): void {
  if (context.state === "running" || (resumeInFlight && !fromGesture)) {
    return;
  }
  resumeInFlight = true;
  // ...unchanged...
}

// ensureRunning(fromGesture = true) for the toggle, checkbox and onGesture;
// ensureRunning(false) from onVisibilityChange.
```

Add a test: start a pending resume from visibility, then fire a trusted touch `pointerup`, and assert that `resumeCalls` went up.

### WR-02: A resume that throws synchronously leaves sound both enabled and failed

**File:** `web/src/player/puzzle-sound.ts:182-199`, `web/src/audio/sound-engine.ts:273-279, 296-297`
**Issue:** In `setEnabled(true)`, `unlock` → `ensureRunning` → `resume`. If `context.resume()` throws synchronously (rather than returning a rejected promise), `failed(error)` calls `onFailure` → `disableAfterFailure`. That sets `enabled = false` and `failed = true` and suspends the context. Then `ensureRunning` still returns `true`, so `setEnabled` goes on to call `preference.set(true)` and `setEnabledSignal(true)`. The result is a toggle showing on (`aria-pressed="true"`), the "Sound stopped after an audio error" note visible, and a suspended context. `present` stays silent because `isRunning()` is false. The engine contract says resume and suspend failures are asynchronous, but this branch explicitly handles a synchronous throw and gets the follow-on state wrong.
**Fix:** Have `ensureRunning` report the synchronous failure. Alternatively, check `failed()` after `unlock`:

```ts
guarded((engine) => {
  if (unlock(engine) && !failed()) {
    deps.preference.set(true);
    setEnabledSignal(true);
  }
});
```

Add a unit test with a fake whose `resume()` throws synchronously. Assert `enabled() === false`, `failed() === true`, and that the stored preference is unchanged.

## Info

### IN-01: Sound-only data can fail the puzzle run

**File:** `crates/liquidfun-wasm/src/session.rs:278-283, 327-332, 509-513`; `crates/liquidfun-wasm/src/frame.rs:140-149`; `web/src/physics/frame-sound.ts:38-55`
**Issue:** A tally error returns `StepFailed`. A non-finite impulse makes `set_puzzle_sound` fail, which becomes `FrameCaptureFailed`, and the TS parse fails the frame closed. All of these fail the whole run. D-07 asks for the fail-closed scalar pattern, but D-08 says audio-layer failures must not fail the run. In practice the only paths are an unavailable particle system view, which would already have failed in the hooks, or an infinite contact impulse. So this is a deliberate trade-off rather than a live bug.
**Fix:** Keep it, but record the decision in the phase summary. Alternatively, in Rust, clamp a non-finite impulse to `0.0` and set the count to `0` instead of rejecting the frame.

### IN-02: The first observed step counts every existing particle-body contact as a new splash

**File:** `crates/liquidfun-wasm/src/puzzle/sound.rs:70-81`
**Issue:** `previous` starts empty, so step 1 of a run session reports every particle-fixture contact as a new contact. For a level whose liquid starts resting on a body, the first presented frame can cross `SPLASH_MIN_CONTACTS_PER_STEP`. If any particle also exceeds 0.3 m/s, Play produces a splash plus a burst-onset impact. The speed gate usually hides this, but nothing guarantees it.
**Fix:** Treat the first observation as a baseline (`splash_contacts = 0` when no previous step has been observed). Alternatively, seed `previous` when the session is constructed.

### IN-03: The checkbox can disagree with the state, and focus is lost when sound becomes unavailable

**File:** `web/src/components/PuzzleSoundToggle.tsx:39-72, 88-95`
**Issue:** If construction fails in the drawer's `onChange`, the user has already checked the box. The bound value `available() && soundEnabled()` stays `false → false`, so Solid never writes `checked = false`, and the box stays checked while disabled. On the HUD, `<Show when={soundAvailable()}>` replaces the focused button with a new disabled button, so keyboard focus drops to `body`.
**Fix:** In `onChange`, set `event.currentTarget.checked = props.controller.soundEnabled()` after calling `setSoundEnabled`. For the HUD, render one button and switch `disabled`, `aria-label` and the icon instead of swapping elements with `Show`.

### IN-04: A click on the drawer checkbox's label text counts as an unlock gesture

**File:** `web/src/player/puzzle-sound.ts:36`
**Issue:** `SOUND_CONTROL_SELECTOR` matches `.puzzle-sound-section input` but not the wrapping `<label class="player-check">`. With a stored "on", pressing the label text creates and resumes the context in `onGesture`. The click then fires `change` and turns sound off, which immediately suspends it. Nothing audible happens, but a context is created as the player turns sound off.
**Fix:** Use `".puzzle-sound-toggle, .puzzle-sound-section label, .puzzle-sound-section input"`.

### IN-05: `silence()` calls `stop()` a second time, and a synchronous `suspend()` throw can hide an earlier error

**File:** `web/src/audio/sound-engine.ts:213-220, 362-371`
**Issue:** Every source already has `stop(end)` scheduled, so `silence()` calls `stop(now)` a second time. The current spec allows this, but older WebKit releases threw `InvalidStateError` on a repeated `stop()`. There, `stopAll` would rethrow on every Play, Stop or Retry while a voice is active, and `stopVoices` would turn sound off for the session. Separately, in `suspend()`, a synchronous throw from `context.suspend()` inside `finally` would replace an error rethrown by `stopAll`.
**Fix:** Wrap each `source.stop(now)` in its own try/catch. The `gain` mute plus `disconnect()` already silences the voice, so log the error rather than rethrowing it. In `suspend()`, call `context.suspend()` inside its own try/catch that reports to `onFailure`.

---

_Reviewed: 2026-10-13T20:14:21Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
