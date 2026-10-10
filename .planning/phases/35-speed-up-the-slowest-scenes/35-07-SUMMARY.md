---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-10T00:12:01Z
phase: 35-speed-up-the-slowest-scenes
plan: "07"
subsystem: particle-storage-and-lifetime
tags: [rust, performance, particle-creation, eviction-index, hasher, fingerprint]

requires:
  - phase: 35-06
    provides: "Engine A1 + A5 + A6 (spot-A6), ABBA method, abba.sh, keep_rule.py, before-full.jsonl"
provides:
  - "A8 kept: ungrouped particle creation skips the O(n) group-lane clone and group-record rebuild (5695f4b39)"
  - "A9 kept: lifetime eviction index uses a deterministic in-tree hasher and an in-place resequence with bulk BTreeMap builds (f7041fc75)"
  - "35-PROFILES.md A8 and A9 rows, 35-07 run notes and the tesla-valve Target record"
affects: [35-08, 36]

tech-stack:
  added: []
  patterns:
    - "Fast path equal to the full rebuild by construction, with a debug_assert_eq! against the rebuild and tests against a verbatim copy of the old function"
    - "Deterministic BuildHasherDefault hasher for engine-issued keys in lookup-only maps"

key-files:
  created:
    - crates/liquidfun/src/particle/storage/creation_fast_path_tests.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-07-SUMMARY.md
  modified:
    - crates/liquidfun/src/particle/storage.rs
    - crates/liquidfun/src/particle/storage/creation.rs
    - crates/liquidfun/src/particle/lifetime/eviction.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "A8 kept: ungrouped creation reuses group_records (with the rebuild's empty-record normalization) instead of rebuilding them; tesla-valve -7.8% / -12.2% in ABBA, fountain and water-wheel about -20%, 25/25 fingerprints, no confirmed regression"
  - "A9 kept: eviction index with an in-tree ParticleIdHasher, in-place resequence and bulk BTreeMap builds (sub-steps a, b and c); tesla-valve -9.1% / -8.1%, fountain and water-wheel about -27%, 25/25 fingerprints, no confirmed regression"
  - "A plain clone of group_records is not equal to the rebuild: it would keep the cached statistics timestamp of an empty retained group, which the rebuild resets. The fast path re-applies retain_empty_after_member_removal to empty records so records stay identical"
  - "A8 and A9 were built and timed separately, then checked in one combined full check cycle"

patterns-established:
  - "When a fast path replaces a rebuild, test it against a verbatim copy of the old function and keep a debug_assert_eq! against the rebuild"

requirements-completed: []

duration: 411min
completed: 2026-10-10
---

# Phase 35 Plan 07: Tesla-Valve Emission and Lifetime Index Summary

**Both attempts are kept.**

- **A8:** ungrouped particle creation no longer clones the group lane or rebuilds group records on each call. Tesla-valve dropped from 3.90–3.96 to 3.48–3.59 ms/step.
- **A9:** the lifetime eviction index now uses a deterministic in-tree hasher and resequences in place. That took tesla-valve from 3.56–3.60 to 3.27 ms/step.
- **Together:** about −17% for tesla-valve. Fountain and water-wheel, which also emit particles, got about 40% faster (fountain 1.15 to 0.67 ms/step, water-wheel 1.23 to 0.74).
- All 25 fingerprints stayed bit-identical.

## Performance

- **Duration:** about 6 h 51 min (2026-10-09T17:20Z to 2026-10-10T00:12Z).
  - Coding and timing took about 1 h 10 min.
  - The first test build (`cargo build -p liquidfun --lib --tests`) took 31 min.
  - The full checks took 5 h 30 min: workspace clippy 21 min, workspace build 42 min, and `cargo test -p liquidfun --all-features` 4 h 15 min.
- **Started:** 2026-10-09T17:20:21Z
- **Completed:** 2026-10-10T00:12:01Z
- **Tasks:** 2
- **Files modified:** 4 engine files (3 modified, 1 new), plus 35-PROFILES.md

## Accomplishments

