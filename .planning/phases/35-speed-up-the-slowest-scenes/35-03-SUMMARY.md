---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-08T14:41:40.455Z
phase: 35-speed-up-the-slowest-scenes
plan: "03"
subsystem: particle-solver
tags: [rust, performance, particle-contacts, proxy-sort, fingerprint]

requires:
  - phase: 35-02
    provides: "Phase before binary, before-full.jsonl fingerprints, ABBA keep-rule method, target profiles"
provides:
  - "Retained sorted contact-proxy order across particle iterations (ProxyOrderCache in ParticleStorage)"
  - "35-PROFILES.md Attempts rows A1 (kept) and A2 (reverted) with Target records for liquid-tumbler, stacked-drip and particles"
affects: [35-04, 35-05, 36]

tech-stack:
  added: []
  patterns:
    - "Sort hints live in an always-equal newtype so ParticleStorage equality and rollback ignore them"
    - "Fast paths fall back to the original row-order path on any error so error identity is unchanged"

key-files:
  created: []
  modified:
    - crates/liquidfun/src/particle/contact_scan.rs
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/runtime.rs
    - crates/liquidfun/src/particle/storage/creation.rs
    - crates/liquidfun/src/particle/body_contact/tests.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "A1 kept: retained proxy order + sort_unstable_by_key((tag, row)) beat the base min in both ABBA pairs for liquid-tumbler, stacked-drip and particles, with 25/25 fingerprints equal"
  - "A2 reverted: the bounded insertion sort was faster than A1 on liquid-tumbler and stacked-drip, but particles and washing-machine regressed in both pairs"
  - "clear_contact_scan only moves a non-empty proxy buffer into the cache, so a repeated clear keeps the sort hint"

patterns-established:
  - "Before timing, check that no cargo/rustc process is running, and record uptime per run in target/phase35/<attempt>.uptime"

requirements-completed: []

duration: 732min
completed: 2026-10-08
---

# Phase 35 Plan 03: Proxy Order Reuse Summary

**Particle contact proxies now keep the previous iteration's sorted order and are re-sorted in place with the total `(tag, row)` key. This is bit-identical for all 25 scenes. In ABBA pairs it took liquid-tumbler from about 24.2–24.4 to 23.3 ms/step, with smaller gains for stacked-drip and particles. A bounded insertion-sort variant was tried and reverted.**

## Performance

- **Duration:** about 12 h 12 min of wall time. Almost all of it was macOS syspolicyd launch stalls. Assessments ran about 6 minutes per fresh executable, and the `spot-A2` warmup launch alone waited 3 h 14 min. Hands-on work took about 1.5 h.
- **Started:** 2026-10-08T02:29:54Z
- **Completed:** 2026-10-08T14:41:40Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- `rebuild_proxies` has a reuse branch. When the retained buffer has `positions.len()` entries, it retags every proxy in place and calls `sort_unstable_by_key(|proxy| (proxy.tag, proxy.row))`. A different length, an out-of-range row or any tag error falls back to the unchanged row-order rebuild, so errors come from the first failing row as before.
- `ParticleStorage` gains `proxy_order_cache: ProxyOrderCache`, a newtype whose `PartialEq` always returns `true`. `clear_contact_scan` swaps the sorted buffer into the cache, so `contact_proxies()` is still empty after a step. `take_contact_proxies` hands the cache back when the current proxies are empty.
- 6 behavior tests:
  - `retained_order_matches_fresh_rebuild_after_motion`
  - `retained_order_with_other_length_falls_back`
  - `retained_order_reports_the_same_tag_error`
  - `retained_order_reports_the_same_diameter_error`
  - `clear_contact_scan_empties_current_proxies_and_keeps_the_order`
  - `storage_equality_ignores_retained_proxy_order`

  A mutation check confirmed the motion test fails when the reuse-branch sort is removed.
- A1 A/B against `spot-base-03` (ABBA `--runs 5`). After-median for pair 1 / pair 2, with the base range in parentheses:
  - liquid-tumbler: 23.345 / 23.259 (base 24.179 / 24.404)
  - stacked-drip: 2.550 / 2.525 (base 2.628 / 2.668)
  - particles: 1.552 / 1.563 (base 1.576 / 1.627)

  All three are below the base minimum in both pairs. All 25 fingerprints in `target/phase35/A1-full.jsonl` equal `before-full.jsonl`. The full catalog run flagged fountain and drawing-particles; isolated ABBA reruns cleared both.
- A2 (the bounded insertion sort) against `spot-A1`:
  - liquid-tumbler: 21.587 / 21.679 (base 22.851 / 23.097)
  - stacked-drip: 2.341 / 2.347 (base 2.465 / 2.490)
  - particles regressed in both pairs: 1.545 / 1.560 vs A1 max 1.502 / 1.546
  - washing-machine regressed in both isolated pairs: 2.084 / 2.080 vs A1 max 2.042 / 2.058

  Fingerprints were 25/25 equal, but the keep rule failed. A2 was never committed; its diff is saved in `target/phase35/attempts/A2.patch`.

## Task Commits

1. **Task 1: Retain the sorted proxy order across particle iterations**: `91b27a6d6` (perf)
1. **Task 2: A/B A1 and A2, keep or revert, and record**: `b31eb61b9` (docs)

## Files Created/Modified

