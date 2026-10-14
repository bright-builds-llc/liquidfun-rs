---
phase: 45-curiosity-groundwork-and-feasibility-spikes
plan: "03"
subsystem: liquidfun-wasm-scenes
tags: [scene-catalog, refactor, prep-02, d-07, d-10]
generated_by: gsd-executor
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: "2026-10-14T16:11:25.575Z"
requires:
  - "45-01: pinned 25-scene fingerprint baseline (behavior-neutral refactor must keep it)"
provides:
  - "crate::scene::ALL_SCENE_IDS: &[SceneId] in web SCENE_IDS order (cfg any(test, not(wasm32)))"
  - "scene::tests::all_scene_ids_match_the_web_catalog_order ties ALL_SCENE_IDS to scenes.ts and parse_scene_id"
affects: [45-04, 45-05, 45-09, 45-10, 46, 47, 48, 49, 50, 51]
tech-stack:
  added: []
  patterns:
    - "Array lengths in const context from a slice constant: [T; ALL_SCENE_IDS.len()]"
    - "include_str! catalog-order test against web/src/catalog/scenes.ts"
key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene.rs
    - crates/liquidfun-wasm/src/scene_spot.rs
    - crates/liquidfun-wasm/src/scene/gravity_slider.rs
    - crates/liquidfun-wasm/src/puzzle/tests.rs
key-decisions:
  - "45-03: ALL_SCENE_IDS (scene.rs) is the single Rust scene list in web SCENE_IDS order; SURVEY_SCENES length, gravity-slider seen arrays and the puzzle catalog-length check derive from it; variant_index stays an exhaustive match"
requirements-completed: []
duration: 10min
completed: 2026-10-14
---

# Phase 45 Plan 03: ALL_SCENE_IDS Summary

**Rust scene counts now come from one list, `ALL_SCENE_IDS`, kept in web catalog order. A test ties it to `scenes.ts` and `parse_scene_id`, and no Rust scene-count literal `25` remains.**

## Performance

- **Duration:** about 10 min
- **Started:** 2026-10-14T16:02Z
- **Completed:** 2026-10-14T16:11Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `pub(crate) const ALL_SCENE_IDS: &[SceneId]` right after the `SceneId` enum. It lists all 25 scenes in `web/src/catalog/scenes.ts` `SCENE_IDS` order, from `WaveMachine` to `TeslaValve`, behind `#[cfg(any(test, not(target_arch = "wasm32")))]`.
- Added `all_scene_ids_match_the_web_catalog_order`. It parses `SCENE_IDS` through `include_str!`, maps each id through `parse_scene_id`, and asserts the result equals `ALL_SCENE_IDS`. It fails on an order mismatch, a length mismatch, or a catalog id that has no parse arm.
- Replaced every Rust scene-count site:

| Site | Before | After |
| --- | --- | --- |
| `scene_spot.rs:46` | `[(&str, SceneId, SurveyCue); 25]` | `[(&str, SceneId, SurveyCue); ALL_SCENE_IDS.len()]` (entries and cues unchanged) |
| `scene/gravity_slider.rs` `seen` | `[false; 25]` | `[false; ALL_SCENE_IDS.len()]` |
| `scene/gravity_slider.rs` assertion | `[true; 25]` | `[true; ALL_SCENE_IDS.len()]` |
| `scene/gravity_slider.rs` helper | local `fn all_scene_ids() -> [SceneId; 25]` (enum order) | deleted; the loop uses `for &id in ALL_SCENE_IDS` |
| `puzzle/tests.rs:77` | `assert_eq!(scene_ids.len(), 25)` | `assert_eq!(scene_ids.len(), ALL_SCENE_IDS.len())` |

- `variant_index` is still an exhaustive `match`, so a new `SceneId` variant without an arm fails to compile. The `seen` check now also proves that `ALL_SCENE_IDS` covers every variant.

## Task Commits

1. **Task 1 RED: failing catalog-order test.** `83352e367` (test)
1. **Task 1 GREEN: `ALL_SCENE_IDS` plus the `SURVEY_SCENES` length.** `993abcb35` (feat)
1. **Task 2: gravity slider and puzzle counts derived.** `2d97bb62a` (refactor)

