---
phase: 37-puzzle-core-in-the-bridge
plan: "01"
subsystem: determinism-tooling
tags: [spike, determinism, wasm, fnv1a, frame-hash, GOAL-04]
generated_by: gsd-execute-phase
lifecycle_mode: yolo
phase_lifecycle_id: 37-2026-10-11T21-49-11
generated_at: "2026-10-11T23:27:10.857Z"
spike_contract: exact
requires:
  - scene_spot::fingerprint::Fnv1a (Phase 35, unchanged)
  - SessionCore::create / advance / capture_frame and ProofFrame lane getters
  - web/scripts/readme-svg.ts wasm loader pattern
provides:
  - test-only native frame_hash_lanes / frame_hash (crates/liquidfun-wasm/src/scene_spot/frame_hash.rs)
  - Bun fnv1a64Bytes / fnv1a64Lanes / hex16 and spike parse/compare helpers (web/scripts/puzzle-replay/)
  - loadGeneratedWasm() and repoRoot for later puzzle replay scripts (web/scripts/puzzle-replay/wasm.ts)
  - `just puzzle-spike` reproducing target/puzzle-spike/spike-report.txt
  - 25-scene baseline target/phase37/spot-base.jsonl and base.txt from phase base 958d6fad7
affects:
  - 37-04 and 37-08 (25/25 fingerprint gate against target/phase37/base.txt)
  - puzzle replay fixtures (exact release-vs-wasm contract; debug asserts fallback form)
tech-stack:
  added: []
  patterns:
    - one FNV-1a 64 frame hash, implemented natively (test-only) and in Bun, pinned by shared vectors
    - fail-closed spike line parsing (exact field shape, contiguous steps)
key-files:
  created:
    - crates/liquidfun-wasm/src/scene_spot/frame_hash.rs
    - web/scripts/puzzle-replay.ts
    - web/scripts/puzzle-replay/hash.ts
    - web/scripts/puzzle-replay/spike.ts
    - web/scripts/puzzle-replay/wasm.ts
    - web/tests/puzzle-replay.test.ts
    - scripts/puzzle-spike.sh
  modified:
    - crates/liquidfun-wasm/src/scene_spot.rs
    - justfile
key-decisions:
  - "Spike (300 steps, phase base 958d6fad7): release-native vs wasm identical for dam-break and water-wheel, theo-jansen diverges at step 70; spike_contract is exact between release-native and wasm"
  - "Debug native diverges from release (dam-break @91, water-wheel @39, theo-jansen @2), so the default debug cargo test replay asserts the fallback form (same outcome, +5% fill margin, outcome-step tolerance)"
  - "theo-jansen trig divergence confirms D-03 angle snapping: static tools use the tau/24 baked-vertex table with body angle 0; no change to crates/liquidfun or any [profile]"
requirements: [GOAL-04]
duration: 26min
completed: 2026-10-11
---

# Phase 37 Plan 01: Native-vs-wasm determinism spike and shared frame hash Summary

**Release-native and wasm frame hashes are bit-identical for 300 steps of dam-break and water-wheel, so the puzzle replay contract is exact between release-native and wasm. Theo-jansen (motorized linkage) diverges at step 70, and native debug diverges from release in all three scenes.**

## Performance

- **Duration:** about 26 min (23:01:37Z to 23:27:10Z). About 18 min of that was the first debug test build, which waited on a concurrent rust-analyzer `cargo check --workspace`.
- **Tasks:** 2 of 2
- **Files:** 9 (7 created, 2 modified)

## Spike Result

Measured by `just puzzle-spike` at HEAD `958d6fad7fa76884182a8f130afb4e90c11b1139`, rustc 1.97.0 (2d8144b78 2026-07-07), Bun 1.4.2. Each scene ran 300 steps of `advance(1)` followed by `captureFrame()`. The hash is FNV-1a 64 over the particle positions, then the rigid segments, then the rigid circles.

| Scene | release-vs-wasm | debug-vs-release | debug-vs-wasm |
| --- | --- | --- | --- |
| dam-break | identical | diverges at step 91 | diverges at step 91 |
| water-wheel | identical | diverges at step 39 | diverges at step 39 |
| theo-jansen | diverges at step 70 | diverges at step 2 | diverges at step 2 |

Verbatim `target/puzzle-spike/spike-report.txt`:

```
head 958d6fad7fa76884182a8f130afb4e90c11b1139
rustc rustc 1.97.0 (2d8144b78 2026-07-07)
bun 1.4.2
== release-vs-wasm
COMPARE dam-break identical 300
COMPARE water-wheel identical 300
COMPARE theo-jansen diverges-at 70 of 300
== debug-vs-release
COMPARE dam-break diverges-at 91 of 300
COMPARE water-wheel diverges-at 39 of 300
COMPARE theo-jansen diverges-at 2 of 300
== debug-vs-wasm
COMPARE dam-break diverges-at 91 of 300
COMPARE water-wheel diverges-at 39 of 300
COMPARE theo-jansen diverges-at 2 of 300
```

**Policy:** the replay contract is exact (same outcome and outcome step) between release-native and wasm, asserted by `cargo test --release` and the Bun wasm replay. The default debug `cargo test` always asserts the fallback form: the same outcome, a fill margin of at least +5% of `full_count`, and an outcome-step tolerance. Static tools use the τ/24 baked-vertex table with body angle 0, so they never call `sin`/`cos`. Nothing under `crates/liquidfun` or any `[profile]` setting changed.

## Baseline (D-21)

- Phase base: `958d6fad7fa76884182a8f130afb4e90c11b1139` (`target/phase37/phase-base.txt`). This is the HEAD at plan start, recorded before any repository edit, with `git status --porcelain -- crates` empty before and after.
- `target/phase37/bin/spot-base --runs 1` wrote `target/phase37/spot-base.jsonl`. `target/phase37/base.txt` holds 25 `scene fingerprint` lines (for example `dam-break 4fa65d209b7e782f`).

## Accomplishments

- Added a native, test-only frame hash that reuses the unchanged `Fnv1a`. The spike test is `#[ignore]`, so plain `cargo test` does not run it.
- Added a matching Bun hash that reads the lanes' raw little-endian bytes, which keeps `-0.0` exact. It throws on a big-endian host.
- Both sides are pinned to the same vectors: no lanes gives `cbf29ce484222325`, and the lanes `[1.0, -0.0]`, `[]`, `[0.5]` give `99e01cab84d730f5`. The Bun side also checks `"a"` against `af63dc4c8601ec8c`.
- Added `parseSpikeLines`, which rejects a malformed line or a step out of sequence, and `compareSpikes`, which rejects a missing scene or a step-count mismatch and reports the first diverging step.
- `scripts/puzzle-spike.sh` (`just puzzle-spike`) rebuilds wasm first (T-37-02). It requires 900 lines per hash file and puts the HEAD, rustc and Bun versions in the report header (T-37-01).

## Task Commits

1. **Task 1: Baseline, then build and run the spike.** Tests and spike tooling.
1. **Task 2: Local checks and commit.** `d9711bf7e` (`test(37-01): add native-vs-wasm determinism spike and shared frame hash`).

The plan asks for one commit, so both tasks share `d9711bf7e`.

## Check Results (`target/phase37/checks-01.results`)

| Check | Result |
| --- | --- |
| `cargo fmt --all --check` | exit=0 |
| `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings` | exit=0 |
| `cargo build -p liquidfun-wasm --all-targets --all-features` | exit=0 |
| `cargo test -p liquidfun-wasm --all-features` | exit=0 |
| `bun scripts/bright-builds-check.ts all` | exit=0 |
| `cd web && bun run typecheck` | exit=0 |
| `cd web && bunx vitest run tests/puzzle-replay.test.ts tests/frame.test.ts tests/session.test.ts` | exit=0 |

`cargo test -p liquidfun-wasm --lib scene_spot::frame_hash`: 2 passed, 1 ignored. `git diff --stat 958d6fad7 -- crates/liquidfun` prints nothing.

## Deviations from Plan

- `spike.ts` was formatted with `bunx prettier --write`, which only changed how `SPIKE_SCENES` wraps. Prettier is not a project dependency or check, so this is a style choice and has no effect on behavior.
- The orchestrator asked for `generated_by: gsd-execute-phase`, and this file uses it. The plan's output section says `gsd-executor`. The other provenance fields match the plan.

- GOAL-04 is still open in REQUIREMENTS.md. `requirements mark-complete` was run and then reverted, because GOAL-04 needs a recorded solution that replays natively and in the browser. That work belongs to Plans 04, 05, 06 and 08. This plan only fixes the contract that GOAL-04's replay will use.

Otherwise the plan was executed as written.

## Known Stubs

None.

## Self-Check: PASSED

- FOUND: crates/liquidfun-wasm/src/scene_spot/frame_hash.rs, web/scripts/puzzle-replay.ts, web/scripts/puzzle-replay/{hash,spike,wasm}.ts, web/tests/puzzle-replay.test.ts, scripts/puzzle-spike.sh
- FOUND: commit d9711bf7e
- FOUND: target/phase37/base.txt (25 lines), target/phase37/phase-base.txt, target/puzzle-spike/spike-report.txt (9 COMPARE lines)
