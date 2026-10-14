# Phase 45: Curiosity Groundwork and Feasibility Spikes - Research

**Researched:** 2026-10-14
**Domain:** Repository plumbing (xtask fingerprint gate, catalog-derived counts, TS file splits) plus two native Rust feasibility spikes in the private `liquidfun-wasm` bridge
**Confidence:** HIGH for code facts, commands and sites (read or run in this session at HEAD `895bca7ba`). MEDIUM for the spike measurement recipes, which are this research's operational proposals. LOW for spike outcomes, which are unknown by design.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Fingerprint baseline and id-join gate (PREP-01)

- **D-01:** Capture the baseline first, before any code edit in this phase, at the current HEAD. HEAD `7ef1baa6d` differs from the milestone base `b151838be` only in `.planning/` docs, so record both hashes in provenance and confirm with `git diff --stat b151838be HEAD -- crates web tools` that no code moved. Cross-check against `target/phase44/` results if a fingerprint list is present there.
- **D-02:** Commit the baseline as a small machine-readable file outside `.planning/` (recommended: `docs/benchmarks/scene-fingerprints.json`). It holds `scene id → fingerprint` in catalog order plus provenance: git commit, capture stamp, host (OS, arch), `rustc -V`, build profile, and the scene-spot stamp directory it came from. Raw scene-spot output stays in `target/`.
- **D-03:** Implement the gate as a tested Rust xtask subcommand (functional core: a pure join/compare function with unit tests; imperative shell: run or read scene-spot output, load the baseline, print a report), exposed through a thin `just` recipe (for example `just scene-fingerprint-gate`). Semantics: pass when every protected id is present with an identical fingerprint; fail when a protected id is missing or changed (naming each); report new ids as accepted additions. A shell `join | awk` is acceptable only as the documented manual fallback, not the gate.
- **D-04:** The gate can either run a fresh scene-spot or read an existing `scene-spot.json` (so a phase can reuse one capture). Determinism inside a run stays enforced by scene-spot's existing 3-run `NondeterministicRun` check.
- **D-05:** Growing the protected set is a separate, explicit step: a later phase updates the baseline file in its own commit after its scene ships, never in the same commit as a scene or shared-code change. Document this rule next to the baseline and in the per-scene checklist.
- **D-06:** The final-tree gate run for this phase must report 25/25 protected fingerprints unchanged (success criterion 5).

#### Count de-hardcoding (PREP-02)

- **D-07:** Add `pub(crate) const ALL_SCENE_IDS: [SceneId; N]` beside `SceneId` in `crates/liquidfun-wasm/src/scene.rs`. Size `gravity_slider.rs` `seen` arrays and the all-ids list from it; keep `variant_index` as the exhaustive `match` (compile-time completeness). Add a test that `ALL_SCENE_IDS` matches the web `SCENE_IDS` order (the `include_str!` parse pattern already used by `scene_spot/tests.rs`). Replace `puzzle/tests.rs` `len() == 25` with `ALL_SCENE_IDS.len()`. Keep `SURVEY_SCENES` explicit (per-scene cue) with its existing order test.
- **D-08:** Web tests and e2e derive counts from `SCENE_IDS.length`, the last title from `SCENES.at(-1)`, and focusable counts as `SCENE_IDS.length + 2`. Keep the explicit id and title lists in `scenes.test.ts` that are intentional UI-SPEC contracts; only the numeric counts change.
- **D-09:** User-facing copy becomes number-free rather than a derived number-word (for example "All demos" style wording in `PAGE_SUMMARY` at `web/src/player/runtime.ts`), and e2e imports the copy constant instead of re-typing it. Also fix the "25-preview" comment in `PlaygroundShell.tsx`. Historical records such as `docs/benchmarks/scene-survey.md` stay as dated records.
- **D-10:** Verify de-hardcoding with a grep over `crates/liquidfun-wasm/src`, `web/src`, `web/tests`, `web/e2e` and `web/scripts` for `25` used as a scene count and for `twenty-five` (case-insensitive), with any remaining hit justified in the summary.

#### File splits

- **D-11:** Split `web/src/catalog/scene-records.ts` (617 lines) and `web/src/catalog/previews.tsx` (562 lines) into cohesive modules (for example by catalog family or milestone), with the existing entry points re-exporting or spreading the parts so imports and `SCENES` order stay unchanged. Pure move refactor: no record, preview or order changes. The split must leave room for at least one new record and one new preview (about 30 lines each) per target module.
- **D-12:** Also check other files a new scene touches that are within about 60 lines of the limit (for example `web/tests/controls.test.ts` at 529) and split only where the first new scene would cross the limit. Planner's discretion.

#### Per-scene checklist

- **D-13:** Write the checklist as a repository doc (recommended: `docs/adding-a-scene.md`), formatted with mdformat and checked with `just markdown-check`, and add a one-line pointer in the `## Repo-Local Guidance` section of `AGENTS.md` (outside the managed block). It lists every Rust and web site, marks which are compile-forced versus test-checked, and names the gates: fingerprint gate, README plan coverage, portrait bounds, HUD safe frame, scale legend, credits path rule (top-level `scene/<id>.rs`), particle budget and recorded ms/step. Start from the research table in `.planning/research/v1.6-curiosity-scenes/ARCHITECTURE.md`, updated to reflect the post-de-hardcoding state.

#### Spike conduct (PREP-03, shared)

- **D-14:** Follow the Phase 40-08 D-25 spike precedent: spike code lives as `#[ignore = "..."]` native tests in the existing `liquidfun-wasm` lib test binary (no new test binary — macOS scans each new binary for about 2 minutes), committed, run in release mode with logs in `target/phase45/`, then removed in a later commit. Record both commit hashes so Phases 50 and 51 can restore the code.
- **D-15:** Spikes use toy meter scale: particle radius 0.04–0.06 m, at most about 3,000 particles, geometry within about 10 m of the origin, thick overlapping polygon walls, default gravity 10 m/s². Record the particle count and native release ms/step of each candidate. Pass rules are judged at gravity 10; results at 2 and 80 are informational only if cheap.
- **D-16:** Spike results go into a dedicated phase artifact per spike (for example `45-SIPHON-SPIKE.md` and `45-HOURGLASS-SPIKE.md`) with the fixed geometry, each candidate's numbers in order, the pass rule verdict, and the decision. Spike numbers are authoring measurements, not evidence for shipped claims (D-18 precedent).
- **D-17:** Time box: each spike gets one plan (one executor pass). When the time box runs out, untried candidates are recorded as "not tried" with the reason; the decision is made from what was measured.
- **D-18:** `crates/liquidfun` stays unchanged by default. The only allowed engine addition is an off-by-default `World::set_particle_flags` matching upstream semantics, added only if the hourglass freeze candidate is reached and cannot work through destroy and re-create. If added, it needs focused tests and the fingerprint gate must still pass 25/25.

#### Siphon spike

- **D-19:** Fixed geometry: two tanks at different heights, an inverted-U tube at least 5 particle diameters wide whose crest is clearly above the source surface, walls at least 3 diameters thick, tube primed full at step 0.
- **D-20:** Candidates in order: (1) plain water, the control, expected to fail; (2) `VISCOUS` with strength at most 1.5; (3) `TENSILE | VISCOUS`, also measuring whether the head difference still drives flow since `TENSILE` zeroes weight pressure; (4) a scene-level push on surface particles (fake atmosphere) applied in a per-step hook along `world.gravity()`; (5) weighted dynamic lids on both tank surfaces; (6) a modeled tube pull, last resort, gated on the inlet being submerged.
- **D-21:** Pass rule (written before measuring): net transfer continues for at least 2 s after the source surface is 0.3 m below the crest, and the final level difference is under 0.15 m. Also check no leaks through walls and no evictions.
- **D-22:** Decision rule: **go** if a natural candidate (1–3) passes; **reshape** if only a stand-in (4–6) passes — Phase 50 then ships with a caption that names the stand-in plainly; **drop** if none passes. Pick the earliest passing candidate in order. Never call plain overflow a siphon.

#### Hourglass spike

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

### Deferred Ideas (OUT OF SCOPE)

