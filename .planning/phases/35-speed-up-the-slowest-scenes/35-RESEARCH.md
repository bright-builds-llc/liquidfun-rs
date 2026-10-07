# Phase 35: Speed Up the Slowest Scenes - Research

**Researched:** 2026-10-07
**Domain:** Native Rust particle-solver hot paths (LiquidFun port), headless survey tooling, bit-exact behavior preservation
**Confidence:** HIGH for tooling, profiles, and the two prototyped fixes; MEDIUM for the unprototyped fix candidates

## Summary

All five targets spend most of their step time in the shared particle pipeline that runs once per particle iteration: `World::update_particle_contacts` (proxy rebuild + full sort + window scan), `World::update_body_contacts`, `run_collision` (CCD ray casts), and the `damping`/`pressure` contact loops. I recorded samply profiles of all five on the current HEAD (`1e5908c45`, the same engine code as baseline `96a3ac6a6`; only docs/tests changed since). For liquid-tumbler the top self-time functions are `pressure::damping` 23%, `contact_scan::consider_window` 10%, `pressure::pressure` 7%, `refresh_solver_weights` 6%, the `ContactProxy` stable sort about 11% inclusive, and `ChainShape::child_edge` 4.7%.

I prototyped two fixes in a throwaway copy of the repo (`git archive`, scratchpad only). Both left all 25 per-scene end-state fingerprints bit-identical. Both also passed `cargo test -p liquidfun --all-features` (75 test binaries) and `cargo test -p liquidfun-wasm` (307 tests).
1. Reuse the previous iteration's sorted proxy order, recompute tags in place, then sort. liquid-tumbler went from 24.44 to about 22.0 ms/step. With an insertion sort, stacked-drip went from 2.64 to 2.45 and particles from 1.58 to 1.52.
2. Build the chain child edge once per child instead of once per particle in body-contact generation. liquid-tumbler went from about 22.1 to about 21.2.

Making the particle `damping` loop branchless (`select_unpredictable`, two variants including a signed-zero-safe one) kept fingerprints identical but was **slower** (liquid-tumbler 24.0 to 25.3 ms vs 21.2 to 22.1 without it). Damping is an inherent dependent scatter; record that attempt as rejected.

Washing-machine is different: 30% of its samples are the **full-scan** CCD path at particle iteration 0 for the moving drum fixtures (`filtered_collision_hits` falls back to `for particle in 0..n` when the fixture moved). Tesla-valve is spread out: per-fixture AABB proxy queries (about 13% self in `visit_sorted_tag_indices_in_aabb`), per-particle emission via `create_particle_with_def` → `prepare_create` (8.4%, O(n) clone per created particle), and the lifetime `EvictionIndex` rebuild with SipHash `HashMap` (8.2%).

**Primary recommendation:** Plan 01 adds `--scene` and an FNV-1a end-state fingerprint (computed in `scene_spot.rs` through a cfg-widened `SessionCore::read_particles`), records a fresh before run, and profiles the five targets. Then attack the shared proxy-sort hot path first, then geometry hoists, then the scene-specific collision (washing-machine) and emission/lifetime (tesla-valve) paths. Do every before/after comparison A/B against a saved "before" binary with fingerprints checked.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Targets
- **D-01:** Target the survey's top five by median ms/step: liquid-tumbler (24.440), tesla-valve (4.118), stacked-drip (2.640), washing-machine (2.104), particles (1.580). The baseline is the committed survey in `docs/benchmarks/scene-survey.md` (commit `96a3ac6a6`); re-record a fresh "before" on the current HEAD before the first change, because later commits may have moved the numbers.
- **D-02:** liquid-tumbler is the priority. It is about 6× slower than the next scene, mainly because it runs `PARTICLE_ITERATIONS = 61` substeps over 3,800 particles. That count is authored behavior and must not drop; the gain has to come from cheaper work per particle-solver iteration.
- **D-03:** The "visibly drops frames in the browser" check is a light, optional observation: if the playground can be opened locally (dev server plus the in-app browser pane), note any scene that visibly stutters and add it to the target list only if it is not already covered. If the browser check cannot run, record that, and the native top five remain the target set. No new browser timing harness.

#### Where fixes go
- **D-04:** Prefer fixes in the shared engine hot paths (`crates/liquidfun/src/particle/**`, contact/solver code) because the top scenes share the particle pipeline, so one fix can help several. Use scene-module changes only when the profile names scene-specific per-step work (for example a hook doing redundant work each step).
- **D-05:** Never change authored scene settings to gain speed: particle counts, iterations, substeps, radii, damping, geometry, gravity, flags, presets and controls stay the same (REQUIREMENTS Out of Scope). No nondeterministic parallelism, no SIMD that changes results, no new `unsafe` unless it is narrow, `SAFETY:`-documented and measured to be necessary.
- **D-06:** Speed outranks compact memory: bounded, measured memory increases (caches, reused scratch buffers, precomputed tables) are allowed (carried forward from the v1.4 decision).

#### Behavior preservation
- **D-07:** "Unchanged behavior" means a bit-identical simulation trajectory. Add an end-state fingerprint to the survey tooling (a hash of live particle position/velocity bits plus body transforms after the warmup and timed steps of the first run) and report it per scene. A kept change must leave all 25 scene fingerprints identical to the before run. A change that alters float results (reassociation, fused ops, reordered accumulation) is rejected, not documented away.
- **D-08:** The fingerprint lives in the measurement tooling (`crates/liquidfun-wasm/src/scene_spot.rs`, the bin and the xtask driver), never in production scene modules (Phase 34 D-11, REQUIREMENTS acceptance).

#### Measurement and keep rule
- **D-09:** Profile with `samply` (installed) against the existing `[profile.profiling]` (release plus debug symbols), driving one scene through the survey bin. The existing `advance_profiled` / `DiagnosticStepProfile` phase timers may be used for a per-phase breakdown. Each target's profile note names its hot path (function and share of samples).
- **D-10:** Add a `--scene <id>` filter (repeatable) to the survey bin and xtask so a single target can be timed and profiled without running all 25. The full catalog run stays the default and keeps its coverage check.
- **D-11:** Keep rule: compare before and after on the same machine with the same flags (60 warmup, 120 steps, at least 3 runs; use 5 runs when the spread is wide). A change is kept only if the after median is below the before minimum for the targeted scene, no other scene's median rises above its before maximum, and all fingerprints match. Otherwise revert it and note the attempt, its numbers and the reason.

