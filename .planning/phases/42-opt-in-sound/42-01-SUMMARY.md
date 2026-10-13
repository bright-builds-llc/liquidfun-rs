---
phase: 42-opt-in-sound
plan: "01"
subsystem: puzzle-bridge
tags: [rust, wasm, typescript, sound, puzzle, determinism, fingerprint]
requires:
  - "Phase 41 puzzle bridge (SessionCore puzzle sources, ProofFrame puzzle getters, frame-puzzle.ts)"
provides:
  - "crates/liquidfun-wasm/src/puzzle/sound.rs: SoundSignals and SoundTally (observe/signals), read-only per-step aggregates"
  - "ProofFrame splashContacts / impactCount / impactImpulse getters (undefined for scenes)"
  - "FrameData::set_puzzle_sound fail-closed validation (requires puzzle status, finite non-negative impulse, no impulse without an impact)"
  - "web/src/physics/frame-sound.ts parsePuzzleSound into RenderFrame.maybeSound (required on puzzle frames, rejected on scene frames)"
  - "target/phase42/sound-probe.txt per-step tuning data for all 14 fixtures (Plan 02 thresholds)"
  - "target/phase42/base.txt pre-phase 25-scene fingerprint baseline for the phase gate"
affects:
  - "42-02 triggers/voices tune splash and impact thresholds from the probe"
  - "42 puzzle loop folds per-step maybeSound across a presented frame"
tech-stack:
  added: []
  patterns:
    - "Puzzle-only Option<SoundTally> on SessionCore observing the existing StepReport after on_after_step"
    - "Separate optional frame group (maybe_sound) beside maybe_puzzle, mirrored fail-closed in TS"
    - "Membership-only HashSet with BuildHasherDefault<DefaultHasher>; counts iterate the deterministic body_contacts order"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/sound.rs
    - crates/liquidfun-wasm/src/puzzle/sound/tests.rs
    - web/src/physics/frame-sound.ts
    - web/tests/frame-sound.test.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/frame/puzzle.rs
    - web/src/physics/frame.ts
    - web/tests/frame-puzzle.test.ts
    - web/tests/puzzle-session.test.ts
    - web/tests/puzzle-host-fixture.ts
decisions:
  - "42-01: splashContacts = particle-fixture body contacts present this step and absent the previous step; impactCount = non-sensor rigid Begin transitions; impactImpulse = largest summed (clamped >= 0) normal impulse of those contacts in contact_solves + continuous_contact_solves; crates/liquidfun untouched"
  - "42-01: A2 held: float-up Begin contacts have a same-step solve with impulse > 0 (release probe: 5 impact steps, impulse max 0.062 N·s on the solution), so the deferred-report fallback was not needed"
  - "42-01: baseline target/phase42/base.txt from scene-spot 2026-10-13T19-08-15Z at bd685b08f (last Rust commit 0cd2e0f31), identical to Phase 41 after-final; after-01 (2026-10-13T19-21-46Z) 25/25 identical"
  - "42-01: only float-up produces rigid impacts (impact_steps 0 in the other five levels); per-step splash peaks 86-153 with p90 4-40, so Plan 02 needs the D-15 splash-onset impact and a splash threshold"
metrics:
  duration: 15min
  completed: 2026-10-13
  tasks: 3
  files: 12
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 42-2026-10-13T18-16-08
generated_at: "2026-10-13T19:21:56.064Z"
---

# Phase 42 Plan 01: Read-only sound aggregates on puzzle frames Summary

Puzzle frames now carry three read-only per-step sound scalars: `splashContacts`, `impactCount` and `impactImpulse`. A puzzle-only `SoundTally` computes them from the step's existing `StepReport` and `ParticleSystemView::body_contacts`. TS parses them fail closed into `RenderFrame.maybeSound`. Tests show that turning the tally off leaves stepping bit-identical, and the 25 scene fingerprints are unchanged.

## Performance

- **Duration:** about 15 min
- **Started:** 2026-10-13T19:07:49Z
- **Completed:** 2026-10-13T19:22Z
- **Tasks:** 3
- **Files:** 12 (4 created, 8 modified), plus gitignored evidence under `target/phase42/`

## Accomplishments

- **Task 1, baseline:** phase base `bd685b08fa5b2d2c7bd22833490b2db644ad677a`; the last Rust commit is `0cd2e0f31d57f728802989211c416fb6a9dbd1f1` and the Rust tree had no changes. The scene-spot stamp is `target/dam-break-perf/2026-10-13T19-08-15Z/`. `base.txt` has 25 lines, and the cross-check `phase41-cross-check: identical` (against `target/phase41/after-final.txt`) is recorded in `baseline.log`.
- **Task 2, Rust:** added `puzzle/sound.rs`: `SoundSignals`, `SoundTally::observe/signals`, `rigid_impacts`, `begin_impulse`, `is_same_contact` and `summed_normal_impulse`. `SessionCore` gets `maybe_sound: Option<SoundTally>` (Some only for puzzles). It observes after `on_after_step` in both `advance` and `advance_profiled`, records `sound tally failed: …` on error, and attaches the signals in `annotate_frame` after `set_puzzle_status`. `FrameData::set_puzzle_sound` adds the Rust-side validation. Added the test-only `without_sound_tally()`. `session.rs` is now 602 lines (limit 610).
- **Task 3, TS:** `RawProofFrame` has three optional getters. `PuzzleFrameSound` and `RenderFrame.maybeSound` were added, and `parsePuzzleSound` lives in `frame-sound.ts` (type-only import of `frame.ts`). The three raw puzzle-frame fakes return zero sound.

