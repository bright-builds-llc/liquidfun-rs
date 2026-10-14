---
phase: 45-curiosity-groundwork-and-feasibility-spikes
verified: 2026-10-14T17:25:00Z
status: passed
score: 5/5 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: 2026-10-14T17:25:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 45: Curiosity Groundwork and Feasibility Spikes Verification Report

**Phase Goal:** The playground can grow past 25 scenes without silently breaking anything, and the two riskiest scenes have measured go, reshape or drop decisions before any scene work starts.
**Verified:** 2026-10-14T17:25:00Z
**Status:** passed
**Re-verification:** No (initial verification)

## Goal Achievement

### Observable Truths (ROADMAP success criteria, merged with plan must-haves)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | A committed, pinned list of the 25 scene ids and their fingerprints, with host and toolchain provenance, is recorded before any scene edit. An id-join gate passes when protected ids are identical, fails when one is missing or changed, and accepts new ids. | VERIFIED | `docs/benchmarks/scene-fingerprints.json` was committed in `f31ac087a`, the first code-area commit of the phase (`git diff --stat b151838be f31ac087a -- crates web tools justfile` is empty). It holds 25 ids in catalog order plus provenance: git commit, milestone base, stamp, os, arch, CPU, rustc 1.97.0, release profile, 60/120/3 steps and runs. It also holds the growth rule. The sorted list equals `target/phase43/after-final.txt` (diff empty). The file is unchanged since `f31ac087a`. Gate behavior was checked first-hand: the fresh run PASSes 25/25. A copy with `soup` changed and `sparky` removed FAILs with "1 changed, 1 missing" and names both ids. A copy with an extra id PASSes and lists `new-scene` under "added (accepted)". |
| 2 | Scene counts derive from `ALL_SCENE_IDS` and the web catalog. No test or user-facing copy hard-codes 25 or "twenty-five". The near-limit catalog files are split. A written per-scene checklist exists. | VERIFIED | `ALL_SCENE_IDS` is defined at `crates/liquidfun-wasm/src/scene.rs:77`. The test `all_scene_ids_match_the_web_catalog_order` uses `include_str!` on scenes.ts. The following derive from `ALL_SCENE_IDS.len()`: `SURVEY_SCENES` (the survey), the `gravity_slider.rs` `seen` array and `puzzle/tests.rs`. `variant_index` remains an exhaustive match. Web counts use `SCENE_IDS.length` and `SCENES.at(-1)` in scenes.test.ts, demo-media-model.test.ts and shell.spec.ts. `PAGE_SUMMARY` names no number, and e2e imports it. The D-10 grep finds no "twenty-five" anywhere. The only `25` literals left are non-count values: ticks, percent, D-25 decision tags, `SMALL_PARTICLE_COLUMNS` and camera pan. The catalog split leaves `scene-records.ts` at 20 lines and family modules at 127–244 lines; `previews.tsx` is 95 lines and its family modules are 60–247 lines. All are far under 628. SCENES sha256 is identical before and after the split. `ScenePreview` has a `never` guard. `docs/adding-a-scene.md` (111 lines) lists the R/W/T sites as compile, test or convention, plus all gates. AGENTS.md:44 points to it. |
| 3 | The siphon spike records measured results for each candidate in order against the written pass rule and ends with a go, reshape or drop decision. | VERIFIED | `45-SIPHON-SPIKE.md` records the rules in commit A `dcf7992b3`, together with the harness, before measurement. It reports all 14 configurations of candidates 1–6 in order, each with particles, ms/step, t*, rule A, rule B, diff_end, leaks and lost. I checked several rows against `target/phase45/siphon-spike.log`: 14 RESULT lines, and the c1 and c6-1.00 values match. Decision: **drop** under D-22. Commit B `6366fe335` removes the code. |
| 4 | The hourglass spike records the measured deposit slope against the written rule (at least 20°, ±5° over 5 s, no jam) and the flip cost, and ends with a go or drop decision that the owner confirms. | VERIFIED | `45-HOURGLASS-SPIKE.md` records the rules in commit A `09ab08395`. It reports 9 configurations (2 controls and 7 candidate configurations), each with median, min and max slope, jam status and water margin. Flip cost covers both the batched and per-particle methods, and the table matches the log (9 RESULT and 9 FLIP lines). Decision: **drop**. The owner's confirmation is recorded in commit `30655d6e4`, and the "pending" wording is gone. Commit B `effc9c98f` removes the code. The D-18 trigger did not fire, so `crates/liquidfun` is unchanged. |
| 5 | The id-join gate reports 25/25 protected fingerprints unchanged on the final tree, and normal local checks pass. | VERIFIED | I ran a fresh `just scene-fingerprint-gate` at HEAD `39bcf48d1`, after the review fixes. Result: "PASS, protected: 25/25 unchanged", capture `target/dam-break-perf/2026-10-14T17-22-42Z`. The 45-10 run (`checks-10.results`) recorded exit 0 for all 26 gates at `75b4b4303`. Since then only `.planning/**`, `justfile` and xtask gate files changed. I re-ran `cargo fmt --all --check` (0), `cargo clippy -p xtask --all-targets -D warnings` (clean), `cargo test -p xtask fingerprint_gate` (32/32), `bun scripts/bright-builds-check.ts all` (0 findings) and `just markdown-check` (0). |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `docs/benchmarks/scene-fingerprints.json` | Pinned baseline with provenance | VERIFIED | kind `scene_fingerprint_baseline`, 25 scenes, growth_rule, provenance complete |
| `tools/xtask/src/playground/fingerprint_gate.rs` | Pure compare core plus shell | VERIFIED | 420 lines. Contains `compare_fingerprints`, parse/render, step-count precondition (WR-01) and filtered-capture hint (IN-05). |
| `tools/xtask/src/playground/fingerprint_gate/tests.rs` | AAA unit tests, at least 80 lines | VERIFIED | 434 lines; 24 core tests plus 8 arg/usage tests pass |
| `justfile` `scene-fingerprint-gate` | Thin recipe | VERIFIED | `[positional-arguments]`, forwards `"$@"` |
| `crates/liquidfun-wasm/src/scene.rs` `ALL_SCENE_IDS` | Catalog-order list and test | VERIFIED | Test parses scenes.ts via `include_str!` |
| `web/src/player/runtime.ts` `PAGE_SUMMARY` | Number-free copy | VERIFIED | "...Every demo runs this repository's engine through WebAssembly." |
| `web/src/catalog/scene-records*.ts`, `previews*.tsx`, `preview-kit.tsx`, `scene-view-bounds.ts` | Family split | VERIFIED | Entry spreads CLASSIC/TESTBED/MACHINE in order; `never` guard in `previews.tsx` |
| `docs/adding-a-scene.md` | Per-scene checklist | VERIFIED | Lists sites, gates, growth rule; mdformat-clean |
| `45-SIPHON-SPIKE.md`, `45-HOURGLASS-SPIKE.md` | Rules, results, decision, hashes, owner confirmation | VERIFIED | Both contain `## Decision`, `## Commits`, `## Owner confirmation` |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `playground.rs` | `fingerprint_gate::run` | `"scene-fingerprint-gate"` match arm | WIRED | playground.rs:39 |
| `fingerprint_gate.rs` | `spot::capture` | fresh mode | WIRED | `ObservedSource::Fresh => spot::capture(&[])?`; exercised by fresh run |
| `fingerprint_gate.rs` | `docs/benchmarks/scene-fingerprints.json` | `DEFAULT_BASELINE` | WIRED | Fresh run reports that baseline |
| `ALL_SCENE_IDS` | web `SCENE_IDS` | `include_str!` order test | WIRED | scene.rs:323 |
| `SURVEY_SCENES` | `ALL_SCENE_IDS` | array length | WIRED | scene_spot.rs:46 |
| `web/e2e/shell.spec.ts` | `PAGE_SUMMARY`, `SCENE_IDS.length` | import | WIRED | shell.spec.ts:5, :12, :349 |
| `scene-records.ts` | family modules | spread in catalog order | WIRED | SCENES sha unchanged |
| `AGENTS.md` | `docs/adding-a-scene.md` | Repo-Local Guidance link | WIRED | AGENTS.md:44 |
| `REQUIREMENTS.md` SAND/SIPHON | SEED-007 / SEED-008 | returned-to-seed notes | WIRED | Both seeds exist (dormant). ROADMAP marks Phases 50 and 51 dropped. PROJECT.md points to the seeds. |

