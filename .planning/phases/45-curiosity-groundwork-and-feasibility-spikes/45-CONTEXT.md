---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 45-2026-10-14T15-08-35
generated_at: 2026-10-14T15:09:40.835Z
---

# Phase 45: Curiosity Groundwork and Feasibility Spikes - Context

**Gathered:** 2026-10-14
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Prepare the repository so the playground can grow past 25 scenes without silently breaking anything, and measure whether the two riskiest v1.6 scenes (siphon, sand hourglass) are feasible before any scene work starts.

Delivers (PREP-01, PREP-02, PREP-03):

1. A committed, pinned baseline of the 25 existing scene ids and end-state fingerprints with host and toolchain provenance, captured before any scene edit, plus a gate that joins on scene id.
1. Scene counts derived from one catalog list (`ALL_SCENE_IDS` in Rust, `SCENE_IDS` in the web catalog); no test or user-facing copy hard-codes 25 or "twenty-five".
1. Splits of the near-limit catalog files so a new scene record and preview fit under the 628-line limit.
1. A written per-scene checklist of every Rust and web site a new scene touches.
1. Time-boxed siphon and hourglass feasibility spikes with measured results against written pass rules and a recorded go, reshape or drop decision for each.

Not in this phase: any new playground scene, catalog record, preview, README plan or caption for the six curiosity scenes. Those ship in Phases 46–51.

</domain>

<decisions>
## Implementation Decisions

### Fingerprint baseline and id-join gate (PREP-01)

- **D-01:** Capture the baseline first, before any code edit in this phase, at the current HEAD. HEAD `7ef1baa6d` differs from the milestone base `b151838be` only in `.planning/` docs, so record both hashes in provenance and confirm with `git diff --stat b151838be HEAD -- crates web tools` that no code moved. Cross-check against `target/phase44/` results if a fingerprint list is present there.
- **D-02:** Commit the baseline as a small machine-readable file outside `.planning/` (recommended: `docs/benchmarks/scene-fingerprints.json`). It holds `scene id → fingerprint` in catalog order plus provenance: git commit, capture stamp, host (OS, arch), `rustc -V`, build profile, and the scene-spot stamp directory it came from. Raw scene-spot output stays in `target/`.
- **D-03:** Implement the gate as a tested Rust xtask subcommand (functional core: a pure join/compare function with unit tests; imperative shell: run or read scene-spot output, load the baseline, print a report), exposed through a thin `just` recipe (for example `just scene-fingerprint-gate`). Semantics: pass when every protected id is present with an identical fingerprint; fail when a protected id is missing or changed (naming each); report new ids as accepted additions. A shell `join | awk` is acceptable only as the documented manual fallback, not the gate.
- **D-04:** The gate can either run a fresh scene-spot or read an existing `scene-spot.json` (so a phase can reuse one capture). Determinism inside a run stays enforced by scene-spot's existing 3-run `NondeterministicRun` check.
- **D-05:** Growing the protected set is a separate, explicit step: a later phase updates the baseline file in its own commit after its scene ships, never in the same commit as a scene or shared-code change. Document this rule next to the baseline and in the per-scene checklist.
- **D-06:** The final-tree gate run for this phase must report 25/25 protected fingerprints unchanged (success criterion 5).

### Count de-hardcoding (PREP-02)

