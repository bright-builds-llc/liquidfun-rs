---
phase: 43-gpu-water-render-mode
plan: "01"
subsystem: render-bridge
tags: [rust, wasm, typescript, water, frame-contract, determinism, fingerprint]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 43-2026-10-13T22-55-49
generated_at: "2026-10-13T23:56:53.367Z"
requires:
  - "Phase 42 frame contract (ProofFrame optional groups, frame-sound.ts parser precedent)"
provides:
  - "crates/liquidfun-wasm/src/session/speeds.rs: max_particle_speed (moved), particle_speeds, attach_particle_speeds"
  - "SessionCore::capture_frame_with_speeds(bool); per-capture flag, never stored on SessionCore"
  - "FrameData::set_particle_speeds fail-closed validation and ProofFrame.particleSpeeds() getter (undefined unless requested)"
  - "ProofSession.captureFrame(particleSpeeds?: boolean) wasm binding; omitted or false returns the frame unchanged"
  - "web/src/physics/frame-speeds.ts parseParticleSpeeds into RenderFrame.maybeParticleSpeeds"
  - "target/phase43/base.txt pre-phase 25-scene fingerprint baseline for the phase gate"
affects:
  - "43 water pass reads RenderFrame.maybeParticleSpeeds for foam"
  - "43 worker/loop must call captureFrame(true) only while Water is requested"
tech-stack:
  added: []
  patterns:
    - "Stateless per-capture flag: bool.then(...).transpose() inside the existing view read, attached after FrameData::new"
    - "Optional frame lane (maybe_particle_speeds) mirrored fail closed in TS, returned without a copy"
key-files:
  created:
    - crates/liquidfun-wasm/src/session/speeds.rs
    - crates/liquidfun-wasm/src/session/speeds/tests.rs
    - crates/liquidfun-wasm/src/frame/speeds.rs
    - web/src/physics/frame-speeds.ts
    - web/tests/frame-speeds.test.ts
  modified:
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/frame.rs
    - crates/liquidfun-wasm/src/lib.rs
    - web/src/physics/frame.ts
decisions:
  - "Speed lane is a stateless per-capture flag (captureFrame(true)); lane values are length_squared().sqrt() from the same velocity read, so max(lane) == maxSpeed bit for bit"
  - "SessionCore::capture_frame() (no lane) is now #[cfg(test)]: wasm uses capture_frame_with_speeds, native callers are all tests"
  - "Baseline target/phase43/base.txt from scene-spot 2026-10-13T23-40-01Z at 2653fd987, identical to Phase 42 after-postfix; after-01 (2026-10-13T23-56-39Z) 25/25 identical"
metrics:
  duration: 17min
  completed: 2026-10-13
  tasks: 3
  files: 9
---

# Phase 43 Plan 01: Optional per-particle speed lane Summary

Captures now attach a per-particle speed lane only when asked, through `captureFrame(true)`. Rust computes it from the velocity read the capture already makes, and TS parses it fail closed into `RenderFrame.maybeParticleSpeeds`. Tests show that capturing with the lane leaves stepping and puzzle status bit-identical. All 25 scene fingerprints match the pre-phase baseline.

## What Was Built

- **Task 1 (baseline, no commit):** phase base `2653fd9875b7fed480dd6438d40a0a2df2eba864`; last Rust commit logged; Rust tree clean. Scene-spot stamp `target/dam-break-perf/2026-10-13T23-40-01Z/`. `target/phase43/base.txt` has 25 lines, and `diff` against `target/phase42/after-postfix.txt` is empty (`phase42-cross-check: identical` in `baseline.log`).
- **Task 2 (52eec199e):** `session/speeds.rs` holds the moved `max_particle_speed`, the new `particle_speeds` and `attach_particle_speeds`. `capture_frame_with_speeds(bool)` computes the lane inside the existing view block. `FrameData` gains `maybe_particle_speeds` (initialized `None`), with `set_particle_speeds` validation and the `particleSpeeds` getter in `frame/speeds.rs`. `ProofSession::capture_frame(particle_speeds: Option<bool>)` uses `unwrap_or(false)`. There are 11 `speeds` tests: 8 in session and 3 in frame. Two of them are 120-step on/off proofs for Dam Break and the First Pour solution: positions match as f32 bits and the status rows are equal.
- **Task 3 (62e62e041):** `parseParticleSpeeds` treats a missing getter or an `undefined` result as absent. It throws on a non-Float32Array, a wrong length, NaN, Infinity or a negative value. It returns the same array instance. After the wasm rebuild, `captureFrame(particle_speeds?: boolean | null)` and `ProofFrame.particleSpeeds(): Float32Array | undefined` appear in the generated d.ts. `just puzzle-replay-wasm` gives 6 LEVELHASH and 14 REPLAY passes with unchanged outcomes. Scene-spot `2026-10-13T23-56-39Z` gives `after-01.txt` 25/25 identical to `base.txt`.