- Siphon Lift-inlet and Prime actions and tracer water (SIPHON-03), hourglass neck-width jamming (SAND-03): future requirements, not this phase.
- Animated hourglass rotation: stretch goal for Phase 51 if the spike passes.
- Raising the 48-circle frame cap for rigid grains: not needed unless a later phase proves it.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| PREP-01 | The 25 existing scenes behave exactly as before. A committed, pinned list of their end-state fingerprints is compared by scene id after every phase, and new scene ids may be added. | §1 scene-spot facts and JSON shape; §2 gate design (pure `compare` + shell, `just` recipe); baseline JSON shape and jq derivation; cross-check sources found in `target/` |
| PREP-02 | Adding a scene needs no edits to hard-coded scene counts or "twenty-five" copy. Counts and copy derive from the scene catalog. | §3 verified site table (every 25 / twenty-five hit with file:line and its replacement); §4 split seams; §5 post-refactor per-scene checklist |
| PREP-03 | Time-boxed siphon and hourglass spikes record measured results against written pass rules, and each ends with a recorded go, reshape or drop decision before its scene phase. | §6 spike harness (API facts, harness shape, metric definitions, flip and freeze mechanics); recommended plans 06 and 07 including the D-27 owner checkpoint |
</phase_requirements>

## Project Constraints (from CLAUDE.md, AGENTS.md and standards)

- Bright Builds: functional core / imperative shell; early returns; `maybe_` prefix for optional bindings; files over 628 lines fail `bun scripts/bright-builds-check.ts all` (`FILE_LINE_LIMIT = 628`, `scripts/bright-builds-check.ts:8`; no `.bright-builds-rules-checks.tsv` exists). Functions over about 161 lines are refactor triggers.
- Rust: no `unwrap()` in production code; propagate errors; `expect()` only for impossible states; tests are Arrange/Act/Assert with one concept each. Workspace lints are `clippy::all` plus **`clippy::pedantic`** at warn and `missing_docs` warn (`Cargo.toml` `[workspace.lints]`); CI-style runs use `-D warnings`. Spike files therefore need the cast `#![allow(..., reason = "...")]` header the 40-08 spike used.
- Tau for full rotations (`std::f32::consts::TAU`) in any angle code (hourglass slope uses `atan`, which is fine).
- Markdown: non-GSD Markdown is formatted with mdformat 1.0.0 and checked by `just markdown-check`. `.planning/**`, `AGENTS.md` and `CLAUDE.md` are excluded by `.mdformat.toml`. Use repeated `1.` markers for ordered lists.
- Determinism over speed; `crates/liquidfun` unchanged by default (D-18); runtime behavior of the 25 scenes must not move.
- Standing authorization (AGENTS.md): executors may commit and push to `main` without further approval, but never invent a human approval or timestamp. This matters for D-27.
- GSD notes from memory: run executors sequentially on the main tree (gitignored `target/` baselines and generated wasm must be visible); never stage `.planning/config.json`; executor SUMMARY.md files need `generated_by`, `lifecycle_mode`, `phase_lifecycle_id` (`45-2026-10-14T15-08-35`) and `generated_at` frontmatter; `gsd-tools commit` drops the `Co-Authored-By` trailer.

## Summary

Every integration site in the milestone research still holds at HEAD `895bca7ba`. `git diff --stat b151838be HEAD -- crates web tools scripts justfile` is empty, so the 25 fingerprints at HEAD equal the milestone base by construction. Two independent fingerprint lists already exist in `target/` and agree: `target/phase43/after-final.txt` and the scene-spot stamp `target/dam-break-perf/2026-10-14T01-07-10Z/scene-spot.json` (git `66181d36e`, 25 scenes, full catalog). `crates/` did not change between `66181d36e` and `b151838be`, so both are valid D-01 cross-checks. `target/phase44/` contains no fingerprint list.

The gate fits as a new module beside `tools/xtask/src/playground/spot.rs`. Do not put it inside `spot.rs`, which is 576 lines. It reuses `spot::run` (with a small refactor so the fresh mode gets the stamp path back), `stamp::parse_stamp_name` and `stamp::evidence_dir`. `scene-spot.json` already carries every provenance field D-02 needs, and its `scenes` array is already in catalog order, so the baseline can be derived from one stamp with `jq`.

De-hardcoding is smaller than the milestone table suggests. `web/scripts/readme-svg/plans.ts` and `web/scripts/demo-media/model.ts` already derive from `SCENE_IDS`. The remaining sites are 3 Rust lines, 5 web-test lines, 9 `shell.spec.ts` lines, one copy constant and one comment. A slice constant (`&[SceneId]`) can size arrays with `ALL_SCENE_IDS.len()` in const context, so no literal `25` remains. This was verified with `rustc` 1.97. Sizing `SURVEY_SCENES` with it also turns the survey length into a compile-time check.

For the spikes there are two important source facts:

- `World::destroy_particle_group` destroys the group **without** its particles. The milestone STACK's "destroy group then recreate" flip would duplicate the sand. Use `destroy_particles_in_shape` (or mark every particle and call `compact_pending_particles` once), then `create_particle_group` with `ParticleGroupSource::positions`.
- `set_particle_position` rebuilds every proxy on each call (O(n) per call, O(n²) for a flip). `set_particle_velocity` alone does **not** rebuild (`commit_kinematic_edit`, `particle/storage/lifecycle.rs:233-240`), so per-particle velocity damping and post-flip velocity restore are cheap.

**Primary recommendation:** Plan 01 captures and commits the baseline with no code edit. Plans 02–05 cover the gate, Rust counts, web counts/copy and file splits. Plans 06 and 07 run self-contained spike modules that drive a `World` directly, plus the D-27 checkpoint. Plans 08 and 09 cover the checklist doc and the final gate (25/25).

## 1. Scene-spot: how fingerprints are produced today

| Fact | Where | Confidence |
| --- | --- | --- |
| `just playground-scene-spot` = `cargo xtask playground scene-spot` (no args) | `justfile:180-181` | VERIFIED |
| xtask runs `cargo run -p liquidfun-wasm --release --quiet --bin playground-scene-spot -- --warmup 60 --steps 120 --runs 3 [--scene id]...` | `tools/xtask/src/playground/spot.rs:61-89`; defaults `spot/args.rs:5-7` | VERIFIED |
| Writes `target/dam-break-perf/<UTC-stamp>/scene-spot.json` (exclusive stamp dir) and prints `wrote <path>` on **stderr** | `spot.rs:46-51`; `stamp.rs:9,26-32` | VERIFIED |
| Expected ids come from `include_str!("../../../../web/src/catalog/scenes.ts")` parsed by `catalog_scene_ids` | `spot.rs:24,32`; `spot/survey_table.rs:15-41` | VERIFIED |
| `--scene <id>` is repeatable and catalog-checked; filtered runs set `full_catalog: false` and `scene_filter` | `spot/args.rs:42-46,69+`; `spot.rs:220-244` | VERIFIED |
| Fingerprint = FNV-1a 64 over end state after 60 warmup and 120 measured steps of the first run. All 3 runs must match (`NondeterministicRun`) | `crates/liquidfun-wasm/src/scene_spot.rs:314-356`; `scene_spot/fingerprint.rs` | VERIFIED |
| Survey order = `SURVEY_SCENES` (catalog order, `wave-machine` first), checked against `scenes.ts` by `survey_scenes_match_catalog_scene_ids` | `scene_spot.rs:46-104`; `scene_spot/tests.rs:27-37` | VERIFIED |
| `validate_scene_samples` enforces 16-digit lowercase hex `fingerprint`, `timed_out: false`, finite spreads | `spot.rs:142-209` (`is_fingerprint` is private at `:204`) | VERIFIED |
| Simulated run time ≈ 21 s for 25 scenes (Σ median × 180 steps × 3 runs). The Phase 43 gate log shows about 22 s end-to-end with warm builds | computed from stamp `2026-10-14T01-07-10Z`; `target/phase43/checks-06.results` | VERIFIED |

`scene-spot.json` top-level keys (verified from the latest stamp): `kind` (`"native_scene_spot"`), `not_timing_authority`, `disclaimer`, `scene_filter`, `full_catalog`, `git_head`, `os`, `arch`, `cpu_brand`, `logical_cores`, `compiler` (`"rustc 1.97.0 (2d8144b78 2026-07-07)"`), `warmup_steps`, `measured_steps`, `runs`, `scenes[]`. Each scene object has `scene`, `interaction`, `runs`, `warmup_steps`, `measured_steps`, `start_particles`, `end_particles`, `median_ms_per_step`, `min_ms_per_step`, `max_ms_per_step`, `timed_out` and `fingerprint`.

Existing cross-check lists (read-only look):