- **D-07:** Add `pub(crate) const ALL_SCENE_IDS: [SceneId; N]` beside `SceneId` in `crates/liquidfun-wasm/src/scene.rs`. Size `gravity_slider.rs` `seen` arrays and the all-ids list from it; keep `variant_index` as the exhaustive `match` (compile-time completeness). Add a test that `ALL_SCENE_IDS` matches the web `SCENE_IDS` order (the `include_str!` parse pattern already used by `scene_spot/tests.rs`). Replace `puzzle/tests.rs` `len() == 25` with `ALL_SCENE_IDS.len()`. Keep `SURVEY_SCENES` explicit (per-scene cue) with its existing order test.
- **D-08:** Web tests and e2e derive counts from `SCENE_IDS.length`, the last title from `SCENES.at(-1)`, and focusable counts as `SCENE_IDS.length + 2`. Keep the explicit id and title lists in `scenes.test.ts` that are intentional UI-SPEC contracts; only the numeric counts change.
- **D-09:** User-facing copy becomes number-free rather than a derived number-word (for example "All demos" style wording in `PAGE_SUMMARY` at `web/src/player/runtime.ts`), and e2e imports the copy constant instead of re-typing it. Also fix the "25-preview" comment in `PlaygroundShell.tsx`. Historical records such as `docs/benchmarks/scene-survey.md` stay as dated records.
- **D-10:** Verify de-hardcoding with a grep over `crates/liquidfun-wasm/src`, `web/src`, `web/tests`, `web/e2e` and `web/scripts` for `25` used as a scene count and for `twenty-five` (case-insensitive), with any remaining hit justified in the summary.

### File splits

- **D-11:** Split `web/src/catalog/scene-records.ts` (617 lines) and `web/src/catalog/previews.tsx` (562 lines) into cohesive modules (for example by catalog family or milestone), with the existing entry points re-exporting or spreading the parts so imports and `SCENES` order stay unchanged. Pure move refactor: no record, preview or order changes. The split must leave room for at least one new record and one new preview (about 30 lines each) per target module.
- **D-12:** Also check other files a new scene touches that are within about 60 lines of the limit (for example `web/tests/controls.test.ts` at 529) and split only where the first new scene would cross the limit. Planner's discretion.

### Per-scene checklist

- **D-13:** Write the checklist as a repository doc (recommended: `docs/adding-a-scene.md`), formatted with mdformat and checked with `just markdown-check`, and add a one-line pointer in the `## Repo-Local Guidance` section of `AGENTS.md` (outside the managed block). It lists every Rust and web site, marks which are compile-forced versus test-checked, and names the gates: fingerprint gate, README plan coverage, portrait bounds, HUD safe frame, scale legend, credits path rule (top-level `scene/<id>.rs`), particle budget and recorded ms/step. Start from the research table in `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md`, updated to reflect the post-de-hardcoding state.

### Spike conduct (PREP-03, shared)

- **D-14:** Follow the Phase 40-08 D-25 spike precedent: spike code lives as `#[ignore = "..."]` native tests in the existing `liquidfun-wasm` lib test binary (no new test binary — macOS scans each new binary for about 2 minutes), committed, run in release mode with logs in `target/phase45/`, then removed in a later commit. Record both commit hashes so Phases 50 and 51 can restore the code.
- **D-15:** Spikes use toy meter scale: particle radius 0.04–0.06 m, at most about 3,000 particles, geometry within about 10 m of the origin, thick overlapping polygon walls, default gravity 10 m/s². Record the particle count and native release ms/step of each candidate. Pass rules are judged at gravity 10; results at 2 and 80 are informational only if cheap.
- **D-16:** Spike results go into a dedicated phase artifact per spike (for example `45-SIPHON-SPIKE.md` and `45-HOURGLASS-SPIKE.md`) with the fixed geometry, each candidate's numbers in order, the pass rule verdict, and the decision. Spike numbers are authoring measurements, not evidence for shipped claims (D-18 precedent).
- **D-17:** Time box: each spike gets one plan (one executor pass). When the time box runs out, untried candidates are recorded as "not tried" with the reason; the decision is made from what was measured.
- **D-18:** `crates/liquidfun` stays unchanged by default. The only allowed engine addition is an off-by-default `World::set_particle_flags` matching upstream semantics, added only if the hourglass freeze candidate is reached and cannot work through destroy and re-create. If added, it needs focused tests and the fingerprint gate must still pass 25/25.

### Siphon spike

