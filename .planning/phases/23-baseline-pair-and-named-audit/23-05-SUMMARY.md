---
phase: 23-baseline-pair-and-named-audit
plan: "05"
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:16:03Z
subsystem: observability-tooling
tags: [xtask, playground-cli, dhat-heap, samply-symbols]

requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: optional dhat-heap on dam-break-bench and copy-only audit-bundle stamps
provides:
  - sidecar-first classify_profile_symbols allocator/Vec heuristic
  - dam-break-heap xtask that spawns dhat only after needle match
  - just playground-dam-break-heap one-line alias
affects:
  - 23-06 fake-cargo heap CLI tests
  - 23-08 live heap run-or-skip notes

tech-stack:
  added: []
  patterns:
    - sidecar-first symbol scan via Path::with_extension("syms.json") then *syms* then gzip JSON
    - heap cargo uses --profile profiling --features dhat-heap and never --release

key-files:
  created:
    - tools/xtask/src/playground/symbols.rs
    - tools/xtask/src/playground/heap.rs
  modified:
    - tools/xtask/src/playground.rs
    - tools/xtask/src/main.rs
    - justfile

key-decisions:
  - "Scan rust.json.syms.json first (Path::with_extension on rust.json.gz), then other *syms* files, then decompressed gzip JSON strings."
  - "Hex-only 0x addresses and non-JSON gzip are SkipNoSymbols, not no-allocator."
  - "Heap cargo is --profile profiling --features dhat-heap and never --release so target/release/dam-break-bench stays the gate binary."
  - "SkipNoAllocator and SkipNoSymbols return Err before mint_exclusive_stamp or cargo spawn."
  - "Do not mark PERF-HEAP complete in this plan; live dump or skip notes remain 23-08."
  - "Run playground unit tests as cargo test -p xtask playground because xtask has no lib target."

patterns-established:
  - "just playground-dam-break-heap is a one-line cargo xtask printer with no cmake, samply, dhat, or --features flags."
  - "heap-identity.json is kind dhat_heap with not_timing_authority true and dump under a sibling exclusive stamp."
  - "clone alone is not a needle; core::clone:: is."

requirements-completed: []

duration: 10min
completed: 2026-09-21
---

# Phase 23 Plan 05: Gated dam-break-heap Summary

**Fail-closed sidecar-first allocator/`Vec` heuristic plus `dam-break-heap` that runs `dhat` only after a needle match, using `--profile profiling --features dhat-heap` so the unprofiled `--release` gate binary stays clean.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-09-21T04:06:07Z
- **Completed:** 2026-09-21T04:16:03Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments

- Added `classify_profile_symbols`: scan `rust.json.syms.json` first, then other `*syms*` files, then gzip JSON (`threads[].stringArray` or recursive strings).
- Distinguished needle match (`HeapGate::Run`), readable symbols with zero needles (`SkipNoAllocator`), and missing/invalid/hex-only symbols (`SkipNoSymbols`).
- Wired `cargo xtask playground dam-break-heap` and `just playground-dam-break-heap` so dhat runs only after a match, into a sibling stamp with `LIQUIDFUN_DHAT_HEAP_FILE`.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: TDD classify_profile_symbols sidecar-first heuristic** - `225a38b` (test)
2. **Task 1 GREEN: implement sidecar-first classify_profile_symbols** - `3eb71b1` (feat)
3. **Task 2: Implement dam-break-heap command and one-line just alias** - `5395715` (feat)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## RED excerpt

Plan command `cargo test -p xtask --lib playground::symbols` failed first (no library target):

```text
error: no library targets found in package `xtask`
```

Binary-unit RED (`cargo test -p xtask playground::symbols -- --test-threads=1`) then failed 6 tests against `todo!("classify_profile_symbols")`.

GREEN: 6 symbol tests passed. After Task 2, `cargo test -p xtask playground -- --test-threads=1` passed 48 tests. `cargo clippy -p xtask --all-targets --no-deps -- -D warnings` exits 0.

## Files Created/Modified

- `tools/xtask/src/playground/symbols.rs` — fail-closed heuristic, gzip via `flate2::read::GzDecoder`, unit tests under `target/xtask-test-fixtures/`
- `tools/xtask/src/playground/heap.rs` — gated spawn, skip-before-mint, `heap-identity.json`, skip-path unit tests
- `tools/xtask/src/playground.rs` — `mod heap;`, dispatcher arm, five-command USAGE including `--stamp`
- `tools/xtask/src/main.rs` — playground help mentions audit bundle and private heap
- `justfile` — `playground-dam-break-heap` one-line alias after audit-bundle

## Physical line counts

Counted with Python `sum(1 for _ in path.open())` including blanks and comments:

| File | Lines | Cap |
| --- | ---: | ---: |
| `tools/xtask/src/playground.rs` | 71 | 628 |
| `tools/xtask/src/playground/symbols.rs` | 336 | 628 |
| `tools/xtask/src/playground/heap.rs` | 469 | 628 |

## Decisions Made

- Sidecar-first: `Path::with_extension("syms.json")` on `rust.json.gz` yields `rust.json.syms.json` even when the gzip is raw non-JSON bytes.
- Address-only `0x…` strings and uncompressed `fake-samply-json-gz` are `SkipNoSymbols` (`could not read symbols`), not no-allocator.
- `clone` in `MycloneHelper` does not match; `core::clone::` does.
- Heap cargo argv is `run -p liquidfun-wasm --profile profiling --bin dam-break-bench --features dhat-heap` and never `--release` (D-11 / Pitfall 3).
- Skip paths return `PlaygroundError` kind `heap` before `mint_exclusive_stamp`. Matching paths mint a sibling stamp and set `LIQUIDFUN_DHAT_HEAP_FILE` to `{stamp}/dhat-heap.json`.
- Do not mark REQUIREMENTS `PERF-HEAP` complete: this plan ships the gate and command; live dump or skip notes remain 23-08; fake-cargo CLI is 23-06.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] xtask has no lib target**
- **Found during:** Task 1 RED verification
- **Issue:** Plan command `cargo test -p xtask --lib playground::symbols` fails with `no library targets found in package xtask` (same as 22-01/23-02).
- **Fix:** Ran `cargo test -p xtask playground::symbols -- --test-threads=1` and later `cargo test -p xtask playground -- --test-threads=1` against binary unit tests.
- **Files modified:** none (verification command only)
- **Verification:** RED 6 failed; GREEN 6 symbol + 48 playground passed
- **Committed in:** n/a

***

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for the xtask binary-only test layout. No live Dam Break, no fake-cargo dump writer, and no samply/dhat flags in just.

## Issues Encountered

The plan's acceptance `rg` snippet `dam-break-audit-bundle, or \`dam-break-heap\`` omits the closing backtick after `dam-break-audit-bundle`. Both `fn run` and `missing_subcommand_is_a_usage_error` use the exact interfaces string with backticks around each command. `cargo test -p xtask playground` covers both sites.

## Known Stubs

None. `todo!()` RED stubs were replaced in GREEN. Fake-cargo dump spawn coverage remains 23-06 by design, not a stub in this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-06: fake-cargo CLI tests in `tools/xtask/tests/playground_cli/heap.rs` only.
- Do not run a live 60+600 dhat here (23-08).
- Do not enable `dhat-heap` on the unprofiled `--release` pair argv.
- Do not treat a dhat dump as the 3× number.

## Self-Check: PASSED

***
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*

