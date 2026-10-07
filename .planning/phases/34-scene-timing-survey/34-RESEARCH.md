# Phase 34: Scene Timing Survey - Research

**Researched:** 2026-10-07
**Domain:** Extending an existing native Rust scene timer (liquidfun-wasm bin + xtask driver)
**Confidence:** HIGH (all findings verified against the codebase and one live 25-scene probe run)

## Summary

Phase 34 extends the existing five-scene timer. `scene_spot.rs` is a `cfg(not(target_arch = "wasm32"))` module inside the `liquidfun-wasm` lib, so it can call the `pub(crate)` `SessionCore::create/advance/apply_action/apply_pointer/live_particle_count` directly. No visibility changes are needed. The bin only calls `run_scene_spot` and prints one JSON line per scene. xtask runs it through `cargo run --release`, validates it against a hard-coded `REQUIRED_SCENES` list, and writes `target/dam-break-perf/<stamp>/scene-spot.json`. [VERIFIED: codebase]

A throwaway probe outside the repo built the full 25-scene survey with the D-09 cues, 3 runs, 60 warmup and 120 timed steps, all in release mode. It finished in **about 27 s total** on an Apple M4 Max. Liquid Tumbler took about half of that (25 ms/step, about 4.5 s per run). Every other scene stayed at or below 4 ms/step. Run-to-run spread was small, under about 8% of the median. The defaults are fine, and the 3-minute per-scene timeout leaves plenty of headroom. [VERIFIED: local probe run]

The cues behave as specified. Float or Sink `drop-body` is a valid action. Impulse pointer-up at (1, 2) falls inside its box [-2,2]×[0,4]. Drawing pointer-up at (0, 2) falls inside the walls (interior x∈(-2,2), y∈(0,4)) and stamps **24 particles**, starting from 0. Sparky goes from 0 to 550 particles with no input, which confirms D-10. [VERIFIED: codebase + probe]

**Primary recommendation:** Add one ordered 25-entry `SURVEY_SCENES` table, plus a cue enum, to `scene_spot.rs`. Compute median/min/max inside `scene_spot.rs` and emit one aggregated JSON line per scene. In xtask, replace `REQUIRED_SCENES` with a `SCENE_IDS` parse of `include_str!`'d `web/src/catalog/scenes.ts`, then rank and print a Markdown table. Update the fake cargo in the xtask test fixture so it emits all catalog scenes.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

**Survey scope and coverage check**

- **D-01:** Extend the existing tool in place (`crates/liquidfun-wasm/src/scene_spot.rs`, `src/bin/playground_scene_spot.rs`, `tools/xtask/src/playground/spot.rs`). Do not add a new crate, harness or evidence framework. The shelved harness on `wip/v1.4-phase34-harness` is not used.
- **D-02:** The survey covers all 25 catalog ids, Dam Break included. The list lives in one Rust table in `scene_spot.rs` (hyphenated id → `SceneId`).
- **D-03:** A coverage test in `liquidfun-wasm` reads `SCENE_IDS` from `web/src/catalog/scenes.ts` through `include_str!` and asserts that it matches the survey list exactly, with each id resolving through `parse_scene_id`. Adding a catalog scene without a survey entry fails `cargo test`.
- **D-04:** The xtask driver drops its hard-coded five-scene `REQUIRED_SCENES` list and validates against the same `SCENE_IDS` parsed from `web/src/catalog/scenes.ts`, so the two lists cannot drift.

**Timing method**

- **D-05:** Keep the defaults of 60 untimed warmup steps and 120 timed `advance(1)` steps, and add `--runs <n>` with a default of 3. Each run builds a fresh `SessionCore`. Report the median, min and max ms/step across runs. The spread is the visible noise indicator Phase 35 uses for "beyond noise".
- **D-06:** Run in release mode through `cargo run --release` as today, serially, one scene at a time. Keep the per-scene wall timeout.

**Reported fields**

- **D-07:** Per scene, report the median, min and max ms/step, the live particle count at the end of the timed window (the start count too), and an interaction label of `default` or `scripted`.
- **D-08:** `cargo xtask playground scene-spot` prints a ranked Markdown table to stdout (slowest first) and still writes the JSON report under the existing `target/dam-break-perf/<stamp>/scene-spot.json` stamp. The existing `not_timing_authority` flag and disclaimer stay, with wording updated to drop the five-scene description.

