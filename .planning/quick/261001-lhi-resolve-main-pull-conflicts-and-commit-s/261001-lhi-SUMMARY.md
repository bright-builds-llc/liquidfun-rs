---
phase: quick
plan: 261001-lhi
subsystem: playground
tags: [rust, wasm, solidjs, merge-resolution]
requires:
  - phase: main
    provides: Upstream physics through 63e661b and preview assets through d4cf257.
provides:
  - Reconciled local wave tank changes and investigated bubbler reservoir compatibility.
affects: [playground]
tech-stack:
  added: []
  patterns: [Scene-specific gravity-relative particle escape bounds]
key-files:
  created:
    - web/src/catalog/wave-tank-controls.ts
  modified:
    - crates/liquidfun-wasm/src/scene/wave_tank.rs
    - crates/liquidfun-wasm/src/session/escape.rs
    - web/src/catalog/portrait-bounds.ts
key-decisions:
  - Preserve the local 50 m tank and its four controls alongside upstream changes.
  - Keep 80 m escape bounds exclusive to WaveTank; preserve other scenes' existing bounds.
  - Retain upstream 3000-particle bubbler initialization because both 1800-particle candidates failed the catch regression.
requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: direct-fallback
generated_at: "2026-10-01T20:39:51Z"
status: source-published
---

# Quick task 261001-lhi: Main pull reconciliation

The 50 m wave tank and its four controls coexist with upstream portrait frames and scene refinements. The bubbler retains the verified upstream 3000-particle initializer and elevator catch behavior.

## Execution status

Conflict reconciliation, native verification, direct WASM behavioral checks, independent acknowledgment, the source commit and its push are complete. The coordinating agent is finalizing the companion tracking commit.

AGENTS.md standing authorization, AGENTS.bright-builds.md, standards-overrides.md, the verification/testing/language standards, and PROJECT-SCOPE.md informed the work. Linux qualification remains optional. The original recovery stash is retained.

## Verification