- **A8, `prepare_create` in `creation.rs`:**
  - For `maybe_group == None`, the new `group_records_after_ungrouped_append` clones `group_records` and calls `retain_empty_after_member_removal` on each empty record. The old code cloned `groups`, pushed `None` and ran `rebuild_group_records_for_system`, which includes `membership_ranges` and `validate_groups`.
  - **Why the result is equal:** an appended `None` adds no membership range. `validate_groups` already fixes the non-empty prefix to match the ranges in order, so `set_range` with the same range is a no-op. The rebuild re-applies the empty-record normalization to the trailing empty records, and the fast path does the same.
  - An explicit `groups.len() >= i32::MAX` check keeps the rebuild's lane limit.
  - A `debug_assert_eq!` compares the result with the full rebuild in every debug build.
  - The grouped path, the error checks and their order, and `solver_state.prepare_append` are unchanged.
  - `prepare_create` and `commit_create` became `pub(super)` so the tests in `storage` can use them.
- **A8 tests (`creation_fast_path_tests.rs`, 4 tests).** Each compares against a verbatim copy of the old `prepare_create`:
  - 300 ungrouped creates, also checking `validate_create_reserving` with one free slot.
  - 300 creates into a storage that has a 3-row group and an emptied `CAN_BE_EMPTY` group with a cached statistics timestamp.
  - Grouped creates, into an existing group and a new one.
  - Six error cases, where the precedence is capacity, then group, then identity.
  - Storage is compared with `ParticleStorage` `PartialEq`, and `group_records` with `assert_eq!`.
- **A9, `EvictionIndex` in `eviction.rs`:**
  - (a) `by_particle` is now a `HashMap<ParticleId, Placement, BuildHasherDefault<ParticleIdHasher>>`. The hasher mixes each written integer with a multiply-rotate step and finishes with SplitMix64. Nothing iterates this map.
  - (b) `resequence_to_storage_order` walks the infinite map, then the finite map in reverse. It assigns `POSITION_GAP * (i + 1)` and updates each placement through `get_mut`. It rebuilds both `BTreeMap`s with `collect` over keys that are already sorted, and sets `next_position = POSITION_GAP * (n + 1)`. These are the same values the old `from_ordered_entries` rebuild produced. A particle without a placement is skipped, as the old `filter_map` did.
  - (c) `from_ordered_entries` pre-sizes the map.
  - No new crate: `cargo tree -p liquidfun -e normal --depth 1` still lists only `bitflags`. `DefaultHasher` and `RandomState` do not appear in `eviction.rs`.
- **A9 tests.**
  - `resequence_matches_rebuild_from_entries` runs 2,000 fixed-seed upserts and removals with finite, zero and negative expirations, and 50 resequences. At every resequence it checks the whole index with `==`, `storage_order()`, and `oldest(rank)` for every rank. At every step it checks `storage_order()` and `oldest(0)`.
  - `index_equality_still_compares_contents` checks that the derived `PartialEq` still compares contents.
  - Before the change, both new tests passed against the original resequence body.
- **A/B results** (ABBA `--runs 5`, ms/step; base median (min–max) to after median):

  | Attempt | Scene | Pair 1 | Pair 2 |
  | --- | --- | --- | --- |
  | A8 vs `spot-base-07` | tesla-valve | 3.895 (3.876–3.928) → 3.593 | 3.960 (3.928–4.015) → 3.477 |
  | A8 | fountain | 1.147 → 0.915 | 1.157 → 0.918 |
  | A8 | water-wheel | 1.229 → 1.012 | 1.235 → 0.987 |
  | A8 | sparky | 0.134 → 0.136 (max 0.139) | 0.137 → 0.135 |
  | A9 vs `spot-base-07b` (= `spot-A8`) | tesla-valve | 3.598 (3.512–3.698) → 3.269 | 3.555 (3.514–3.608) → 3.266 |
  | A9 | fountain | 0.910 → 0.665 | 0.927 → 0.672 |
  | A9 | water-wheel | 0.993 → 0.739 | 1.012 → 0.733 |
  | A9 | sparky | 0.135 → 0.135 | 0.132 → 0.138 (max 0.133) |