**Idle scenes**

- **D-09:** Script interaction only where the default scene does nothing visible. Reuse the cues the README previews already use (`web/scripts/readme-svg/plans.ts`): Float or Sink `drop-body` action, Impulse `pointer-up` at (1, 2), Drawing Particles `pointer-up` at (0, 2). Apply each cue through the public `SessionCore::apply_action` and `apply_pointer` before warmup, so the timed window covers the reaction.
- **D-10:** Sparky stays `default`: its circles emit particles on contact without input. Every other scene is `default`.
- **D-11:** Survey cue code lives in `scene_spot.rs` only, never in production scene modules.

**Committed table**

- **D-12:** `docs/benchmarks/scene-survey.md` records the ranked table (rank, scene, median/min/max ms/step, end particles, interaction) plus the commit, machine (OS, arch, CPU, cores), rustc, the exact command, and the warmup, steps and runs counts. The page states that these are local observations, not public speed claims. Format with mdformat and pass `just markdown-check`.
- **D-13:** Keep the `justfile` recipe as the existing one line (`cargo xtask playground scene-spot`). An existing test pins that exact text.

### Claude's Discretion

- Exact table column names, the median computation for an even run count, and helper structure in xtask.
- Whether the ranked-table renderer is shared between stdout and the doc, or the doc is pasted from the stdout output.

### Deferred Ideas (OUT OF SCOPE)

- Browser frame-drop measurement belongs to Phase 35 if it is needed.
- Whole-catalog before/after belongs to Phase 36.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| PERF-07 | Time every playground catalog scene natively with one command, rank by median ms/step, and fail a check when a catalog scene is missing | 25-entry table + `include_str!` coverage test (liquidfun-wasm) + xtask validation against parsed `SCENE_IDS`. The probe shows a full run takes about 27 s. Cue semantics verified. |

</phase_requirements>

## Project Constraints (from CLAUDE.md / AGENTS.md)

- No `unwrap()` in production code. Propagate errors through the closed `SceneSpotError` / `PlaygroundError`. Use `expect` only in tests. Use `let ... else` for early returns.
- Name optional values with `maybe_`. Use tau rather than pi for any angles (none are expected here).
- Unit tests check one concern each and carry `// Arrange` / `// Act` / `// Assert` comments.
- Keep a functional core and an imperative shell. Pure helpers (median, SCENE_IDS parsing, ranking, table rendering) should be unit-testable without running the simulation.
- Run `just markdown-check` (mdformat 1.0.0 + mdformat-gfm, Python 3.13) after any non-GSD Markdown change. Never mdformat `.planning/**`.
- Documentation must not overclaim. The survey is a local observation and not a timing authority, as the existing `not_timing_authority` disclaimer says.
- The project is a hobby project: local checks only, with no new frameworks, crates or evidence systems (memory note: "prefer lightweight process").

## Standard Stack

No new dependencies. Everything already exists. [VERIFIED: codebase]

| Piece | Location | Role |
| --- | --- | --- |
| `SessionCore` (`pub(crate)`) | `crates/liquidfun-wasm/src/session.rs` | `create(SceneId)`, `advance(1)`, `apply_action(&str)`, `apply_pointer(kind: &str, x: f32, y: f32)`, `live_particle_count()` (cfg `any(test, not(wasm32))`) |
| `SceneId`, `parse_scene_id` (`pub(crate)`) | `crates/liquidfun-wasm/src/scene.rs` | All 25 ids are already mapped |
| `serde_json` | already an xtask dependency | Parse bin JSON lines, write report |
| `host_identity`, `rustc_version`, `stamp::mint_exclusive_stamp` | `tools/xtask/src/playground/{identity,spot,stamp}.rs` | Commit, OS, arch, CPU, cores, rustc, stamp dir |
| mdformat 1.0.0 + mdformat-gfm 1.0.0 | `~/.local/bin/mdformat` | Aligns the table in the doc |

## Architecture Patterns

### Files to change (and only these)

