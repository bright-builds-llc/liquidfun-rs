---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 261010-ibo
generated_at: 2026-10-10T21:20:17.704Z
phase: quick-261010-ibo
plan: "01"
subsystem: particle-solver
tags: [rust, performance, chain-shape, ccd, body-contacts, fingerprint, abba]

requires:
  - phase: 35-04
    provides: "attempts/A3.patch and A3b.patch, abba.sh, keep_rule.py, before-full.jsonl"
  - phase: 35-08
    provides: "spot-final (final engine A1+A5+A6+A8+A9, SHA-256 bd2b2eb5...)"
provides:
  - "35-PROFILES.md Attempts rows A3r and A3br (both reverted under D-11), quick 261010-ibo run notes, liquid-tumbler target record, Phase summary and Phase 36 notes"
  - "target/phase35/attempts/A3r.patch and A3br.patch (diffs ported onto the final engine)"
  - "target/phase35/abba-gated.sh: ABBA runner with a pre-registered, result-blind quiet-host gate"
affects: [36]

tech-stack:
  added: []
  patterns:
    - "Result-blind quiet-host gate: wait before each leg; void a set on cargo/rustc or load > 10 at leg end; at most 2 re-runs, the third counts"

key-files:
  created:
    - .planning/quick/261010-ibo-retry-phase-35-chain-edge-hoist-a3-and-a/261010-ibo-SUMMARY.md
  modified:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "A3r reverted under D-11: liquid-tumbler -8.8% / -10.4% with 25/25 fingerprints, but jelly-drop and water-wheel (no chain fixture) were above the base max in both isolated pairs"
  - "A3br reverted under D-11: liquid-tumbler -8.5% / -10.3% with 25/25 fingerprints, but wave-tank (no chain fixture) was above the base max in both isolated pairs"
  - "D-11 applied as written; labelled diagnostics after each decision did not change it"

requirements-completed: []

duration: 183min
completed: 2026-10-10
---

# Quick 261010-ibo: Chain-Edge Hoist Retry Summary

**The chain child edge hoist was retried on the final Phase 35 engine in both forms. A3r and A3br cut liquid-tumbler by 8.5–10.4% in both targeted ABBA pairs, with all 25 fingerprints bit-identical. Both were still reverted under D-11 as written. For A3r, jelly-drop and water-wheel were above their base max in both isolated pairs. For A3br, wave-tank was. None of these scenes has a chain fixture. The engine on HEAD is unchanged.**

## Performance

- **Duration:** about 3 h 3 min (2026-10-10T18:17:38Z to 21:20Z). Most of this was waiting: the gate held legs while other repositories ran cargo, and a 50-minute quiet-window wait timed out. The clippy runs took 19–21 min each because they contended with the editor's workspace checks.
- **Tasks:** 3 of 3
- **Files modified:** 35-PROFILES.md (committed). Six engine and test files were changed twice, timed, and restored, for a net change of zero.
- **Launch:** fresh binaries started in under a second, with no `_dyld_start` stall. Focused `cargo test` runs finished in seconds.

## Accomplishments

- **Base verified.** `git diff f7041fc75 HEAD -- crates/` was empty, and `spot-final` hashed to `bd2b2eb5…`. The base was hard-linked as `spot-base-ibo`.
- **A3r (A3 ported onto A6's `collect_fixture_hits`).**
  - The CCD loop now iterates `CcdChild` records. A6's pad match and its `maybe_velocities_finite` cache are unchanged.
  - New test `chain_child_edge_records_cast_like_chain_children`: CCD hits from prebuilt edges match `Shape::ray_cast(.., child)` bit for bit.
  - Timing results:
    - Targeted pairs: 21.772 → 19.849 and 22.674 → 20.308.
    - Fingerprints: 25/25.
    - Isolated pairs: jelly-drop 0.232 > 0.230 and 0.249 > 0.234; water-wheel 0.742 > 0.739 and 0.759 > 0.745.
  - Decision: **reverted**.
- **A3br (`A3b.patch`, applied cleanly).**
  - Targeted pairs: 22.143 → 20.263 and 22.495 → 20.170.
  - Fingerprints: 25/25.
  - Isolated pairs: wave-tank 0.854 > 0.852 and 0.852 > 0.850.
  - Decision: **reverted**.
- **Records written before diagnostics.** The protocol (`ibo-protocol.txt`, 18:50:38Z) was pre-registered before the first leg. `A3r.decision` and `A3br.decision` were each written before any diagnostic run.
- **Diagnostics afterwards (labelled, not decision inputs).** Jelly-drop and water-wheel did not exceed the base max in either pair. Wave-tank exceeded it in pair 1 only.
- **35-PROFILES.md updated** with the A3r and A3br rows, run notes, the liquid-tumbler target record, the Phase summary Attempts column and bullets, and Phase 36 leftover idea 1. The original A3 and A3b rows are byte-unchanged.

## Task Commits

1. **Task 1** (port A3r, tests, build `spot-A3r`): no commit, as the plan specifies.
1. **Task 2** (gated ABBA, decisions, A3br fallback): no commit. Both attempts were reverted, so `git diff HEAD -- crates/` is empty.
1. **Task 3** (record): `4d9ba0e82` docs(35): record chain-edge hoist retry (quick 261010-ibo).

## Files Created/Modified

- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: the changes listed under Accomplishments.
- Local and gitignored, all under `target/phase35/`:
  - Runner and protocol: `abba-gated.sh`, `ibo-protocol.txt`
  - Decisions: `A3r.decision`, `A3br.decision`
  - Source digests: `A3r.source-sha`, `A3br.source-sha`
  - Patches: `attempts/A3r.patch`, `attempts/A3br.patch`
  - Binaries: `bin/spot-base-ibo` (hard link), `bin/spot-A3r`, `bin/spot-A3br`
  - Timing output: the `A3r*`, `A3br*`, `*-diag*` and `*.uptime` files, with the five `*.void-*` directories
  - Logs: `ibo-warm.log`, `*.keep.txt`

## Decisions Made

See `key-decisions` in the frontmatter.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] File-length limit on `body_contact/tests.rs` (A3r and A3br)**
- **Issue:** Both patches put the file at 653 lines, over the 628-line limit.
- **Fix:** As the plan prescribed, the two A3 body-contact tests moved to the sibling module `body_contact/chain_edge_tests.rs`. `storage` and `legacy` became `pub(super)`. No limit was raised.

