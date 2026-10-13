---
phase: 41-level-select-and-progress
plan: "01"
subsystem: puzzle-bridge
tags: [rust, wasm, typescript, fingerprint, fnv1a, puzzle, progress]
requires:
  - "Phase 40 six-level registry (LEVEL_IDS / PUZZLE_IDS)"
provides:
  - "PuzzleLevelInfo.levelHash (Rust getter, JS levelHash) — 16 lowercase hex digits per level"
  - "crates/liquidfun-wasm/src/puzzle/level-hashes.txt pin file shared by native test and wasm replay"
  - "TS PuzzleLevelInfo.levelHash parsed fail-closed"
  - "target/phase41/base.txt pre-phase 25-scene fingerprint baseline for the 41-08 gate"
affects:
  - "41 restore/save plans (PROG-04) key saved placements on levelHash"
tech-stack:
  added: []
  patterns:
    - "FNV-1a 64 over an encoding-versioned Debug text of a const spec"
    - "One pin file checked by both the native Rust test and the wasm replay"
key-files:
  created:
    - crates/liquidfun-wasm/src/puzzle/level_hash.rs
    - crates/liquidfun-wasm/src/puzzle/level_hash/tests.rs
    - crates/liquidfun-wasm/src/puzzle/level-hashes.txt
    - web/scripts/puzzle-replay/level-hashes.ts
    - web/tests/puzzle-level-hashes.test.ts
  modified:
    - crates/liquidfun-wasm/src/puzzle.rs
    - crates/liquidfun-wasm/src/puzzle/info.rs
    - web/src/physics/puzzle-session.ts
    - web/scripts/puzzle-replay.ts
    - web/tests/puzzle-level-info.test.ts
    - web/tests/puzzle-session.test.ts
    - web/tests/puzzle-host-fixture.ts
decisions:
  - "41-01: level hash = FNV-1a 64 of level-hash-v1|{LevelSpec:?}|grid={GRID_METERS:?}|tick={STEPS_PER_OPEN_TICK}, 16 lowercase hex; pins in puzzle/level-hashes.txt (first-pour f0d2fdf9d2efb050 ... grand-finale 88c192beb348641e)"
  - "41-01: wasm replay checks every PUZZLE_IDS levelHash against the same pin file before the fixtures (LEVELHASH lines); native == wasm for all six levels"
  - "41-01: D-17 baseline target/phase41/base.txt from scene-spot 2026-10-13T16-42-00Z at f277ecad9, identical to Phase 40 after-final; after-01 (2026-10-13T16-52-29Z) 25/25 identical"
metrics:
  duration: 11min
  completed: 2026-10-13
  tasks: 3
  files: 12
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 41-2026-10-13T15-49-07
generated_at: "2026-10-13T16:52:44.260Z"
---

# Phase 41 Plan 01: Level fingerprint (levelHash) and phase baseline Summary

The wasm bridge now exposes `levelInfo().levelHash()`: FNV-1a 64 over the encoding-versioned `Debug` text of each `LevelSpec`. All six levels are pinned in `level-hashes.txt`, and both the native Rust test and the wasm replay check that file, so the native and wasm hashes are shown to match. TS parses the hash fail-closed. The pre-phase 25-scene baseline is recorded and unchanged after the change.

## Tasks

| Task | Name | Commit | Key files |
| ---- | ---- | ------ | --------- |
| 1 | Record the 25-scene fingerprint baseline at the phase base | none (gitignored `target/`) | target/phase41/{phase-base.txt,base.txt,baseline.log} |
| 2 | Rust level fingerprint, `levelHash` getter and pinned per-level test | 0cd2e0f31 | puzzle/level_hash.rs, level_hash/tests.rs, level-hashes.txt, info.rs, puzzle.rs |
| 3 | TS `levelHash` parse, fakes, wasm replay pin check, 25/25 check | dfa7d00d8 | physics/puzzle-session.ts, scripts/puzzle-replay.ts, scripts/puzzle-replay/level-hashes.ts, tests |

## Baseline evidence (Task 1)

- Phase base commit: `f277ecad9f85ded5cf71b8a44966b893cf9efd0d` (`target/phase41/phase-base.txt`).
- `git diff --quiet 31dec5b1e HEAD -- crates tools xtask Cargo.toml Cargo.lock` and the working-tree check both exited 0.
- Scene-spot stamp: `target/dam-break-perf/2026-10-13T16-42-00Z/`; `base.txt` has 25 lines.
- `phase40-cross-check: identical` (`target/phase40/after-final.txt` == `target/phase41/base.txt`).

## Pinned level hashes

```
first-pour f0d2fdf9d2efb050
thick-syrup 1acd4f5d5af07d9c
jelly-pad d48dd7deeeb39443
float-up a94c5fd51fb29304
timed-gate bebe1822f5fb7ea9
grand-finale 88c192beb348641e
```

## Replay evidence (`target/phase41/replay-01.log`, `just puzzle-replay-wasm`, exit 0)

```
LEVELHASH first-pour pass f0d2fdf9d2efb050
LEVELHASH thick-syrup pass 1acd4f5d5af07d9c
LEVELHASH jelly-pad pass d48dd7deeeb39443
LEVELHASH float-up pass a94c5fd51fb29304
LEVELHASH timed-gate pass bebe1822f5fb7ea9
LEVELHASH grand-finale pass 88c192beb348641e
```

All 14 `REPLAY … pass` lines are unchanged from Phase 40. For example, first-pour solution won at 201 with 689/811/262, and the empty run lost at 720 with 0/0/none.

## Verification

- TDD RED, Rust: with a header-only pin file, `level_hashes_match_the_pin_file_in_level_order` failed and 8 tests passed. GREEN: `cargo test -p liquidfun-wasm --lib puzzle::level_hash` gave 9 passed and 1 ignored (the printer).
- TDD RED, TS: before the implementation, 6 tests failed across `puzzle-level-info` and `puzzle-level-hashes`. GREEN: those tests pass now.
- Rust pre-commit: `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -D warnings` (default members), the same clippy for `-p liquidfun-wasm` and for the `wasm32-unknown-unknown` lib, `cargo build -p liquidfun-wasm --all-targets --all-features`, and `cargo test -p liquidfun-wasm` (574 passed, 24 ignored) all passed. `bun scripts/bright-builds-check.ts all` reported 0 findings.
- TS: `bun run typecheck` passes after the wasm rebuild, and `bun run test:unit` passes 98 files with 1394 tests. `bun run build:app` succeeds.
- 25/25 check: scene-spot `2026-10-13T16-52-29Z`; `diff target/phase41/base.txt target/phase41/after-01.txt` is empty.
- `git diff --stat f277ecad9 -- crates/liquidfun` is empty. `levels_tests.rs` is still 620 lines.

## Deviations from Plan

None. The plan executed as written. I also ran wasm32 clippy and `build:app` beyond the listed pre-commit checks; both passed.

Note: `bun run typecheck` fails until the wasm package is regenerated, because the generated `.d.ts` lacks `levelHash`. `just puzzle-replay-wasm` regenerates it. That is expected ordering, not a defect.

## Known Stubs

None.

## Threat Flags

None. The getter is read-only. T-41-01 is covered by the fail-closed hex check and T-41-04 by the strict two-field, 16-hex, duplicate-rejecting parsers. The Rust pin test compares the whole ordered list, so a duplicate or missing id fails it. T-41-03 is covered by the empty engine diff and the 25/25 fingerprints.

## Self-Check: PASSED

All five created source files, both target/phase41 baseline files and commits 0cd2e0f31 and dfa7d00d8 exist.