```
crates/liquidfun-wasm/src/scene_spot.rs        # SURVEY_SCENES table, cue enum, runs, median/min/max, coverage test
crates/liquidfun-wasm/src/bin/playground_scene_spot.rs  # add --runs flag
tools/xtask/src/playground/spot.rs             # SCENE_IDS validation, --runs passthrough, ranking + Markdown table
tools/xtask/src/playground.rs                  # USAGE string: add [--runs <n>] (no test pins exact USAGE)
tools/xtask/tests/fixtures/fake_upstream_tool.rs  # fake cargo must emit all catalog scenes in the new JSON shape
tools/xtask/tests/playground_cli/spot.rs       # replace five-scene assertion with catalog set
docs/benchmarks/scene-survey.md                # new committed table
docs/playground-scene-spot-check.md            # stale five-scene description: add a pointer to scene-survey.md (small edit)
```

`BENCHMARKING.md` already links to `docs/playground-scene-spot-check.md`. Adding a one-sentence link to `scene-survey.md` is optional. [VERIFIED: grep]

### Pattern 1: One ordered survey table plus a cue enum (liquidfun-wasm)

```rust
#[derive(Debug, Clone, Copy)]
enum SurveyCue {
    None,
    Action(&'static str),
    PointerUp { x: f32, y: f32 },
}

const SURVEY_SCENES: [(&str, SceneId, SurveyCue); 25] = [
    ("wave-machine", SceneId::WaveMachine, SurveyCue::None),
    ("dam-break", SceneId::DamBreak, SurveyCue::None),
    // ... catalog order from web/src/catalog/scenes.ts ...
    ("float-or-sink", SceneId::FloatOrSink, SurveyCue::Action("drop-body")),
    ("impulse", SceneId::Impulse, SurveyCue::PointerUp { x: 1.0, y: 2.0 }),
    ("drawing-particles", SceneId::DrawingParticles, SurveyCue::PointerUp { x: 0.0, y: 2.0 }),
    // ...
];
```

- Keep the table in **catalog order** so the coverage test can be an ordered `assert_eq!` on `Vec<&str>`. That one assertion catches missing, extra, duplicate and reordered ids.
- The interaction label comes from the cue: `None` → `"default"`, anything else → `"scripted"`.
- Apply the cue right after `SessionCore::create` and **before** the start-particle snapshot or warmup. For Drawing, decide whether `start_particles` is read before the cue (0) or after it (24). Reading before the cue is recommended, so that `start` means "as constructed", as it does today. Map a cue failure to a new closed variant such as `SceneSpotError::CueFailed { scene }`.
- `apply_pointer` takes `kind: &str`, so pass `"up"`. [VERIFIED: session.rs:266]

### Pattern 2: Aggregate per scene in the bin, rank in xtask

- `time_scene` stays a single run that returns ms/step and counts. A new `survey_scene(…, runs)` loops `runs` times with a fresh `SessionCore` each time, keeps the timeout **per run** (same semantics, more headroom), and reduces the results with a pure `summarize(&mut [f64]) -> (median, min, max)`.
- Median for an even count: the mean of the two middle values (discretion item; the default `runs = 3` is odd). Sort with `f64::total_cmp`, which avoids `partial_cmp(...).unwrap()`.
- Reject `--runs 0` in both the bin (`run_scene_spot` returns an error, as it already does for `measured_steps == 0`) and xtask `parse_spot_counts` (usage error).
- End particles are deterministic across runs (the probe saw identical counts in all three runs). Report the last run's counts. Do not add a cross-run equality check, since that would add machinery without much value.
- Suggested JSON line fields: `scene, interaction, runs, warmup_steps, measured_steps, start_particles, end_particles, median_ms_per_step, min_ms_per_step, max_ms_per_step, timed_out`. Drop `wall_ms` and `ms_per_step` from the line, or keep `ms_per_step` as the median. Whichever is chosen, update the xtask `wall_ms` finiteness check and the fake tool to match.

### Pattern 3: xtask validates against `SCENE_IDS` through `include_str!`

```rust
const CATALOG_SCENES_TS: &str = include_str!("../../../../web/src/catalog/scenes.ts");

fn catalog_scene_ids(source: &str) -> Result<Vec<&str>, PlaygroundError> {
    let Some(start) = source.find("export const SCENE_IDS = [") else { /* error */ };
    let body = &source[start..];
    let Some(open) = body.find('[') else { /* error */ };
    let Some(close) = body.find(']') else { /* error */ };
    Ok(body[open + 1..close]
        .split(',')
        .map(|item| item.trim().trim_matches('"'))
        .filter(|item| !item.is_empty())
        .collect())
}
```

