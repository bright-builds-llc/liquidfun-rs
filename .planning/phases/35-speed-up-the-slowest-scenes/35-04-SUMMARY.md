---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-09T01:42:18.091Z
phase: 35-speed-up-the-slowest-scenes
plan: "04"
subsystem: particle-solver
tags: [rust, performance, chain-shape, ccd, body-contacts, fingerprint]

requires:
  - phase: 35-03
    provides: "A1 retained proxy order in the base, ABBA keep-rule method, spot-before and before-full.jsonl"
provides:
  - "35-PROFILES.md Attempts rows A3 (committed, then reverted) and A3b (never committed), both reverted under D-11, with run notes and the liquid-tumbler Target record"
  - "Saved diffs target/phase35/attempts/A3.patch and A3b.patch for a possible later retry"
affects: [35-05, 36]

tech-stack:
  added: []
  patterns:
    - "target/phase35/abba.sh: interleaved ABBA runner that refuses to time while cargo or rustc runs and logs uptime per leg"
    - "Diagnostic ABBA pairs taken after a keep/revert decision are labelled as such and never change the decision"

key-files:
  created:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-04-SUMMARY.md
  modified:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md
    - .planning/phases/35-speed-up-the-slowest-scenes/deferred-items.md

key-decisions:
  - "A3 reverted: hoisting the chain child edge cut liquid-tumbler 9.4% / 10.4% with 25/25 fingerprints, but soup-stirrer (no chain) exceeded its base max in both isolated pairs"
  - "A3b reverted: the no-branch Shape::Edge variant cut liquid-tumbler 7.6% / 7.3% with 25/25 fingerprints, but fountain (no chain) exceeded its base max in both isolated pairs"
  - "Neither regression reproduced in a later diagnostic pair; D-11 was applied as written and no third re-roll was run"

patterns-established:
  - "Run the full pre-commit suite only after timing, so syspolicyd launch stalls never overlap a timed run"

requirements-completed: []

duration: 659min
completed: 2026-10-09
---

# Phase 35 Plan 04: Chain Edge Hoist Summary

**Building each chain child `EdgeShape` once per child cut liquid-tumbler by about 7–10% in ABBA pairs, with all 25 fingerprints bit-identical. Both variants were still reverted under D-11. In each, a scene that never takes the chain path (soup-stirrer for A3, fountain for A3b) measured about 1.5% above its base max in both isolated pairs. A later diagnostic pair did not reproduce either regression.**

## Performance

- **Duration:** about 11 h of wall time (2026-10-08T14:43Z to 2026-10-09T01:42Z). About 8 h went to syspolicyd launch stalls during the two full `cargo test` runs, which took 4 h 54 min and 4 h 13 min. Each relinked test binary waited 2 to 5 minutes for its first launch. Hands-on work and timing took about 1.5 h.
- **Started:** 2026-10-08T14:43:35Z
- **Completed:** 2026-10-09T01:42:18Z
- **Tasks:** 2
- **Files modified:** 5 engine files committed and then reverted (net zero), plus 35-PROFILES.md and deferred-items.md

## Accomplishments

- **A3, the plan's change, committed as `0a2fe8cb8`:**
  - `body_contact::generate` built `maybe_child_edge` once per chain child and called `edge.distance_to_point` per particle.
  - CCD records carried `CcdChild { index, maybe_aabb, maybe_edge }`, and `push_fixture_particle_hit` cast against the prebuilt edge.
  - Bit-identity tests: `chain_fixture_contacts_match_per_particle_distance`, `polygon_fixture_contacts_unchanged_by_hoist` (both compare `to_bits()` against the per-particle legacy path under a rotated transform) and `ccd_children_carry_chain_edges_only`.
  - The full checks passed before the commit.
