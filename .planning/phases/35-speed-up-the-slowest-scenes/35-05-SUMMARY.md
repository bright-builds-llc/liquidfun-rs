---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-09T09:16:59.651Z
phase: 35-speed-up-the-slowest-scenes
plan: "05"
subsystem: particle-solver
tags: [rust, performance, body-contacts, spatial-query, bitset, fingerprint]

requires:
  - phase: 35-04
    provides: "Engine equal to A1 (spot-base-04), ABBA method, abba.sh, keep_rule.py, before-full.jsonl"
provides:
  - "A5 kept: body-contact candidate rows ordered by a reused bitset walk instead of sort_unstable + dedup (216de3929)"
  - "35-PROFILES.md Attempts rows A4 (reverted, never committed) and A5 (kept), run notes and Target records for liquid-tumbler, tesla-valve and stacked-drip"
  - "Saved diff target/phase35/attempts/A4.patch (per-row AABB query plus its equivalence tests)"
affects: [35-06, 35-07, 35-08, 36]

tech-stack:
  added: []
  patterns:
    - "Caller-owned scratch bitset that is all zero between calls: words are cleared as they are read"
    - "Small-input fallback keeps the original sort below a fixed row count"

key-files:
  created:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-05-SUMMARY.md
  modified:
    - crates/liquidfun/src/particle/body_contact.rs
    - crates/liquidfun/src/particle/body_contact/tests.rs
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "A4 reverted: per-y-row binary search in visit_sorted_tag_indices_in_aabb gained on no target and put liquid-tumbler, elastic-particles, liquid-timer, rigid-particles and surface-tension above the base max in both pairs; 25/25 fingerprints"
  - "A4 was never committed: it failed D-11 decisively, and a commit-then-revert would have cost a multi-hour full test cycle for code that leaves the tree at once (A3b precedent)"
  - "A5 kept: the ordered bitset walk cut liquid-tumbler 3.0% / 3.0% and tesla-valve 0.9% / 2.2% in both ABBA pairs, with 25/25 fingerprints and no confirmed regression"

patterns-established:
  - "Time from a recorded working-tree digest first, then run the full checks, then commit after re-checking the digest"

requirements-completed: []

duration: 422min
completed: 2026-10-09
---

# Phase 35 Plan 05: AABB Query and Candidate Rows Summary

**A5 is kept. It orders body-contact candidate rows with a reused bitset walk read by `trailing_zeros` instead of `sort_unstable` + `dedup`. That cut liquid-tumbler by 3.0% in both ABBA pairs and tesla-valve by 0.9% / 2.2%, with all 25 fingerprints bit-identical. A4, a per-row binary-searched AABB tag query, was slower. It gained on no target and regressed five scenes in both pairs, so it was reverted before commit.**

## Performance

- **Duration:** about 7 h of wall time in this session (2026-10-09T02:15Z to 09:17Z). An earlier, interrupted executor had already written the A4 code and tests and created `spot-base-05`. About 6 h went to the full checks under syspolicyd launch stalls: workspace clippy 20 min, workspace build 51 min, `cargo test -p liquidfun --all-features` 4 h 43 min. Timing took about 15 min in total.
- **Started:** 2026-10-09T02:15:43Z (this session)
- **Completed:** 2026-10-09T09:16:59Z
- **Tasks:** 3
- **Files modified:** 2 engine files (A5), plus 35-PROFILES.md

## Accomplishments

- **A4, per-row AABB tag query (reverted):**
  - Inside the unchanged `first..last` window, two ranged partition-point searches per tag row replaced the x-mask scan. A cost rule kept the linear scan when the searches would cost about as much.
  - The 4 equivalence tests against a verbatim copy of the scan passed: a grid, 2,000 random tags × 200 boxes, empty tags, and error cases.
  - Targeted ABBA against `spot-base-05`, pair 1 / pair 2:
    - tesla-valve +1.0% / +0.8%
    - stacked-drip −0.9% / +1.2%
    - liquid-tumbler +0.7% / +2.9%, above the base max in both pairs
  - Isolated ABBA on the 15 other scenes the full run flagged: elastic-particles, liquid-timer, rigid-particles and surface-tension were above the base max in both pairs. Three of them rose 3–6%.
  - All 25 fingerprints were equal.
  - The diff is in `target/phase35/attempts/A4.patch`.
  - Likely cause: the per-row searches' scattered probes cost more than the sequential scan they skip at these box sizes. The 13.7% self share in tesla-valve is real scan work, not waste.