- **Use compile-time `include_str!` rather than a runtime read from `repository_root()`.** `RepositoryFixture` builds a minimal fake repository with no `web/` directory, so a runtime read would fail in the CLI tests unless the fixture also wrote `scenes.ts`. `include_str!` also makes Cargo rebuild xtask whenever `scenes.ts` changes. There is precedent: xtask tests already `include_str!("../../../../justfile")`, and other workspace crates `include_str!` files under `protocol/` and `reference/`. [VERIFIED: grep]
- Splitting on commas between `[` and `]` works whether Prettier writes one id per line (as it does today) or collapses the list onto one line. The array literal contains no `]`. [VERIFIED: scenes.ts:11-37]
- Validation: the parsed list must have no duplicates (`BTreeSet` length equals `Vec` length), and the sample names (set) must equal the catalog set, with `samples.len() == catalog.len()`. Keep the existing `timed_out == false` check and the finite-number check, retargeted to `median_ms_per_step`.
- The liquidfun-wasm coverage test needs the same tiny parser. Duplicating about 10 lines across two crates is acceptable. Do not create a shared crate (D-01).

### Pattern 4: Rank and render

- Sort by `median_ms_per_step` descending, then by scene id for a stable tie order. Rank starts at 1.
- Suggested columns: `| Rank | Scene | Median ms/step | Min | Max | Start particles | End particles | Interaction |`. Use 3 decimal places for ms/step, which shows sub-0.1 ms scenes such as Drawing (0.011).
- Print the header/disclaimer, the table, then the stamp/artifact lines. Keep `eprintln!("wrote …")`.
- Doc workflow (discretion): render a padded table with a pure `render_ranked_table` function. Paste the stdout table into `docs/benchmarks/scene-survey.md` and run `mdformat`, which realigns the columns. That means no file-writing code for the doc.

### Anti-Patterns to Avoid