- **A3 timing against `spot-base-04`:**
  - liquid-tumbler went from 23.590 to 21.371 (pair 1) and from 23.815 to 21.336 (pair 2).
  - All 25 fingerprints equal `before-full.jsonl`.
  - Soup-stirrer was 1.171 and 1.181 against base max 1.160 and 1.176, a confirmed regression.
  - Result: diff saved to `attempts/A3.patch`, then `git revert` (`7f0b36b18`).
- **A3b, an extra attempt:**
  - Same idea with no new per-row branch. A chain child is queried as a prebuilt `Shape::Edge` at child index 0 through the same `Shape::distance_to_point` / `Shape::ray_cast` call the original code makes. CCD emits one record per chain child, in fixture-then-child order.
  - Its own tests passed: `ccd_chain_children_become_edge_records_in_child_order` and `edge_record_ray_casts_match_chain_child_ray_casts` (bit-compares hits).
  - liquid-tumbler went from 23.383 to 21.608 and from 23.365 to 21.651, with 25/25 fingerprints.
  - Fountain was 1.168 and 1.148 against base max 1.164 and 1.137, a confirmed regression.
  - Result: never committed; diff in `attempts/A3b.patch`.
- **Diagnostic pairs,** run after both decisions: fountain and soup-stirrer showed no regression for either binary. 35-PROFILES.md records this as information, not a decision input.

## Task Commits

1. **Task 1: Hoist the chain child edge in body-contact generation and CCD**: `0a2fe8cb8` (perf)
1. **Task 2: A/B the edge hoist, keep or revert, and record**:
   - `7f0b36b18` (revert of A3)
   - `b4c65f14e` (docs, 35-PROFILES.md A3 and A3b rows)

## Files Created/Modified

- `crates/liquidfun/src/particle/body_contact.rs`, `body_contact/tests.rs`, `world/particle_coupling.rs`, `particle_coupling/scratch.rs`, `particle_coupling/scratch_tests.rs`: changed in `0a2fe8cb8` and restored in `7f0b36b18`. `git diff b31eb61b9 HEAD -- crates/` is empty.
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: A3 and A3b rows, 35-04 run notes, liquid-tumbler Target record.
- `.planning/phases/35-speed-up-the-slowest-scenes/deferred-items.md`: a pre-existing release-only warning.
- Local only (gitignored), all under `target/phase35/`:
  - Binaries: `bin/spot-base-04` (byte-identical to `spot-A1`), `bin/spot-A3`, `bin/spot-A3b`
  - Diffs: `attempts/A3.patch`, `attempts/A3b.patch`
  - Timing output: `A3-*.jsonl`, `A3-iso-*`, `A3-diag-*`, `A3b-*.jsonl`, `A3b-iso-*`, `A3b-diag-*`, and the `*.uptime` files
  - `abba.sh`
  - Test logs: `A3-test-*.log`, `A3-revert-test.log`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Process deviations

**1. [Process] Timing ran before the A3 commit**
- **Issue:** The full checks take hours under launch stalls, and timing must not overlap cargo.
- **Fix:** I built `spot-A3` from the working tree and recorded the diff SHA-256 (`2d0965a6…`). I timed it with no cargo or rustc running, then ran the full checks and committed. The SHA matched at commit time, so the committed source equals the timed source.

**2. [Scope] Extra attempt A3b**
- **Issue:** A3 failed D-11 only on a scene that does not use chains. Both causes of a non-chain cost the plan's design could add (a per-row branch, a larger CCD child record) are removable without losing bit identity.
- **Action:** I tried one variant, A3b, under the same method and keep rule, as 35-03 did with A2. It also failed and was never committed. Because A3b is not on HEAD, the acceptance check "the last commit touching body_contact.rs starts with Revert" still holds.

**3. [Process] Diagnostic pairs after the decisions**
- Separate ABBA pairs for fountain and soup-stirrer ran after both decisions were fixed. Neither regression reproduced. I recorded them as diagnostics only. Re-running decision pairs until they pass would bias the keep rule, so neither decision changed.