- **D-19:** Fixed geometry: two tanks at different heights, an inverted-U tube at least 5 particle diameters wide whose crest is clearly above the source surface, walls at least 3 diameters thick, tube primed full at step 0.
- **D-20:** Candidates in order: (1) plain water, the control, expected to fail; (2) `VISCOUS` with strength at most 1.5; (3) `TENSILE | VISCOUS`, also measuring whether the head difference still drives flow since `TENSILE` zeroes weight pressure; (4) a scene-level push on surface particles (fake atmosphere) applied in a per-step hook along `world.gravity()`; (5) weighted dynamic lids on both tank surfaces; (6) a modeled tube pull, last resort, gated on the inlet being submerged.
- **D-21:** Pass rule (written before measuring): net transfer continues for at least 2 s after the source surface is 0.3 m below the crest, and the final level difference is under 0.15 m. Also check no leaks through walls and no evictions.
- **D-22:** Decision rule: **go** if a natural candidate (1–3) passes; **reshape** if only a stand-in (4–6) passes — Phase 50 then ships with a caption that names the stand-in plainly; **drop** if none passes. Pick the earliest passing candidate in order. Never call plain overflow a siphon.

### Hourglass spike

- **D-23:** Fixed geometry: a point-symmetric two-bulb glass with a neck about 5 particle diameters wide and walls that never move.
- **D-24:** Sand-rule candidates in order: (1) bridge-side stick-slip friction (per-step velocity damping or forces on slow, supported grains); (2) freeze settled grains to `WALL` (destroy and re-create, or `set_particle_flags` per D-18), at most K per step, all released on flip and on any gravity change, never permanent; (3) rigid particle clumps. Rigid circle-body grains are ruled out by the 48-circle frame cap.
- **D-25:** Pass rule (written before measuring): deposit slope at least 20° (from the p95 height against base width of the lower-bulb deposit), stable within ±5° over 5 s, with no jam (sand left above the neck but no flow for more than 1 s counts as a jam).
- **D-26:** Measure the flip cost: an instant point reflection (p, v) → (2c − p, −v) by rebuilding the sand group in one batched call (gathering all sand, including frozen grains), and the per-particle `set_particle_position` alternative for comparison. Never flip by inverting gravity.
- **D-27:** Decision: **go** if any candidate passes, **drop** otherwise. By the owner's 2026-10-14 decision, a failed spike returns SAND-01 and SAND-02 to a seed (record that seed update in this phase if the outcome is drop). Owner confirmation: after the spike, the orchestrator presents the measured result and asks the owner to confirm. If the owner is unavailable, the pre-recorded owner rule is applied and recorded as such, with the confirmation left as an open human item — never record an invented approval.

### Claude's Discretion

- Exact module names and boundaries for the file splits.
- Baseline file name and JSON shape, and the xtask subcommand name, within D-02 and D-03.
- Spike geometry dimensions, measurement sampling interval and helper code, within the written pass rules.
- Plan breakdown and wave ordering, provided the baseline capture (D-01) comes before any code edit and both spikes are independent of the de-hardcoding work.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Milestone scope and requirements

- `.planning/ROADMAP.md` § Phase 45 — goal and the five success criteria
- `.planning/REQUIREMENTS.md` — PREP-01, PREP-02, PREP-03, out-of-scope table and the hourglass owner decision
- `.planning/PROJECT.md` — Transparency, Determinism and Architecture constraints
- `.planning/STATE.md` § Decisions — v1.6 decisions (no engine change by default, id-join gate, hourglass drop rule)
- `PROJECT-SCOPE.md` — hobby-scope policy
- `AGENTS.md` — Repo-Local Guidance (README scene gallery, portrait framing, scale legend, mdformat rules)

### v1.6 research

- `.planning/research/v1.6-curiosity-scenes/SUMMARY.md` — reconciled decisions, spike candidate orders, pass rules
- `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md` — per-scene integration checklist (R1–R12, W1–W8, S1–S3, T1–T6, E1–E5), hard-coded 25 sites, fingerprint gate implications
- `.planning/research/v1.6-curiosity-scenes/PITFALLS.md` — Pitfalls 1 (siphon) and 2 (hourglass), fingerprint drift
- `.planning/research/v1.6-curiosity-scenes/STACK.md` — engine API facts and force calibration
- `.planning/seeds/SEED-004-physics-curiosity-scenes.md` — milestone seed