- `target/phase43/after-final.txt` (sorted `id fingerprint`, 25 lines) equals `target/phase43/base.txt`. Its phase base was `2653fd987…`.
- The latest stamp, `target/dam-break-perf/2026-10-14T01-07-10Z/scene-spot.json` (git `66181d36e`, full catalog, 25 scenes), matches `after-final.txt` exactly.
- `target/phase44/` holds only `checks-03.*` and `screens/`. It has **no fingerprint list**. Phase 44 used `crates-untouched` instead.
- `git diff --stat 66181d36e b151838be -- crates` is empty, so these lists are valid cross-checks for the D-01 capture.

## 2. Fingerprint gate design (PREP-01)

**Where:** `tools/xtask/src/playground/fingerprint_gate.rs`, with tests in `fingerprint_gate/tests.rs` (foo.rs + foo/ layout). Register `mod fingerprint_gate;` and the `"scene-fingerprint-gate"` arm in `tools/xtask/src/playground.rs:33-43`.

**Must also update:**

- `USAGE` (`playground.rs:17`)
- the exact usage-error message **and its test** `missing_subcommand_is_a_usage_error` (`playground.rs:27-31,49-60`), which asserts the full sentence listing the subcommands
- the `playground` line in `tools/xtask/src/main.rs:43` help text

**Functional core (pure, unit-tested):**

```rust
// Source: proposal shaped by D-03; names are discretion.
pub(super) struct SceneFingerprint { pub(super) id: String, pub(super) fingerprint: String } // validated 16-hex
pub(super) struct GateReport {
    pub(super) unchanged: Vec<String>,
    pub(super) changed: Vec<(String, String, String)>, // id, baseline, observed
    pub(super) missing: Vec<String>,                   // protected id absent from the run
    pub(super) added: Vec<(String, String)>,           // new id, fingerprint (accepted)
}
impl GateReport { pub(super) fn passes(&self) -> bool { self.changed.is_empty() && self.missing.is_empty() } }
pub(super) fn compare_fingerprints(protected: &[SceneFingerprint], observed: &[SceneFingerprint])
    -> Result<GateReport, PlaygroundError>; // Err on duplicate ids or empty protected set
pub(super) fn parse_baseline(json: &serde_json::Value) -> Result<Vec<SceneFingerprint>, PlaygroundError>;
pub(super) fn parse_observed(json: &serde_json::Value) -> Result<Vec<SceneFingerprint>, PlaygroundError>; // kind == native_scene_spot
pub(super) fn render_report(report: &GateReport, baseline_label: &str, observed_label: &str) -> String;
```

Unit tests, one concept each, in AAA form:

- all equal: passes
- one changed: fails and names it
- one missing: fails and names it
- one new id: passes and lists the addition
- a duplicate id in either input: error
- an empty baseline: error
- a malformed fingerprint: parse error
- the wrong `kind`: parse error
- report text lists `changed`/`missing`/`added` ids in catalog order

Reuse `is_fingerprint`; make it `pub(super)` in `spot.rs`.

**Imperative shell:**

- `--spot <path/to/scene-spot.json>` or `--stamp <utc>` (validate with `stamp::parse_stamp_name`, resolve under `stamp::evidence_dir`) reads an existing capture (D-04).
- With neither flag, the gate runs a fresh capture by calling spot's internals. Refactor `spot::run` into `pub(super) fn capture(args) -> Result<PathBuf, PlaygroundError>` plus the existing print, which adds about 6 lines to `spot.rs`.
- `--baseline <path>` defaults to `docs/benchmarks/scene-fingerprints.json`.
- Print the report, and exit non-zero (`Err(PlaygroundError::new("gate", ...))`) on failure.
- Print a **warning, not a failure**, when the observed `compiler`/`arch`/`os` differ from the baseline provenance. Bit-exact `f32` hashes are only comparable on the same toolchain and arch.

**Report shape** (for pasting into summaries):

```
Scene fingerprint gate: PASS
- protected: 25/25 unchanged
- changed: none
- missing: none
- added (accepted): none
- baseline: docs/benchmarks/scene-fingerprints.json (captured at <commit>, <stamp>)
- observed: target/dam-break-perf/<stamp>/scene-spot.json (git <head>, full_catalog true)
```

**Just recipe** (thin):

```just
# Compare the pinned scene fingerprints by id with a fresh or saved scene-spot run.
scene-fingerprint-gate *args:
    cargo xtask playground scene-fingerprint-gate {{args}}
```

**Baseline file** `docs/benchmarks/scene-fingerprints.json` (D-02 shape; discretion on names):

```json
{
  "kind": "scene_fingerprint_baseline",
  "schema_version": 1,
  "growth_rule": "Protected ids only grow. A later phase appends its shipped scene in its own commit, never with a scene or shared-code change. See docs/adding-a-scene.md.",
  "provenance": {
    "git_commit": "<HEAD at capture>",
    "milestone_base": "b151838be...",
    "scene_spot_stamp": "target/dam-break-perf/<stamp>",
    "captured_at": "<stamp as UTC>",
    "os": "macos", "arch": "aarch64", "cpu_brand": "Apple M4 Max",
    "compiler": "rustc 1.97.0 (2d8144b78 2026-07-07)",
    "build_profile": "release",
    "warmup_steps": 60, "measured_steps": 120, "runs": 3
  },
  "scenes": [ { "id": "wave-machine", "fingerprint": "ac22aea195c85cf0" }, "... 25 entries in catalog order ..." ]
}
```

Derivation from one stamp, which is also the documented manual path for growing the set later:

```bash
S=target/dam-break-perf/<stamp>
jq --arg base "$(git rev-parse b151838be)" --arg stamp "$S" '{
  kind: "scene_fingerprint_baseline", schema_version: 1,
  growth_rule: "…",
  provenance: { git_commit: .git_head, milestone_base: $base, scene_spot_stamp: $stamp,
    captured_at: ($stamp | split("/") | last), os, arch, cpu_brand, compiler,
    build_profile: "release", warmup_steps, measured_steps, runs },
  scenes: [ .scenes[] | { id: .scene, fingerprint } ] }' "$S/scene-spot.json" > docs/benchmarks/scene-fingerprints.json
```

Documented manual fallback, not the gate: `join base.txt after.txt | awk '$2 != $3 {print "CHANGED", $1; bad=1} END {exit bad}'`, plus a line-count check on the join.

## 3. Hard-coded 25 / "twenty-five" sites (re-grepped at HEAD)

| Site | Current | Replacement |
| --- | --- | --- |
| `crates/liquidfun-wasm/src/scene/gravity_slider.rs:122` | `let mut seen = [false; 25];` | `[false; ALL_SCENE_IDS.len()]` |
| `gravity_slider.rs:144-147` | `assert_eq!(seen, [true; 25], …)` | `[true; ALL_SCENE_IDS.len()]` |
| `gravity_slider.rs:191-219` | local `fn all_scene_ids() -> [SceneId; 25]` (catalog-**un**ordered, enum order) | delete; iterate `ALL_SCENE_IDS` (keep `variant_index` `:161-189` as the exhaustive match) |
| `crates/liquidfun-wasm/src/puzzle/tests.rs:77` | `assert_eq!(scene_ids.len(), 25);` | `ALL_SCENE_IDS.len()` |
| `crates/liquidfun-wasm/src/scene_spot.rs:46` | `const SURVEY_SCENES: [(&str, SceneId, SurveyCue); 25]` | `; ALL_SCENE_IDS.len()]` (stays explicit; length now compile-checked) |
| `web/src/player/runtime.ts:10-11` | `PAGE_SUMMARY` "… All twenty-five demos run this repository's engine through WebAssembly." | number-free, e.g. "… Every demo runs this repository's engine through WebAssembly." |
| `web/src/components/PlaygroundShell.tsx:73` | comment "25-preview scene list counts" | number-free comment |
| `web/tests/scenes.test.ts:140` | test title "lists twenty-five locked scenes" | number-free title |
| `scenes.test.ts:174` | `toHaveLength(25)` | `toHaveLength(SCENE_IDS.length)` |
| `scenes.test.ts:239` | `readyCount toBe(25)` | `SCENE_IDS.length` |
| `scenes.test.ts:433,441` | title "all twenty-five"; `Array.from({ length: 25 }, …)` | number-free title; `SCENE_IDS.length` |
| `web/tests/demo-media-model.test.ts:28,32` | `toHaveLength(25)`; `.at(-1)).toBe("tesla-valve")` | `SCENE_IDS.length`; `SCENE_IDS.at(-1)` |
| `web/e2e/shell.spec.ts:57-62` | `"All twenty-five demos"` ×2 | `import { PAGE_SUMMARY } from "../src/player/runtime"`; header `.not.toContainText(PAGE_SUMMARY)`, footer `.toContainText(PAGE_SUMMARY)` |
| `shell.spec.ts:235-236,252-253` | comments "twenty-five … 27 focusables"; loops `< 28` | `const FOCUSABLE_COUNT = SCENE_IDS.length + 2;` loops `< FOCUSABLE_COUNT + 1` |
| `shell.spec.ts:247` | `lastLink` `/Tesla Valve\b/` | `new RegExp(\`${escapeRegExp(lastTitle)}\\b\`)` where `lastTitle` comes from `SCENES.at(-1)` with an explicit undefined guard (`noUncheckedIndexedAccess` is on; `escapeRegExp` already exists at `:5`) |
| `shell.spec.ts:323,332,335` | title "twenty-five static previews"; `toHaveCount(25)` ×2 | number-free; `toHaveCount(SCENE_IDS.length)` |