**4. [Rule 3 - Blocking] wasm32 check uses `--lib`**
- This carries over from 35-03 and `deferred-items.md`. `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` fails on the native-only bins on unchanged code. I verified `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown` instead, and it passed on A3.

**5. [Process] No separate RED commit**
- Every commit must pass the full checks. I ran the A3 CCD test against the new type and it failed first: `ccd_children_carry_chain_edges_only` assumed fixture order, and the body lists fixtures newest-first. I fixed the test, not the code. The body-contact equivalence tests pass on both the old and new code by design.

---

**Total deviations:** 5, all process or scope. No authored scene settings changed: `git diff 0cc0c0986 HEAD --stat -- crates/liquidfun-wasm/src/scene/` is empty.

## Issues Encountered

- **syspolicyd launch stalls (environmental).** Fresh test executables waited 2 to 5 minutes each, one at a time. The two `cargo test --all-features` runs took 4 h 54 min (A3) and 4 h 13 min (after the revert). The workspace clippy and build took about 60 minutes. The spot binaries launched immediately.
- **Host noise and the regression check.** Load ranged from 6.2 to 10.5. Each attempt re-checked many flagged scenes (7 for A3, 12 for A3b), and one per attempt crossed its base max in both isolated pairs by about 1.5%. Neither reproduced in a later pair. Under this load, the D-11 regression check appears to give false positives often enough that a real bit-identical gain of about 8% on liquid-tumbler was rejected twice. Changing the rule needs the user's decision (D-11 is locked). One option is to re-attempt `attempts/A3b.patch` on a quieter host, or with a confirmation rule set before the run.
- **Verification evidence:**
  - On A3 (`0a2fe8cb8`):
    - `cargo fmt --all --check`: pass
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean
    - `cargo build --workspace --all-targets --all-features`: pass
    - `cargo build -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: pass
    - `bun scripts/bright-builds-check.ts all`: 0 findings; `particle_coupling.rs` is 513 lines, under the 628 limit
    - `cargo test --all-features`: 75 `test result: ok`, exit 0; lib 472 passed, including the 3 new tests
    - `cargo test -p liquidfun-wasm --all-features`: 316 passed
  - On A3b (uncommitted): `cargo test -p liquidfun --all-features --lib -- body_contact particle_coupling` passed 17 tests, including the 4 new ones.
  - After the revert (`7f0b36b18`, crates equal `b31eb61b9`): `cargo test -p liquidfun --all-features` gave 75 `test result: ok`, exit 0; lib 469 passed, doctests 22 passed.
  - `keep_rule.py before-full.jsonl A3-full.jsonl ""` and `... A3b-full.jsonl ""` both printed `fingerprint mismatches: none`.
- **Not run:** the full workspace test suite (`cargo test --workspace`, including xtask and differential). It is outside this plan's required set and would queue about 150 more fresh executables.
- **Pre-existing:** a release-only `unreachable expression` warning at `particle/solver/boundary/support.rs:98`, logged in deferred-items.md.

## Known Stubs

None.

## Next Phase Readiness

- HEAD engine code equals `b31eb61b9` (A1 kept). 35-05 should build its plan-start base from HEAD; `spot-base-04` is that binary.
- liquid-tumbler's chain edge hot spot (4.9% self) is still there. `attempts/A3b.patch` is the cleaner candidate if the user wants a retry under a different confirmation protocol.
- PERF-08 and PERF-09 stay Pending.

## Self-Check: PASSED

- FOUND: target/phase35/attempts/A3.patch
- FOUND: target/phase35/attempts/A3b.patch
- FOUND: target/phase35/bin/spot-base-04
- FOUND: commit 0a2fe8cb8
- FOUND: commit 7f0b36b18
- FOUND: commit b4c65f14e
- FOUND: `| A3 |` row in 35-PROFILES.md with Decision `reverted`, Fingerprints `yes (25/25)` and Commit `0a2fe8cb8`
- FOUND: last commit touching body_contact.rs starts with `Revert`