## Verification

- `cargo test -p liquidfun-wasm --lib scene::tests::all_scene_ids_match_the_web_catalog_order`: ok. Before the constant existed, the RED run failed with `E0432 unresolved import super::ALL_SCENE_IDS`.
- Each targeted run was started on its own:
  - `cargo test -p liquidfun-wasm --lib scene::`: 250 passed.
  - `cargo test -p liquidfun-wasm --lib scene_spot::`: 21 passed, 1 ignored.
  - `cargo test -p liquidfun-wasm --lib puzzle::tests`: 18 passed.
- `cargo clippy -p liquidfun-wasm --all-targets --all-features -- -D warnings`: clean.
- `cargo build -p liquidfun-wasm --all-targets --all-features`: no warnings or errors.
- `cargo check -p liquidfun-wasm --lib --target wasm32-unknown-unknown`: clean, and the output does not mention `ALL_SCENE_IDS`, so the cfg gate works.
- `bun scripts/bright-builds-check.ts all`: findings=0.
- `grep -rnE '\[(false|true); ?25\]|; ?25\]|len\(\), 25\)|SceneId; 25' crates/liquidfun-wasm/src`: no matches.
- `git diff --stat b151838be -- crates/liquidfun`: empty, so the engine crate is unchanged.

## Remaining `25` hits (justified)

None of these is a scene count:

- `scene/dam_break.rs` `SMALL_PARTICLE_COLUMNS: u8 = 25` is a particle column count. The plan lists it as a known false positive.
- `puzzle/tools/tests.rs:396` and `puzzle/validate/tests.rs:471` use `[0, 25, -1, 300]`, which are valve open-tick probes.
- `puzzle/validate/tests.rs:565` uses `[-16, 87, 0, 25]`, which are pose grid probes.
- Float constants such as `0.25`, `1.25` and `3.25` are geometry and tuning values.

## Decisions Made

- `ALL_SCENE_IDS` is a slice, not an array, so its declaration contains no literal length. It is cfg-gated the same way as the native and test-only session helpers (`session.rs:554,563`), because its only consumers are tests and the native-only `scene_spot`.

## Deviations from Plan

### Commit grouping

**1. [Rule 3 - Blocking] The `SURVEY_SCENES` change went into the Task 1 GREEN commit.**
- **Found during:** Task 1 GREEN.
- **Issue:** With only the constant, a native non-test `cargo build -p liquidfun-wasm` warned `constant ALL_SCENE_IDS is never used`. That warning would fail `clippy -D warnings` at that commit.
- **Fix:** The first non-test consumer, the `scene_spot.rs` `SURVEY_SCENES` length, was committed together with the constant, so no commit carries a dead-code warning. The gravity slider and puzzle test sites went into the Task 2 refactor commit. The plan allowed one or two commits for this work.
- **Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene_spot.rs`
- **Commit:** `993abcb35`

The work was otherwise executed as written.

## Issues Encountered

- `cargo check -p liquidfun-wasm --target wasm32-unknown-unknown` without `--lib` fails on the native-only bins `dam-break-bench` and `dam-break-timers` with `unresolved import liquidfun_wasm::dam_break_bench`. This existed before this plan. It is the v1.4 leftover "wasm32 bins" listed in REQUIREMENTS.md. The lib target builds clean for wasm32, and that is the target the cfg gate affects. Out of scope.

## Notes

- PREP-02 is not marked complete. This plan covers the Rust half only; Plans 04, 05, 09 and 10 cover the rest.
- The change is behavior-neutral. Scene construction, the survey entries and the cues did not change; only type-level array lengths and test iteration changed. The `scene-fingerprint-gate` 25/25 check is left to Plan 10, as planned.

## Next Phase Readiness

- A new scene now needs a `SceneId` variant, a `parse_scene_id` arm, a `variant_index` arm (enforced by the compiler), an `ALL_SCENE_IDS` entry in catalog order (enforced by tests), and a `SURVEY_SCENES` entry (enforced by the array length). No `25` literal needs editing.

## Self-Check: PASSED

- Modified files exist; commits 83352e367, 993abcb35 and 2d97bb62a are in the log.