Already derived, so no edit is needed: `web/scripts/readme-svg/plans.ts:145-163` and `web/scripts/demo-media/model.ts:290-312` (both compare with `SCENE_IDS.length`), `tools/xtask/src/playground/spot.rs` (parses `scenes.ts`), `web/e2e/hud-safe-frame.spec.ts` (iterates `SCENE_IDS`), and `web/e2e/player.spec.ts` (no count literal).

e2e may import from `../src` (`shell.spec.ts:3,17` already does). `runtime.ts` imports only `../catalog/scenes`, `../components/scene-controls` and a type, so importing it from e2e is safe. `tsconfig.e2e.json` extends the main config, so `bun run typecheck` covers it.

**Known false positives for the D-10 grep** (justify them in the summary): `web/tests/animated-svg.test.ts:143` `toBe(25)` (a percent), `crates/liquidfun-wasm/src/scene/dam_break.rs:22` `SMALL_PARTICLE_COLUMNS: u8 = 25`, the "D-25" comments in `web/src/puzzle/*.ts` and `web/src/catalog/puzzle-bounds.ts`, and decimal geometry such as `3.25`/`1.25`. Suggested grep pair:

```bash
grep -rnIiE 'twenty-five|twenty five' crates/liquidfun-wasm/src web/src web/tests web/e2e web/scripts
grep -rnIE '\[(false|true); ?25\]|; ?25\]|toHaveLength\(25\)|toBe\(25\)|toHaveCount\(25\)|length: 25|len\(\), 25\)|< ?2[78]\b|25-preview' crates/liquidfun-wasm/src web/src web/tests web/e2e web/scripts
```

**`ALL_SCENE_IDS` details:**

- Put it in **catalog order** (`wave-machine` first, matching `SCENE_IDS`), not enum declaration order (`DamBreak` first, `scene.rs:45-71`).
- Prefer the slice form `pub(crate) const ALL_SCENE_IDS: &[SceneId] = &[...]`, so no literal `25` remains. `[false; ALL_SCENE_IDS.len()]` and `[T; ALL_SCENE_IDS.len()]` compile in const context; this was verified in a scratch crate with rustc 1.97.0. The array form `[SceneId; 25]` also satisfies D-07 but leaves one justified literal.
- Gate it with `#[cfg(any(test, not(target_arch = "wasm32")))]`, the same pattern as `session.rs:554,563`. Its users are tests and the native-only `scene_spot`. Without the gate, `wasm-pack build` warns about dead code. That is not fatal, because `scripts/web-build.ts` sets no `-D warnings`, but it is noise.
- Add the order test in `scene.rs` `mod tests`: parse `include_str!("../../../web/src/catalog/scenes.ts")` with the same bracket/comma split as `scene_spot/tests.rs:9-23`, map each id through `parse_scene_id`, and `assert_eq!` against `ALL_SCENE_IDS`. This test also makes a missing `parse_scene_id` arm (R4) fail at test time.

## 4. File splits (D-11, D-12)

`bun scripts/bright-builds-check.ts all` currently passes (`file-lengths scanned=1562 findings=0`, about 0.3 s).

**`web/src/catalog/scene-records.ts` (617 lines).** Lines 1-51 are imports. Lines 53-122 are the `*_VIEW_BOUNDS` constants, which `scenes.ts:1-9,95-102` re-exports. Lines 124-617 are `SCENES`. Records are contiguous in catalog order, so split by family:

| Module (suggested) | Records (lines now) | ≈ lines |
| --- | --- | --- |
| `scene-records-classic.ts` (v1.1) | wave-machine … water-wheel (126-309) | ~190 + imports |
| `scene-records-testbed.ts` (v1.3 testbed ports) | particles … sparky (310-513) | ~205 + imports |
| `scene-records-machines.ts` (v1.3 custom + later) | hydraulic-fountain … tesla-valve (514-616), plus `WASHING_MACHINE_`/`TESLA_VALVE_VIEW_BOUNDS` | ~140 |
| `scene-records.ts` (entry) | remaining bounds constants; `export const SCENES = [...CLASSIC, ...TESTBED, ...MACHINES]` | ~90 |

- Keep the re-exports `scenes.ts` imports from `./scene-records`, or update the single import in `scenes.ts`.
- Each part imports only what it uses. `noUnusedLocals` catches leftovers.
- Parts import `type SceneRecord` from `./scenes`. That is a type-only cycle and already the case today.
- The v1.6 records go into a new `scene-records-curiosity.ts`, created by Phase 46 or here as the empty target. Discretion: an empty exported array is acceptable, but Phase 46 can create it.
- **Proof of a pure move:** before and after, run `cd web && bun -e 'import {SCENES} from "./src/catalog/scenes"; const s=JSON.stringify(SCENES); console.log(new Bun.CryptoHasher("sha256").update(s).digest("hex"))'`. At HEAD it prints `d64119508d2669a206c0837084ae8b3794e2f41db25f2896bd24cfb337a70ff3` (26,476 bytes, run this session).

**`web/src/catalog/previews.tsx` (562 lines).**

- Lines 1-25 hold the palette constants and `PreviewFrame`. Shared helpers are `VerticalWallBasin` (`:140`) and `SoupBasin` (`:212`). The 25 preview functions are in no particular order (`:27-506`). `ScenePreview` (`:509-562`) is the only export, and `components/DemoNavigation.tsx:3` is its only importer.
- Split into:
  - `preview-kit.tsx`: palette, `PreviewFrame`, `VerticalWallBasin`, `SoupBasin`
  - family modules (`previews-classic.tsx`, `previews-testbed.tsx`, `previews-machines.tsx`) that export their preview components
  - `previews.tsx`, which keeps only the `ScenePreview` switch (~60 lines)
- Optional, behavior-neutral and recommended: add an exhaustiveness guard after the switch (`const unreachable: never = props.sceneId; return unreachable;`). This turns the silent W5 site, a missing case that renders nothing because `noImplicitReturns` is off in `web/tsconfig.json`, into a compile error. It changes no preview.
- Review the moved code with `git diff --color-moved=plain`. The e2e `shell.spec.ts` preview count is the runtime check.

**Other near-limit files a new scene touches (D-12):**

| File | Lines | Growth per new scene | Verdict |
| --- | --- | --- | --- |
| `web/tests/controls.test.ts` | 529 | ~25-35 per new HUD control test (Tesla Valve precedent `:495+`); the screw, and maybe ferro/RT, add one each | Crosses about the third control. Optional now; otherwise Phase 47 or 48 splits it. Not required by the first scene. |
| `web/e2e/player-helpers.ts` | 580 | 1 (`SCENE_HASH_PATHS`) | fine (+6 over the milestone) |
| `crates/liquidfun-wasm/src/session/tests.rs` | 580 | 1 (parse list) | fine |
| `tools/xtask/src/playground/spot.rs` | 576 | 0 per scene; +~6 for the gate refactor | keep the gate out of this file |
| `crates/liquidfun-wasm/src/session.rs` | 600 | 0 | do not add spike hooks here |
| `web/tests/scenes.test.ts` | 480 | ~6-8 | fine for six scenes |
| `web/tests/portrait-bounds.test.ts` | 461 | ~10-15 | fine for six scenes |

## 5. Per-scene checklist (post-de-hardcoding state, for `docs/adding-a-scene.md`)

