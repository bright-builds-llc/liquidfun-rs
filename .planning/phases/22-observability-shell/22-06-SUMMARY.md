---
phase: 22-observability-shell
plan: "06"
subsystem: observability-tooling
tags: [package-isolation, benchmarking, gitignored-stamps, bright-builds]

requires:
  - phase: 22-observability-shell
    provides: exclusive stamp mint plus dam-break-profile and dam-break-timers dispatch
provides:
  - BENCHMARKING.md gitignored stamp note without public Dam Break numbers
  - package verify and cargo-tree proof that liquidfun stays bitflags-only
affects:
  - Phase 23 named-function audit
  - PERF-PAIR
  - PERF-PROFILE
  - PERF-TIMERS

tech-stack:
  added: []
  patterns:
    - Committed docs name gitignored stamp paths; they do not paste host dumps or fill manifest.toml
    - Isolation proof is cargo xtask package verify plus cargo tree -p liquidfun --edges normal

key-files:
  created: []
  modified:
    - BENCHMARKING.md
    - crates/liquidfun/src/world/particle_object.rs
    - crates/liquidfun-wasm/src/lib.rs
    - crates/liquidfun-wasm/src/session.rs
    - crates/liquidfun-wasm/src/scene/dam_break.rs
    - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
    - crates/liquidfun-wasm/src/scene/float_or_sink.rs
    - crates/liquidfun-wasm/src/scene/fountain.rs
    - crates/liquidfun-wasm/src/scene/jelly_drop.rs
    - crates/liquidfun-wasm/src/scene/water_wheel.rs

key-decisions:
  - "BENCHMARKING.md names gitignored target/dam-break-perf stamps and denies profiled/timer sibling stamps as the 3x number; it does not paste wall-ms."
  - "liquidfun production deps stay bitflags-only; cargo tree --edges normal is liquidfun -> bitflags only."
  - "Independent AI review remains a later policy step; this plan records automated evidence and does not self-approve the phase."

patterns-established:
  - "Exploratory Dam Break copy may name gitignored pair.json/pair.md paths; it must not become a Phase 12 reviewed claim."
  - "just playground-dam-break-{bench,profile,timers} stay one-line cargo xtask aliases."

requirements-completed: [PERF-PAIR, PERF-PROFILE, PERF-TIMERS]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T00:54:20Z

duration: 8min
completed: 2026-09-21
---

# Phase 22 Plan 06: Isolation and Honest Benchmarking Copy Summary

**Package isolation still holds after the observability shell: `liquidfun` stays `bitflags`-only, `BENCHMARKING.md` names gitignored `target/dam-break-perf/<utc-stamp>/` stamps without pasting pair numbers, and Bright Builds file-length plus `package verify` pass.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-09-21T00:46:04Z
- **Completed:** 2026-09-21T00:54:20Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added one Exploratory local diagnosis note in `BENCHMARKING.md`: a successful unprofiled pair writes gitignored `target/dam-break-perf/<utc-stamp>/pair.json` and `pair.md`; profile and timer sibling stamps are not the 3× number; do not commit `.json.gz` / `.trace` or copy pair numbers into `reference/performance/manifest.toml`.
- Proved `cargo xtask package verify`, `cargo tree -p liquidfun --edges normal` (`liquidfun` → `bitflags` only), `just markdown-check`, focused clippy/tests, and `bun scripts/bright-builds-check.ts all`.
- Left `docs/playground-dam-break-timing.md` and `reference/performance/manifest.toml` (`reviewed_reports = []`) unmodified. Did not run a live Dam Break pair. Did not self-approve the phase.

## Task Commits

Each task was committed atomically:

1. **Task 1: Document gitignored pair stamps in BENCHMARKING.md** - `44da072` (docs)
2. **Task 2: Isolation, recipe, and Bright Builds gates** - `33a7ab6` (style)

**Plan metadata:** docs commit after STATE/ROADMAP updates

## Command Evidence

`cargo xtask package verify`:

```text
package verified: 238 entries built and tested outside the repository
```

`cargo tree -p liquidfun --edges normal`:

```text
liquidfun v0.0.0 (/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun)
└── bitflags v2.13.0
```

`cargo fmt --all --check`: exit 0.

`cargo clippy -p xtask -p liquidfun-wasm -p liquidfun --all-targets --all-features -- -D warnings`: exit 0 after lint-only fixes recorded below.

`cargo test -p xtask --test playground_cli -- --test-threads=1`: 8 passed.

`cargo test -p liquidfun-wasm --lib dam_break_timers -- --test-threads=1`: 5 passed.

`bun scripts/bright-builds-check.ts all`:

```text
SUMMARY file-lengths scanned=1117 exceptions=0 findings=0
SUMMARY lessons sources=1 lessons=11 bytes=7560 estimated_tokens=2520 findings=0
SUMMARY all findings=0
```

