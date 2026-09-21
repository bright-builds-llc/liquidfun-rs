---
phase: 23-baseline-pair-and-named-audit
plan: "06"
subsystem: observability-tooling
tags: [xtask, playground-cli, dhat-heap, fake-cargo]

requires:
  - phase: 23-baseline-pair-and-named-audit
    provides: sidecar-first classify_profile_symbols and dam-break-heap xtask
provides:
  - fake-cargo dump writer for run plus dhat-heap
  - playground_cli heap success coverage for sibling dhat dump
  - playground_cli heap fail-closed skip coverage for no-allocator vs no-symbols
affects:
  - 23-07 live pair profile and audit-bundle stamps
  - 23-08 live heap run-or-skip notes

tech-stack:
  added: []
  patterns:
    - fake cargo run plus dhat-heap writes LIQUIDFUN_DHAT_HEAP_FILE before the profiling build path
    - heap CLI success mints a sibling stamp; skip paths write no dump

key-files:
  created: []
  modified:
    - tools/xtask/tests/playground_cli/heap.rs
    - tools/xtask/tests/fixtures/fake_upstream_tool.rs

key-decisions:
  - "Reorder fake cargo so run + dam-break-bench + dhat-heap writes LIQUIDFUN_DHAT_HEAP_FILE before --profile profiling install_profiling_bench."
  - "Keep the unprofiled run + dam-break-bench JSON printer and the profiling build path for dam-break-profile."
  - "Do not mark PERF-HEAP complete in this plan; live dump or skip notes remain 23-08."
  - "Skip CLI tests passed on first run because 23-05 already returns Err before mint_exclusive_stamp."

patterns-established:
  - "Fake cargo dhat-heap dumps are at least 16 nonempty bytes at LIQUIDFUN_DHAT_HEAP_FILE; not real gzip."
  - "Heap CLI tests use RepositoryFixture plus a local THIRD stamp unix for the sibling dump."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T04:24:57Z

duration: 6min
completed: 2026-09-21
---

# Phase 23 Plan 06: Fake-cargo heap CLI Summary

**Fake-cargo `dhat-heap` dump writer plus `playground_cli` tests proving `dam-break-heap` runs only on a needle match with `--profile profiling`, and fail-closes with distinct stderr and no dump for no-allocator vs no-symbols.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-09-21T04:19:22Z
- **Completed:** 2026-09-21T04:24:57Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments

- Reordered fake cargo so `run` + `dam-break-bench` + `dhat-heap` writes nonempty bytes to `LIQUIDFUN_DHAT_HEAP_FILE` and prints native_rust bench JSON, instead of hitting `install_profiling_bench`.
- Covered needle-match CLI success: sibling stamp gets nonempty `dhat-heap.json` and `heap-identity.json` with `kind=dhat_heap`, `not_timing_authority=true`, `cargo_profile=profiling`, and `dhat-heap` in features; cargo argv uses `--profile profiling` and never `--release`.
- Covered fail-closed skip CLI paths with distinct stderr and zero dumps.

## Task Commits

Each task was committed atomically:

1. **Task 1 RED: failing dam-break-heap CLI tests** - `735836f` (test)
2. **Task 1 GREEN: fake cargo dhat-heap dump writer** - `5540879` (feat)
3. **Task 2: skip-path CLI coverage** - `5567a42` (test)

**Plan metadata:** docs commit after STATE/ROADMAP updates

_Note: TDD tasks may have multiple commits (test → feat → refactor)_

## RED excerpt

Task 1 `cargo test -p xtask --test playground_cli -- --test-threads=1 heap` failed 3 tests against missing `dhat-heap.json` because fake cargo still treated `--profile profiling` as `install_profiling_bench`:

```text
error: playground/heap: missing .../2001-09-09T01-46-42Z/dhat-heap.json: No such file or directory
```

GREEN: 4 heap tests passed. After Task 2, 6 heap tests and 20 `playground_cli` tests passed.

## Files Created/Modified

- `tools/xtask/tests/fixtures/fake_upstream_tool.rs` — dhat-heap run writes `LIQUIDFUN_DHAT_HEAP_FILE` then native_rust JSON
- `tools/xtask/tests/playground_cli/heap.rs` — success, cargo-argv, source-stamp, no-allocator, and no-symbols CLI tests

## Physical line counts

Counted with Python `sum(1 for _ in path.open())` including blanks and comments:

| File | Lines | Cap |
| --- | ---: | ---: |
| `tools/xtask/tests/playground_cli/heap.rs` | 274 | 628 |
| `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | 282 | 628 |

## Decisions Made

- Fake cargo must match `run` + `dam-break-bench` + `dhat-heap` **before** `--profile profiling`, or heap CLI never writes the dump (Pitfall 3 / D-11).
- Unprofiled pair fake (`run` + `dam-break-bench` without `dhat-heap`) and profiling **build** (`install_profiling_bench`) stay in place.
- Do not mark REQUIREMENTS `PERF-HEAP` complete: this plan is fake-tool CLI coverage; live dump or skip notes remain 23-08.
- Task 2 skip tests needed no production patch; 23-05 already returns `PlaygroundError` kind `heap` before mint.

## Deviations from Plan

None - plan executed exactly as written.

---

**Total deviations:** 0 auto-fixed
**Impact on plan:** Task 2 tests passed on first run because skip-before-mint already shipped in 23-05. That is planned TDD against existing wiring, not a production change.

## Issues Encountered

Task 2 RED did not fail: no-allocator and no-symbols CLI paths already return nonzero from `heap::run` without spawning cargo. Tests were committed as coverage, not as a GREEN production patch.

## Known Stubs

None. Fake dumps remain fixture bytes under `target/xtask-test-fixtures/`; they cannot rank physics functions (D-01).

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-07 live pair, profile, and audit-bundle stamps.
- Do not run a live 60+600 dhat here (23-08).
- Do not enable `dhat-heap` on the unprofiled `--release` pair argv.
- Do not treat a dhat dump as the 3× number.
- Do not `git add` `dhat-heap.json` or `*.json.gz`.

## Self-Check: PASSED

---
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*