- All four required core commands passed in order using `CARGO_TARGET_DIR=target/main-pull-resolution/native-check`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`. The final suite passed 1032 tests. `core-isolated-summary.json` records exit code 0 for each command and actual completion times; separate `*-core-final-isolated.log` logs preserve output.
- The earlier default-cache core attempt was incomplete: 438 unit, 24 broad-phase integration and 18 contract integration tests passed, while subsequent executables stalled before reaching Rust. After preserving logs and two process samples, the coordinating agent directed stopping only the current test child to release Cargo; Cargo reported exit 101 with SIGTERM for `collision_distance`. A second default-cache attempt also stalled. These retained attempts are not the passing final evidence.
- The coordinating agent reran formatting after integrating the upstream elevator catch refinement at `63e661b`; the core source did not change in that upstream update.
- No unresolved index entries remain and `git diff --check` passes.
- Final web checks passed: typecheck, 402 unit tests, production WASM/web build and 28 Playwright tests. The coordinating agent also confirmed Markdown and managed checks passed. A prior browser failure was retained and fixed by closing the previous controls sheet before navigating to Fountain; the final full suite passed.
- An affected WASM Clippy attempt blocked behind an unrelated language-server Cargo job after the core test stopped. That queued attempt was stopped separately and retained; affected checks subsequently completed successfully in `target/main-pull-resolution/native-check` with separate attempt logs.
- Isolated strict affected-crate Clippy passed in 1.50 seconds after narrow prerequisite repairs (`cargo-clippy-wasm-isolated-attempt-2.log`).
- Isolated `cargo build -p liquidfun-wasm --all-targets --all-features` passed in 5.78 seconds (`cargo-build-wasm-isolated-attempt-1.log`).
- Isolated `cargo test -p liquidfun-wasm --all-features` passed: 284 unit tests, zero failures, 82.52 seconds; zero doctests (`cargo-test-wasm-isolated-attempt-1.log`). Unlike the original cache, this test executable entered Rust normally. The unchanged upstream second-dwell catch regression is included in that passing suite.
- Subsequent strict affected-crate lint and the renamed 3000-particle test passed. A managed file-length finding was resolved by moving the unchanged constant press-speed assertion from hydraulic fountain tests into a production compile-time assertion. Formatting, affected strict lint, the focused runtime descent test and managed checks passed after that move.
- Immediately before the source commit, the four required core commands passed again in order in the isolated directory. `code-precommit-{fmt,clippy,build,test}.log` retain those successful results, and the staged source digest still matched the independently reviewed digest exactly.
- The rebuilt WASM upstream baseline passed the second-dwell catch check at 900 steps: plate center Y `0.0469999993`, 276 on deck versus 240 below, 3000 total particles. Both horizontal tilt directions retained all 2500 wave tank particles. The coordinating agent retained `wasm-scene-check-upstream-baseline.log`.

## Decisions and simplification

- The 1800-particle reservoir candidates failed the latest upstream catch regression. The unchanged upstream 3000-particle filled polygon passed, so it is the final initializer; the disconnected experimental reservoir module was removed after preserving candidate evidence.
- Particle escape logic shares its proxy and gravity guards. One scene match selects the wave tank's 80 m fall/sideways bounds; other scenes retain 12 m along gravity and 48 m sideways/against gravity.
- The local tank retains width, speed, amplitude and slant controls with its portrait framing. Upstream fountain controls and framing remain intact.
- Two compatible upstream commits arrived during verification. The coordinating agent preserved the working changes, fast-forwarded main to `63e661b`, and reconciled the bubbler module while retaining upstream elevator-motion/catch modules and temporarily retaining an experimental bounded reservoir. Behavioral verification later replaced that reservoir with the verified upstream initializer.
- A final asset-only upstream update arrived before committing. Main fast-forwarded to `d4cf257`; physics remained based on `63e661b`, and the reviewed source digest was unchanged. The final resolution uses upstream's verified 3000-particle initializer.

## Deviations from Plan

### Reservoir compatibility investigation

The new upstream second-dwell catch regression requires more than 200 particles on the elevator deck and a larger on-deck load than the under-plate load. Direct rebuilt WASM execution at 900 steps showed the initial 50 by 36, 1800-particle reservoir had 20 particles on deck and 178 below. A wider 75 by 24 layout retained 1800 particles but also failed, with 23 on deck and 185 below. The regression threshold was preserved. After two failed candidates, the coordinating agent revised the plan to measure the unchanged upstream initializer at `63e661b`; the 1800 cap was a reconciliation assumption, not an explicit user requirement. Failed candidate source and patch copies are retained under `target/main-pull-resolution/`.

### Review-driven correctness repair

The independent reviewer identified valid water at `(49, 1)` being evicted when gravity pointed right: the original 12 m gravity-relative fall limit cut through the 50 m tank. The resolution threads `SceneId` through both regular and profiled particle eviction and assigns WaveTank 80 m limits in every gravity-relative direction. Regression coverage checks both pool ends under four gravity directions, normal-scene escape limits, the proxy guard, and real `SessionCore` advances with horizontal tilt.

### Narrow verification repairs

- Core Clippy repairs collapse a nested particle-system filter and remove a redundant iterator conversion; physics behavior is unchanged.
- The washing-machine sock position parameter was renamed to avoid a Clippy similar-name failure.
- A stacked-drip sample constant is test-only. Native/test particle counting and its snapshot import are excluded from browser builds where unused, while remaining available to native scene spot checks.
- A missing particle-radius import was fixed in a temporary experimental reservoir candidate. That candidate and its module were later discarded after failing the catch regression; the import repair is not part of final production code. Failed source and patch records remain preserved under `target/main-pull-resolution/`.
- The authorized simulation-performance Markdown received formatter-only changes.
- Strict affected-crate Clippy exposed additional existing style blockers. The coordinating agent explicitly expanded prerequisite repairs to that crate: lossless bounded integer conversions, compile-time invariant assertions, float step accounting for bounded test windows, a result type alias, range predicates, documentation markup and redundant closure/binding removal. No warning suppression or catch-threshold weakening was introduced.

The added prerequisite repairs touch `hydraulic_fountain.rs` and its tests, `impulse.rs`, `soup_family.rs`, `soup_stirrer.rs`, `theo_jansen.rs`, `wave_machine.rs` and its tests, `washing_machine.rs` and its tests, `stacked_drip.rs` and its tests, and the bubbler test modules. All affected-crate lint, build and tests pass after these changes.

## Commits and publication

Source commit: `0fa50e1b4fd41b4a00e2b2324f9df11faa49e796` — `fix(playground): reconcile scene changes with latest main`. It contains 32 reviewed source, test and simulation-performance documentation paths. Implementation and verification were captured together in one coherent integration commit, as directed by the coordinating agent. GSD and task tracking are excluded from that commit and remain owned by the coordinating agent.

The verified commit was created on local `main`, based on `d4cf257`, with origin confirmed as `git@github.com:bright-builds-llc/liquidfun-rs.git`. The coordinating agent successfully pushed it normally to `origin/main` (`d4cf257..0fa50e1`). This executor did not push. The coordinating agent owns the final companion tracking commit and its publication.

## Independent review

Separate AI reviewer `/root/independent_review` acknowledged the final complete relevant source diff at actual review time `2026-10-01 21:07:04 UTC`, bound to SHA256 `07e45e462804d0f583437cc1bccbb15a70d552d04b0afdd915d3bfe23b6845fe`. The reviewer found no remaining actionable defects after inspecting final source, the additional upstream changes, prerequisite repairs and verification evidence. This is an independent AI acknowledgment, not human approval. The append-only review record is retained in `target/main-pull-resolution/review.md`.

## Residual limits

Local verification supports the reconciled playground paths; it does not claim full upstream parity or strict Linux qualification. No new network, authentication, file-access or trust-boundary surface was introduced.

## Host execution limitation

Samples of the stalled core executables show `_dyld_start` before Rust with a 96 KB footprint. The coordinating agent observed normal macOS assessment activity and a separate enumeration stall in the old 48 GB dependency cache, matching the active macOS host-stall lesson. Signatures, security metadata and system settings were not changed. All required native verification passed using an isolated build directory. The original cache issue remains separate from source correctness; no reboot or destructive cache cleanup was needed.

## Self-Check: PASSED

- Both created source artifacts, `web/src/catalog/wave-tank-controls.ts` and `web/tests/scene-surface.test.ts`, exist.
- This summary exists at its planned path.
- Git confirms `0fa50e1b4fd41b4a00e2b2324f9df11faa49e796` is a commit.
- A scan of all committed changed paths found no TODO, FIXME or placeholder markers; independent review and native/browser behavior checks cover the implemented paths.
- The source commit diff against `d4cf257` hashes to the acknowledged source digest, and the coordinating agent confirmed its successful normal push. Companion tracking publication remains owned by the coordinating agent.