## Verification

- `cargo test -p liquidfun-wasm --lib puzzle::sound`: 11 passed, 1 ignored (the probe).
- `cargo test -p liquidfun-wasm --release --lib puzzle::replay`: 34 passed. The replay pins are unchanged.
- `cargo test -p liquidfun-wasm` (full lib): 585 passed, 25 ignored.
- `cargo fmt --all`, the workspace clippy and `-p liquidfun-wasm` clippy with `--all-targets --all-features -D warnings`, `cargo build -p liquidfun-wasm --all-targets --all-features`, and wasm32 lib clippy are all clean. `bun scripts/bright-builds-check.ts all` reports 0 findings.
- `bun run typecheck` passes. `bun run test:unit`: 105 files, 1565 tests passed.
- `just web-wasm`, then `just puzzle-replay-wasm`: 6 `LEVELHASH … pass` lines, 14 `REPLAY … pass`, no failures (`target/phase42/replay-01.log`). The replay parses every wasm frame through `parseRenderFrame`, so the wasm build emits the new getters on puzzle frames.
- 25/25 check: `diff target/phase42/base.txt target/phase42/after-01.txt` is empty (stamp `2026-10-13T19-21-46Z`).
- `git diff --stat bd685b08f -- crates/liquidfun crates/liquidfun-wasm/src/puzzle/fixtures crates/liquidfun-wasm/src/scene.rs crates/liquidfun-wasm/src/scene` is empty.

## A2 (same-step solve for Begin contacts)

A2 held. The `float_up_solution_reports_a_rigid_impact_with_impulse` test passes without the deferred-report fallback. In release, the float-up solution shows 5 impact steps with an impulse up to 0.062008 N·s. The empty run shows 6 steps, up to 0.101071 N·s.

## Sound probe summary lines (verbatim, `target/phase42/sound-probe.txt`, release)

Columns are `step splash_contacts impact_count impact_impulse max_speed goal_count`. `splash_p90` is taken over all printed steps. `impulse_p90` is taken over steps with `impact_count > 0`.

```
## first-pour-solution.txt splash_max=136 splash_p90=28 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## first-pour-empty.txt splash_max=136 splash_p90=15 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## thick-syrup-solution.txt splash_max=108 splash_p90=5 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## thick-syrup-empty.txt splash_max=108 splash_p90=4 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## jelly-pad-solution.txt splash_max=153 splash_p90=18 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## jelly-pad-empty.txt splash_max=153 splash_p90=11 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## float-up-solution.txt splash_max=152 splash_p90=40 impact_steps=5 impulse_max=0.062008 impulse_p90=0.062008
## float-up-empty.txt splash_max=152 splash_p90=33 impact_steps=6 impulse_max=0.101071 impulse_p90=0.101071
## timed-gate-solution.txt splash_max=86 splash_p90=12 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## timed-gate-empty.txt splash_max=86 splash_p90=11 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## timed-gate-early.txt splash_max=86 splash_p90=11 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## timed-gate-late.txt splash_max=86 splash_p90=10 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## grand-finale-solution.txt splash_max=98 splash_p90=14 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
## grand-finale-empty.txt splash_max=98 splash_p90=14 impact_steps=0 impulse_max=0.000000 impulse_p90=0.000000
```

## Task Commits

1. **Task 1: baseline.** No commit; the evidence is gitignored under `target/phase42/`.
1. **Task 2: Rust SoundTally, frame attachment, getters, tests, probe.** `1a7f1ac99` (feat)
1. **Task 3: TS maybeSound parse, fakes, wasm replay, 25/25.** `56b542f69` (feat)

## Deviations from Plan

### Process notes

**1. TDD RED and GREEN in one commit per task**

- The tests and implementation were written together and committed as one `feat` commit per task. The plan allowed one or two commits for Tasks 2 and 3. A RED-only Rust commit would not have compiled on `main`. No behavior changed.

**2. Probe p90 definition**

- The plan did not define the probe's p90 statistics. `splash_p90` uses nearest rank over every printed step. `impulse_p90` uses only steps with an impact, because an all-step impulse p90 would always be 0. The probe output starts with a `# columns:` line that states both definitions. It does not affect the 14 `## ` summary lines.

**3. Sound fakes in the scene-shaped frame-puzzle test**

- `frame-puzzle.test.ts`'s "puzzle methods all returning undefined is a scene frame" test sets the three sound getters to `() => undefined`. This matches how the generated scene frames behave. Its assertion is unchanged.

None of these changes affect the plan's behavior or acceptance criteria.

## Known Stubs

None.

## Next Phase Readiness

- Plan 02 can fold `maybeSound` per presented frame and tune its thresholds from the probe lines above. Rigid impacts are audible only in float-up, which confirms D-15's splash-onset impact.

## Self-Check: PASSED

- All four created files exist on disk.
- Commits `1a7f1ac99` and `56b542f69` are in `git log`.
- `target/phase42/{phase-base.txt,base.txt,baseline.log,sound-probe.txt,after-01.txt,replay-01.log,sound-test-01.log}` are all present.
