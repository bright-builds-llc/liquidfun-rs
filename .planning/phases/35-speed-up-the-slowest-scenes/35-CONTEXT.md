---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 35-2026-10-07T17-30-21
generated_at: 2026-10-07T17:31:19.566Z
---

# Phase 35: Speed Up the Slowest Scenes - Context

**Gathered:** 2026-10-07
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Profile the slowest playground scenes from the Phase 34 survey, name each one's hot path, and keep only native Rust changes that make those scenes measurably faster (beyond the survey's run-to-run noise) while leaving scene behavior, settings, controls and visuals unchanged. Rust and web tests stay green. The whole-catalog before/after table and doc wrap-up belong to Phase 36.

</domain>

<decisions>
## Implementation Decisions

### Targets
- **D-01:** Target the survey's top five by median ms/step: liquid-tumbler (24.440), tesla-valve (4.118), stacked-drip (2.640), washing-machine (2.104), particles (1.580). The baseline is the committed survey in `docs/benchmarks/scene-survey.md` (commit `96a3ac6a6`); re-record a fresh "before" on the current HEAD before the first change, because later commits may have moved the numbers.
- **D-02:** liquid-tumbler is the priority. It is about 6× slower than the next scene, mainly because it runs `PARTICLE_ITERATIONS = 61` substeps over 3,800 particles. That count is authored behavior and must not drop; the gain has to come from cheaper work per particle-solver iteration.
- **D-03:** The "visibly drops frames in the browser" check is a light, optional observation: if the playground can be opened locally (dev server plus the in-app browser pane), note any scene that visibly stutters and add it to the target list only if it is not already covered. If the browser check cannot run, record that, and the native top five remain the target set. No new browser timing harness.

### Where fixes go
- **D-04:** Prefer fixes in the shared engine hot paths (`crates/liquidfun/src/particle/**`, contact/solver code) because the top scenes share the particle pipeline, so one fix can help several. Use scene-module changes only when the profile names scene-specific per-step work (for example a hook doing redundant work each step).
- **D-05:** Never change authored scene settings to gain speed: particle counts, iterations, substeps, radii, damping, geometry, gravity, flags, presets and controls stay the same (REQUIREMENTS Out of Scope). No nondeterministic parallelism, no SIMD that changes results, no new `unsafe` unless it is narrow, `SAFETY:`-documented and measured to be necessary.
- **D-06:** Speed outranks compact memory: bounded, measured memory increases (caches, reused scratch buffers, precomputed tables) are allowed (carried forward from the v1.4 decision).

### Behavior preservation
- **D-07:** "Unchanged behavior" means a bit-identical simulation trajectory. Add an end-state fingerprint to the survey tooling (a hash of live particle position/velocity bits plus body transforms after the warmup and timed steps of the first run) and report it per scene. A kept change must leave all 25 scene fingerprints identical to the before run. A change that alters float results (reassociation, fused ops, reordered accumulation) is rejected, not documented away.
- **D-08:** The fingerprint lives in the measurement tooling (`crates/liquidfun-wasm/src/scene_spot.rs`, the bin and the xtask driver), never in production scene modules (Phase 34 D-11, REQUIREMENTS acceptance).

### Measurement and keep rule
- **D-09:** Profile with `samply` (installed) against the existing `[profile.profiling]` (release plus debug symbols), driving one scene through the survey bin. The existing `advance_profiled` / `DiagnosticStepProfile` phase timers may be used for a per-phase breakdown. Each target's profile note names its hot path (function and share of samples).
- **D-10:** Add a `--scene <id>` filter (repeatable) to the survey bin and xtask so a single target can be timed and profiled without running all 25. The full catalog run stays the default and keeps its coverage check.
- **D-11:** Keep rule: compare before and after on the same machine with the same flags (60 warmup, 120 steps, at least 3 runs; use 5 runs when the spread is wide). A change is kept only if the after median is below the before minimum for the targeted scene, no other scene's median rises above its before maximum, and all fingerprints match. Otherwise revert it and note the attempt, its numbers and the reason.

