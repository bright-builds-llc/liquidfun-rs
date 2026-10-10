---
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 261010-mrp
generated_at: 2026-10-10T22:02:47.117Z
phase: quick-261010-mrp
plan: "01"
subsystem: performance-measurement
tags: [performance, abba, a-a-test, noise-floor, d-11, calibration]

requires:
  - phase: quick-261010-ibo
    provides: "abba-gated.sh, ibo-protocol.txt, A3r/A3br timing sets and decisions, spot-base-ibo"
  - phase: 35-04
    provides: "A3 and A3b timing sets, keep_rule.py, before-full.jsonl"
provides:
  - "35-PROFILES.md section '## D-11 calibration (quick 261010-mrp)': A/A false-alarm count, noise distributions, proposed floor F and what-if table"
  - "target/phase35/mrp-protocol.txt, aa_calib.py, mrp-results.txt, AA{1,2,3}.round (local, gitignored)"
affects: [36]

tech-stack:
  added: []
  patterns:
    - "A/A calibration: run the unchanged keep-rule protocol on a byte-identical copy of the base binary to measure the false-alarm rate"

key-files:
  created:
    - .planning/quick/261010-mrp-calibrate-d-11-keep-rule-with-a-a-runs-o/261010-mrp-SUMMARY.md
  modified:
    - .planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md

key-decisions:
  - "D-11 as written would have reverted 1 of 3 no-op A/A rounds (float-or-sink +1.854% / +2.580%); 0 of 3 false gains"
  - "Proposed floor (not adopted): M = 3.096% (color-mixer, round 2, pair 2), F = 4%; under F, A3, A3b, A3r and A3br would each have been kept"
  - "D-11 and every recorded decision unchanged; the user decides the floor"

requirements-completed: []

duration: 35min
completed: 2026-10-10
---

# Quick 261010-mrp: D-11 A/A Calibration Summary

**I ran three gated A/A rounds through the unchanged §Method protocol. Each compared `spot-base-ibo` with a byte-identical copy of itself. D-11's regression clause would have reverted 1 of the 3 no-op rounds: float-or-sink was above its base max in both isolated pairs, at +1.854% and +2.580%. The largest A/A isolated-pair ratio was 3.096%, so the formula fixed before timing gives a floor of F = 4%. Under F, all four chain-edge attempts (A3, A3b, A3r and A3br) would have been kept. F is a proposal only, and no decision changed.**

## Performance

- **Duration:** about 35 min (2026-10-10T21:28Z to 22:03Z). About 17 minutes of this was the pre-wait before round 1's targeted third run.
- **Tasks:** 3 of 3
- **Files modified:** 1 committed (35-PROFILES.md, 135 lines added, 0 deleted)

## Accomplishments

- **A/A binary.** `bin/spot-AA` is a `cp` of `spot-base-ibo`. Both have SHA-256 `bd2b2eb5…`, and their inodes differ (550580308 and 545134829). It was launched once untimed in 0.36 s.
- **Pre-registration.** `mrp-protocol.txt` was written at 21:29:57Z. The first AA1 leg started at 21:30:33Z. `aa_calib.py` has SHA-256 `0393cf95…`, and its selftest reproduced the A3r confirmations. The script was not changed after timing.
- **Three counted rounds**, 21:49:38Z to 22:01:11Z, at 1-minute loads of 3.06–6.72:
  - Round 1 confirmed float-or-sink, so D-11 would have reverted the no-op.
  - Rounds 2 and 3 confirmed nothing.
  - Fingerprints were 25/25 equal in every round.
  - liquid-tumbler showed no false gain in any round.
- **Noise.** In the full runs, 20 of 75 scene medians were above the base max. In the isolated runs, 9 of 40 pairs were.
- **Floor.** M = 3.096% and F = 4%. The leave-one-round-out F_k values were 4%, 3% and 4%, and no held-out round had a scene above its F_k in both pairs.
- **What-if.** Under F = 4%, A3, A3b, A3r and A3br would each be **keep**. Each had a target gain in both pairs, 25/25 fingerprints and no floor-confirmed scene. The D-11-as-written recomputation reproduced all five recorded confirmations, with no MISMATCH.

## Task Commits

1. **Task 1** (A/A binary, `aa_calib.py`, protocol): no commit. The files are local and gitignored.
1. **Task 2** (three gated rounds, `mrp-results.txt`): no commit. The files are local and gitignored.
1. **Task 3** (record): `30e1e62fe` docs(35): record D-11 A/A calibration (quick 261010-mrp)

## Files Created/Modified

- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: the new section `## D-11 calibration (quick 261010-mrp)` is appended after the last line. No existing line changed.
- Local files under `target/phase35/`:
  - `bin/spot-AA`
  - `aa_calib.py`
  - `mrp-protocol.txt`
  - `mrp-results.txt`
  - `AA{1,2,3}.round`
  - the `AA*.jsonl`, `AA*.uptime` and `AA*.keep.txt` files
  - `AA1.void-1791667895/` and `AA1.void-1791667958/`

## Decisions Made

See `key-decisions` in the frontmatter. None of them change D-11 or a recorded decision.

## Deviations from Plan

### Process deviations

**1. [Process] Result-blind pre-wait before round 1's targeted third run.** Round 1's targeted set was voided twice. Both times, another repository's `cargo test -p open-bitcoin-node` was running at a leg end. Before the counted third run (`NO_VOID=1`), I waited for 300 s with no cargo or rustc, with a 3,000 s cap. The wait took 1,005 s. This follows ibo deviation 3 and is logged as `PRE-WAIT` in `AA1.uptime`. The third run logged no `VOID-IGNORED`, so no counted leg started or ended while cargo or rustc ran.

**2. [Pre-registration] One script fix before hashing.** A parse-only run with no A/A data crashed on an empty per-scene list. I fixed it before computing the hash, and the protocol records this. The floor-confirmed what-if output from that run was not read. No addendum was needed after timing.

## Issues Encountered

- Two voided sets, both described above. There were 3 GATE-WAITED entries of 15 s each and no GATE-TIMEOUT.
- The isolated sets in round 2 took about 4 s per leg because the 8 listed scenes are light. This was expected and is not an error.

## Verification

- Task 1 verify: passed (hash, separate inode, selftest, protocol hash and formula present, no AA timing files before the protocol, `crates/` clean).
- Task 2 verify: passed (3 `.round` files, 25-line full files, targeted legs present, `^F` line, no MISMATCH, `crates/` clean).
- Task 3: `git show --numstat` shows 135 added and 0 deleted lines in 35-PROFILES.md only. `git diff HEAD -- crates/` is empty. `=` and `.planning/config.json` were not touched. The plan's trailer check, `git log -1 --format=%B | tail -1`, reads the blank line that `%B` always appends, so it fails on any commit. With blank lines stripped, and with `--format='%(trailers:only)'`, the last line is the required `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. All the other Task 3 checks passed as written.

## Known Stubs

None.

## Next Phase Readiness

- The user decides whether to adopt F = 4%, another floor, or keep D-11 as written. A re-judgement of A3r needs a fresh run after that decision. `attempts/A3r.patch` and `A3br.patch` still apply to the current engine.

## Self-Check: PASSED

- FOUND: target/phase35/bin/spot-AA, aa_calib.py, mrp-protocol.txt, mrp-results.txt, AA1.round, AA2.round, AA3.round
- FOUND: commit 30e1e62fe (35-PROFILES.md only, additions only)
- FOUND: `## D-11 calibration (quick 261010-mrp)` in 35-PROFILES.md
- `git diff HEAD -- crates/` is empty