- **Regression checks:**
  - A8: the full run listed 10 scenes while the load rose to 12.8. In the isolated ABBA of all 10, pair 1 listed 6 of them (load 7.4 → 13.1 during the after runs). Pair 2 listed none, so no regression is confirmed.
  - A9: sparky was over its base max in targeted pair 2 only. The full run listed 4 scenes. The isolated ABBA of those 4 plus sparky listed none in pair 1, and dam-break and jelly-drop in pair 2. No scene was over its base max in both pairs.
- **Fingerprints:** `keep_rule.py before-full.jsonl A8-full.jsonl ""` and the same check for `A9-full.jsonl` both printed `fingerprint mismatches: none` for all 25 scenes.

## Task Commits

1. **Task 1: A8, ungrouped-append fast path:** `5695f4b39` (perf), recorded in `0a508779a` (docs)
1. **Task 2: A9, cheaper lifetime eviction index:** `f7041fc75` (perf), recorded in `7e7e9ddb6` (docs)

## Files Created/Modified

- `crates/liquidfun/src/particle/storage/creation.rs` (466 lines): `group_records_after_ungrouped_append` and the fast-path branch in `prepare_create`
- `crates/liquidfun/src/particle/storage/creation_fast_path_tests.rs` (270 lines): the 4 A8 tests and the reference `prepare_create`
- `crates/liquidfun/src/particle/storage.rs`: the test module declaration
- `crates/liquidfun/src/particle/lifetime/eviction.rs` (397 lines): `ParticleIdHasher`, the in-place `resequence_to_storage_order`, and 2 new tests
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: A8 and A9 rows, 35-07 run notes, and the tesla-valve Target record
- Local only (gitignored), under `target/phase35/`:
  - Binaries: `bin/spot-base-07` (hard link of `spot-A6`), `bin/spot-A8`, `bin/spot-base-07b` (hard link of `spot-A8`) and `bin/spot-A9`
  - Provenance: `A8.source-sha`, `A9.source-sha`, `A9-final.source-sha` and `A8A9-tree.source-sha`
  - Timing output: `A8-*`, `A8-iso-*`, `A9-*` and `A9-iso-*` files (`.jsonl` and `.uptime`)
  - Logs: `A8-unit.log`, `A9-unit.log`, `A8A9-checks-1.log`, `A8A9-checks.log` and `checks-07.sh`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Correctness] The A8 fast path normalizes empty records instead of using a plain clone**
- **Found during:** Task 1, step 1 (checking the invariant)
- **Issue:** The plan's step 3 used `self.group_records.clone()`. Step 2 said not to ship a shortcut that changes records. The rebuild calls `retain_empty_after_member_removal` on every empty record. That resets `statistics.maybe_source_timestamp` to `None`, and `refresh_rigid_statistics` or `update_group_statistics` can set a timestamp on an empty retained group. A plain clone would keep the timestamp, so `group_records` would differ. `ungrouped_create_into_storage_with_groups_matches_reference` builds that state and asserts the timestamp before and after.
- **Fix:** The fast path clones the records and applies the same normalization to each empty record. This is O(number of groups). It equals the rebuild exactly, and the `debug_assert_eq!` checks that.
- **Commit:** `5695f4b39`

**2. [Rule 1 - Correctness] Explicit lane-length check in the fast path**
- **Issue:** The rebuild's `validate_groups` rejects a lane longer than `i32::MAX`. With `free_slots = 1`, `prepare_append` alone would not.
- **Fix:** `groups.len() >= i32::MAX` returns `InvalidGroupRange`, as the rebuild did.
- **Commit:** `5695f4b39`

**3. [Rule 3 - Blocking] Clippy lints after timing (doc backticks, `assert!` equality)**
- **Issue:** The first full check run failed clippy on two `doc_markdown` lints in the A9 hasher doc and two `manual_assert_eq` lints in the A9 tests (`A8A9-checks-1.log`).
- **Fix:** I reworded the doc comment and switched to `assert_eq!` and `assert_ne!`. A release rebuild was byte-identical to the timed `spot-A9` (SHA-256 `bd2b2eb5…`), so the timing evidence applies to the committed source.
- **Commit:** `f7041fc75`