| # | Site | After Phase 45 | Forced by |
| --- | --- | --- | --- |
| R1 | `crates/liquidfun-wasm/src/scene/<id>.rs` (+ `<id>/tests.rs`, `<id>/geometry.rs`); credit path must be top-level (`web/src/catalog/links.ts:22-33` rejects `/`) | new | credits test |
| R2 | `scene.rs` `mod <id>;` | add | compile / dead-code lint |
| R3/R5 | `SceneId` variant + `build_scene` arm (`scene.rs:45-71, 214-251`) | add | compile (exhaustive) |
| R4 | `parse_scene_id` arm (`scene.rs:73-102`) | add | **test** (new `ALL_SCENE_IDS` catalog-order test) |
| R6 | `particle_iterations` (`scene.rs:108-115`) | only if substeps > 2 | optional |
| R7 | `ALL_SCENE_IDS` (new, catalog order) + `variant_index` (`gravity_slider.rs` tests) | append; add arm | test (order) + compile (exhaustive match) |
| R8 | `session/tests.rs:14-42` parse list | append (convention; now redundant with R4 test) | none |
| R9 | `SURVEY_SCENES` (`scene_spot.rs:46`) | append with cue | **compile** (length = `ALL_SCENE_IDS.len()`) + order test |
| R10 | `scene_spot/tests.rs` `only_idle_scenes_are_scripted` | only for non-Default cue | test |
| R12 | `session/escape.rs:33-41` | only if playfield > 12 m fall / 48 m | optional |
| W1 | `web/src/catalog/scenes.ts` `SCENE_IDS` | append (source of truth) | — |
| W2 | record in a `scene-records-*.ts` part | add | `scenes.test.ts` |
| W3/W4 | `<id>-controls.ts`; `DEFAULT_PRESET_VALUES` (control ids share one namespace) | if live controls | — |
| W5 | preview + `ScenePreview` case | add | compile if the guard is added; else e2e count |
| W6 | `portrait-bounds.ts:69` `PORTRAIT_VIEW_BOUNDS` | add | compile + `portrait-bounds.test.ts` |
| S1/S2 | `web/scripts/readme-svg/plans.ts` plan; `just readme-svg` assets | add | `readme-svg.test.ts:55` coverage assert |
| S3 | `web/scripts/demo-media/model.ts` `SCENE_CAPTURE_PLANS` | add | module-load assert + test |
| T1 | `scenes.test.ts` `UI_SPEC_DESCRIPTIONS`/`UI_SPEC_HINTS` (Record) + explicit id/title lists | add entries | compile + test |
| T2 | `scene-catalog-controls.test.ts:21` `RECREATING_CONTROL_IDS` | append | test |
| T3 | `scene-catalog-credits.test.ts:68` `expectedPaths` | add | compile |
| T4 | `portrait-bounds.test.ts:45` `MIN_HEIGHT_FRACTION` (+ `:76` optional, wall points) | add | compile + test |
| T6 | `controls.test.ts` HUD control test | per live control | convention |
| E1 | `web/e2e/player-helpers.ts:30` `SCENE_HASH_PATHS` | add | compile (`bun run typecheck` includes e2e) |
| E2 | `web/e2e/player.spec.ts:40` `POINTER_CONTROL` (Partial) | interactive scenes | **silent**, convention |
| E5 | `player-helpers.ts:23` `ALL_SCENE_TIMEOUT_MS = 380_000` | raise if loops time out | maybe |
| — | No edit any more: `PAGE_SUMMARY`, shell.spec counts, scenes.test counts, demo-media counts, gravity slider sizes, puzzle count | derived | — |

Gates to name in the doc:

- `just scene-fingerprint-gate`, plus the separate baseline-growth commit rule (D-05)
- README plan coverage (`just readme-svg` / `readme-svg.test.ts`)
- portrait frame test
- `web/e2e/hud-safe-frame.spec.ts`
- `web/tests/scale-legend.test.ts`
- credits path rule
- particle budget about 3,000 or less, with the recorded native release ms/step (the `median_ms_per_step` from scene-spot)
- `bun scripts/bright-builds-check.ts all`

Also link the AGENTS.md sections for README gallery, canvas framing and scale legend.

## 6. Spike harness (PREP-03)

### 6.1 Template: the 40-08 powder spike (`git show e79597945`)

- It added `crates/liquidfun-wasm/src/puzzle/powder_spike.rs` (508 lines) and two lines in `puzzle.rs` (`#[cfg(test)] mod powder_spike;`). Commit `2879193cb` removed it.
- Header: a module doc with the run command, plus `#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss, reason = "...")]`.
- Helpers to copy: `quad(min_x, min_y, max_x, max_y) -> [Vec2; 4]`, nearest-rank `percentile`, `mean`, `xs`/`ys`, and a compare loop that prints `SPIKE <label> t <s> <metric> …` lines.
- Run: `cargo test -p liquidfun-wasm --release --lib <module> -- --ignored --nocapture --test-threads=1 2>&1 | tee target/phase45/<spike>.log`.
- It built worlds through a leaked `LevelSpec` and `SessionCore::create_puzzle_from_spec`. **That route has no per-step hook and no `&mut World` access.** `SessionCore` exposes only `read_particles(&self, …)` (`session.rs:563-566`), and `from_built` is private (`session.rs:148`). So it cannot run candidates 4-6 (siphon) or 1-2 (hourglass).

### 6.2 Recommended harness: drive a `World` directly (self-contained, no shared-file edits)

Recommended files (each plan creates one, then deletes it): `crates/liquidfun-wasm/src/siphon_spike.rs` and `hourglass_spike.rs`. Each is registered with one `#[cfg(test)] mod …;` line in `crates/liquidfun-wasm/src/lib.rs`. This keeps the spikes out of `scene.rs`, which plan 03 edits, and out of `basin_family.rs`/`soup_family.rs` (D-14 and the fingerprint safety rule).

Step loop mirroring `SessionCore::advance` (`session.rs:218-268`), apart from eviction:

```rust
// Source: session.rs:4,164-168,228-245 (same types and order)
use liquidfun::{NoDecisionHook, StepConfiguration, StepLimits, World};
let config = StepConfiguration::new(1.0 / 60.0, 8, 3)?.with_particle_iterations(iterations)?;
for _ in 0..steps {
    candidate.on_advance(&mut world, system)?;          // SceneHooks-shaped: liftable into Phase 50/51
    let started = std::time::Instant::now();
    world.step(config, &mut NoDecisionHook, StepLimits::default())?;
    ms_total += started.elapsed().as_secs_f64() * 1e3;  // time hook + step together for ms/step
}
```

- **Walls:** `crate::scene::attach_basin_fixture(&mut world, body, &[Vec2; 4])` (`scene.rs:264-283`, `pub(crate)`, density 0, friction 0.2) for thick, overlapping convex quads on one static body.
- **Particle system:**
  - `ParticleSystemDef::default().with_radius(r)?` (`with_viscous_strength`, `with_surface_tension_*_strength`, `with_maximum_count`; `particle/definition/system_definition.rs:72-398`)
  - `create_particle_system_with_def`
  - fill with `ParticleGroupSource::positions(vec)` + `ParticleGroupRecipe::new(source, ParticleGroupDestination::New).with_particle_flags(flags).with_color(c).with_transform(Transform::IDENTITY)?` and `world.create_particle_group(system, &recipe)` (template `scene/hydraulic_fountain.rs:326-347`; `TENSILE | VISCOUS` template `scene/liquid_timer.rs:120-145`)
  - `ParticleGroupSource::filled_shapes` also works for blocks
- **Eviction:** `session/escape.rs` is private to `session`, so the direct harness has no eviction pass. Measure **leaks** as particles outside the union of interior regions plus a one-diameter margin, and **losses** as live count below the start count. A real eviction implies a leak first, so "0 leaks and 0 lost" satisfies D-21's "no leaks, no evictions".
- **Gravity:** `world.set_gravity(Vec2::new(0.0, -10.0))`. Every scene force uses `world.gravity()` (`world/config.rs:273`).

### 6.3 Engine API facts for the candidates (verified in source this session)

