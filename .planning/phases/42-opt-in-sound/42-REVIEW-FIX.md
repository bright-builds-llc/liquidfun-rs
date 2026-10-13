---
phase: 42-opt-in-sound
fixed_at: 2026-10-13T20:35:22Z
review_path: .planning/phases/42-opt-in-sound/42-REVIEW.md
iteration: 1
findings_in_scope: 7
fixed: 7
skipped: 0
status: all_fixed
---

# Phase 42: Code Review Fix Report

**Fixed at:** 2026-10-13T20:35:22Z
**Source review:** .planning/phases/42-opt-in-sound/42-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 7 (2 warning, 5 info; fix_scope = all)
- Fixed: 7
- Skipped: 0

Every regression test added below was checked against the pre-fix code (by stashing the source change) and failed there, except where noted.

## Fixed Issues

### WR-01: A resume that never settles blocks every later resume, including resumes from real gestures

**Files modified:** `web/src/audio/sound-engine.ts`, `web/src/player/puzzle-sound.ts`, `web/tests/sound-engine.test.ts`, `web/tests/puzzle-sound.test.ts`
**Commit:** 6772b0980
**Applied fix:** `ensureRunning()` is only called from gesture handlers (the toggle, the checkbox and `onGesture`). It now always reaches `context.resume()` unless the context is already running. A new `resumeWithoutGesture()` covers the visibility path. It never constructs a context and keeps the in-flight dedupe, so background resumes do not stack. The old test "does not stack resume calls while one is in flight" asserted the buggy behavior, so it now asserts that a gesture resumes again. New tests check that the no-gesture path is deduped and never constructs. The glue test the reviewer asked for is also added: start a pending resume from visibility, then fire a trusted touch `pointerup`, and `resumeCalls` goes up.

### WR-02: A resume that throws synchronously leaves sound both enabled and failed

**Files modified:** `web/src/player/puzzle-sound.ts`, `web/tests/puzzle-sound.test.ts`, `web/tests/sound-fake-context.ts`
**Commit:** 5b16e51eb
**Applied fix:** `setEnabled(true)` now checks `!failed()` after `unlock()` before it stores "on" and sets the enabled signal. `ensureRunning` returning false was not used for this: `unlock` treats false as "unavailable" (D-05), which is the wrong state for a runtime failure (D-16). The fake context gained a `"throw"` resume behavior. The new test checks `enabled() === false`, `failed() === true`, `available() === true`, that nothing was stored, and that the error was logged.

### IN-01: Sound-only data can fail the puzzle run

**Files modified:** `crates/liquidfun-wasm/src/puzzle/sound.rs`, `crates/liquidfun-wasm/src/puzzle/sound/tests.rs`
**Commit:** 55011b67c
**Status:** fixed: requires human verification (a policy trade-off between D-07 fail-closed parsing and D-08)
**Applied fix:** The tally (the producer) now keeps only finite summed impulses (`strongest_finite_impulse`), and the impact itself still counts. A non-finite contact impulse therefore yields a valid frame and no longer becomes `FrameCaptureFailed`. Deliberately left unchanged:
- `FrameData::set_puzzle_sound` validation and the TS `parsePuzzleSound` fail-closed checks. They still reject contract violations, so the puzzle-status parse is not weakened.
- The `StepFailed` path for an unavailable particle-system view. That is a world invariant the hooks rely on too, not sound-only data.

There is no frame channel for reporting a dropped impulse to the TS side. Adding one would change the frame contract, so the code comment documents the drop instead. The tally only reads the step report, so simulation is unchanged.

### IN-02: The first observed step counts every existing particle-body contact as a new splash

**Files modified:** `crates/liquidfun-wasm/src/puzzle/sound.rs`, `crates/liquidfun-wasm/src/puzzle/sound/tests.rs`
**Commit:** a47a73c03
**Status:** fixed: requires human verification (logic change)
**Applied fix:** The tally now records step 1's contacts as a baseline and reports all-zero signals for that step. The recorded probe (`target/phase42/sound-probe.txt`) shows this was a real problem: every level reported 86 to 153 "new" splash contacts on step 1. Jelly Pad (0.307 m/s) and Grand Finale (0.350 m/s) are already above the 0.3 m/s splash gate on that step, so Play could open with a splash and a burst-onset impact.