### Precedents

- `.planning/phases/40-timed-valve-and-teaching-levels/40-01-PLAN.md` — prior fingerprint baseline procedure
- `.planning/phases/40-timed-valve-and-teaching-levels/40-08-SUMMARY.md` — D-25 powder spike precedent (ignored tests, commit-then-remove, measured decision)
- `.planning/phases/40-timed-valve-and-teaching-levels/40-09-SUMMARY.md` — buoyancy depth table and leak findings
- `docs/playground-scene-spot-check.md` — scene-spot usage
- `docs/benchmarks/scene-survey.md` — historical survey record (do not edit)

### Code entry points

- `crates/liquidfun-wasm/src/scene.rs`, `scene/gravity_slider.rs`, `scene_spot.rs`, `scene_spot/fingerprint.rs`, `scene_spot/tests.rs`, `puzzle/tests.rs`, `session/tests.rs`
- `tools/xtask/src/playground/spot.rs` — xtask scene-spot plumbing
- `web/src/catalog/scenes.ts`, `scene-records.ts`, `previews.tsx`, `web/src/player/runtime.ts`
- `web/tests/scenes.test.ts`, `web/tests/demo-media-model.test.ts`, `web/e2e/shell.spec.ts`, `web/e2e/player.spec.ts`
- `scripts/bright-builds-check.ts` — file-length limit (628 lines)
- `justfile` — `playground-scene-spot` recipe

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `scene_spot` fingerprinting (FNV-1a 64 over particle and body state after 60 warmup and 120 measured steps, 3 runs per scene with a determinism check) and `--scene` filtering: the gate reuses this instead of computing new hashes.
- `include_str!` parsing of `web/src/catalog/scenes.ts` in Rust tests: the pattern for checking `ALL_SCENE_IDS` against the web catalog.
- `ParticleGroupSource::positions`, `apply_particle_force(_range)`, particle flags (`VISCOUS`, `TENSILE`, `WALL`), kinematic and dynamic bodies: everything the spikes need without engine changes.
- `crates/liquidfun/src/particle/storage/group.rs` `set_particle_flags_internal` exists internally; a public `World::set_particle_flags` would wrap it (only if D-18 triggers).

### Established Patterns

- Bright Builds functional core / imperative shell; Rust errors propagated, no `unwrap()`.
- Spike code as ignored tests in the existing lib test binary, committed then removed (40-08).
- Phase executors run on the main tree, not worktrees, because `target/` baselines are gitignored.
- `.planning/**` is never mdformatted; repository Markdown elsewhere is.

### Integration Points

- New xtask subcommand next to the existing `playground scene-spot` command, plus a `just` recipe.
- `AGENTS.md` Repo-Local Guidance gets a pointer to the per-scene checklist.

</code_context>

<specifics>
## Specific Ideas

- Gate output should name each changed or missing protected id and list new ids, so later phases can paste the report into their summaries.
- The siphon reshape caption must name the stand-in mechanism plainly; never say "suction" or "atmospheric pressure" unless that is what is modeled.
- The hourglass caption may say "piles" or "angle of repose" only if the spike records a measured slope of at least 20°.

</specifics>

<deferred>
## Deferred Ideas

- Siphon Lift-inlet and Prime actions and tracer water (SIPHON-03), hourglass neck-width jamming (SAND-03): future requirements, not this phase.
- Animated hourglass rotation: stretch goal for Phase 51 if the spike passes.
- Raising the 48-circle frame cap for rigid grains: not needed unless a later phase proves it.

</deferred>

---

*Phase: 45-curiosity-groundwork-and-feasibility-spikes*
*Context gathered: 2026-10-14*
