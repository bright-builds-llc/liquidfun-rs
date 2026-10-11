---
phase: 36-re-survey-and-wrap-up
verified: 2026-10-11T03:41:08.902Z
status: passed
score: 15/15 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-10-11T02-24-50
generated_at: 2026-10-11T03:41:08.902Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 36: Re-survey and Wrap Up Verification Report

**Phase Goal:** Maintainers can see the before/after picture for the whole catalog.
**Verified:** 2026-10-11T03:41:08.902Z
**Status:** passed
**Re-verification:** No (initial verification)

SC3 reading: per locked decision D-10 and the orchestrator's instruction, SC3 ("v1.4 is archived with any leftover ideas recorded as future work") is verified as "leftovers recorded and ready to archive". `/gsd-complete-milestone` and any `v1.4` tag are separate later steps; the tag needs the user's go-ahead.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| SC1 | The survey reruns on final source, and `docs/benchmarks/scene-survey.md` shows before/after for all 25 scenes | VERIFIED | `## v1.4 before/after` has 25 rows. I recomputed every row with an independent script from `target/phase35/v14-{b1,a1,a2,b2}.jsonl` and found 0 mismatches: rank order, before/after medians, change % for both pairs, min-max and class all match. Counts are 24 Faster, 1 Within noise, 0 Slower. The after binary SHA-256 `a7d837b1…` matches the file on disk. Its source HEAD `5c664d20c` has `crates/` equal to `edcafebc0`. `git diff c4252c01b HEAD -- crates/ tools/ web/ Cargo.* justfile` is empty. |
| SC2 | Performance notes in user-facing docs cite only these measured local results | VERIFIED | README.md:264 and BENCHMARKING.md:30-33 add only a link plus "local observations, not public speed claims", with no numbers. playground-scene-spot-check.md has no timing numbers and points to the survey. The PERF-CANARY2 paragraph is dated and scoped after b8dc38a0c. A scan for `fps`, `times faster`, `% faster` and `speedup` in non-planning Markdown finds no new v1.4 or universal claim. The remaining ms/step pages are pre-existing, self-contained named-workload reports. |
| SC3 | v1.4 leftovers recorded as future work and ready to archive (D-10 reading) | VERIFIED | REQUIREMENTS.md `### Left over from v1.4` lists all 5 D-09 ideas and all 3 pre-existing deferred items. WR-01 is fixed, so it is correctly absent, and no scene was Slower. `git tag -l 'v1.4*'` is empty, so no archive or tag was created early. |
| P1-1 | A filtered stamp records `scene_filter` (request order) and `full_catalog: false`, with a disclaimer limited to the requested scenes | VERIFIED | spot.rs:26 `FILTERED_DISCLAIMER`, :212 `disclaimer_for`, :231-232 the new fields. The CLI test at playground_cli/spot.rs:151-165 checks these. |
| P1-2 | A full-catalog stamp keeps the byte-identical disclaimer and adds `scene_filter: []` and `full_catalog: true` | VERIFIED | spot.rs:25. The unit test `disclaimer_for_full_catalog_keeps_the_original_text` and the CLI test at :88-94 pass. |
| P1-3 | The summary and the stamp show the same disclaimer from one source | VERIFIED | spot.rs:44 `let disclaimer = disclaimer_for(&spot_args.scenes);` is passed to both writers, with no read-back fallback. |
| P1-4 | The fix touches only tools/xtask | VERIFIED | `git show --stat c4252c01b` lists only the two xtask files. |
| P1-5 | Full checks passed before the fix(36) commit | VERIFIED | `target/phase36/checks-01.results` shows fmt, clippy, build, bright-builds-check and test all at exit=0. The test run ended at 03:18:49Z; commit c4252c01b is at 03:19:02Z. |
| P2-1 | The protocol was pre-registered before the first timed leg | VERIFIED | `v14-protocol.txt` written_utc is 03:21:56Z; the first `start` in v14.uptime is 03:22:04. It holds both binary SHA-256s, the script SHA-256 and the selftest results. The `v14_table.py` SHA-256 on disk, `ec492383…`, matches the protocol. |
| P2-2 | spot-v14-final is built from final `crates/`; spot-before is the unchanged baseline | VERIFIED | `shasum` gives spot-before `5bfc7b61…0742` and spot-v14-final `a7d837b1…b10f`, matching the doc and protocol. |
| P2-3 | One counted, gated ABBA run over 25 scenes, 60/120, `--runs 5`, with no cross-day evidence | VERIFIED | v14.uptime shows b1, a1, a2, b2 with 2 `GATE-WAITED`. Every record has runs 5, warmup 60 and measured 120. The doc warns against comparing across sections. |
| P2-4 | All 6 legs have 25/25 fingerprints equal to before-full.jsonl | VERIFIED | My independent script found every scene set and fingerprint equal in b1, a1, a2, b2, rank-b1 and rank-a1. Both tables print `PUBLICATION: ok`. |
| P2-5 | The before/after section meets D-05: ranked by after, both pairs, min-max, class, provenance, 35-PROFILES link | VERIFIED | scene-survey.md:31-114 |
| P2-6 | A refreshed `--runs 3` ranking with a named source; the Phase 34 table unchanged under `## Original survey (Phase 34)` | VERIFIED | All 25 rank rows match `v14-rank-b1.jsonl`, and the source is stated. The Phase 34 table lines diff identical to `9d8718bc1`. |
| P3-1 | PERF-10 checked and Complete only after the survey commit | VERIFIED | REQUIREMENTS.md shows `[x] **PERF-10**` and `| PERF-10 | Phase 36 | Complete |`. The survey commit e5640ea88 comes before 1cc89564a. |