#### Plan shape and records
- **D-12:** The roadmap says one plan per targeted scene. The planner may instead group by shared hot path when several targets share one, as long as every target ends with a profile note and a kept-or-reverted record. Start with a plan that adds the tooling (D-07, D-10) and records the fresh before run plus profiles of all five targets.
- **D-13:** Record profiles, attempts (kept and reverted) and before/after numbers in a phase-local `35-PROFILES.md` in the phase directory. Leave `docs/benchmarks/scene-survey.md` alone except for tooling-description changes the new flags or fields require; the whole-catalog before/after rewrite is Phase 36.
- **D-14:** Every commit passes the normal local checks: `cargo fmt --all`, clippy with `-D warnings`, the build, `cargo test`, and the web tests when web code or shared assets change. Run `just markdown-check` after editing non-GSD Markdown.

### Claude's Discretion
- The fingerprint hash function and the exact state fields beyond positions, velocities and body transforms.
- The order in which targets are attacked after liquid-tumbler, and how plans group shared hot paths.
- Specific optimization techniques (allocation reuse, loop restructuring, data layout, early-outs), within D-05 and D-07.

### Deferred Ideas (OUT OF SCOPE)
- A browser frame-timing harness or WASM-specific profiling, unless the optional browser observation (D-03) shows a scene that native timing misses.
- Opt-in parallel or SIMD stepping (REQUIREMENTS Future Requirements).
- Whole-catalog before/after table and user-facing performance notes (Phase 36).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-08 | Maintainers can inspect profile-guided fixes for the slowest scenes, each kept only with a before/after survey gain beyond run-to-run noise. | Profiles of all five targets with named hot paths and shares (§Profiles); the samply headless recipe and analysis script (§Code Examples); two prototyped keepers with measured gains; one prototyped reject; the A/B measurement method (§Pitfalls 1–2). |
| PERF-09 | Visitors see unchanged scene behavior, settings, controls and visuals after the fixes; existing Rust and web tests stay green. | The fingerprint design is verified deterministic across runs, processes, and release vs profiling builds; it covers all 25 scenes (§Architecture Pattern 1). Float-order rules are in §Don't Hand-Roll / §Pitfalls. Test commands and timings are in §Verification Commands. |
</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- Production code: no `unwrap()`. Propagate errors with closed error enums. Use `expect()` only with invariant messages for impossible states. Use the `maybe_` prefix for optional internals, `let...else` guards, and tau for full rotations. [VERIFIED: CLAUDE.md]
- Safe Rust is the default. Any new `unsafe` must be narrow, carry a `SAFETY:` comment, be measured, and have focused tests (also D-05). [VERIFIED: CLAUDE.md]
- Determinism: stable ordering and reproducible scenarios take precedence. Never use `HashMap`/`HashSet` iteration in solver-visible order. No `-ffast-math`, fused ops, or `-march=native`. [VERIFIED: CLAUDE.md "What NOT to Use"]
- Tests: one concern per test, with explicit `// Arrange`, `// Act`, `// Assert` comments. [VERIFIED: CLAUDE.md, standards/core/testing.md]
- File length: tracked source files must stay at or under **628 physical lines** (`bun scripts/bright-builds-check.ts all`). Files near the limit: `tools/xtask/src/playground/spot.rs` (547), `crates/liquidfun/src/particle/storage/runtime.rs` (574), `crates/liquidfun/src/world/particle_coupling.rs` (494). Put new code in submodules (`foo.rs` + `foo/`, never `mod.rs`). [VERIFIED: ran the check, 0 findings currently]
- `.planning/**` must never be formatted with mdformat. Run `just markdown-check` after changing non-GSD Markdown (mdformat 1.0.0 under Python ≥ 3.13). [VERIFIED: AGENTS.md]
- Pre-commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, `cargo test --all-features`, plus web checks when web code or shared assets change. [VERIFIED: justfile, CLAUDE.md]
- The `justfile` must not contain the string `samply`, `cmake`, or `dhat`, and the `playground-scene-spot` recipe text is pinned verbatim (`tools/xtask/tests/playground_cli/spot.rs::justfile_keeps_the_one_line_playground_scene_spot_alias`). **Do not add a samply recipe to the justfile.** [VERIFIED: test source]
- GSD workflow: work happens through `/gsd-execute-phase`. Standing authorization covers commits and pushes to `main`. Independent review may be done by AI. [VERIFIED: AGENTS.md]
- User memory: prefer lightweight process and small iterations, and commit fixes promptly. The v1.4 harness was shelved for overgrowth, so **do not build a compare/evidence harness**. Compare with jq/python one-liners in the scratchpad. [VERIFIED: MEMORY.md]

## Standard Stack

No new dependencies are needed. Everything below is already installed or in the workspace.

| Tool | Version (verified) | Purpose | Notes |
|------|--------------------|---------|-------|
| rustc / cargo | 1.97.0 | Build/test | `rust-toolchain.toml` pin [VERIFIED: `rustc --version`] |
| samply | 0.13.1 | CPU sampling profiler | `~/.cargo/bin/samply`; `--save-only --unstable-presymbolicate` works headless on macOS [VERIFIED: ran it] |
| `[profile.profiling]` | inherits release, `debug = true`, `strip = false` | Symbolized optimized binary | Timings match `--release` (24.44 ms liquid-tumbler both) [VERIFIED: ran both] |
| dsymutil / atos | Xcode CLT | Optional line-level attribution | `dsymutil <bin>` first, then `atos -i -o <bin> -l 0x100000000 <addr>` [VERIFIED: ran it] |
| python3 | 3.14.6 | Parse samply JSON for text "top functions" | stdlib only (`gzip`, `json`, `bisect`) [VERIFIED] |
| bun | 1.4.2 | Web unit tests (`bun run test:unit`, vitest) | about 4 s [VERIFIED: ran it] |
| wasm-pack | 0.15.0 | `just web-wasm` / `just web-build` | wasm32 target installed [VERIFIED] |
| mdformat | 1.0.0 (+gfm, frontmatter) | `just markdown-check` | [VERIFIED] |

