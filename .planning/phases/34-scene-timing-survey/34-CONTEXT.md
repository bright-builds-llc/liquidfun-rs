---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 34-2026-10-07T05-23-23
generated_at: 2026-10-07T05:25:02.462Z
---

# Phase 34: Scene Timing Survey - Context

**Gathered:** 2026-10-07
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Extend the existing native five-scene `just playground-scene-spot` timer so one command times all 25 playground catalog scenes and prints them ranked by median ms/step. Commit the ranked table in `docs/benchmarks/scene-survey.md`. No physics, scene, settings or visual changes. No speedups (Phase 35) and no re-survey (Phase 36).

</domain>

<decisions>
## Implementation Decisions

### Survey scope and coverage check
- **D-01:** Extend the existing tool in place (`crates/liquidfun-wasm/src/scene_spot.rs`, `src/bin/playground_scene_spot.rs`, `tools/xtask/src/playground/spot.rs`). Do not add a new crate, harness or evidence framework. The shelved harness on `wip/v1.4-phase34-harness` is not used.
- **D-02:** The survey covers all 25 catalog ids, Dam Break included. The list lives in one Rust table in `scene_spot.rs` (hyphenated id → `SceneId`).
- **D-03:** A coverage test in `liquidfun-wasm` reads `SCENE_IDS` from `web/src/catalog/scenes.ts` through `include_str!` and asserts that it matches the survey list exactly, with each id resolving through `parse_scene_id`. Adding a catalog scene without a survey entry fails `cargo test`.
- **D-04:** The xtask driver drops its hard-coded five-scene `REQUIRED_SCENES` list and validates against the same `SCENE_IDS` parsed from `web/src/catalog/scenes.ts`, so the two lists cannot drift.

### Timing method
- **D-05:** Keep the defaults of 60 untimed warmup steps and 120 timed `advance(1)` steps, and add `--runs <n>` with a default of 3. Each run builds a fresh `SessionCore`. Report the median, min and max ms/step across runs. The spread is the visible noise indicator Phase 35 uses for "beyond noise".
- **D-06:** Run in release mode through `cargo run --release` as today, serially, one scene at a time. Keep the per-scene wall timeout.

### Reported fields
- **D-07:** Per scene, report the median, min and max ms/step, the live particle count at the end of the timed window (the start count too), and an interaction label of `default` or `scripted`.
- **D-08:** `cargo xtask playground scene-spot` prints a ranked Markdown table to stdout (slowest first) and still writes the JSON report under the existing `target/dam-break-perf/<stamp>/scene-spot.json` stamp. The existing `not_timing_authority` flag and disclaimer stay, with wording updated to drop the five-scene description.

### Idle scenes
- **D-09:** Script interaction only where the default scene does nothing visible. Reuse the cues the README previews already use (`web/scripts/readme-svg/plans.ts`): Float or Sink `drop-body` action, Impulse `pointer-up` at (1, 2), Drawing Particles `pointer-up` at (0, 2). Apply each cue through the public `SessionCore::apply_action` and `apply_pointer` before warmup, so the timed window covers the reaction.
- **D-10:** Sparky stays `default`: its circles emit particles on contact without input. Every other scene is `default`.
- **D-11:** Survey cue code lives in `scene_spot.rs` only, never in production scene modules.

### Committed table
- **D-12:** `docs/benchmarks/scene-survey.md` records the ranked table (rank, scene, median/min/max ms/step, end particles, interaction) plus the commit, machine (OS, arch, CPU, cores), rustc, the exact command, and the warmup, steps and runs counts. The page states that these are local observations, not public speed claims. Format with mdformat and pass `just markdown-check`.
- **D-13:** Keep the `justfile` recipe as the existing one line (`cargo xtask playground scene-spot`). An existing test pins that exact text.

### Claude's Discretion
- Exact table column names, the median computation for an even run count, and helper structure in xtask.
- Whether the ranked-table renderer is shared between stdout and the doc, or the doc is pasted from the stdout output.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope
- `.planning/ROADMAP.md` §Phase 34 and §Shared acceptance rules — success criteria and measurement rules
- `.planning/REQUIREMENTS.md` — PERF-07
- `PROJECT-SCOPE.md` — hobby-project scope; local checks only

### Existing tool
- `crates/liquidfun-wasm/src/scene_spot.rs` — current five-scene stepper to extend
- `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs` — CLI flag parsing
- `tools/xtask/src/playground/spot.rs` — xtask driver, validation and report
- `tools/xtask/tests/playground_cli/spot.rs` — existing CLI tests (justfile alias pin, five-scene set assertion to update)
- `crates/liquidfun-wasm/src/session.rs` — `SessionCore` API (`create`, `advance`, `apply_action`, `apply_pointer`, `live_particle_count`)
- `crates/liquidfun-wasm/src/scene.rs` — `SceneId`, `parse_scene_id`
- `web/src/catalog/scenes.ts` — `SCENE_IDS`, the catalog source of truth
- `web/scripts/readme-svg/plans.ts` — existing interaction cues for idle scenes

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `run_scene_spot` / `time_scene`: warmup, timed loop, timeout and particle counts are already in place. Generalize the scene list and add runs and cues.
- `host_identity`, `rustc_version` and `stamp::mint_exclusive_stamp` in xtask cover the machine, commit and stamp fields.

### Established Patterns
- The bin prints JSON lines and xtask parses them, validates them and writes a pretty JSON report.
- Tests use Arrange/Act/Assert comments. Errors are a closed enum with `Display`, and there is no `unwrap` in production code.

### Integration Points
- `justfile` `playground-scene-spot` → `cargo xtask playground scene-spot` → `cargo run --release --bin playground-scene-spot`.

</code_context>

<specifics>
## Specific Ideas

- The roadmap names Drawing and Sparky as idle examples. Inspection shows Sparky is active by default (contact-triggered emission), so only Drawing, Impulse and Float or Sink get cues.

</specifics>

<deferred>
## Deferred Ideas

- Browser frame-drop measurement belongs to Phase 35 if it is needed.
- Whole-catalog before/after belongs to Phase 36.

</deferred>

---

*Phase: 34-scene-timing-survey*
*Context gathered: 2026-10-07*