**Score:** 15/15 truths verified (3 roadmap SCs and 12 plan truths; overlapping plan truths are merged).

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `tools/xtask/src/playground/spot.rs` | disclaimer constants, `disclaimer_for`, new stamp fields | VERIFIED | 576 lines (limit 628); wired into `run`, `assemble_report` and `render_summary` |
| `tools/xtask/tests/playground_cli/spot.rs` | CLI assertions for full and filtered runs | VERIFIED | 4/4 spot CLI tests pass |
| `docs/benchmarks/scene-survey.md` | before/after section, current ranking, original survey | VERIFIED | matches the raw evidence |
| `docs/playground-scene-spot-check.md` | points to the survey, no outdated numbers | VERIFIED | contains `benchmarks/scene-survey.md` |
| `README.md`, `BENCHMARKING.md` | link sentences only | VERIFIED | no numbers |
| `.planning/REQUIREMENTS.md` | leftover list, PERF-10 Complete | VERIFIED | all 8 items present |
| `target/phase35/v14-protocol.txt`, `v14_table.py`, `v14-table.txt`, `v14-rank-table.txt` | local evidence | VERIFIED | present; hashes and timing consistent |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `spot::run` | `spot::disclaimer_for` | one value passed to both writers | WIRED | spot.rs:44 |
| v14-table.txt | v14_table.py | frozen SHA-256 in the protocol | WIRED | hash matches; `PUBLICATION: ok` |
| scene-survey.md | v14-table.txt / raw JSONL | rows pasted from the script output | WIRED | independent recompute: 0 mismatches |
| README.md | scene-survey.md | link sentence | WIRED | README.md:264 |
| REQUIREMENTS.md PERF-10 | docs(36) survey commit | ordering in git log | WIRED | e5640ea88 comes before 1cc89564a |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| scene-survey.md before/after table | per-scene medians, min-max, class | `target/phase35/v14-*.jsonl` from the gated ABBA run | Yes: 25 scenes × 4 legs, 5 runs each | FLOWING |
| scene-survey.md current ranking | per-scene median, min, max, particles | `target/phase35/v14-rank-b1.jsonl` | Yes | FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Markdown formatting | `just markdown-check` | exit 0 | PASS |
| Public doc contracts | `cargo xtask docs check` | exit 0; "docs verified" | PASS |
| Doc contract tests | `cargo test -p xtask --test docs_contract` | 43 passed | PASS |
| scene-spot CLI | `cargo test -p xtask --test playground_cli spot` | 4 passed, including filtered and full-catalog cases | PASS |
| Tables match evidence | independent Python recompute (scratchpad) | 25/25 rows each table, 0 mismatches, fingerprints 25/25 × 6 legs | PASS |
| Full workspace tests | `target/phase36/checks-01.results` (not re-run; no code changed since c4252c01b) | test exit=0 (1402 s) | PASS |

I ran no timing, as instructed.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PERF-10 | 36-01, 36-02, 36-03 | Maintainers can read a whole-catalog before/after timing table for the final v1.4 source | SATISFIED | scene-survey.md `## v1.4 before/after`, verified against raw evidence. The traceability row says Complete. |

No orphaned requirements: REQUIREMENTS.md maps only PERF-10 to Phase 36.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| tools/xtask/src/playground/spot.rs | 26 | `FILTERED_DISCLAIMER` says "(see scene_filter)" but stdout does not print the filter (36-REVIEW IN-03, open) | Info | The terminal reader has to open `scene-spot.json` to see the filter. The stamp is correct. |

No TODO, FIXME or placeholder text, and no stubs, in the phase-modified files. 36-REVIEW WR-01, IN-01 and IN-02 are fixed in b8dc38a0c. I confirmed this in the current text: the PERF-CANARY2 paragraph is dated and scoped, the History paragraph says "v1.2 commit, predate the v1.3 and v1.4 engine changes", and the ranking section says "run directly with no xtask stamp".

### Human Verification Required

None. Every criterion can be checked against repo artifacts, the local evidence and non-destructive commands.

### Gaps Summary

No gaps block the goal. Notes for the milestone archive:

1. IN-03 is the only open review finding, and it is info-level. It is recorded only in 36-REVIEW.md, not in the REQUIREMENTS.md leftover list. You may want to add it to `### Left over from v1.4` when archiving.
1. 35-PROFILES §Notes for Phase 36 also mentions the chain child edge hoist and the `pressure::damping` loop. The hoist was later kept as A3r2, so it is not a leftover. The damping loop is a closed dead end: A0 was slower and D-07 locks its float order. Leaving both off the future-work list is consistent with D-09.
1. Other user-facing pages still carry ms/step tables: `docs/benchmarks/tesla-valve/README.md`, `docs/native-performance-audit.md`, `docs/playground-dam-break-timing.md` and `docs/simulation-performance.md`. These are pre-existing, self-contained named-workload reports with their own evidence, and README.md:263-264 frames them that way. They make no v1.4 scene-performance claim, so SC2 holds.

---

_Verified: 2026-10-11T03:41:08.902Z_
_Verifier: Claude (gsd-verifier)_