- Do not add cue logic to `crates/liquidfun-wasm/src/scene/*.rs` (D-11).
- Do not change `SessionCore` visibility. `scene_spot.rs` is in-crate, so `pub(crate)` is enough. (CONTEXT's word "public" just means the session API rather than hook internals.)
- Do not use `ProofSession::particle_count()` / `SessionCore::particle_count()` for counts. It is the **construction-time** count and is never updated (session.rs:66,106,400). Keep `live_particle_count()`. The probe showed Fountain reporting `1 -> 1` through the cached field and `1 -> 3200` live. [VERIFIED: session.rs + probe]
- Do not run the full survey inside `cargo test`. Liquid Tumbler alone takes about 4.5 s per run in release and far longer in debug. Unit tests should cover parsing, the coverage check, `summarize`, ranking and rendering, plus at most a cue-applies-OK check that does not step the simulation.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead |
| --- | --- | --- |
| Host/commit/compiler fields | New probes | `host_identity()` + `rustc_version()` already in xtask |
| Stamp directories | New output layout | `stamp::mint_exclusive_stamp` + existing `scene-spot.json` |
| Table alignment in the doc | Custom padding perfectionism | `mdformat` (gfm plugin aligns tables) |
| A TS parser | Regex or JS tooling | The 10-line bracket/comma split above |
| Statistics | Percentile libs | Sort with `total_cmp`, take the middle values |

## Common Pitfalls

1. **Fake cargo still prints five scenes.** `tools/xtask/tests/fixtures/fake_upstream_tool.rs::print_scene_spot_sample` hard-codes five ids in the old JSON shape. After D-04, `scene_spot_persists_native_scene_spot_without_pair` will fail. Fix: have the fake emit every id from the same `include_str!("../../../../web/src/catalog/scenes.ts")`, which works because the fake is compiled with plain `rustc <file>` and `include_str!` resolves relative to the source file. Alternatively, hard-code 25 ids, but that list can drift. Update the CLI test assertion to compare against the parsed catalog set, or at least `len() == 25` plus a few names. The fake should also accept and ignore `--runs <n>`. [VERIFIED: fixture code]
2. **The xtask unit test `validate_scene_samples_requires_five_named_scenes`** needs rewriting for the catalog list.
3. **Pinned justfile text.** Leave `playground-scene-spot:\n    cargo xtask playground scene-spot` exactly as it is (D-13; `justfile_keeps_the_one_line_playground_scene_spot_alias`). Change defaults in code, not in the recipe.
4. **Drawing ranks last at about 0.011 ms/step.** One brush stamp makes 24 particles. The label `scripted` explains this, and the doc should mention it in one line so Phase 35 does not misread it. [VERIFIED: probe]
5. **Impulse cue timing.** The README applies the shove at sample 20, after the scene settles. D-09 applies it before warmup, at step 0. The shove still lands (the box test uses (1, 2)), but the 120-step timed window begins 60 steps after the impulse. This is acceptable under the locked decision; just do not expect an Impulse-specific spike.
6. **Stale commit in the doc.** `host_identity().git_head` does not record a dirty tree. Commit the code changes first, then run the survey on that clean HEAD and paste the result. The doc commit follows, so the recorded commit is the code's commit, not the doc's.
7. **Release-build warning noise.** `cargo run --release` prints a pre-existing `unreachable_code` warning from `crates/liquidfun/src/particle/solver/boundary/support.rs:79-98` (only under `cfg(not(debug_assertions))`). It goes to stderr and is harmless, because xtask ignores stderr on success. Fixing it is out of scope; note it if desired. [VERIFIED: cargo build output]
8. **Thermal and ordering noise.** Liquid Tumbler runs about 4.5 s hot per run. Scenes run serially in catalog order, so later scenes can inherit a warm or throttled CPU. The min/max columns expose this, and nothing else is needed for a hobby survey.
9. **Workspace default-members is only `crates/liquidfun`.** Bare `cargo clippy`/`cargo test` at the root will **not** cover liquidfun-wasm or xtask, so use `-p` flags (see Validation).

## Code Examples

### Probe results (release, 3 runs × (60 + 120), Apple M4 Max, 2026-10-07, HEAD `eeec35a2a`) [VERIFIED: local probe]

| Scene | median ms/step | min | max | start → end particles |
| --- | --- | --- | --- | --- |
| liquid-tumbler | 24.99 | 24.90 | 25.46 | 3800 → 3800 |
| tesla-valve | 4.00 | 4.00 | 4.02 | 0 → 2850 |
| stacked-drip | 2.75 | 2.71 | 2.77 | 2000 → 2000 |
| washing-machine | 2.12 | 2.10 | 2.13 | 3168 → 3168 |
| particles | 1.65 | 1.63 | 1.66 | 4569 → 4569 |
| liquid-bubbler | 1.56 | 1.54 | 1.57 | 3000 → 3000 |
| water-wheel | 1.34 | 1.31 | 1.40 | 1 → 3200 |
| soup-stirrer / soup / fountain | about 1.16–1.21 | | | |
| wave-tank, wave-machine, hydraulic-fountain, liquid-timer, impulse | 0.77–0.98 | | | |
| remaining 9 | 0.011–0.56 | | | drawing 0 → 24, sparky 0 → 550 |

Total wall time is about 27.4 s plus the incremental release build. This is preliminary context for the planner only. The committed doc must come from the real `just playground-scene-spot` run after implementation.

### Coverage test shape (liquidfun-wasm, `#[cfg(test)]` only, so wasm builds never see `include_str!`)

```rust
#[test]
fn survey_scenes_match_catalog_scene_ids() {
    // Arrange
    let catalog = catalog_scene_ids(include_str!("../../../web/src/catalog/scenes.ts"));

    // Act
    let survey: Vec<&str> = SURVEY_SCENES.iter().map(|(id, _, _)| *id).collect();

    // Assert
    assert_eq!(survey, catalog);
}

#[test]
fn survey_scene_ids_resolve_through_parse_scene_id() {
    // Arrange / Act / Assert
    for (id, scene, _) in SURVEY_SCENES {
        assert_eq!(parse_scene_id(id).expect("catalog id should parse"), scene);
    }
}
```

`liquidfun-wasm` is `publish = false`, so a `../../../web/...` path never affects packaging. [VERIFIED: Cargo.toml]

## State of the Art

| Old | New (this phase) |
| --- | --- |
| 5 hard-coded scenes in both bin and xtask | 25 catalog scenes; both sides checked against `scenes.ts` |
| 1 run, mean ms/step | `--runs 3` (default), median/min/max |
| xtask prints a stamp summary only | Ranked Markdown table (slowest first) + stamp/artifact lines |
| `docs/playground-scene-spot-check.md` five-scene sample | `docs/benchmarks/scene-survey.md` ranked table; old doc points to it |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| A1 | `start_particles` should be read before the cue | Pattern 1 | Cosmetic only: Drawing shows 0 or 24 at start |
| A2 | Even-count median = mean of the two middle values | Pattern 2 | None for default runs=3 |
| A3 | Per-run (not per-scene-total) timeout is acceptable under D-06 "keep the per-scene wall timeout" | Pattern 2 | If per-scene total is wanted, wrap all runs in one `Instant`; still far under 3 min |

## Open Questions (RESOLVED)

1. **Should the old `docs/playground-scene-spot-check.md` be rewritten or only pointed at the new doc?**
   - Recommendation: add a short note at the top that links to `docs/benchmarks/scene-survey.md` and say its five-scene table is historical. That is the minimal edit. Run `just markdown-check`.
   - RESOLVED: add pointer + mark five-scene table historical (34-02 Task 2).

## Environment Availability

| Dependency | Required By | Available | Version |
| --- | --- | --- | --- |
| Rust toolchain (pinned) | everything | ✓ | 1.97.0 via `rust-toolchain.toml` (system default is 1.91.1, but the repo pin overrides it) |
| mdformat + gfm | doc formatting | ✓ | 1.0.0 / gfm 1.0.0 |
| Python 3.13 | mdformat config exclusions | ✓ | `/opt/homebrew/bin/python3.13` |
| just | recipe | ✓ (repo uses it) | — |

Note: the first run of a freshly linked xtask test binary was observed stalled at `_dyld_start` for several minutes. That looks like a macOS executable scan, not a code issue. If CLI tests seem hung right after a rebuild, wait before diagnosing. [VERIFIED: `sample` of the stalled process]

## Validation (local checks)

- `cargo fmt --all --check`
- `cargo clippy -p liquidfun-wasm -p xtask --all-targets --all-features -- -D warnings`
- `cargo test -p liquidfun-wasm --lib scene_spot` (coverage, summarize and cue tests)
- `cargo test -p xtask --bin xtask playground::spot` and `cargo test -p xtask --test playground_cli spot` (fixture CLI test + justfile pin)
- `just playground-scene-spot` (real run, about 30 s; produces the table for the doc)
- `just markdown-check` after writing `docs/benchmarks/scene-survey.md` / editing the old spot doc

## Security Domain

Not material. This is a local developer CLI with no network access, auth, or untrusted input beyond integer flags, which are already parsed as `u32` with closed errors. The stamp path guard (`parse_stamp_unix` rejects path separators) is unchanged. V5 input validation means `--runs` is parsed like `--warmup`/`--steps` and rejected when 0.

## Sources

### Primary (HIGH confidence)

- Codebase: `crates/liquidfun-wasm/src/{scene_spot.rs,bin/playground_scene_spot.rs,lib.rs,session.rs,scene.rs,scene/{impulse,drawing_particles,float_or_sink}.rs}`, `tools/xtask/src/playground/{spot.rs,identity.rs}`, `tools/xtask/src/playground.rs`, `tools/xtask/tests/playground_cli/{spot.rs,support.rs}`, `tools/xtask/tests/fixtures/fake_upstream_tool.rs`, `web/src/catalog/scenes.ts`, `web/scripts/readme-svg/plans.ts`, `.mdformat.toml`, `scripts/markdown-check.sh`, `justfile`, `Cargo.toml`
- Local runs: existing five-scene `cargo run -p liquidfun-wasm --release --bin playground-scene-spot` (8.7 s incl. cargo), plus a 25-scene probe built in the scratchpad against the `liquidfun-wasm` rlib (27.4 s)

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH, because nothing new is added and all APIs were read in source
- Architecture: HIGH, because the change points and the test-fixture coupling were verified
- Pitfalls: HIGH, because the fake cargo and cached particle count were verified by code and probe

**Research date:** 2026-10-07
**Valid until:** 2026-11-06 (or until a catalog scene is added)