The baseline also zeroes step-1 rigid `Begin` impacts, which goes slightly beyond the review text but has the same root cause. Nothing moves before the first step, so those contacts existed at spawn. Float Up reported a step-1 impact, and impacts are always audible (minimum strength 0.2). Float Up still has four rigid impacts after step 1, and `float_up_solution_reports_a_rigid_impact_with_impulse` still passes. New test: `first_step_reports_spawn_contacts_as_a_silent_baseline`, which covers all 14 fixtures.

### IN-03: The checkbox can disagree with the state, and focus is lost when sound becomes unavailable

**Files modified:** `web/src/components/PuzzleSoundToggle.tsx`, `web/src/styles/puzzle-sound.css`, `web/e2e/puzzle-sound.spec.ts`, `web/e2e/sound-helpers.ts`
**Commit:** 3faf07c2d
**Applied fix:**
- **HUD toggle.** It is now one `<button>` in every state. Unavailable switches the name, title, icon, `aria-describedby` and `data-sound-state`, and the click becomes a no-op. Unavailable uses `aria-disabled="true"` rather than `disabled`, because a disabled button still loses focus under the HTML focus-fixup rule. This follows the existing tray and selection-bar precedent. A CSS rule gives it the `button:disabled` look.
- **Drawer checkbox.** `onChange` now writes `available() && soundEnabled()` back to the box.

What the investigation found:
- **Construction failure, checkbox.** Here the reviewer's premise does not hold. Solid compiles `checked` as its own render effect with no previous-value comparison, and `available()` changes, so the box was already rewritten.
- **Synchronous `resume()` throw, checkbox.** This is the real stale-checkbox path. `available` stays true and `enabled` is written false over false, so the binding never re-runs.

Regression coverage:
- New Playwright helpers `installThrowingAudio` (the constructor throws) and `installThrowingResumeAudio` (`resume()` throws synchronously).
- Three new tests. "keeps focus on the HUD toggle when context creation fails" and "unchecks the drawer checkbox when resume throws" both fail on the old code. "unchecks the drawer checkbox when context creation fails" passes on both old and new code. It is kept as a guard for that path.

### IN-04: A click on the drawer checkbox's label text counts as an unlock gesture

**Files modified:** `web/src/player/puzzle-sound.ts`, `web/e2e/puzzle-sound.spec.ts`
**Commit:** 6f2a0a51d
**Applied fix:** `SOUND_CONTROL_SELECTOR` now includes `.puzzle-sound-section label`. The Playwright regression runs in the form-factor row, because there the checkbox is visible without any other gesture. In the drawer, opening the sheet is itself a gesture. The test stores "on", clicks the label's far end (away from the input), and checks that no context is built and "off" is stored. It failed on the old code, where one context was constructed.

### IN-05: `silence()` calls `stop()` a second time, and a synchronous `suspend()` throw can hide an earlier error

**Files modified:** `web/src/audio/sound-engine.ts`, `web/tests/sound-engine.test.ts`
**Commit:** 8cbf2587c
**Applied fix:**
- **Repeated `stop()`.** Each `source.stop(now)` in `silence()` is wrapped in its own try/catch and logged through the engine's `logError`. The gain mute and `disconnect()` still run, so the voice is silent either way.
- **Throwing `suspend()`.** `suspend()` now calls a new `suspendContext()`, which reports both a synchronous throw and a rejection to `onFailure`. A `stopAll` error still propagates.

Recursion through `disableAfterFailure` is bounded by its `alreadyFailed` guard. Three new unit tests cover these paths, and all three failed on the old code.

## Verification

- `cd web && bun run typecheck`: clean.
- `cd web && bunx vitest run`: 113 files, 1725 tests passed.
- `cargo test -p liquidfun-wasm`: 588 passed, 0 failed (25 ignored, which were already ignored). This includes the replay pins and `removing_the_tally_leaves_positions_and_status_bit_identical`.
- `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`: clean.
- `cargo fmt --all --check`: clean.
- `just playground-scene-spot`, then the 42-05 jq recipe: 25/25 fingerprints identical to `target/phase42/base.txt` (written to `target/phase42/after-review-fix.txt`).
- `just web-build`, then `bun run test:puzzle`: 99 passed, including the 4 new sound e2e tests. "reaches the same run with sound on and off" still matches its fingerprint.
- `bun scripts/bright-builds-check.ts all`: findings=0.

---

_Fixed: 2026-10-13T20:35:22Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