## Verification

- `cargo test -p liquidfun-wasm --lib speeds`: 11 passed (log `target/phase43/speeds-test-01.log`); `cargo test -p liquidfun-wasm`: 599 passed, 25 ignored.
- `cargo fmt --all`; clippy for the workspace and `-p liquidfun-wasm --all-targets --all-features -D warnings`: clean; `cargo build -p liquidfun-wasm --all-targets --all-features`: ok; `bun scripts/bright-builds-check.ts all`: findings=0.
- `cd web && bun run typecheck && bun run test:unit`: 114 files, 1735 tests passed.
- Final line counts: `session.rs` 600 (≤ 600), `frame.rs` 556 (≤ 560), `frame/speeds.rs` 110 (< 120, so its tests stay inline).
- `git diff --stat 2653fd987 -- crates/liquidfun crates/liquidfun-wasm/src/scene.rs crates/liquidfun-wasm/src/scene crates/liquidfun-wasm/src/puzzle/fixtures` is empty.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] `SessionCore::capture_frame()` became dead code in non-test builds**
- **Found during:** Task 2 clippy
- **Issue:** Once the wasm binding called `capture_frame_with_speeds`, only test code (scene modules, puzzle tests, scene-spot `frame_hash`, all `#[cfg(test)]`) still called the no-flag `capture_frame()`. `-D dead-code` failed the lib build.
- **Fix:** Marked it `#[cfg(test)]`, following the Phase 37 precedent for test-only helpers rather than adding an `allow(dead_code)`. All 37+ test call sites keep the same signature.
- **Files modified:** crates/liquidfun-wasm/src/session.rs
- **Commit:** 52eec199e

**2. [Rule 3 - Blocking] `session.rs` line budget**
- **Issue:** Inlining the attach step left `session.rs` at 602 lines, over the 600 cap.
- **Fix:** Added `speeds::attach_particle_speeds(&mut frame, maybe_speeds)` (a one-line call) and `bool::then(...).transpose()` for the lane, which keeps all lane logic in `session/speeds.rs`.
- **Commit:** 52eec199e

**3. [Minor] Puzzle test core built from `REFERENCE_PLANK_POSE` instead of the fixture parser**
- **Issue:** `puzzle::replay::fixture` is `pub(in crate::puzzle)`, so `session/speeds/tests.rs` cannot call `parse_fixture` or `lanes`.
- **Fix:** Built the core with `build_puzzle_core("first-pour", &[ToolKind::Plank.code()], &REFERENCE_PLANK_POSE)`. This is the same `plank 8 64 23` placement as `first-pour-solution.txt`, and `api_tests.rs` uses the same approach.

**Note on TDD:** I wrote tests and implementation together and committed each task once. The plan allowed one or two commits for Tasks 2–3, and every behavior bullet has a dedicated test.

## Out-of-scope Observations

- `cargo clippy -p liquidfun-wasm --target wasm32-unknown-unknown` (all targets) fails in `bin/playground_scene_spot.rs` and `bin/dam_break_timers.rs` because native-only modules are configured out. This is the known v1.4 leftover ("wasm32 bins"), and it existed before this plan. The wasm32 `--lib` check is clean.

## Known Stubs

None.

## Self-Check: PASSED