**Hash for the fingerprint:** hand-rolled FNV-1a 64 over little-endian `to_bits()` bytes (about 6 lines, no dependency). Do not use `std::collections::hash_map::DefaultHasher`, because its algorithm is not guaranteed stable across Rust releases and the before/after comparison must survive a toolchain bump. [ASSUMED: std docs state DefaultHasher's algorithm is unspecified; consistent with training knowledge]

## Profiles (recorded this session, HEAD 1e5908c45, Apple M4 Max, samply 4 kHz)

Samples were restricted to stacks under `SessionCore::advance`, so construction is excluded. They include warmup steps because samply records the whole process. The survey times only the measured window, but the per-step mix is the same.

### Inclusive share by phase

| Phase | liquid-tumbler | tesla-valve | stacked-drip | washing-machine | particles |
| --- | ---: | ---: | ---: | ---: | ---: |
| particle contacts (`World::update_particle_contacts`) | 27.7% | 13.4% | 38.7% | 20.5% | 36.6% |
| └ proxy rebuild + sort (`contact_scan::rebuild_proxies`) | 12.3% | 3.5% | 11.7% | 4.4% | 9.0% |
| body contacts (`World::update_body_contacts`) | 17.5% | 23.9% | 15.6% | 10.1% | 6.8% |
| collision (`SystemPassExecutor::run_collision`) | 14.2% | 26.5% | 13.7% | 35.8% | 7.9% |
| └ full-scan CCD (iteration 0, moving fixture) | 0.0% | 0.0% | 1.2% | **30.0%** | 2.7% |
| `pressure::damping` | **23.5%** | 5.3% | 17.5% | 11.3% | **23.4%** |
| `pressure::pressure` | 8.0% | 2.8% | 6.2% | 5.0% | 7.5% |
| `ParticleStorage::refresh_solver_weights` | 6.1% | 1.3% | 3.5% | 2.8% | 6.0% |
| `begin_boundary` (lane copy into `BoundaryCandidate`) | 2.3% | 0.5% | 2.5% | 0.7% | 1.5% |
| scene `on_advance` (emission) | 0 | **8.5%** | 0 | 0 | 0 |
| scene `on_after_step` (`destroy_particles_in_shape`) | 0 | 4.4% | 0 | 0 | 0 |
| `ParticleLifetimeState::solve_lifetimes` | 0 | **8.8%** | 0 | 0.4% | 0.9% |
| `backup_step_limit_state` (per-step world clone) | 0.3% | 1.5% | 0.7% | 2.2% | 4.1% |

[VERIFIED: samply profiles + `phase_table.py`, scratchpad]

### Named hot paths per target (self time)

- **liquid-tumbler:**
  - `pressure::damping` 23.1%. Line-level: the particle-contact loop, `pressure.rs:173-183`. Samples sit on the branch landing and the velocity gathers.
  - `consider_window` 10.0%.
  - `pressure::pressure` 7.3%.
  - `refresh_solver_weights` 6.0%.
  - `ContactProxy` stable sort (`driftsort`/quicksort/smallsort) about 11% inclusive.
  - `fill_stored_contacts` 5.1%.
  - `ChainShape::child_edge` 4.7%. It is rebuilt and validated per particle: 2.8% from `body_contact::generate` → `distance_to_point` and 1.9% from CCD `ray_cast`.
  - `sort_unstable::<usize>` in `body_contact::collect_candidate_rows` 4.1% inclusive.
  - `_platform_memmove` 3.7%, about half of it from `BoundaryCandidate::new_with_buffers`.
  - `validate_groups`/`membership_ranges` from `swap_solver_candidate` 1.2%.
  - `maybe_current_contact_proxies` revalidation 1.3%.
- **tesla-valve** (`PARTICLE_ITERATIONS = 4`; 0 → 2,850 particles; the emission cost is separate from solver cost):
  - `_platform_memmove` 8.3%, from `prepare_create` (×2 per created particle) and `backup_step_limit_state`.
  - `push_fixture_particle_hit` 7.1%.
  - `visit_sorted_tag_indices_in_aabb` 6.7% (collision queries) + 6.3% (body-contact queries).
  - `consider_window` 5.8%.
  - `body_contact::generate` 5.8%.
  - `PolygonShape::ray_cast` 5.3%.
  - `damping` 4.9%.
  - SipHash `Hasher::write` 3.5%, `BTreeMap::insert` 2.3% (`EvictionIndex::resequence_to_storage_order`).
  - `membership_ranges` 2.7% (`prepare_create` → `rebuild_group_records_for_system`).
  - Emission: `TeslaValveHooks::on_advance` → `World::create_particle_with_def` 8.4% inclusive.
- **stacked-drip** (`PARTICLE_ITERATIONS = 12`): `damping` 16.9%, `consider_window` 16.8%, `fill_stored_contacts` 10.1%, `pressure` 5.0%, proxy sort about 8.3%, `body_contact::generate` 3.6%, `refresh_solver_weights` 3.4%.
- **washing-machine** (motorized revolute drum of polygon ribs):
  - `push_fixture_particle_hit` 16.5% self, about 27% inclusive when called directly from the full-scan loop in `filtered_collision_hits`.
  - `collision_start_from_previous_transform` 8.6%.
  - `particle_travel_aabb` 4.6%.
  - `damping` 11.0%.
  - `consider_window` 10.9%.
- **particles:**
  - `damping` 23.0%.
  - `consider_window` 18.4%.
  - `fill_stored_contacts` 9.1%.
  - `pressure` 6.8%.
  - `refresh_solver_weights` 5.9%.
  - Proxy sort about 5.6%.
  - Note: 22% of the *whole-process* samples are scene **construction** (`create_particle_group` → `append_group_particle` → `prepare_create`, O(n²)). That cost is not in the timed survey window. Do not chase it for PERF-08.

The `DiagnosticStepProfile` phase timers (`SessionCore::advance_profiled`) only give coarse parents (contact_update, rigid_solve, particle_prepare, particle_solve, ...). samply function shares are the right "named hot path" evidence for D-09. [VERIFIED: profile.rs, session.rs:198]

## Architecture Patterns

### Pattern 1: Fingerprint and `--scene` in survey tooling only (D-07, D-08, D-10)

**State access.** `SessionCore` owns `world`/`particle_system` privately. `SessionCore::read_particles(&self, impl FnOnce(&World, ParticleSystemId) -> T)` already exists but is `#[cfg(test)]` (`session.rs:413-416`). Widen it to `#[cfg(any(test, not(target_arch = "wasm32")))]`, matching `live_particle_count`. That is the only production-file change, and it is an attribute, not logic. Then compute the fingerprint entirely in `scene_spot.rs` (D-08) using public engine APIs:
- `World::particle_system_view(system)` → `positions()`, `velocities()`, `maybe_colors()`, `particle_ids()` (`crates/liquidfun/src/particle/view.rs:50-137`).
- `World::world_observation(WorldObservationLimits::reviewed())` → `bodies()`, each `BodyObservation::snapshot()` → `position()`, `angle()`, `linear_velocity()`, `angular_velocity()` (`world/observation.rs:68`, `observation/collection.rs:38`, `world/body.rs:519-567`). It succeeded for all 25 scenes. Max particle contacts seen was 25,538, against the reviewed cap of 65,536. On error, fail closed with a new `SceneSpotError::Fingerprint { scene }` rather than hashing a sentinel.
- Recommended hashed fields, in order: particle count; per-particle position x/y bits, velocity x/y bits, and color components when the color lane exists (visuals; color-mixer mixes colors); body count; per-body position x/y, angle, linear velocity x/y, angular velocity bits in observation order. [VERIFIED: prototype hashed positions/velocities/bodies]

**When:** after warmup and measured steps of **run 1**, outside the timed window (`time_run` already reads `end_particles` there). Return it via `RunSample`, and keep the first run's value in `SceneSpotSample` as `fingerprint: String` (16 lowercase hex chars). Emit it in `to_json` as `"fingerprint":"997fde4d8b3b9f43"`.

**Verified determinism:** fingerprints were identical across 2 runs in one process, across separate processes, and between `--release` and `--profile profiling` builds, for all 25 scenes. [VERIFIED: prototype runs]

**`--scene` (repeatable):**
- **Bin** (`src/bin/playground_scene_spot.rs`): parse `--scene <id>` into a list. Pass it to a `run_scene_spot` variant (for example `run_scene_spot(warmup, steps, runs, &scene_filter)`, where an empty filter means the full catalog). Reject unknown ids with a closed error (`SceneSpotError::UnknownScene`) using `SURVEY_SCENES` ids. Reject or dedupe duplicates. Output stays in catalog order.
- **xtask** (`tools/xtask/src/playground/spot.rs`): parse repeatable `--scene`, validate each id against `catalog_scene_ids(CATALOG_SCENES_TS)`, and forward `--scene <id>` to the bin. **Coverage check:** with no filter, keep `validate_scene_samples` exactly as is (full catalog). With a filter, require the output set to equal the requested set. Require a 16-hex `fingerprint` field per sample. spot.rs is at 547/628 lines, so put argument parsing and validation in a new `spot/args.rs` (or similar) submodule.
- **Ranked table:** keep the table columns unchanged so Phase 36 can paste the same shape. Keep fingerprints in the JSON only (superseded: no stdout list; see Open Questions).
- **Fake tool** `tools/xtask/tests/fixtures/fake_upstream_tool.rs::print_scene_spot_sample` must honor `--scene` and print a `fingerprint` field, or the CLI integration test fails.

**Tests that pin output and need updates:**
- `crates/liquidfun-wasm/src/scene_spot/tests.rs::to_json_reports_survey_fields` (field list; add a `fingerprint` assertion). `run_scene_spot_rejects_zero_runs` / `_zero_measured_steps` call `run_scene_spot(0, 1, runs)`, so update the call sites if the signature changes.
- `tools/xtask/src/playground/spot.rs` unit tests: `valid_sample()` must gain `fingerprint` if validation requires it, and `parse_spot_counts_*` covers the new flag.
- `tools/xtask/tests/playground_cli/spot.rs`: the justfile recipe pin (unchanged recipe text is fine), `report["runs"]`, and the `| Rank | Scene | Median ms/step |` prefix. Add a CLI test for `--scene`.
- `tools/xtask/src/playground.rs::USAGE` string. Its tests only check `contains("scene-spot")`, so update the text to mention `--scene <id>`.
- `docs/benchmarks/scene-survey.md` "Reproduce" text may mention the new flag and field (D-13 allows tooling-description changes only). Run `just markdown-check`.

### Pattern 2: Bit-identical optimization rules (D-07)

A change preserves bits if every float value is produced by the same IEEE operations on the same operands in the same order. Allowed and forbidden changes:
- **Allowed:** moving or hoisting identical computations (computing the chain child `EdgeShape` once per child), skipping work whose result is provably discarded, reusing buffers instead of allocating, changing sort algorithms when the sort key is a **total** order (the output is then unique), and replacing `HashMap` hashers or structures that are only used for keyed lookup.
- **Forbidden:** reassociation (`a+b+c` → `a+(b+c)`), `mul_add`/FMA, accumulating in a different order (for example fusing weight accumulation into the contact scan changes the `weights[i]` summation order from body-contacts-then-particle-contacts), precomputing a composed transform `T_cur·T_prev⁻¹` in place of applying the two transforms sequentially, SIMD lane reductions, and changing contact or hit order.
- **Signed zero trap:** `v + 0.0` turns `-0.0` into `+0.0`. The identity adds are `v + (-0.0)` and `v - (+0.0)`.

### Pattern 3: Measurement loop per attempt (D-11)

1. On the tooling commit (before any engine change), build once: `cargo build --release -p liquidfun-wasm --bin playground-scene-spot`, then `cp target/release/playground-scene-spot <scratch>/spot-before`.
1. After each change, build again and copy to `<scratch>/spot-after-<attempt>`.
1. Run A/B interleaved on the same machine and flags: `spot-before --scene X --runs 3`, `spot-after --scene X --runs 3`, repeated once more (B/A order) if the gap is under about 5%. Use `--runs 5` when the spread is wide.
1. Full-catalog fingerprint check with both binaries (`--runs 1` is enough for fingerprints; about 15 s each).
1. Full-catalog timing (3 runs, before then after) for the "no other scene's median above its before max" rule.
1. Record in `35-PROFILES.md`: attempt, commit, numbers, fingerprints equal yes/no, kept/reverted, and reason.

### Recommended plan breakdown (D-12)

| Plan | Scope | Targets helped | Evidence from this research |
|---|---|---|---|
| 35-01 | Tooling (`--scene`, fingerprint, fake tool, tests, doc reproduce text), fresh before run (full catalog, 3 runs), save before binary, samply profiles of all 5, `35-PROFILES.md` skeleton with profile notes, optional D-03 browser glance | all | §Pattern 1, §Profiles |
| 35-02 | **Particle-contact proxy order reuse** (retain last sorted `ContactProxy` order across iterations, recompute tags in place, sort with total key) | LT, stacked-drip, particles (+ every scene) | Prototype: LT 24.44 → 22.0; stacked-drip → 2.45; particles → 1.52 (insertion sort); fingerprints identical |
| 35-03 | **Geometry hoists in contact/collision**: chain child edge once per child in `body_contact::generate` and in CCD (`CcdFixtureRecord.children` carries an `Option<EdgeShape>`); optional `collect_candidate_rows` sort → row-marker/bitset; per-row x-range binary search in `visit_sorted_tag_indices_in_aabb` | LT (edge, rows), tesla-valve + stacked-drip (AABB query) | Prototype: edge hoist LT 22.1 → 21.2, fingerprints identical; the AABB query and bitset are unprototyped (MEDIUM) |
| 35-04 | **Moving-fixture CCD at iteration 0** (washing-machine): sound conservative spatial filter instead of the full `0..n` scan, and/or per-fixture hoisting of `transform_is_finite` validation in `collision_start_from_previous_transform` | washing-machine (also soup-stirrer, water-wheel, theo-jansen) | Profile: 30% of washing-machine is the full-scan path; unprototyped, needs a soundness argument + unit tests (MEDIUM-LOW) |
| 35-05 | **Tesla-valve emission and lifetime**: avoid O(n) clones in `ParticleStorage::prepare_create` (twice per emitted particle) for ungrouped particles; make `EvictionIndex` cheaper (no SipHash/rehash growth: `with_capacity`, slot-indexed `Vec`, or a deterministic hasher) | tesla-valve (fountain, water-wheel, sparky emit too) | Profile: 8.4% + 8.2%; unprototyped (MEDIUM) |
| (record) | `damping` branchless attempt: rejected | LT, particles, stacked-drip | Prototype: two variants, both slower (+13–14%) with identical fingerprints |

Attack order after liquid-tumbler: 35-02 (largest, broadest), 35-03, then 35-04 (washing-machine) and 35-05 (tesla-valve) in either order. Each plan ends with a kept-or-reverted record per targeted scene.

### Anti-Patterns to Avoid
- **Comparing against the committed survey numbers.** Cross-invocation drift is about ±3% (particles measured 1.57, 1.59, 1.63, 1.65 in different invocations of the same code). Always A/B with saved binaries.
- **Insertion sort without a budget on reused proxies.** When particles are destroyed and created in the same step (tesla-valve, sparky), the length can stay equal while rows map to unrelated positions. The reused order is then arbitrary, and insertion sort degrades to O(n²) (about 14 M moves at 3,800). Use `sort_unstable_by_key(|p| (p.tag, p.row))` or `sort_by` (pattern-adaptive, O(n log n) worst case), or an insertion sort with a shift budget that falls back to a full sort.
- **Building an evidence or compare harness.** Use one-liners. Keep `35-PROFILES.md` as the record.
- **Adding samply to the justfile.** A test forbids it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CPU profiling | Custom timers in hot loops | `samply record --save-only --unstable-presymbolicate` + the small python reader below | Timers perturb hot loops; samply is installed and works headless |
| Sorting proxies | A custom radix sort (first attempt) | `slice::sort_unstable_by_key` / `sort_by` with the total key `(tag, row)` | std sorts are pattern-adaptive on nearly sorted input; a total key makes any correct sort bit-identical |
| Behavior check | Tolerance-based comparisons | Exact `to_bits()` fingerprint (FNV-1a) | D-07 requires bit identity; tolerance hides reordering |
| Before/after compare | xtask subcommand or new stamp format | jq/python in the scratchpad over bin JSON lines | Lightweight process (user memory); Phase 34 shelved harness growth |
| Faster `HashMap` hasher | A new crate dependency (`ahash`, `rustc-hash`) | A slot-indexed `Vec` or a tiny in-tree deterministic hasher, if needed | Minimizes dependencies; lookups only, so order does not matter |

**Key insight:** In this engine the profitable wins are **redundant work per particle iteration** (re-sorting nearly sorted proxies from scratch, rebuilding and validating identical geometry per particle, full scans where a spatial query would do). They are not arithmetic speedups. The arithmetic loops (`damping`, `pressure`, `consider_window`) are dependent scatters whose float order is locked by D-07.

## Code Examples

### Fingerprint (scene_spot.rs; FNV-1a over bits)

```rust
// Source: prototype verified in scratchpad (all 25 scenes deterministic)
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

struct Fnv1a(u64);

impl Fnv1a {
    fn mix_u32(&mut self, bits: u32) {
        for byte in bits.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(FNV_PRIME);
        }
    }
    fn mix_f32(&mut self, value: f32) {
        self.mix_u32(value.to_bits());
    }
}

fn end_state_fingerprint(world: &World, system: ParticleSystemId) -> Option<u64> {
    let mut hash = Fnv1a(FNV_OFFSET);
    let view = world.particle_system_view(system).ok()?;
    hash.mix_u32(u32::try_from(view.positions().len()).ok()?);
    for (position, velocity) in view.positions().iter().zip(view.velocities()) {
        hash.mix_f32(position.x);
        hash.mix_f32(position.y);
        hash.mix_f32(velocity.x);
        hash.mix_f32(velocity.y);
    }
    // also mix color components when view.maybe_colors() is Some
    let observation = world.world_observation(WorldObservationLimits::reviewed()).ok()?;
    hash.mix_u32(u32::try_from(observation.bodies().len()).ok()?);
    for body in observation.bodies() {
        let snapshot = body.snapshot();
        hash.mix_f32(snapshot.position().x);
        hash.mix_f32(snapshot.position().y);
        hash.mix_f32(snapshot.angle());
        hash.mix_f32(snapshot.linear_velocity().x);
        hash.mix_f32(snapshot.linear_velocity().y);
        hash.mix_f32(snapshot.angular_velocity());
    }
    Some(hash.0)
}
// call site in time_run, run 1 only: session.read_particles(end_state_fingerprint)
// map None -> SceneSpotError::Fingerprint { scene }
```

### Proxy order reuse (prototype shape, `contact_scan.rs` + storage)

```rust
// Source: prototype in scratchpad; fingerprints identical for all 25 scenes;
// liquidfun (75 binaries) and liquidfun-wasm (307 tests) passed.
// rebuild_proxies: when the retained buffer has exactly positions.len() entries it is a
// permutation of rows 0..n (it only ever comes from a full rebuild at that length).
if proxies.len() == positions.len() {
    let mut all_tagged = true;
    for proxy in proxies.iter_mut() {
        let position = positions[proxy.row];
        let Ok(tag) = checked_tag(inverse_diameter * position.x, inverse_diameter * position.y) else {
            all_tagged = false; // fall through to the row-order rebuild so the error is identical
            break;
        };
        proxy.tag = tag;
    }
    if all_tagged {
        proxies.sort_unstable_by_key(|proxy| (proxy.tag, proxy.row)); // total key → unique output
        return Ok(());
    }
}
// ...existing clear + row-order rebuild + sort...
```

Retention: `commit_boundary` (`executor/boundary_runtime.rs:174`) calls `storage.clear_contact_scan()` every iteration, and `ParticleStorage` documents `contact_proxies` as "Cleared before a step returns". Keep that contract and move the cleared vec into a separate cache field instead of clearing it. `take_contact_proxies` hands back the cache when `contact_proxies` is empty. `ParticleStorage` derives `PartialEq`, so wrap the cache in a newtype whose `PartialEq` always returns `true` (cache semantics) so storage equality and rollback tests are unaffected. Storage is cloned once per step by `backup_step_limit_state`, so the cache adds an n×16-byte copy (bounded, D-06).

### Chain edge hoist (`body_contact::generate`)

```rust
// Source: prototype; bit-identical because ChainShape::distance_to_point is exactly
// self.child_edge(child)?.distance_to_point(transform, point) (chain.rs:157-165).
let maybe_child_edge = match &source.shape {
    Shape::Chain(chain) => chain.child_edge(child).ok(),
    _ => None,
};
// per row:
let distance = match &maybe_child_edge {
    Some(edge) => edge.distance_to_point(source.transform, position),
    None => source.shape.distance_to_point(source.transform, position, child),
}
.expect("world-owned checked shapes and transforms remain queryable");
```

### Headless samply + text "top functions" on macOS

```bash
cargo build --profile profiling -p liquidfun-wasm --bin playground-scene-spot
samply record --save-only --unstable-presymbolicate -r 4000 \
  -o "$SCRATCH/lt.json.gz" -- target/profiling/playground-scene-spot --scene liquid-tumbler --runs 1
# writes lt.json.gz + lt.json.syms.json (sidecar symbol table)
python3 "$SCRATCH/top_functions.py" "$SCRATCH/lt.json.gz" 25 'SessionCore>::advance'
```

```python
# top_functions.py: self + inclusive % from samply's processed profile + .syms.json sidecar
import bisect, gzip, json, re, sys
from collections import Counter
path, top, needle = sys.argv[1], int(sys.argv[2]), sys.argv[3] if len(sys.argv) > 3 else None
p = json.load(gzip.open(path)); syms = json.load(open(path.replace('.json.gz', '.json.syms.json')))
strings = syms['string_table']; tables = {}
for lib in syms['data']:
    e = sorted((x['rva'], x['size'], strings[x['symbol']]) for x in lib['symbol_table'])
    tables[lib['debug_name']] = ([r[0] for r in e], e)
t = p['threads'][0]; ft, st, fn, rt = t['frameTable'], t['stackTable'], t['funcTable'], t['resourceTable']
def name(f):
    a, res = ft['address'][f], fn['resource'][ft['func'][f]]
    if res is None or res < 0: return '?'
    lib = p['libs'][rt['lib'][res]]['debugName']
    if lib not in tables: return lib
    s, e = tables[lib]; i = bisect.bisect_right(s, a) - 1
    return re.sub(r'::h[0-9a-f]{16}$', '', e[i][2]) if i >= 0 else lib
selfc, incl, total = Counter(), Counter(), 0
for s in t['samples']['stack']:
    names = []
    while s is not None: names.append(name(st['frame'][s])); s = st['prefix'][s]
    if not names or (needle and not any(needle in n for n in names)): continue
    total += 1; selfc[names[0]] += 1
    for n in set(names): incl[n] += 1
for label, c in (('self', selfc), ('inclusive', incl)):
    print('==', label)
    for n, k in c.most_common(top): print(f'{100*k/total:6.2f}%  {n[:160]}')
```

For line-level attribution, run `dsymutil target/profiling/playground-scene-spot`, then `atos -i -o <bin> -l 0x100000000 0x1<rva-hex>` (RVA = `frameTable.address`). [VERIFIED: ran all of the above]

## Common Pitfalls

### Pitfall 1: Cross-invocation timing drift masquerades as gain or regression
**What goes wrong:** The keep rule compares the after median against the before min. A separate invocation minutes later can drift by about 3%, which is larger than the within-invocation spread (particles: 1.574–1.615 committed; 1.57, 1.59, 1.63, 1.65 in four invocations of identical code this session).
**How to avoid:** Use saved before/after binaries, interleaved A/B/A/B, the same flags, no other heavy load, and 5 runs when the spread is wide. Re-run any apparent non-target regression with `--scene <id> --runs 5` before concluding.
**Warning signs:** A "gain" that appears in only one of two orderings.

### Pitfall 2: The full-catalog "no other scene regressed" check is noisy for tiny scenes
**What goes wrong:** drawing-particles (0.011 ms) and sparky (0.14 ms) have spreads near timer resolution.
**How to avoid:** Apply the rule, re-run suspicious scenes in isolation, and record the re-run in `35-PROFILES.md`. Never waive the fingerprint check.

### Pitfall 3: Reused proxy order and worst-case insertion sort
See Anti-Patterns. The length check is enough for correctness, because the output is unique under the total key. Performance needs an O(n log n) fallback.

### Pitfall 4: Signed zero and "harmless" branchless rewrites
`x + 0.0` is not an identity for `-0.0`. Branchless damping also measured slower here: unconditional stores lengthen the dependent store-to-load chain. Do not retry it beyond one recorded attempt.

### Pitfall 5: Error-path identity
When an optimization changes iteration order, an error that used to come from the first failing row in row order may now come from a different row or with a different variant. Fall back to the original row-order path on any error so the error stays the same (see the proxy example).

### Pitfall 6: Shared engine changes affect all 25 scenes and the WASM build
Every engine change must re-check all 25 fingerprints, not just the target. The web runs the same Rust compiled to wasm32. Op-identical Rust gives identical IEEE results there too, but run `just web-wasm` (or `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown`) to prove it compiles. Since fingerprints are bit-identical, README SVG/WebP previews need no regeneration.

### Pitfall 7: Release-only warning already present
`cargo build --release` / `--profile profiling` warn "unreachable expression" at `crates/liquidfun/src/particle/solver/boundary/support.rs:98` (`cfg(not(debug_assertions))` early return). Clippy in dev profile does not see it. It predates this phase. Fix it only if a plan touches that file (restructure as `#[cfg(debug_assertions)]` / `#[cfg(not(debug_assertions))]` blocks).

### Pitfall 8: Washing-machine CCD filter soundness
A spatial filter for moving fixtures at iteration 0 must be conservative. The ray start is `T_cur(T_prev⁻¹(p))`, so its displacement grows with distance from the body origin (bounded by `|Δx| + 2|sin(Δθ/2)|·|p − x_prev|`). Skipping a particle that would have hit changes results. The fingerprint catches this only in the surveyed scenes, so add unit tests with a rotating fixture comparing filtered and full-scan hit lists exactly. The existing `hits.sort_by_key(|hit| hit.particle)` (stable) makes query-order versus scan-order irrelevant: both yield the same per-particle fixture/child order, as the static-fixture path already relies on.

## State of the Art (in this repo)

| Old Approach | Current Approach | Notes |
|---|---|---|
| Rebuild proxies in row order + full stable sort every particle iteration | (proposed) retain last order, re-tag, adaptive sort | Upstream LiquidFun also sorts proxies per `UpdateContacts`. The repo keeps that shape; the total key keeps output identical |
| Build a `ChainShape` child `EdgeShape` per query | (proposed) build once per child per pass | `ChainShape::child_edge` validates every vertex each call (`edge.rs:35-64`) |
| `0..n` CCD scan for moving fixtures at iteration 0 | (proposed) conservative spatial query | The static and iteration>0 path already uses `query_particles_for_fixture` |

## Prior Tesla Valve Work (avoid re-doing it)

`docs/benchmarks/tesla-valve/README.md` records a 2026-10-02 browser campaign. It covered:
1. Spatial wall contacts (indexed body contacts; about 80% faster).
1. Immutable geometry sharing and reusable buffers.
1. A query-only drain index (AABB queries skip neighbour-pair enumeration).
1. A live simulation worker.
1. Path/GPU metadata caches.

The remaining tesla-valve costs named above (per-fixture AABB query scan width, emission `prepare_create` clones, `EvictionIndex` rebuilds, the CCD ray casts) are not among those stages. `web/` has an existing `bun run bench:tesla` browser runner. It is not needed for this phase (D-03: no new browser harness; native survey timing is the keep rule) and must not be confused with the survey. [VERIFIED: docs/benchmarks/tesla-valve/README.md, web/package.json]

## Determinism Audit (risks a refactor might perturb)

- **HashMap/HashSet in production:** `particle/lifetime/eviction.rs` (`by_particle`), `particle/storage/validation.rs` (`records_by_id`, `seen`), `association.rs`, `world/diagnostics/reconstruction.rs`, and `solver/manifest/witness_registry.rs`. In the eviction and validation modules they are used only for keyed lookup and duplicate detection; no iteration over them was found (`grep` for `.iter()/.keys()/.values()/.drain()` returned none). Changing their hashers or structures is safe; introducing iteration over them is not. [VERIFIED: grep]
- **Parallelism:** none in the step path.
- **Order-dependent float accumulation:** `recompute_contact_weights` (body contacts then particle contacts), `pressure`/`damping`/`extra_damping` (body loop then particle loop, Gauss-Seidel style in-place velocity updates), `static_pressure` accumulation, `filtered_collision_hits` hit order (stable sort by particle), and `body_contact::generate` (fixture, then child, then ascending row; `collect_candidate_rows` sorts and dedups rows to keep that order). Any change must preserve these orders exactly.
- **Body order:** `World::body_order` (newest-first) drives fixture sources and CCD records. Keep it.

## Verification Commands

| Check | Command | Observed runtime |
|---|---|---|
| Format | `cargo fmt --all` (CI: `cargo fmt --all --check`) | seconds |
| Lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (`just clippy` omits `--workspace`; CI uses it) | ~1–2 min cold [ASSUMED] |
| Build | `cargo build --workspace --all-targets --all-features` | ~1–2 min cold [ASSUMED] |
| Engine tests | `cargo test -p liquidfun --all-features` | 13–26 s warm [VERIFIED] |
| Playground tests | `cargo test -p liquidfun-wasm` | **~3.4 min** (307 tests, debug) [VERIFIED] |
| Survey tooling tests | `cargo test -p liquidfun-wasm --lib scene_spot`; `cargo test -p xtask --bin xtask spot`; `cargo test -p xtask --test playground_cli spot` | seconds |
| Full suite | `cargo test --workspace --all-features` (CI) | several minutes [ASSUMED] |
| WASM compile | `cargo build -p liquidfun-wasm --target wasm32-unknown-unknown` or `just web-wasm` | ~1 min [ASSUMED] |
| Web unit tests | `cd web && bun run test:unit` | ~4 s [VERIFIED] |
| Starter checks | `bun scripts/bright-builds-check.ts all` (628-line limit) | <1 s [VERIFIED] |
| Markdown | `just markdown-check` | seconds |
| Survey (full) | `just playground-scene-spot` (= `cargo xtask playground scene-spot`) | ~25 s build + about 15 s per run of all 25 scenes |
| Fingerprints only | `<bin> --runs 1` (full catalog) | ~15 s |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|---|---|---|---|---|
| rustc/cargo | everything | ✓ | 1.97.0 | — |
| samply | D-09 profiles | ✓ | 0.13.1 | `xctrace` (present) |
| dsymutil/atos | optional line attribution | ✓ | Xcode CLT | function-level only |
| python3 | profile parsing | ✓ | 3.14.6 | jq |
| bun | web tests, starter checks | ✓ | 1.4.2 | — |
| wasm-pack + wasm32 target | `just web-wasm` | ✓ | 0.15.0 | `cargo build --target wasm32-unknown-unknown` |
| mdformat | `just markdown-check` | ✓ | 1.0.0 | — |
| just | recipes | ✓ | 1.48.0 | `cargo xtask ...` |

No missing dependencies.

## Security Domain

Local developer tooling and engine internals only; no network, auth, or user data.

| ASVS Category | Applies | Standard Control |
|---|---|---|
| V2/V3/V4/V6 | no | — |
| V5 Input Validation | yes (CLI args) | `--scene` ids validated against `SURVEY_SCENES` / `SCENE_IDS`; closed error enums; no path/shell interpolation of ids |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|---|---|---|
| A1 | `DefaultHasher`'s algorithm is not guaranteed stable across Rust releases | Standard Stack | Low: FNV-1a is recommended anyway |
| A2 | Cold clippy/build/full-workspace-test/wasm durations | Verification Commands | Low: affects scheduling only |
| A3 | Per-row x-range AABB query, candidate-row bitset, prepare_create fast path, and EvictionIndex rework are bit-identical and worthwhile | Plan breakdown 35-03/05 | Medium: unprototyped; the fingerprint + A/B rule catches failures, so the cost is only wasted attempts |
| A4 | A sound conservative iteration-0 filter for moving fixtures is feasible with float slop | Pitfall 8 / 35-04 | Medium-High: the soundness argument needs care; fallback is per-fixture validation hoisting (smaller gain) |
| A5 | Local dev-server command for the optional D-03 browser glance | Open Questions | Low: optional check |

## Open Questions (RESOLVED)

1. **Should proxy-order retention live in `ParticleStorage` or `ParticleStepScratch`?**
   - Known: `ParticleStepScratch` is per world and taken out during the solve. `ParticleStorage` is per system, derives `PartialEq`/`Clone`, and is cloned per step for rollback.
   - Recommendation: a storage field wrapped in an always-equal newtype. It is simplest and per-system; the prototype used it and all tests passed.
   - RESOLVED: a `ParticleStorage` field in the always-equal `ProxyOrderCache` newtype (35-03 Task 1).
1. **Fingerprint table placement.**
   - Recommendation: a separate "Fingerprints" list plus a JSON field, keeping the ranked table shape for Phase 36.
   - RESOLVED: a per-scene JSON `fingerprint` field in the bin output and the stamp's scenes array only. There is no separate stdout list or `scene_filter` stamp field (lightweight process; later plans read the bin's JSON lines). The ranked table shape is unchanged (35-01).