### Process deviations

**4. Base binaries are hard links, not rebuilds**
- `spot-base-07` links to `spot-A6`; `git diff 2256dd8cd HEAD -- crates/` was empty.
- `spot-base-07b` links to `spot-A8`, since A8 was kept.

**5. A8 and A9 were timed before one combined full check cycle**
- This follows the host-condition guidance. Each attempt was judged on its own pair of binaries.
- The `5695f4b39` tree, A8 without A9, was not checked on its own. A8 and A9 touch independent files.
- The combined tree passed every check, and its digest was unchanged at commit time.

**6. Visibility change**
- `prepare_create` and `commit_create` are now `pub(super)` instead of private. The test module in `storage` needs them to run a verbatim copy of the old path.

**7. [Rule 3 - Blocking] wasm32 check uses `--lib`**
- This carries over from 35-03 to 35-06. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` passed.

**8. No A9 mutation check**
- Unlike 35-06, I did not mutate the code to show that the tests catch a wrong resequence. The test compares the whole index with `==`, including every placement position and `next_position`, against the original rebuild.

---

**Total deviations:** 8: 2 correctness hardenings of A8, 2 blocking fixes, and 4 process deviations. No authored scene settings changed (D-05).

## Issues Encountered

- **Shared-host load.** Timing waited three times for another repository's `cargo` to exit. The load still rose to 12.8–13.1 during the A8 full after run and the A8 isolated pair 1 after runs. Those runs listed scenes that pair 2 did not confirm.
- **Launch stalls.**
  - `cargo test -p liquidfun --all-features` took 4 h 15 min.
  - The `spot-A8` warm launch took 5 min. The `spot-A9` warm launch was immediate.
- **The stray empty file `=`** in the repo root stays untouched. `.planning/config.json` was not staged.
- **Verification evidence (final tree, A8 + A9, digest `005912fb…`):**
  - `cargo fmt --all --check`: pass
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
  - `cargo build --workspace --all-targets --all-features`: pass
  - `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass
  - `bun scripts/bright-builds-check.ts all`: 0 findings
  - `cargo test -p liquidfun --all-features`: 75 `test result: ok`, 1,079 passed, 0 failed. The lib has 485 passed, including the 6 new tests.
  - `cargo test -p liquidfun-wasm --all-features`: 316 passed, 0 failed
  - Fingerprints: 25/25 equal to `before-full.jsonl` for both A8 and A9
- **Not run:** the full workspace test suite, the same scope as 35-05 and 35-06.

## Known Stubs

None.

## Next Phase Readiness

- The HEAD engine is A1 + A5 + A6 + A8 + A9. `spot-A9` is that engine.
- Tesla-valve's emission and lifetime costs are reduced. Re-profile before more work on it. The remaining named costs are the AABB scans and CCD queries.
- PERF-08 and PERF-09 stay Pending.

## Self-Check: PASSED

- FOUND: crates/liquidfun/src/particle/storage/creation_fast_path_tests.rs, with the 4 planned test names
- FOUND: `maybe_group.is_none()` and `debug_assert_eq!` in crates/liquidfun/src/particle/storage/creation.rs
- FOUND: no `DefaultHasher` or `RandomState` in crates/liquidfun/src/particle/lifetime/eviction.rs; `resequence_matches_rebuild_from_entries` passed
- FOUND: target/phase35/bin/spot-base-07, spot-A8, spot-base-07b and spot-A9
- FOUND: commits 5695f4b39, 0a508779a, f7041fc75 and 7e7e9ddb6
- FOUND: the `| A8 |` row (contains `membership_ranges` and a `%` share) and the `| A9 |` row, both with Decision `kept` and Fingerprints `yes (25/25)`
- `git status --porcelain -- crates` is empty
