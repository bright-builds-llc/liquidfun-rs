---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-10T01:37:28.342Z
phase: 35-speed-up-the-slowest-scenes
plan: "08"
subsystem: phase-close-out
tags: [performance, verification, fingerprint, abba, phase-summary]

requires:
  - phase: 35-07
    provides: "Final engine A1 + A5 + A6 + A8 + A9 (spot-A9), ABBA method, keep_rule.py, before-full.jsonl"
provides:
  - "Final verification record: 25/25 fingerprints equal, cumulative spot-before vs spot-final ABBA, Rust and web check results"
  - "35-PROFILES.md Phase summary, closed Target records and Notes for Phase 36"
affects: [36]

tech-stack:
  added: []
  patterns:
    - "Phase-level gain judged by a cumulative ABBA of the saved phase before binary against the final binary"

key-files:
  created:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-08-SUMMARY.md
  modified:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
    - .planning/phases/35-speed-up-the-slowest-scenes/deferred-items.md

key-decisions:
  - "Phase 35 close-out: final HEAD 202075791 is bit-identical to the phase before for all 25 scenes; cumulative ABBA puts all five targets below the before minimum in both pairs (liquid-tumbler -5.8%/-8.5%, tesla-valve -19.7%/-14.6%, stacked-drip -9.9%/-6.4%, washing-machine -31.8%/-31.7%, particles -5.1%/-7.4%)"
  - "spot-final is a hard link of spot-A9: crates/ unchanged since f7041fc75 and target/release/playground-scene-spot has the same SHA-256, so no new binary was linked"
  - "A3 and A3b stay reverted under D-11; the phase summary records the evidence that their flagged regressions may have been host noise and lists a retry as a Phase 36 note"

patterns-established:
  - "Close-out compares the saved before binary with the final binary in one ABBA set, not a chain of per-attempt gains"

requirements-completed: []

duration: 84min
completed: 2026-10-10
---

# Phase 35 Plan 08: Phase Close-Out Summary

**Phase 35 is closed. All 25 scenes behave bit-identically to the phase before run, and all five targets are faster than the phase before binary in both cumulative ABBA pairs. The required Rust and web checks pass. The only failures are two that predate this phase.**

## Performance

- **Duration:** about 84 min (2026-10-10 00:14Z to 01:38Z)
- **Tasks:** 2 of 2
- **Files modified:** 2 planning files, plus this SUMMARY

## Cumulative result (spot-before vs spot-final, `--runs 5`, before, final, final, before)

| Target | Before median (min-max), pair 1 / pair 2 | Final median, pair 1 / pair 2 | Change | Gain beyond noise |
| --- | --- | --- | --- | --- |
| liquid-tumbler | 25.135 (24.735-25.699) / 25.346 (25.133-25.454) | 23.670 / 23.204 | −5.8% / −8.5% | yes |
| tesla-valve | 4.056 (3.965-4.407) / 4.028 (3.980-4.061) | 3.258 / 3.440 | −19.7% / −14.6% | yes |
| stacked-drip | 2.764 (2.713-2.808) / 2.719 (2.658-2.766) | 2.489 / 2.546 | −9.9% / −6.4% | yes |
| washing-machine | 2.161 (2.094-2.280) / 2.182 (2.131-2.190) | 1.474 / 1.490 | −31.8% / −31.7% | yes |
| particles | 1.657 (1.627-1.673) / 1.713 (1.662-2.025) | 1.573 / 1.587 | −5.1% / −7.4% | yes (smallest margin) |

Timing ran 00:24:56–00:27:27Z. Load averages were 5.64 to 8.87. No cargo process from this repository was running. Another repository's `cargo test` was in progress, mostly waiting at launch, and a third repository's `cargo test` started at about 00:25Z. ABBA interleaving is the mitigation for that load.

## Accomplishments

- The final fingerprints were checked with `spot-final --runs 3 > target/phase35/final-full.jsonl` (25 lines). `keep_rule.py` against `before-full.jsonl` printed `fingerprint mismatches: none` (25/25) and no medians above the before max, so no bisect or revert was needed.
- Ran the cumulative ABBA for the five targets. All five gained in both pairs.
- The authored-behavior guard was empty: `git diff 809582431 HEAD --stat -- crates/liquidfun-wasm/src/scene/ web/src/` printed nothing. Only engine internals and tests under `crates/liquidfun/src/{particle,world}/` and planning files changed.
- Added three sections to 35-PROFILES.md:
  - `## Final verification`
  - `## Phase summary`, which has one row per target and all attempt IDs A0–A9 with their decisions. A3 and A3b stay reverted under D-11, with the noise evidence presented.
  - `## Notes for Phase 36`