**2. [Rule 3 - Blocking] `clippy::too_many_lines` on `body_contact::generate` (A3br)**
- **Issue:** A5 had made `generate` longer, and A3b took it to 101 of 100 lines.
- **Fix:** The per-child edge lookup moved into `chain_child_edge_shape(shape, child)`. The float operations are unchanged, and the design stayed one `Shape::Edge` per chain child at child index 0.
- **Verification:** All 27 focused tests passed, including A3b's 4.

### Process deviations

**3. [Process] Pre-run quiet wait before the A3r full set's last re-run.** After two voids, I waited up to 50 minutes for 5 minutes with no cargo or rustc. The wait was result-blind and never came. The re-run then went ahead behind the normal gate and was valid. This is logged in `A3r-full.uptime`.

**4. [Process] Voided sets stop at the voiding leg.** The script exits at the leg where the void condition appears instead of running the remaining legs. It moves the files aside unread, and the protocol states this.

**5. [Lifecycle frontmatter] Values set by the orchestrator.** The orchestrator's prompt set `lifecycle_mode: yolo` and `phase_lifecycle_id: 261010-ibo`. These override the plan's `direct-fallback` / `quick-261010-ibo`.

**6. [Protocol addendum] A3br identity.** After the A3r decision, I appended the A3br binary and source digests to `ibo-protocol.txt` as an identity-only addendum. The rules did not change. This append is why the file's mtime is 20:48Z, while it records that it was written at 18:50:38Z.

## Issues Encountered

- **Voided sets.** Five sets were voided, all because another repository's cargo or rustc was running at the end of a leg (bitaxe-esp-miner editor checks, open-bitcoin `cargo test`): A3r-full twice, A3r-iso once, and A3br targeted twice. The A3br targeted set's third run counted regardless and logged no void condition. There were no GATE-TIMEOUTs.
- **Counted sets** had 1-minute loads between 4.1 and 6.3, lower than in 35-04 (6.2–10.5).
- **The pattern across attempts.** Even at this lower load, each attempt again had one or two non-chain scenes above the base max in both isolated pairs. Across the four chain-edge attempts so far, a different scene was flagged each time. The 35-PROFILES.md notes record this neutrally. Keeping the hoist would need a user decision on the D-11 confirmation rule, not another re-run.
- **Verification evidence:**
  - Focused tests, `cargo test -p liquidfun --all-features --lib -- body_contact particle_coupling`: 27 passed on A3r and 27 on A3br.
  - `cargo clippy -p liquidfun --all-targets --all-features -- -D warnings`: clean on both, after fix 2 for A3br.
  - `bun scripts/bright-builds-check.ts all`: 0 findings on both, after fix 1.
  - `keep_rule.py before-full.jsonl <ID>-full-a1.jsonl ""`: `fingerprint mismatches: none` for both.
  - The plan's Task 2 and Task 3 verify commands both passed.
  - Not run, because no attempt was kept: the full workspace checks (workspace clippy and build, the wasm32 `--lib` build and `cargo test --workspace`). The plan runs them only for a kept attempt, and HEAD's engine is unchanged.
- **Pre-existing:** the release-only `unreachable expression` warning at `boundary/support.rs:98` is already listed in deferred-items.md.

## Known Stubs

None.

## Next Phase Readiness

- The HEAD engine still equals `f7041fc75`, and `spot-final` remains the Phase 36 re-survey binary.
- `attempts/A3r.patch` and `A3br.patch` apply to the current engine if the user changes the confirmation rule.
- STATE.md and ROADMAP.md were not touched. The orchestrator owns them.

## Self-Check: PASSED

- FOUND: target/phase35/ibo-protocol.txt, abba-gated.sh, A3r.decision, A3br.decision
- FOUND: target/phase35/attempts/A3r.patch, attempts/A3br.patch
- FOUND: target/phase35/bin/spot-base-ibo, spot-A3r, spot-A3br
- FOUND: A3r-full-a1.jsonl (25 lines), A3br-full-a1.jsonl (25 lines)
- FOUND: commit 4d9ba0e82
- FOUND: `| A3r | quick 261010-ibo` and `| A3br | quick 261010-ibo` rows; no A3 or A3b row line removed
- `git diff HEAD -- crates/` is empty