| API | Where | Note |
| --- | --- | --- |
| `apply_particle_force(particle, force)` | `world/particle_object/system.rs:359-366` | One particle; routes through the range path. Errors on `WALL` particles, so filter them out first |
| `apply_particle_force_range(system, &[ids], force)` | `system.rs:377-394`; `force / count` at `particle/force.rs:79-80` | Total force split across a contiguous source-ordered range |
| Force consumed once per world step at substep dt | `particle/solver/preparation.rs:53-87` (`velocity_per_force = time_step * inverse_mass`, then cleared) | For an average acceleration `a` over the step: `F = m·a·particle_iterations`, `m = density·(0.75·2r)²` (`PARTICLE_STRIDE = 0.75`, `math/settings.rs:84`). Treat this as a code-read calibration; the spike sweeps a dimensionless multiplier anyway |
| `apply_particle_linear_impulse(particle, impulse)` | `system.rs:401` | Immediate velocity change (alternative to force) |
| `set_particle_velocity(particle, v)` | `system.rs:288-294` → `commit_kinematic_edit` (`particle/storage/lifecycle.rs:227-243`) | **Velocity-only edit does not rebuild proxies**; cheap. Use it for stick-slip damping and the post-flip `−v` |
| `set_particle_position(particle, p)` | `system.rs:275-281` | **Rebuilds all proxies and clears contacts on every call** (`lifecycle.rs:238-260`). O(n) per call, so O(n²) for a whole-pile flip |
| View lanes | `world.particle_system_view(system)` → `particle_ids()`, `positions()`, `velocities()`, `flags()`, `group_ids()`, `weights()`, `maybe_colors()` (`particle/view.rs:44-140`) | `weights` ≈ packing (surface particles have low weight) |
| `destroy_particle_group(group)` | `world/object/lifecycle.rs:245-257` | **Destroys the group shell only, not its particles** |
| `destroy_particle_group_particles(group, false)` | `world/particle_object/group_lifecycle.rs:21-47` | Marks members pending; clones the system once |
| `compact_pending_particles(system)` | `group_lifecycle.rs:59+` | One compaction after any number of marks |
| `mark_particle_for_destruction(particle)` | `world/particle_object/particle.rs:148-160` | Mark only; compact once |
| `destroy_particle(particle)` | `world/object/lifecycle.rs:264-270` → `destroy_particle_now` (`particle.rs:217-234`) | Compacts immediately, so it is O(n) per call. Do not loop it |
| `destroy_particles_in_shape(system, &shape, transform)` | `particle.rs:171-215` | AABB query, then mark and compact once. Bridge precedent `scene/tesla_valve.rs:208`, `drawing_particles.rs:208` |
| `create_particle_with_def(system, None, &ParticleDef::default().with_flags(WALL).with_color(c).with_position(p)?)` | `particle.rs:24-58`; precedent `tesla_valve.rs:191-200` | Freeze recreate |
| `create_particle_group(system, &recipe) -> ParticleGroupId`; members via `world.particle_group_view(group)?.member_ids()` | `world/particle_object/group.rs:22-29,70+` | Assumed: members are in `positions` order. Assert it in the spike |
| Group recipe | `particle/group/recipe.rs:51-183` | `with_particle_flags`, `with_group_flags` (RIGID clumps), `with_linear_velocity` (one velocity for the whole group, so set per-particle `−v` afterwards), `with_color`, `with_strength` |
| `set_particle_flags_internal` | `particle/storage/group.rs:350-363`, **`pub(in crate::particle)`** on storage; only caller is its unit test (`group/tests.rs:226`) | A public `World::set_particle_flags` (D-18 fallback) needs a `world/particle_object/` method plus a visibility widening (`pub(crate)`). Upstream `b2ParticleSystem::SetParticleFlags` is declared at `b2ParticleSystem.h:496` |
| Pressure clamp; `POWDER`/`TENSILE` zero own weight pressure | `particle/solver/pressure.rs:86` and `:89-100` | Siphon candidate 3 must measure whether head difference still drives flow |
| Dynamic body | `BodyDef::new(BodyType::Dynamic, position, angle, true)` (`scene/float_or_sink.rs:193`) + a fixture with density > 0 | Siphon lids (candidate 5) |
| Flags | `particle/definition.rs:18-55` | `WALL = 1<<2`, `VISCOUS = 1<<5`, `POWDER = 1<<6`, `TENSILE = 1<<7` |
| Viscous strength > ~2.0 fails the step | Phase 40 measurement (`40-RESEARCH.md` Pitfall 3) | D-20 caps it at 1.5 |

### 6.4 Measurement definitions (fix these in the plan **before** measuring)

D-21 and D-25 lock the rules. The operational formulas below are discretion ("measurement sampling interval and helper code"), but the plan must record them before the first run, so they cannot be tuned to a result. Sample every 0.25 s (15 steps) at gravity 10.

**Siphon** (regions are axis-aligned interior boxes `S` = source tank, `D` = destination tank, `T` = tube):

- `source_surface` is the p95 of `y` over particles in `S`; `dest_surface` is the same for `D`. `crest` is the inner **lower** wall height at the tube apex, the highest point water must clear. Record the inner upper height as well.
- `t*` is the first sample where `source_surface ≤ crest − 0.3`. If it never happens, rule A fails.
- Rule A (D-21): `count(D)` rises over `[t*, t* + 2 s]`, with every 0.5 s window positive (continuing, not one late splash) and a total of at least 10 particles.
- Rule B: `|source_surface − dest_surface| < 0.15 m` at the end of the run (for example 30 s), or once transfer has been zero for 2 s.
- Also record 0 leaks, 0 lost, particle count and ms/step.
- **Geometry implication:** the equalized level must sit well below `crest − 0.3`. Start the source surface only about 0.1-0.2 m below the crest, and size the tanks so the final common level is at least 0.5 m below the crest. Otherwise rule A triggers trivially or never. With r = 0.05, the tube inner width is at least 0.5 m and walls are at least 0.3 m thick (D-19). Spacing 0.075 m gives about 178 particles/m², so 3,000 particles cover about 17 m², which is ample.
- **Candidate 3 extra:** compare the pre-`t*` transfer rate with candidate 1. Report whether `TENSILE` water still moves from high to low (D-20).
- **Candidates 4 and 6** need a "surface particle" rule (low `weights()` and inside `S ∪ D`) and an "inlet submerged" gate (`source_surface` above the inlet mouth top). Sweep the force multiplier `α` in `a = α·|g|` along `world.gravity()`.

**Hourglass** (point-symmetric about `c`; generate the lower-half walls and mirror them, `q → 2c − q`, for exact symmetry):

- `upper = count(y > c.y)`, `lower = count(y < c.y)`, and flow per 0.25 s.
- **Jam:** `upper > 0` and `lower` unchanged for more than 60 consecutive steps (D-25: more than 1 s).
- **Slope (pitfall!):**
  - Do **not** use `atan(p95_height / (base_width / 2))` alone. In a confined bulb a flat water fill can score at least 20°. For example, a 0.2 m deep layer in a 1 m wide bulb gives atan(0.4) ≈ 21.8°.
  - Use the p95 deposit height **above the deposit's edge height**:
    - Bin the lower-bulb particles by `x` (bin width 2 diameters) and take each bin's top surface.
    - `edge = mean(top of the two outermost occupied bins)`.
    - `slope = atan((p95_y − edge) / (base_width / 2))`, where `base_width` is the occupied-bin span.
  - Also run the **same geometry with water** as a control (40-08 precedent). A valid metric gives water a slope near 0°.
- **Stability:** max − min slope over a 5 s window after the pile forms is at most 10° (±5°).
- Record particle count, ms/step, frozen count and freezes per step (candidate 2), and flip ms.

**Flip cost (D-26):**

1. Batched: read positions and velocities of **all** particles, including frozen `WALL` ones. Destroy them all with one `destroy_particles_in_shape` box covering the glass interior, or mark them all and call one `compact_pending_particles`. Then call one `create_particle_group` at `2c − p` with sand flags (which releases frozen grains), then `set_particle_velocity(id, −v)` per member. Time the whole sequence.
1. Comparison: `set_particle_position(id, 2c − p)` for every particle, timed once. Expect O(n²).

**Freeze (candidate 2) via destroy and re-create:**

- Each step, pick at most K slow, supported grains (`|v| < v_freeze` for M steps, high `weight`, inside the deposit).
- Mark them, call `compact_pending_particles` once, then `create_particle_with_def(…WALL…)` at the same positions.
- Keep frozen ids out of every force call (`force.rs` rejects `WALL`).
- Release all of them on flip, and on any gravity change (simulate one gravity change to prove release).
- Reach for `World::set_particle_flags` only if this path demonstrably cannot work (D-18), and say why in the artifact.

### 6.5 Decision recording