### Data-Flow Trace (Level 4)

Not applicable. The phase adds tooling, tests, refactors and docs; it adds no new dynamic-data rendering. The footer copy is a static constant.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Fresh gate on final tree | `just scene-fingerprint-gate` | PASS, 25/25 unchanged, added none (HEAD 39bcf48d1) | PASS |
| Gate fails on change or missing | `--spot` copy with soup changed and sparky removed | FAIL "1 changed, 1 missing", both named, recipe exit 1 | PASS |
| Gate accepts new ids | `--spot` copy with an extra `new-scene` | PASS, listed under "added (accepted)" | PASS |
| Gate unit tests | `cargo test -p xtask --bin xtask fingerprint_gate` | 32 passed | PASS |
| File-length and lessons checks | `bun scripts/bright-builds-check.ts all` | findings=0 | PASS |
| Markdown | `just markdown-check` | exit 0 | PASS |
| No spike leftovers or engine change | `ls src \| grep spike`, `git diff --stat b151838be HEAD -- crates/liquidfun Cargo.lock` | empty | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PREP-01 | 45-01, 45-02, 45-09, 45-10 | Pinned fingerprints compared by id; new ids allowed | SATISFIED | Truths 1 and 5 |
| PREP-02 | 45-03, 45-04, 45-05, 45-09, 45-10 | No hard-coded counts or "twenty-five" copy | SATISFIED | Truth 2 |
| PREP-03 | 45-06, 45-07, 45-08, 45-10 | Time-boxed spikes with measured results and decisions | SATISFIED | Truths 3 and 4 |