`just markdown-check`: exit 0.

Static honesty checks: `justfile` has three one-line `cargo xtask playground dam-break-{bench,profile,timers}` aliases and no `samply`/`cmake`; workspace has `[profile.profiling]` and no `[profile.release]`; `crates/liquidfun/Cargo.toml` `[dependencies]` is `bitflags` only; `step_profiled` is absent from `dam_break_bench.rs` / `bin/dam_break_bench.rs`; no `playground/mod.rs`; `git ls-files '*.json.gz' '*.trace'` is empty; `git diff -- docs/playground-dam-break-timing.md reference/performance/manifest.toml` is empty.

## Files Created/Modified

- `BENCHMARKING.md` — gitignored stamp path, not-the-3× sibling stamps, no gzip/trace commit
- `crates/liquidfun/src/world/particle_object.rs` — clippy `doc_markdown` backticks on historical budgets
- `crates/liquidfun-wasm/src/scene/dam_break.rs` — digit separators on locked recipe literals
- `crates/liquidfun-wasm/src/scene/dam_break/tests.rs` — matching radius literal separators
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` — matching radius/spacing separators
- `crates/liquidfun-wasm/src/scene/fountain.rs` — emit-spacing separators
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` — radius separators
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` — radius and emit-spacing separators
- `crates/liquidfun-wasm/src/session.rs` — matching test radius separators
- `crates/liquidfun-wasm/src/lib.rs` — consume `pointer_action` in the wasm existence test

## Decisions Made

- Followed D-14: one BENCHMARKING.md sentence (split for wrap) names gitignored stamps and forbids gzip/trace commit and manifest copies. Did not paste wall-ms, ratios, host names, or CPU brands.
- Followed D-12: isolation is `package verify` plus `cargo tree`; production `liquidfun` deps remain `bitflags` only. Did not add serde, samply, dhat, CMake, or flate2.
- Followed D-06/D-07: just recipes stay one-line cargo xtask aliases.
- Independent AI review is required later by owner policy. This plan only records automated evidence and does not approve its own work.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Clippy `doc_markdown` on historical particle budgets**
- **Found during:** Task 2 (`cargo clippy -p liquidfun ... -D warnings`)
- **Issue:** Pre-existing `4_096 / 16_384 / 2_000_000 / 8_192` in `particle_object.rs` docs (commit `3a047bf`) failed `-D warnings`.
- **Fix:** Wrapped each number in backticks. No physics or dependency change.
- **Files modified:** `crates/liquidfun/src/world/particle_object.rs`
- **Verification:** clippy on liquidfun/xtask/wasm exits 0
- **Committed in:** `33a7ab6`

**2. [Rule 3 - Blocking] Clippy `unreadable_literal` on playground recipe decimals**
- **Found during:** Task 2 (same clippy command)
- **Issue:** Locked scene radii/spacings lacked digit separators; `dam_break_bench.rs` already allowed or formatted the same decimals.
- **Fix:** Inserted `_` separators (`0.063_245_55`, `0.101_193`, and the other clippy-named literals). Values unchanged.
- **Files modified:** wasm scene sources plus matching tests in `session.rs` and `dam_break/tests.rs`
- **Verification:** clippy exits 0
- **Committed in:** `33a7ab6`

**3. [Rule 3 - Blocking] Clippy `no_effect_underscore_binding` on wasm pointer_action test**
- **Found during:** Task 2 (lib-test clippy)
- **Issue:** `_method` / `_js_name` bindings had no side effect under `-D warnings`.
- **Fix:** Bind `method` and assert `size_of_val`; keep the `"pointerAction"` string assert.
- **Files modified:** `crates/liquidfun-wasm/src/lib.rs`
- **Verification:** clippy exits 0
- **Committed in:** `33a7ab6`

***

**Total deviations:** 3 auto-fixed (3 blocking, lint-only)
**Impact on plan:** Required isolation clippy command now passes. No profiler/serde leak into `liquidfun`. No public timing-doc refresh.

## Issues Encountered

The plan's clippy command covers `liquidfun` and `liquidfun-wasm` with `--all-targets --all-features`, which earlier Phase 22 plans did not run. Pre-existing pedantic lints in playground scenes and particle docs blocked that command until the lint-only fixes above.

## User Setup Required

None - no external service configuration required. Live samply / Dam Break pair remain optional developer proof, not definition of done.

## Next Phase Readiness

- Phase 22 plans 01–06 now have summaries. Isolation, one-line just aliases, and honest BENCHMARKING.md copy are in place.
- Ready for independent AI review of Phase 22 (implementing agent must not approve its own work).
- Ready for Phase 23 named-function audit. Do not treat this SUMMARY as that review. Do not fill `manifest.toml`. Do not quote profiled or timer walls as the 3× number.

## Self-Check: PASSED

---
*Phase: 22-observability-shell*
*Completed: 2026-09-21*