- **A5, ordered bitset walk (kept, `216de3929`):**
  - `collect_candidate_rows` takes a `row_marks: &mut Vec<u64>` that `generate` allocates once per call.
  - With 32 or more visited rows, it sets one bit per row and reads the marked words in ascending order with `trailing_zeros`, clearing each word as it goes. It only walks the words between the lowest and highest marked row. Fewer rows keep the sort.
  - Targeted ABBA against `spot-base-05b`, pair 1 / pair 2:
    - liquid-tumbler 23.424 → 22.724 and 23.698 → 22.978
    - tesla-valve 3.839 → 3.805 and 3.920 → 3.833
    - stacked-drip 2.524 → 2.482 (a gain) and 2.584 → 2.546 (below the base max but not the min)
  - Fingerprints: 25/25 equal.
  - The full run flagged fountain, stacked-drip and washing-machine. None of them regressed in the isolated ABBA, and stacked-drip was below the base min in both isolated pairs.

## Task Commits

1. **Task 1: A4, per-y-row binary-searched AABB tag query:** no commit. A4 was timed from the working tree, failed D-11, and was saved to `attempts/A4.patch` (see Deviations).
1. **Task 2: A/B A4, keep or revert, and record:** `25bf95d68` (docs)
1. **Task 3: A5, ordered bitset walk; A/B, keep, and record:**
   - `216de3929` (perf)
   - `245413995` (docs)

## Files Created/Modified

- `crates/liquidfun/src/particle/body_contact.rs`: adds `MIN_ROWS_FOR_ROW_MARKS`, the `row_marks` parameter, `order_rows_with_marks`, and one `row_marks` buffer in `generate`.
- `crates/liquidfun/src/particle/body_contact/tests.rs`:
  - New tests: `candidate_rows_match_sorted_dedup` (500 particles on a scrambled grid, 6 boxes, one shared mark buffer, both paths exercised), `candidate_rows_small_query_uses_same_order` and `candidate_rows_failed_query_returns_false_and_empty`.
  - The two existing callers pass a mark buffer.
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: A4 and A5 rows, 35-05 run notes, and Target records.
- Local only (gitignored), all under `target/phase35/`:
  - Binaries: `bin/spot-base-05` and `bin/spot-base-05b` (hard links of `spot-base-04`), `bin/spot-A4`, `bin/spot-A5`
  - Attempt and provenance files: `attempts/A4.patch`, `A4.source-sha`, `A5.source-sha`
  - Timing output: `A4-*.jsonl`, `A4-iso-*`, `A5-*.jsonl`, `A5-iso-*`, and the `*.uptime` files
  - `A5-checks.log`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Process deviations

**1. [Process] A4 was never committed, so there is no commit-then-revert**
- **Found during:** Task 1 and Task 2
- **Issue:** The plan commits A4 after the full checks and reverts it if D-11 fails. I timed A4 first, as 35-04 did. It failed D-11 decisively, with no target gain and five scenes above the base max in both pairs. A commit-then-revert would have needed a full `cargo test` cycle under launch stalls (about 70 relinked test executables, roughly 4–5 h) for code that leaves the tree at once.
- **Action:** I saved the A4 diff, including the untracked test file, to `target/phase35/attempts/A4.patch` and checked that it applies cleanly. Then I restored `proxy.rs` and removed the test file. This is the same handling as A3b in 35-04.
- **Effect on acceptance:** "The last commit touching proxy.rs starts with Revert" does not apply, because no A4 commit exists. `git diff aed221fc0 HEAD -- crates/liquidfun/src/particle/proxy.rs` is empty. The A4 row records `none (never committed; diff in target/phase35/attempts/A4.patch)`.

