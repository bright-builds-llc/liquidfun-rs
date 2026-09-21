---
phase: 23-baseline-pair-and-named-audit
plan: "04"
subsystem: observability-tooling
tags: [dhat, dhat-heap, dam-break-bench, isolation]

requires:
  - phase: 22-observability-shell
    provides: unprofiled dam-break-bench gate binary and pair.rs --release argv
provides:
  - optional non-default dhat-heap feature on liquidfun-wasm
  - cfg-gated dhat Alloc and Profiler in dam-break-bench only
  - Cargo.lock record of crates.io dhat 0.3.3
affects:
  - 23-05 heap command wiring
  - 23-06 heap CLI tests

tech-stack:
  added: [dhat 0.3.3]
  patterns:
    - optional crate-local dhat-heap = ["dep:dhat"] never default
    - #[global_allocator] lives only in dam-break-bench.rs behind cfg(all(feature = "dhat-heap", not(target_arch = "wasm32")))

key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/Cargo.toml
    - crates/liquidfun-wasm/src/bin/dam_break_bench.rs
    - Cargo.lock

key-decisions:
  - "Keep dhat 0.3.3 crate-local on liquidfun-wasm; do not add it to workspace.dependencies or liquidfun."
  - "Honor LIQUIDFUN_DHAT_HEAP_FILE as an OsString path; unset env may use Profiler::new_heap()."
  - "Do not mark PERF-HEAP complete in this plan; xtask heap spawn and run-or-skip remain 23-05 and 23-08."
  - "pair.rs cargo argv stays --release without --features so the gate binary stays uninstrumented."

patterns-established:
  - "Heap instrumentation is a private optional bin feature; the unprofiled --release gate path never enables it."
  - "dhat unsafe stays inside the dhat crate; workspace unsafe_code = forbid is unchanged."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 23-2026-09-21T02-44-37
generated_at: 2026-09-21T03:41:20Z

duration: 5min
completed: 2026-09-21
---

# Phase 23 Plan 04: Optional dhat-heap Summary

**Private optional `dhat` 0.3.3 heap instrumentation on `dam-break-bench` only, never default and never a `liquidfun` dependency.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-09-21T03:36:57Z
- **Completed:** 2026-09-21T03:41:20Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added non-default `dhat-heap = ["dep:dhat"]` and optional `dhat` 0.3.3 on private `liquidfun-wasm` only.
- Wired `#[global_allocator]` plus `dhat::Profiler` in `src/bin/dam_break_bench.rs` under `cfg(all(feature = "dhat-heap", not(target_arch = "wasm32")))`, honoring `LIQUIDFUN_DHAT_HEAP_FILE`.
- Left `lib.rs`, `dam-break-timers`, `liquidfun`, and `pair.rs` `--release` argv uninstrumented; `cargo deny --locked check` passed.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add optional dhat-heap on dam-break-bench only** - `bcbdb69` (feat)
2. **Task 2: Prove default gate binary stays unfeatured and deny/tree isolation** - no commit (verification-only; command outputs recorded below)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Files Created/Modified

- `crates/liquidfun-wasm/Cargo.toml` — empty default features, `dhat-heap = ["dep:dhat"]`, optional `dhat` 0.3.3
- `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` — cfg-gated `dhat::Alloc` and Profiler before parse/run; still prints JSON on stdout
- `Cargo.lock` — crates.io `dhat` 0.3.3 plus its optional-dep graph (`backtrace`, `mintex`, `thousands`, …)

## Isolation evidence (Task 2)

```text
$ cargo check -p liquidfun-wasm --bin dam-break-bench
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.38s
CHECK_NO_FEATURES_EXIT:0

$ cargo tree -p liquidfun --edges normal
liquidfun v0.0.0 (/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun)
└── bitflags v2.13.0

$ cargo tree -p liquidfun-wasm --edges features -i dhat
error: package ID specification `dhat` did not match any packages

$ cargo tree -p liquidfun-wasm --features dhat-heap -i dhat
dhat v0.3.3
└── liquidfun-wasm v0.0.0 (/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun-wasm)

$ cargo deny --locked check
advisories ok, bans ok, licenses ok, sources ok
DENY_EXIT:0

$ rg -n 'name = "dhat"' Cargo.lock
928:name = "dhat"

$ rg -n 'dhat-heap|--features' tools/xtask/src/playground/pair.rs
(no matches)

$ git diff -- crates/liquidfun/Cargo.toml crates/liquidfun/src
(empty)
```

`pair.rs` `run_rust_bench` argv remains `cargo run -p liquidfun-wasm --release --quiet --bin dam-break-bench` with no `--features`. Did not run Dam Break. Did not add xtask heap command or just aliases.

## Decisions Made

- Keep `dhat` 0.3.3 crate-local on `liquidfun-wasm`; do not add it to `[workspace.dependencies]` or `crates/liquidfun`.
- Place `#[global_allocator]` at bin item scope (not inside `fn run`, not in `lib.rs`) with the plan's `not(target_arch = "wasm32")` cfg.
- Honor `LIQUIDFUN_DHAT_HEAP_FILE` as `OsString`; unset may call `Profiler::new_heap()`. This plan does not run the bin, so it does not create a repo-root dump.
- Do not mark REQUIREMENTS `PERF-HEAP` complete: feature compile is this plan; heap spawn is 23-05; run-or-skip notes are 23-08.

## Deviations from Plan

None - plan executed exactly as written.

The plan's `<interfaces>` snippet showed `#[global_allocator]` next to the Profiler inside `fn run`. Rust requires the allocator attribute at item scope, which matches RESEARCH Pattern 2. That is the specified wiring, not a scope change.

---

**Total deviations:** 0 auto-fixed
**Impact on plan:** None.

## Issues Encountered

None. `dhat` 0.3.3 downloaded from crates.io and compiled. `cargo deny --locked check` reported existing duplicate-version warnings (testbed graph) and exited 0; `deny.toml` was not weakened.

## Known Stubs

None. Unset `LIQUIDFUN_DHAT_HEAP_FILE` still uses dhat's default cwd `dhat-heap.json`; 23-05 must set the env to a stamp path (Pitfall 7). That is deferred command wiring, not a stub in this plan.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for 23-05: allocator/`Vec` heuristic and `dam-break-heap` spawn with `--profile profiling --features dhat-heap` and `LIQUIDFUN_DHAT_HEAP_FILE`.
- Do not enable `dhat-heap` on the unprofiled `--release` pair argv.
- Do not add `dhat` to `liquidfun`.
- Do not treat dhat dump timings as the 3× number.

## Self-Check: PASSED

---
*Phase: 23-baseline-pair-and-named-audit*
*Completed: 2026-09-21*