### Plan shape and records
- **D-12:** The roadmap says one plan per targeted scene. The planner may instead group by shared hot path when several targets share one, as long as every target ends with a profile note and a kept-or-reverted record. Start with a plan that adds the tooling (D-07, D-10) and records the fresh before run plus profiles of all five targets.
- **D-13:** Record profiles, attempts (kept and reverted) and before/after numbers in a phase-local `35-PROFILES.md` in the phase directory. Leave `docs/benchmarks/scene-survey.md` alone except for tooling-description changes the new flags or fields require; the whole-catalog before/after rewrite is Phase 36.
- **D-14:** Every commit passes the normal local checks: `cargo fmt --all`, clippy with `-D warnings`, the build, `cargo test`, and the web tests when web code or shared assets change. Run `just markdown-check` after editing non-GSD Markdown.

### Claude's Discretion
- The fingerprint hash function and the exact state fields beyond positions, velocities and body transforms.
- The order in which targets are attacked after liquid-tumbler, and how plans group shared hot paths.
- Specific optimization techniques (allocation reuse, loop restructuring, data layout, early-outs), within D-05 and D-07.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and requirements
- `.planning/ROADMAP.md` §Phase 35 — goal and success criteria
- `.planning/REQUIREMENTS.md` — PERF-08, PERF-09, Acceptance rules and Out of Scope table
- `PROJECT-SCOPE.md` — experimental scope; local checks gate commits
- `.planning/STATE.md` §Decisions — v1.4 speed-over-memory and preserved-behavior decisions

### Survey baseline and tooling
- `docs/benchmarks/scene-survey.md` — recorded baseline table, method and reproduce command
- `.planning/phases/34-scene-timing-survey/34-CONTEXT.md` — survey tooling decisions (D-01 to D-13) this phase extends
- `.planning/phases/34-scene-timing-survey/34-02-SUMMARY.md` — recorded run provenance

### Standards
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/verification.md`, `standards/core/testing.md`, `standards/languages/rust.md` — repo workflow, testing and Rust rules

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/liquidfun-wasm/src/scene_spot.rs` and `src/bin/playground_scene_spot.rs`: survey runner with `--warmup`, `--steps` and `--runs`; extend with `--scene` and the fingerprint.
- `tools/xtask/src/playground/spot.rs`: xtask driver, coverage check against `SCENE_IDS`, ranked table output.
- `SessionCore::advance_profiled` (`crates/liquidfun-wasm/src/session.rs:198`) and `DiagnosticStepProfile` (`crates/liquidfun/src/world/observation/profile.rs`): existing per-phase step timers.
- `[profile.profiling]` in the root `Cargo.toml`: release plus debug symbols for samply.

### Established Patterns
- Scenes are built in `crates/liquidfun-wasm/src/scene/*.rs`; `scene::particle_iterations(id)` feeds `with_particle_iterations` in `session.rs`.
- The particle pipeline lives in `crates/liquidfun/src/particle/` (contact scan, solver, body contact, force, storage).
- Tests use Arrange/Act/Assert comments; errors are closed enums; no `unwrap` in production code.

### Integration Points
- `just playground-scene-spot` → `cargo xtask playground scene-spot` → `cargo run --release --bin playground-scene-spot`. A test pins the justfile recipe text.

</code_context>

<specifics>
## Specific Ideas

- liquid-tumbler: 3,800 particles × 61 particle iterations per step. Expect the hot path in per-iteration particle solver work (pressure, contact weights, collision against the glass chain).
- tesla-valve is the only top-five scene whose particle count grows during the timed window (0 → 2,850); its profile should separate emission/spawning cost from solver cost.

</specifics>

<deferred>
## Deferred Ideas

- A browser frame-timing harness or WASM-specific profiling, unless the optional browser observation (D-03) shows a scene that native timing misses.
- Opt-in parallel or SIMD stepping (REQUIREMENTS Future Requirements).
- Whole-catalog before/after table and user-facing performance notes (Phase 36).

</deferred>

---

*Phase: 35-speed-up-the-slowest-scenes*
*Context gathered: 2026-10-07*