1. **D-03 browser glance.**
   - Optional. `web/package.json` has no `dev` script. `cd web && bunx vite` (dev) or `just web-build` followed by `bun run preview` (serves `dist/` on 127.0.0.1:4173) should work, but this is unverified [ASSUMED]. Rust changes need `just web-wasm` first. If the check is skipped, record "not run" in `35-PROFILES.md`.
   - RESOLVED: optional and light; recorded in 35-PROFILES.md, with a `Not run: <reason>` fallback that keeps the native top five as the target set (35-02 Task 2).

## Sources

### Primary (HIGH confidence, verified this session)
- Code read: `crates/liquidfun-wasm/src/{scene_spot.rs, scene_spot/tests.rs, bin/playground_scene_spot.rs, session.rs, scene.rs, scene/liquid_tumbler.rs}`, `tools/xtask/src/playground/{spot.rs, spot/survey_table.rs, profile.rs, symbols.rs}`, `tools/xtask/src/playground.rs`, `tools/xtask/tests/playground_cli/spot.rs`, `tools/xtask/tests/fixtures/fake_upstream_tool.rs`, `crates/liquidfun/src/particle/{solver.rs, solver/pressure.rs, contact_scan.rs, body_contact.rs, proxy.rs, view.rs, storage.rs, storage/runtime.rs, storage/creation.rs, lifetime/eviction.rs, lifetime.rs}`, `crates/liquidfun/src/world/{particle_coupling.rs, particle_coupling/executor.rs, particle_coupling/executor/boundary_runtime.rs, particle_coupling/body_coupling.rs, observation.rs, observation/collection.rs, step/execution.rs}`, `crates/liquidfun/src/collision/{shape.rs, shape/chain.rs, shape/edge.rs}`
- samply 0.13.1 profiles of all five targets; prototype builds, fingerprints, and timings (scratchpad: `lt.json.gz`, `{tesla-valve,stacked-drip,washing-machine,particles}.json.gz`, `top_functions.py`, `callers.py`, `hot_lines.py`, `phase_table.py`, `fp_*.txt`)
- Test runs on the prototype: `cargo test -p liquidfun --all-features` (75 ok), `cargo test -p liquidfun-wasm` (307 ok); `bun run test:unit`; `bun scripts/bright-builds-check.ts all`

### Secondary
- `.planning/phases/34-scene-timing-survey/{34-CONTEXT.md, 34-02-SUMMARY.md, 34-VERIFICATION.md}`, `docs/benchmarks/scene-survey.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `AGENTS.md`, `PROJECT-SCOPE.md`, `standards/*`

## Metadata

**Confidence breakdown:**
- Tooling design: HIGH (APIs verified, prototype fingerprint deterministic for 25 scenes)
- Hot paths: HIGH (direct samply profiles on the target machine)
- Proxy reuse + edge hoist: HIGH (prototyped, fingerprints identical, tests green)
- Washing-machine / tesla-valve fixes: MEDIUM (profile-named, unprototyped)
- Damping: HIGH that branchless is a loss on M4 Max (two variants measured)

**Research date:** 2026-10-07
**Valid until:** about 2026-11-07, or until engine hot-path files change