- All five Target records are now `status: closed` with their final medians.

## Checks on the final HEAD (`2020757911f1757356d2c2ec80e55055aa0eeaa0`)

| Command | Result |
| --- | --- |
| `cargo fmt --all --check` | pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass |
| `cargo build --workspace --all-targets --all-features` | pass |
| `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` | **fail, pre-existing** (deferred 35-01: E0432 in the native-only bins `dam-break-bench`, `dam-break-timers` and `playground-scene-spot`) |
| `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` | pass (the wasm32 check used in 35-03 to 35-07) |
| `bun scripts/bright-builds-check.ts all` | pass |
| `cargo test --all-features` (default member `liquidfun`) | pass: 75 `test result: ok`, 1,079 passed, 0 failed (52.5 min; no launch stall this time) |
| `cargo test -p liquidfun-wasm --all-features` | pass: 316 passed |
| `just web-build` | pass (generated files unchanged) |
| `cd web && bun run test:unit` | pass: 56 files, 472 tests |
| `just web-smoke` (optional) | **failed, pre-existing and unrelated**: 61/62 passed. `e2e/rust-wasm-proof.spec.ts:170` expects the status text `Loading Rust/WASM session…`, which no file under `web/src/` contains. `web/` is unchanged since BEFORE_COMMIT. The failure is logged in deferred-items.md. |

Logs: `target/phase35/checks-08.log`, `checks-08.results`, `web-smoke-08.log`, `final.uptime`, `final-*.keep`.

## Task Commits

1. **Task 1: Final fingerprint, cumulative A/B, and full Rust and web checks** – `c3792ec29` (docs)
1. **Task 2: Phase summary and target-record closure** – `a46743c58` (docs)

## Decisions Made

- **The final binary was reused, not rebuilt.** `git diff f7041fc75 HEAD -- crates/` was empty, and `target/release/playground-scene-spot` had the same SHA-256 as `spot-A9` (`bd2b2eb5…`). `spot-final` was therefore created as a hard link of `spot-A9`. That avoided a new syspolicyd launch assessment and did not change which code was measured.
- **The "Gain beyond noise" column follows the plan's rule:** the final median must be below the before minimum in both cumulative pairs. Every target meets it. The percentages are the measured pair changes only and are not public claims.
- **A3 and A3b stay reverted under D-11.** The phase summary presents the evidence that their flagged regressions may have been host noise, and the retry is listed in Notes for Phase 36:
  - Neither flagged scene takes the chain path.
  - The diagnostic pairs did not reproduce the regressions.
  - The check fired once per attempt across 7 and 12 re-checked scenes.
  - The same kind of single-pair noise appears in A1 and A8.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] The wasm32 check needs `--lib`**
- **Found during:** Task 1, step 5
- **Issue:** The plan's command `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` fails on the native-only bins. This failure predates Phase 35 and is recorded in deferred-items.md under 35-01.
- **Fix:** I ran the plan's command and recorded it as a pre-existing failure. I also ran `--lib`, which passes and is what plans 35-03 to 35-07 used.
- **Files modified:** none
- **Commit:** c3792ec29 (recorded in 35-PROFILES.md)

**2. [Rule 2 - Completeness] Added `cargo test -p liquidfun-wasm --all-features`**
- **Found during:** Task 1, step 5
- **Issue:** `cargo test --all-features` runs only the default member `liquidfun`, so the scene and session tests were not covered.
- **Fix:** I added the wasm crate's test run, which passed 316/316.
- **Commit:** c3792ec29

**3. [Scope] The optional web-smoke failure was logged, not fixed**
- **Found during:** Task 1, step 5
- **Issue:** A stale e2e expectation is unrelated to Phase 35, and `web/` is unchanged.
- **Fix:** I added an entry to deferred-items.md.
- **Commit:** c3792ec29

## Issues Encountered

During timing, other repositories' `cargo test` runs shared the host, with load averages from 5.6 to 8.9. They are recorded in `final.uptime`, and ABBA interleaving is the mitigation.

## Known Stubs

None. This plan changed only planning records.

## Next Phase Readiness

Phase 36 can re-survey at `2020757911f` or a later commit with the same `crates/`. `docs/benchmarks/scene-survey.md` and the README are untouched and still hold the pre-phase table. PERF-08 and PERF-09 are left for the orchestrator's phase-complete step.

## Self-Check: PASSED

- FOUND: .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
- FOUND: .planning/phases/35-speed-up-the-slowest-scenes/35-08-SUMMARY.md
- FOUND: target/phase35/final-full.jsonl (25 lines)
- FOUND: c3792ec29
- FOUND: a46743c58