- `45-SIPHON-SPIKE.md` and `45-HOURGLASS-SPIKE.md` go in the phase dir, which is `.planning`, so not mdformatted.
- Contents: fixed geometry, run command, add and remove commit hashes, log path `target/phase45/<spike>.log`, a per-candidate table in order (particles, ms/step, metrics, verdict), candidates "not tried" with reason (D-17), and the decision by the pre-recorded rule.
- Siphon (D-22): go on candidates 1-3, reshape on 4-6 (name the stand-in), drop on none. There is no owner checkpoint.
- Hourglass (D-27): go if any candidate passes, otherwise drop. Write "Decision by pre-recorded rule: …; owner confirmation: pending". Then a `checkpoint:decision` task (non-autonomous) presents the numbers.
  - On confirmation, record the owner's answer verbatim with the actual time.
  - If the owner is unavailable, write "pre-recorded owner rule (2026-10-14) applied; confirmation open human item". Never invent an approval.
  - On **drop**, in this phase:
    - add a dormant seed, for example `.planning/seeds/SEED-007-sand-hourglass.md` modeled on `SEED-006`, with the spike numbers and a trigger
    - mark SAND-01/SAND-02 as returned to the seed in `REQUIREMENTS.md`
    - mark Phase 51 dropped in `ROADMAP.md`
    - add a pointer line in `PROJECT.md`
  - Do not use a 999.x backlog phase, which hijacks routing (project memory).

## Recommended Plan Breakdown

| Plan | Wave | Depends on | Content | Files (main) | Autonomous |
| --- | --- | --- | --- | --- | --- |
| 45-01 Baseline capture | 1 | — | No code edits. Check `git diff --quiet b151838be HEAD -- crates web tools scripts justfile Cargo.toml Cargo.lock rust-toolchain.toml` and that crates/web/tools are clean. Run `just playground-scene-spot > target/phase45/spot.out 2> target/phase45/spot.err`, and take the stamp from `sed -n 's/^wrote //p'` on the stderr file rather than `ls -t`. Require 25 lines. `diff` the sorted list against `target/phase43/after-final.txt`, which has the same crates as `66181d36e`. Derive and commit `docs/benchmarks/scene-fingerprints.json` with jq, recording the HEAD and `b151838be` hashes | `docs/benchmarks/scene-fingerprints.json` | yes |
| 45-02 Gate subcommand | 2 | 01 | Pure `compare_fingerprints` + parsers + report, with unit tests; shell with `--spot`/`--stamp`/fresh mode and `--baseline`; `spot::capture` refactor; `is_fingerprint` `pub(super)`; usage text, test and main help; `just scene-fingerprint-gate`. Verify with `just scene-fingerprint-gate --stamp <01 stamp>`, expecting 25/25 | `tools/xtask/src/playground.rs`, `playground/fingerprint_gate.rs` (+`/tests.rs`), `playground/spot.rs`, `tools/xtask/src/main.rs`, `justfile` | yes |
| 45-03 Rust counts | 2 | 01 | `ALL_SCENE_IDS` (slice, catalog order, cfg-gated) + catalog-order test; gravity_slider sizes; drop the local `all_scene_ids`; `SURVEY_SCENES` sized by it; `puzzle/tests.rs:77` | `crates/liquidfun-wasm/src/scene.rs`, `scene/gravity_slider.rs`, `scene_spot.rs`, `puzzle/tests.rs` | yes |
| 45-04 Web counts and copy | 2 | 01 | `PAGE_SUMMARY` number-free; PlaygroundShell comment; scenes.test and demo-media-model.test counts; shell.spec imports `PAGE_SUMMARY`, `SCENE_IDS`, `SCENES` | `web/src/player/runtime.ts`, `web/src/components/PlaygroundShell.tsx`, `web/tests/scenes.test.ts`, `web/tests/demo-media-model.test.ts`, `web/e2e/shell.spec.ts` | yes |
| 45-05 Catalog splits | 2 | 01 | scene-records and previews splits (pure move, `SCENES` sha256 unchanged, optional `never` guard) | `web/src/catalog/scene-records*.ts`, `web/src/catalog/preview*.tsx` | yes |
| 45-06 Siphon spike | 2 | 01 | Add spike (commit A), run in release, write `45-SIPHON-SPIKE.md`, remove spike (commit B), decision per D-22 | `crates/liquidfun-wasm/src/lib.rs`, `src/siphon_spike.rs` (transient) | yes |
| 45-07 Hourglass spike | 3 (or 2) | 01 (+06 only to serialize the one-line `lib.rs` edit) | Same pattern; flip cost; freeze; optional D-18 engine API with focused tests; `45-HOURGLASS-SPIKE.md`; **`checkpoint:decision` (D-27)**; seed/roadmap/requirements updates on drop | `crates/liquidfun-wasm/src/lib.rs`, `src/hourglass_spike.rs` (transient), maybe `crates/liquidfun/src/world/particle_object/*.rs` + `particle/storage/group.rs`, `.planning/seeds/…` | **no** (checkpoint) |
| 45-08 Checklist doc | 3 | 02-05 | `docs/adding-a-scene.md` (§5 table + gates + D-05 growth rule), mdformat, `just markdown-check`; one-line pointer in AGENTS.md `## Repo-Local Guidance` | `docs/adding-a-scene.md`, `AGENTS.md` | yes |
| 45-09 Final phase gate | 4 | all | Full local checks (below) + `just scene-fingerprint-gate` 25/25 (D-06) + D-10 grep + `git diff --quiet b151838be -- crates/liquidfun` (unless D-18 triggered) | none (evidence in `target/phase45/`) | yes |

Ordering rationale:

- 01 is first because D-01 requires the capture before any edit.
- 02-06 touch disjoint files.
- The spikes do not depend on de-hardcoding (Discretion bullet 4).
- The hourglass checkpoint sits late, so autonomous work is not blocked waiting for the owner.

## Verification Commands (verified to exist; durations from this session or Phase 43/44 logs)

| Check | Command | Time |
| --- | --- | --- |
| Rust format | `cargo fmt --all --check` | seconds |
| Clippy | `cargo clippy --all-targets --all-features -- -D warnings` | ~1 min warm |
| xtask unit tests (gate) | `cargo test -p xtask --bin xtask playground::` | build + about 2 min macOS scan of the new test binary |
| Bridge unit tests (targeted) | `cargo test -p liquidfun-wasm --lib scene::` / `scene_spot::` / `puzzle::tests` | ~3.5 min for the full crate in the Phase 43 log |
| Spikes | `cargo test -p liquidfun-wasm --release --lib siphon_spike -- --ignored --nocapture --test-threads=1 2>&1 \| tee target/phase45/siphon-spike.log` | release build + scan + run |
| Fingerprints | `just playground-scene-spot` (then `just scene-fingerprint-gate --stamp <s>`), or `just scene-fingerprint-gate` (fresh) | ~22 s simulated plus builds |
| File lengths | `bun scripts/bright-builds-check.ts all` | 0.3 s |
| Web typecheck (incl. e2e) | `cd web && bun run typecheck` | 0.7 s (TypeScript 7) |
| Web unit | `cd web && npx vitest run` (baseline: 119 files, 1,844 tests pass) | ~2 s |
| e2e (shell, previews, copy) | `just web-wasm && (cd web && bun run build:app) && (cd web && CI=true bunx playwright test e2e/shell.spec.ts)` | ~1-2 min |
| HUD safe frame (optional final) | `(cd web && CI=true bunx playwright test e2e/hud-safe-frame.spec.ts)` | ~10 s (Phase 44 log) |
| Markdown | `just markdown-check` | 1 s |

macOS note (project memory): each newly linked binary stalls about 2 minutes at first launch for a debug test binary. A 2 MB release binary is fast. Batch all candidates of one spike into one build, and parametrize inside the test rather than editing and rebuilding per candidate. Run per-crate test filters, never in parallel.

## Common Pitfalls

### Pitfall 1: Flip by group destroy duplicates the sand

**What goes wrong:** `destroy_particle_group` leaves the particles alive (`lifecycle.rs:245`), so recreating at mirrored positions doubles the count.
**How to avoid:** destroy the particles themselves (shape destroy, or mark plus one compaction), then create. Assert the particle count is the same after the flip.

### Pitfall 2: Per-particle loops that compact or rebuild every call

**What goes wrong:** `destroy_particle` (compacts) and `set_particle_position` (rebuilds proxies) inside a loop are O(n²).
**How to avoid:** mark-then-compact, batched group creation, and velocity-only edits.

### Pitfall 3: A slope metric fooled by confinement

**What goes wrong:** a flat fill in a narrow bulb passes "p95 height vs base width".
**How to avoid:** use the edge-relative surface metric plus a same-geometry water control (§6.4). Fix the formula in the plan before running.

### Pitfall 4: Spike results tuned after the fact

**How to avoid:** the plan states candidate parameters (or a fixed small sweep), sample times and formulas before the first run. Record every run in the log, including failures. Mark untried candidates "not tried" (D-17).

### Pitfall 5: Forces on `WALL` particles error out

**What goes wrong:** `apply_particle_force` rejects `WALL` (`force.rs` wall check), and one frozen id in a range call fails the whole call.
**How to avoid:** filter by `flags()` first.