**2. [Process] spot-base-05b is a hard link, not a rebuild**
- After the A4 decision the engine source equalled `aed221fc0`. So `spot-base-05b` is a hard link of `spot-base-05`, which is itself `spot-base-04`. This followed the host-condition guidance to avoid new executables.

**3. [Process] A5 tests ran against the old code through a temporary wrapper**
- The new tests first called a test-local `collect_rows` wrapper with the old 4-argument signature. All 3 passed on the old code, including the checks that both the sort path and the bitset path are exercised. The wrapper was then removed when the signature changed.

**4. [Rule 3 - Blocking] wasm32 check uses `--lib`**
- This carries over from 35-03, 35-04 and `deferred-items.md`. `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` passed.

**5. [Rule 1 - Lint] needless_range_loop**
- The first A5 draft indexed `row_marks` by range. Clippy rejected it, so the loop now iterates `row_marks[first_word..=last_word].iter_mut().enumerate()`. That draft was never timed. `spot-A5` was built after the fix.

---

**Total deviations:** 5, all process or lint. No authored scene settings changed.

## Issues Encountered

- **Shared host.** Another repository's `cargo test` ran twice just before timing windows, and `abba.sh` refused to time or I waited until it exited. Load ranged from 8.0 to 12.1 during A4 and from 5.3 to 8.0 during A5. The A4 full run was not interleaved and ran under rising load (9.9 to 11.3), so 16 scenes were listed. Only the isolated ABBA decided regressions.
- **Small margins.** In A5 pair 1, tesla-valve beat the base min by only 0.004 ms (3.805 vs 3.809). liquid-tumbler's −3.0% / −3.0% carries the keep decision on its own.
- **Verification evidence (A5, `216de3929`; working-tree digest `b357f353…` equal at timing and at commit):**
  - `cargo fmt --all --check`: pass
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
  - `cargo build --workspace --all-targets --all-features`: pass
  - `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass
  - `bun scripts/bright-builds-check.ts all`: 0 findings. `body_contact.rs` is 421 lines and `body_contact/tests.rs` is 556.
  - `cargo test -p liquidfun --all-features` (the same as root `cargo test --all-features`, since `default-members` is `liquidfun`): 75 `test result: ok`, exit 0. That is 1,066 passed and 0 failed; the lib has 472 passed, including the 3 new tests.
  - `cargo test -p liquidfun-wasm --all-features`: 316 passed, exit 0.
  - `keep_rule.py before-full.jsonl A5-full.jsonl ""`: `fingerprint mismatches: none`. The same holds for `A4-full.jsonl`.
- **Not run:** the full workspace test suite (`cargo test --workspace`, including xtask and differential). It is outside the required set and would queue many more fresh executables.

## Known Stubs

None.

## Next Phase Readiness

- HEAD engine = A1 + A5. Later plans should build their base from HEAD (`spot-A5` is that engine).
- tesla-valve's `visit_sorted_tag_indices_in_aabb` share is the scan itself. The per-row search did not pay. Its remaining large costs are emission and lifetime resequencing (35-07).
- PERF-08 and PERF-09 stay Pending.

## Self-Check: PASSED

- FOUND: .planning/phases/35-speed-up-the-slowest-scenes/35-05-SUMMARY.md
- FOUND: target/phase35/attempts/A4.patch
- FOUND: target/phase35/bin/spot-base-05, spot-base-05b, spot-A4, spot-A5
- FOUND: commit 25bf95d68
- FOUND: commit 216de3929
- FOUND: commit 245413995
- FOUND: `| A4 |` row with Decision `reverted` and Fingerprints `yes (25/25)`; `| A5 |` row with Decision `kept`, Fingerprints `yes (25/25)` and Commit `216de3929`
- FOUND: `trailing_zeros` in crates/liquidfun/src/particle/body_contact.rs; `git status --porcelain -- crates` empty