No orphaned requirements. REQUIREMENTS.md maps only PREP-01..03 to Phase 45, and every plan's `requirements` field is covered.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/gravity_slider.rs` | 119-147 | `seen` array is filled from `ALL_SCENE_IDS`, so it cannot catch a variant missing from the list (review IN-02, deferred by owner) | Info | Small. The catalog-order test covers every catalog id. |
| `crates/liquidfun-wasm/src/scene.rs` | 326-337 | Fourth copy of the `SCENE_IDS` parser (review IN-03, deferred) | Info | Maintenance only |
| `fingerprint_gate/tests.rs` | — | Some tests assert only `is_err()` (review IN-04, deferred) | Info | Weaker regression signal |

None of these blocks the goal. All three are recorded in 45-REVIEW.md as owner-deferred.

### Honesty Notes on Spike Records

- Both spike docs no longer contain "pending" wording. The one remaining match is `compact_pending_particles`, an API name. Each Owner confirmation section names its source: the owner's in-chat selection through the Plan 08 checkpoint, "Confirm both drops (Recommended)", quoted with its option description. The recording time 2026-10-14T17:09:11Z is one minute before commit `30655d6e4` (17:10:35Z). No approval details were invented. Minor nuance: "Owner answer (verbatim): confirm" is the orchestrator's resume answer, not the owner's literal words. The literal owner selection is quoted on the next line, and 45-08-SUMMARY explains the difference, so the record is honest.
- The hourglass verdict uses an edge-relative reading of the D-25 slope rule plus a water-control guard. The literal D-25 formula is recorded as well, and it passes every row, including plain water, at about 26°: a confinement artifact. The deviation was written down in commit A, before measuring. The owner was shown both readings side by side and explicitly confirmed the edge-relative interpretation.
- The siphon doc records an arithmetic slip in the plan ("at most 13" configurations against the actual 14). It also records two metric corrections made before measurement in commit A: the surface estimator and the t* floor.

### Human Verification Required

None. The owner confirmation required by success criterion 4 was obtained and recorded on 2026-10-14. All other criteria were verified programmatically.

### Gaps Summary

No gaps. The baseline was pinned before any code edit and matches the independent Phase 43 list. The id-join gate behaves correctly on pass, change, missing and added inputs, and it reports 25/25 unchanged on the current HEAD. Scene counts derive from the catalog. The catalog files are split, with plenty of headroom. The checklist exists and is linked. Both spikes measured every candidate against rules committed before measurement, ended in drop decisions, and those decisions were honestly owner-confirmed, with the requirements returned to dormant seeds.

---

_Verified: 2026-10-14T17:25:00Z_
_Verifier: Claude (gsd-verifier)_