### Pitfall 6: Baseline from the wrong stamp

**What goes wrong:** `ls -td target/dam-break-perf/*/ | head -1` can pick an unrelated stamp from a concurrent run.
**How to avoid:** parse the `wrote <path>` stderr line. Check `.git_head` equals HEAD and `.full_catalog == true`.

### Pitfall 7: The gate compares across toolchains

**What goes wrong:** a different `rustc` or arch can legitimately change `f32` bits.
**How to avoid:** the gate prints a provenance warning. The protected baseline stays tied to rustc 1.97.0 on aarch64 macOS (from `rust-toolchain.toml`).

### Pitfall 8: `ALL_SCENE_IDS` in enum order

**What goes wrong:** it must equal `SCENE_IDS` order (`wave-machine` first), while the enum starts with `DamBreak`.
**How to avoid:** the catalog-order test catches this. Write it in catalog order from the start.

### Pitfall 9: Editing shared builders or `session.rs` for spike convenience

**What goes wrong:** any change to `session.rs` advance, `escape.rs`, `basin_family`/`soup_family` or `crates/liquidfun` can move fingerprints.
**How to avoid:** keep spikes self-contained (§6.2). The only sanctioned engine change is D-18.

### Pitfall 10: `noUncheckedIndexedAccess` on `SCENES.at(-1)` / `SCENE_IDS.at(-1)`

**What goes wrong:** both return `T | undefined`, so e2e and tests need a guard or a test-local `expect` before use.

### Pitfall 11: Staging `.planning/config.json`

**What goes wrong:** it holds the orchestrator's `_auto_chain_active` flag and shows as modified at phase start.
**How to avoid:** stage files explicitly.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | `particle_group_view(group).member_ids()` returns new particles in `ParticleGroupSource::positions` order | §6.3 flip | Velocities restored to the wrong particles. Mitigation: assert positions match before setting `−v` |
| A2 | `F = m·a·particle_iterations` gives an average acceleration `a` per world step (code-read, not measured) | §6.3 | Candidate 4/6 force magnitudes off by a constant. Harmless, because the spike sweeps a multiplier |
| A3 | Creating `WALL` particles mid-run in a system that started without `WALL` updates the aggregate-flag pass gate (`solver/manifest.rs:92,363`) | §6.4 freeze | Frozen grains might not behave as walls. The spike must check that frozen particles stay at zero velocity |
| A4 | The edge-relative slope formula is an acceptable reading of D-25's "p95 height against base width" | §6.4 | If the owner reads D-25 literally, a confined flat fill could pass. Record both numbers and the water control |
| A5 | e2e shell.spec plus vitest are sufficient regression cover for the splits (no SSR render test of previews exists) | §4 | A moved preview with a typo renders differently. Mitigation: `git diff --color-moved` review |

## Open Questions (RESOLVED)

1. **What happens to SIPHON-01/02 if the siphon spike's decision is drop?**
   - D-22 defines go/reshape/drop but not the requirement bookkeeping. D-27's seed rule covers only the hourglass.
   - Recommendation: the siphon plan records the decision. On drop it also mirrors the hourglass handling (seed + REQUIREMENTS/ROADMAP note), so Phase 50 is not planned blindly. The planner should flag this to the orchestrator.
   - RESOLVED: 45-06 records the siphon decision only. 45-08 shows it to the owner with the hourglass result; siphon seed, REQUIREMENTS and ROADMAP updates happen in 45-08 only if the owner explicitly confirms a drop; otherwise Phase 50 records the seed return (ROADMAP Phase 50 criterion 1).
1. **Should `scene-records-curiosity.ts` / `previews-curiosity.tsx` exist as empty targets now?**
   - Either works. Creating them in Phase 46 avoids empty modules.
   - RESOLVED: 45-05 creates no empty curiosity modules; Phase 46 creates `scene-records-curiosity.ts` and `previews-curiosity.tsx` when it needs them.
1. **Should `controls.test.ts` be split now?**
   - Not required for the first scene (§4). Leave it to Phase 47 or 48 unless the planner wants a margin.
   - RESOLVED: 45-05 records the D-12 line counts and defers the `controls.test.ts` split to a later scene phase.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Rust toolchain | all Rust work | ✓ | rustc/cargo 1.97.0 | — |
| bun | web checks, bright-builds check | ✓ | 1.4.2 | — |
| jq | baseline derivation | ✓ | 1.7.1 | the xtask could write it; not needed |
| just | recipes | ✓ | 1.48.0 | call cargo/bun directly |
| wasm-pack | `just web-wasm` before e2e | ✓ | 0.15.0 (the version `web-build.ts` checks) | — |
| Playwright Chromium | shell.spec | ✓ | chromium-1169/1200 cached | — |
| mdformat | `just markdown-check` | ✓ | 1.0.0 (gfm 1.0.0, frontmatter 2.1.2) on Python 3.13.12 | — |
| Disk | builds and logs | ✓ | 91 GiB free (96% used) | keep spike logs small |

No missing dependencies.

## Security Domain

This is local developer tooling with no network, auth or user data. **ASVS V5 (input validation) applies to the gate:**

- Parse `scene-spot.json` and the baseline into typed values. Reject a wrong `kind`, malformed 16-hex fingerprints, duplicate ids and an empty baseline.
- Validate `--stamp` with the existing `stamp::parse_stamp_name`, which rejects `..`, `/` and colons (`stamp.rs:90-104`).
- Validate `--spot` and `--baseline` paths as files.

V2, V3, V4 and V6: not applicable. Threat: a tampered baseline could hide drift. The mitigation is that the baseline is committed, changes only in its own commit (D-05), and is reviewed.

## Sources

### Primary (HIGH, read or run this session at HEAD `895bca7ba`)

- `tools/xtask/src/playground.rs`, `playground/spot.rs`, `spot/args.rs`, `spot/survey_table.rs`, `stamp.rs`, `identity.rs`, `tools/xtask/src/main.rs`, `tools/xtask/Cargo.toml`, `justfile`
- `crates/liquidfun-wasm/src/{lib.rs, scene.rs, scene/gravity_slider.rs, scene_spot.rs, scene_spot/tests.rs, puzzle/tests.rs, session.rs, session/tests.rs, session/escape.rs, scene/liquid_timer.rs, scene/hydraulic_fountain.rs, scene/tesla_valve.rs}`
- `crates/liquidfun/src/{world/particle_object/system.rs, particle.rs, group.rs, group_lifecycle.rs, world/object/lifecycle.rs, particle/storage/lifecycle.rs, particle/storage/group.rs, particle/force.rs, particle/solver/preparation.rs, particle/solver/pressure.rs, particle/definition.rs, particle/view.rs, particle/group/recipe.rs, particle/group/source.rs}`; upstream `b2ParticleSystem.h:496`
- `web/src/catalog/{scenes.ts, scene-records.ts, previews.tsx, links.ts}`, `web/src/player/runtime.ts`, `web/src/components/PlaygroundShell.tsx`, `web/tests/{scenes,demo-media-model}.test.ts`, `web/e2e/shell.spec.ts`, `web/scripts/{readme-svg/plans.ts, demo-media/model.ts}`, `web/tsconfig*.json`, `web/package.json`
- `git show e79597945` (40-08 spike) and `40-08-SUMMARY.md`; `40-01-PLAN.md` Task 1; `target/phase43/*`, `target/phase44/*`, `target/dam-break-perf/2026-10-14T01-07-10Z/scene-spot.json`
- Commands run: `bun scripts/bright-builds-check.ts all`, `bun run typecheck`, `npx vitest run`, `just markdown-check`, a rustc 1.97 const-slice-length check, `git diff --stat b151838be HEAD -- crates web tools scripts justfile`

### Secondary

- `.planning/research/v1.6-curiosity-scenes/{SUMMARY,ARCHITECTURE,PITFALLS,STACK}.md`. Verified and corrected here: the group-destroy flip, per-particle velocity cost, already-derived plan scripts, and the `SURVEY_SCENES` compile-length opportunity.

## Metadata

**Confidence breakdown:**

- Gate, counts and splits: HIGH. Every site was re-read, and the commands were run.
- Spike harness and APIs: HIGH for API facts. MEDIUM for the metric recipes, which are proposals the plan must fix before measuring.
- Spike outcomes: LOW, by design.

**Research date:** 2026-10-14
**Valid until:** about 2026-11-14, or until any edit to `crates/liquidfun-wasm/src/scene*`, `web/src/catalog` or the xtask playground module.