- `crates/liquidfun/src/particle/contact_scan.rs`: `ProxyOrderCache`, `retag_retained_order`, the reuse branch in `rebuild_proxies`, and 4 tests (452 lines)
- `crates/liquidfun/src/particle/storage.rs`: `proxy_order_cache` field
- `crates/liquidfun/src/particle/storage/creation.rs`: field initializer
- `crates/liquidfun/src/particle/storage/runtime.rs`: `take_contact_proxies` and `clear_contact_scan` cache handoff (582 lines, limit 628)
- `crates/liquidfun/src/particle/body_contact/tests.rs`: 2 storage tests
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: A1 and A2 rows, a run-notes paragraph, and Target records
- Local only (gitignored): `target/phase35/bin/spot-base-03`, `spot-A1`, `spot-A2`, `A1-*.jsonl`, `A1-iso-*.jsonl`, `A2-*.jsonl`, `A2-iso-*.jsonl`, `*.uptime`, and `attempts/A2.patch`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Process deviations

**1. [Process] Timing ran before the A1 commit; tests gated the commit afterwards**
- **Issue:** The first `cargo test --all-features` run stalled for over an hour on fresh test binaries. Timing must not overlap a cargo test run.
- **Fix:** I stopped that run, built `spot-A1` from the exact working tree (diff SHA `d5e1cc5b…`, identical at commit time), and ran the A1 and A2 timing with no cargo process running. Then I re-ran the full tests on the same source and committed only after they passed. The committed A1 source equals the timed binary's source.

**2. [Process] A2 was timed on a parked working tree**
- **Issue:** A1 was not committed yet when A2 was built.
- **Fix:** I saved the A2 file to the scratchpad and restored the exact A1 tree with `git checkout` plus the saved A1 patch, then compared diff SHAs. After the A1 commit I copied A2 back in, ran `git diff > target/phase35/attempts/A2.patch` against `91b27a6d6`, and ran `git checkout -- crates/`, as the plan specifies.

**3. [Rule 3 - Blocking] wasm32 check uses `--lib`**
- `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` fails before this plan because the native-only bins cannot compile for wasm32 (see 35-01 and `deferred-items.md`). I verified `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` instead, which passes.

**4. [Rule 1 - Robustness] `clear_contact_scan` keeps the cache on repeated clears**
- The plan's literal swap would move an already empty buffer into the cache on a second clear and lose the hint. The swap now runs only when the current proxies are non-empty. Results are unaffected either way; this only keeps the hint.

**5. [Process] No separate RED commit**
- Every commit must pass fmt, clippy, build and tests, so tests and implementation landed together. The equivalence tests pass on both the old and new code by design. The mutation check above shows that they catch a broken reuse path.

---

**Total deviations:** 5, all process or robustness. No scope change. No authored scene settings changed: `git diff 51041deb0 HEAD --stat -- crates/liquidfun-wasm/src/scene/` is empty.

## Issues Encountered

- **syspolicyd launch stalls (environmental).** Every new executable waited for a Gatekeeper and XProtect assessment, about 6 minutes each, serialized. My own parallel `--list` prewarm of 72 test binaries filled the queue, and killing those processes did not remove their queued assessments. That is why `spot-A2` waited 3 h 14 min. Prewarming only helps once the queue drains; it does not speed it up.
- **Host noise.** Load was 6.2–9.2 during timing. The base binary drifted about 3% between ABBA pairs, and the `spot-A1` numbers taken 5.5 h later were 1–3% faster than its first run. Each row therefore compares only binaries run back to back. The particles gain in A1 pair 1 is narrow: 1.552 against a base minimum of 1.567.
- **Verification evidence, with commands and results:**
  - `cargo fmt --all --check`: pass
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean (one pedantic `doc_markdown` finding was fixed before the commit)
  - `cargo build --workspace --all-targets --all-features`: clean
  - `cargo test --all-features` (default member `liquidfun`): 75 `test result: ok` lines, exit 0. This covers 469 lib tests (including the 6 new ones), all integration tests and 22 doctests.
  - `cargo test -p liquidfun-wasm --all-features`: 316 passed
  - `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass
  - `bun scripts/bright-builds-check.ts all`: 0 findings
  - `python3 target/phase35/keep_rule.py target/phase35/before-full.jsonl target/phase35/A1-full.jsonl ""`: `fingerprint mismatches: none`
- **Not run:** the full workspace test suite (`cargo test --workspace`, including xtask and differential). It was outside this plan's required set, and it would have queued about 150 more executables at about 6 minutes each.

## Known Stubs

None.

## Next Phase Readiness

- 35-04 (chain edge hoist) and 35-05 (AABB query and candidate rows) should build their plan-start base from `b31eb61b9` or later, so A1 is in the base.
- PERF-08 and PERF-09 stay Pending. One kept fix does not complete the phase.

## Self-Check: PASSED

- FOUND: crates/liquidfun/src/particle/contact_scan.rs (ProxyOrderCache, sort_unstable_by_key)
- FOUND: proxy_order_cache in storage.rs, runtime.rs, creation.rs
- FOUND: target/phase35/attempts/A2.patch
- FOUND: target/phase35/A1-full.jsonl (25 lines, fingerprints equal)
- FOUND: commit 91b27a6d6
- FOUND: commit b31eb61b9
