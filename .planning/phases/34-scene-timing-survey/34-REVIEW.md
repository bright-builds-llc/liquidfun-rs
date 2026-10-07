---
phase: 34-scene-timing-survey
reviewed: 2026-10-07T06:24:36Z
depth: standard
files_reviewed: 10
files_reviewed_list:
  - crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/scene_spot/tests.rs
  - tools/xtask/src/playground.rs
  - tools/xtask/src/playground/spot.rs
  - tools/xtask/src/playground/spot/survey_table.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/playground_cli/spot.rs
  - docs/benchmarks/scene-survey.md
  - docs/playground-scene-spot-check.md
findings:
  critical: 0
  warning: 0
  info: 7
  total: 7
status: issues_found
---

# Phase 34: Code Review Report

**Reviewed:** 2026-10-07T06:24:36Z
**Depth:** standard
**Files Reviewed:** 10
**Status:** issues_found

## Summary

I reviewed the diff from `338b6632e` to HEAD for the all-catalog scene survey. The change covers the native `playground-scene-spot` binary with repeated runs and a median/min/max spread, survey cues for idle scenes, xtask validation against `web/src/catalog/scenes.ts`, the ranked Markdown table, the fake-tool fixture, the CLI test and the two docs.

The code is sound for a local dev-only timing tool. Production paths contain no `unwrap()`. The one `expect` (writing to a `String` in `render_ranked_table`) has a real invariant message. Errors propagate through closed enums or `PlaygroundError`. Tests follow Arrange/Act/Assert, and optional values use `maybe_` names. I checked each survey cue against its scene hook:

- Float or Sink handles `drop-body`.
- Impulse acts on `Up` inside its box.
- Drawing Particles stamps on an `Up` with no open stroke.

All three cues therefore exercise the intended path. `survey_scenes_match_catalog_scene_ids` and the xtask catalog validation keep `SURVEY_SCENES` in step with the catalog. `scripts/markdown-check.sh` passes, and every file is under the 628-line limit.

There are no critical or warning findings. The items below are small: misleading error variants, duplicated constants and parsers, one brittle test assertion, and stale doc wording.

## Info

### IN-01: Zero measured steps is reported as a step failure on scene "all"

**File:** `crates/liquidfun-wasm/src/scene_spot.rs:214-219`
**Issue:** `run_scene_spot(_, 0, _)` returns `StepFailed { scene: "all", phase: "measured" }`. Its message is "all measured SessionCore::advance(1) failed", which points the user at a physics failure. The real cause is an invalid argument. xtask rejects `--steps 0` first, but running the binary directly with `--steps 0` shows the wrong message. No test covers this branch.
**Fix:** Add a dedicated variant, as `ZeroRuns` already does:

```rust
/// The caller asked for zero measured steps.
ZeroMeasuredSteps,
// Display: "--steps must be greater than 0"
```

Add a `run_scene_spot_rejects_zero_measured_steps` test.

### IN-02: Particle-count snapshot failures are labeled as scene construction failures

**File:** `crates/liquidfun-wasm/src/scene_spot.rs:285-287, 316-318`
**Issue:** If `live_particle_count()` fails after the timed loop (line 316), the error is `SceneConstruction`. Construction succeeded long before that point, so the message misdirects debugging. (`SessionCore::live_particle_count` itself maps to `SessionError::SceneConstruction`, so this label is inherited.)
**Fix:** Add a `ParticleSnapshot { scene }` variant, or reuse `StepFailed { scene, phase: "particle snapshot" }`.

### IN-03: Survey defaults are duplicated between the binary crate and xtask

**File:** `tools/xtask/src/playground/spot.rs:21-23`, `crates/liquidfun-wasm/src/scene_spot.rs:11-15`
**Issue:** `60 / 120 / 3` are defined twice, and the `--warmup/--steps/--runs` parsing is duplicated too. xtask always passes explicit values, so behavior is consistent today. A future edit to one side would silently change what the binary's own defaults mean compared with the stamp metadata.
**Fix:** Add a one-line cross-reference comment on both sets of constants. Better, have xtask omit flags that equal its defaults so a single source applies. Do not add a dependency just for this.

### IN-04: The `SCENE_IDS` parser exists in three copies

**File:** `tools/xtask/src/playground/spot/survey_table.rs:15-41`, `crates/liquidfun-wasm/src/scene_spot/tests.rs:11-23`, `tools/xtask/tests/fixtures/fake_upstream_tool.rs:296-309`
**Issue:** The copies are deliberate and documented, because the fixture builds with plain `rustc`. They share the same assumptions: double-quoted ids, and no `]` or comment inside the array. If Prettier settings change (single quotes) or someone adds a comment to the array, all three need the same fix. The wasm-side test would fail loudly; the fixture copy would not.
**Fix:** No change is needed now. If the catalog format changes, update all three together. Keep the existing "copy of the xtask split" doc comments so the copies are easy to find.

### IN-05: CLI test hardcodes the catalog size and the rank-1 scene

**File:** `tools/xtask/tests/playground_cli/spot.rs:64, 70`
**Issue:** `len() == 25` and `row.contains("tesla-valve")` fail as soon as a scene is appended to the catalog. AGENTS.md expects scenes to be added. The fixture gives the last catalog id the highest median, so the new scene would rank first even though xtask behavior did not change. `SURVEY_SCENES: [_; 25]` already forces an intentional update elsewhere, so this test adds churn without adding coverage.
**Fix:** Derive both values from the catalog:

```rust
let catalog = include_str!("../../../../web/src/catalog/scenes.ts");
// parse ids as the fixture does; expect len == ids.len() and the first row to contain ids.last()
```

### IN-06: Per-sample counts from the binary are not checked against the requested counts

**File:** `tools/xtask/src/playground/spot.rs:168-192, 210-212`
**Issue:** The report's top-level `warmup_steps`, `measured_steps` and `runs` come from xtask's parsed arguments. The per-scene `runs`, `warmup_steps` and `measured_steps` that the binary emits are stored but never compared with them. If the binary ignored a flag, the stamp would show metadata inconsistent with what was measured. The risk is low because `cargo run` always rebuilds the binary.
**Fix:** In `validate_scene_sample`, also require `scene["runs"] == counts.runs`, and the same for the two step counts. Pass `counts` through to do this.

### IN-07: Historical spot-check doc still reads as current

**File:** `docs/playground-scene-spot-check.md:13-26`
**Issue:** The new preamble says the five-scene table is historical. The page still has a "Reproduce" section that runs `just playground-scene-spot`, which now times 25 scenes and prints a different schema with no `wall_ms`. It also keeps the heading "Current recorded sample". A reader following the page cannot reproduce the table shown.
**Fix:** Rename the heading to "Historical five-scene sample (2026-09-21)". Either drop the Reproduce block or note that it now produces the survey in `benchmarks/scene-survey.md`.

---

_Reviewed: 2026-10-07T06:24:36Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
