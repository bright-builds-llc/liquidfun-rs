---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 36-2026-10-11T02-24-50
generated_at: 2026-10-11T02:25:17.923Z
---

# Phase 36: Re-survey and Wrap Up - Context

**Gathered:** 2026-10-11
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Measure the final v1.4 engine against the pre-Phase-35 engine for all 25 catalog scenes. Publish that before/after table in `docs/benchmarks/scene-survey.md`. Make sure user-facing performance notes cite only these measured local results. Record leftover ideas as future work, so that v1.4 can be archived. No new engine optimizations in this phase.

</domain>

<decisions>
## Implementation Decisions

### Before/after measurement
- **D-01:** The "before" binary is `target/phase35/bin/spot-before`, the Phase 35 baseline built at `809582431`. The "after" binary is a fresh release build of `playground-scene-spot` from the final `crates/`, whose last engine commit is `edcafebc0` (A2r on top of A3r2). Name it `spot-v14-final` and record the SHA-256 of each binary.
- **D-02:** Compare the two in back-to-back interleaved ABBA order over all 25 scenes, with 60 warmup steps, 120 measured steps and `--runs 5`. Run it through `target/phase35/abba-gated.sh` with the same quiet-host gate and void rules as the 261010-ibo, mrp, npx and rif runs. Do not cite as before/after evidence the cross-day comparison between `before-full.jsonl` and the original Phase 34 table; it was recorded on different days under different load.
- **D-03:** For each scene, report the before median, after median and change %, giving pair 1 and pair 2 separately, plus the min and max. Classify each scene:
  - **Faster:** after below the before minimum in both pairs.
  - **Slower:** the after median is more than 4% above the before median in both pairs. This is the amended D-11 floor from 35-CONTEXT.
  - **Within noise:** everything else.
- **D-04:** Fingerprints must be 25/25 equal to `before-full.jsonl`. A mismatch blocks publication; investigate it, never paper over it.

### Survey doc layout
- **D-05:** Keep the existing Phase 34 ranked table in `docs/benchmarks/scene-survey.md`, labelled as the original survey. Add a new "v1.4 before/after" section containing:
  - the table, ranked by after median;
  - provenance: both binaries' source commits and SHA-256s, the machine, rustc, flags, load averages and the stamp;
  - one short paragraph on what changed and a link to `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`.

  Keep the existing "Local observations, not public speed claims" framing. Format with mdformat and pass `just markdown-check`.
- **D-06:** Also refresh the ranked current-survey table using the after binary (`--runs 3` full catalog), so the page shows today's ranking. Alternatively, derive the ranking from the ABBA after legs, but say which source was used.

### Measurement tooling fix (35 review WR-01)
- **D-07:** Before re-surveying, fix review finding WR-01 in `tools/xtask/src/playground/spot.rs`. A `--scene`-filtered stamp must record the filter, for example with `scene_filter` and `full_catalog` fields, and must use disclaimer wording that does not claim to cover every catalog scene. Full-catalog behavior stays the same. Add or extend a CLI test. This is measurement tooling, not production scene code.

### User-facing docs
- **D-08:** Update `docs/playground-scene-spot-check.md` and any README performance mention so they point to the survey page. Any numbers they cite must come from the new table and carry its local, one-machine framing. Make no universal speed claims and no FPS promises. Change the README only where a performance mention already exists or where a single link line fits the existing performance or claims section.

### Wrap-up and archive
- **D-09:** Record the leftover ideas from 35-PROFILES §Notes for Phase 36 as future work in `.planning/REQUIREMENTS.md` §Future Requirements, or as backlog items, briefly:
  - a presortedness test to recover the particles cost from A2r;
  - A7 per-fixture validation;
  - the world backup clone;
  - emission reallocations;
  - the grouped `prepare_create` cost at load time.

  Also record the pre-existing deferred items:
  - the wasm32 native-only bins;
  - the release-only warning at `support.rs:98`;
  - the stale `web-smoke` spec;
  - WR-01, if it is not fixed in this phase.
- **D-10:** Phase 36 ends with PERF-10 complete and the phase verified. The milestone archive (`/gsd-complete-milestone`) runs as a separate step after phase verification. Creating or pushing a `v1.4` git tag needs the user's explicit go-ahead, because this milestone does not authorize release tags, even though v1.3 used a planning tag.

### Claude's Discretion
- The exact table columns and formatting, and whether the before/after table and the current ranking are merged into one table.
- The structure of the WR-01 fix and the field names.
- The wording of the doc updates, within D-05 and D-08.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and requirements
- `.planning/ROADMAP.md` §Phase 36: goal and success criteria
- `.planning/REQUIREMENTS.md`: PERF-10, Acceptance, Out of Scope, Future Requirements
- `PROJECT-SCOPE.md`: experimental scope and transparency of claims

### Prior phase evidence and method
- `.planning/phases/35-speed-up-the-slowest-scenes/35-PROFILES.md`: §Method, the attempts, §Notes for Phase 36, §D-11 calibration, and the A3r2 and A2r rows
- `.planning/phases/35-speed-up-the-slowest-scenes/35-CONTEXT.md`: D-07 fingerprint rule and D-11 plus its amendment (F = 4%)
- `.planning/phases/35-speed-up-the-slowest-scenes/35-REVIEW.md`: WR-01
- `.planning/phases/35-speed-up-the-slowest-scenes/deferred-items.md`
- `.planning/quick/261010-rif-re-judge-a2-bounded-insertion-sort-under/261010-rif-SUMMARY.md`: latest gated ABBA usage
- `docs/benchmarks/scene-survey.md`: the page to update
- `docs/playground-scene-spot-check.md` and `README.md`: user-facing mentions

### Standards
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Under `target/phase35/` (gitignored):
  - `abba-gated.sh`, `keep_rule.py`, `rif_judge.py` and `npx_judge.py`: ABBA runner, gate and judges
  - `bin/spot-before`: baseline binary
  - `before-full.jsonl`: fingerprint baseline
- `tools/xtask/src/playground/spot.rs` and `spot/args.rs`: the survey driver, its `--scene` filter and the stamp writer.

### Established Patterns
- Pre-register the protocol before any timed leg, write the decision or record before any diagnostic, and never time while cargo or rustc runs.
- Fresh binaries now launch instantly, because Developer Tools covers the claude-code process tree. `cargo test --workspace --all-features` takes about 20 minutes.

### Integration Points
- `just playground-scene-spot` → `cargo xtask playground scene-spot` → the bin. The `justfile` recipe text is pinned by a test.

</code_context>

<specifics>
## Specific Ideas

- Expected large gains from Phase 35 kept changes include:
  - washing-machine, about −32%
  - tesla-valve, about −15 to −20%
  - liquid-tumbler, about −20% after A3r2 and A2r
  - stacked-drip, about −13 to −15%
  - water-wheel, fountain and others
- Particles regressed slightly from A2r but should still be faster overall. The table must show honestly whatever the measurement says.

</specifics>

<deferred>
## Deferred Ideas

- New optimizations, such as the presortedness test or A7, are recorded as future work and not attempted in this phase.
- Creating a `v1.4` git tag waits for the user's explicit go-ahead.

</deferred>

---

*Phase: 36-re-survey-and-wrap-up*
*Context gathered: 2026-10-11*
