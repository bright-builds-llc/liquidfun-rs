---
phase: 22-observability-shell
verified: 2026-09-21T01:06:27Z
status: passed
score: 10/10 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 22-2026-09-20T22-34-43
generated_at: 2026-09-21T01:06:27Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 22: Observability shell Verification Report

**Phase Goal:** A developer can re-run the locked Dam Break pair into a dated evidence directory and capture a symbolicated samply CPU profile (plus optional parent-phase timers) without changing physics or the `--release` gate binary.
**Verified:** 2026-09-21T01:06:27Z
**Status:** passed
**Re-verification:** No — initial verification
**Verifier:** independent AI (`gsd-verifier`), not the implementing agent
**Hobby DoD:** Fake cmake/samply CLI proof is sufficient. Live Dam Break pair, real samply session, filling `manifest.toml`, and refreshing `docs/playground-dam-break-timing.md` are out of scope.

## Goal Achievement

The observability shell exists as three thin `just` → xtask recipes that mint exclusive gitignored stamps under `target/dam-break-perf/<YYYY-MM-DDTHH-MM-SSZ>/`. The unprofiled pair remains the 3× authority (`kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`). Profile and timer sibling stamps carry `not_timing_authority: true` and never write `pair.json`. Physics kernels are unchanged except a one-line clippy doc backtick in `particle_object.rs`. Workspace `[profile.profiling]` isolates debuginfo from default `--release`. `liquidfun` production deps remain `bitflags` only.

Independent AI review is already recorded in `22-REVIEW.md` (`status: clean`, digest `52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`, reviewed `2026-09-21T01:00:42Z`, reviewer `gsd-code-reviewer`, not human, not the implementing agent). HEAD `ec7039c` is the review-record commit on top of implementation `2e9aa01`.

### Observable Truths

Merged from ROADMAP success criteria plus PLAN `must_haves` (plan details add to, and do not replace, the four roadmap criteria).

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Developer can run `just playground-dam-break-bench` and find a new dated directory under `target/dam-break-perf/<utc-stamp>/` with unprofiled wall ms, ms/step, Rust/C++ ratio, host, git HEAD, and compilers — not stdout-only — and an existing stamp is not overwritten. (ROADMAP SC1 / PERF-PAIR) | ✓ VERIFIED | `pair::run` validates both samples, computes `rust_over_cpp_ratio`, then `persist_pair_report` mints a stamp and writes `pair.json` + `pair.md`, then prints the same Markdown. CLI test `dam_break_bench_persists_unprofiled_pair_without_profile_cmake_flags` plus `second_pair_in_the_same_unix_second_mints_a_new_stamp` prove persist, ratio, stdout copy, and no clobber. Unit test `pair_report_json_is_unprofiled_wall_clock_pair` asserts `git_head`, os/arch, compilers, wall_ms, ms_per_step. C++ `wall_ms` of 0 is a `bench` error before persist. |
| 2 | Exclusive stamp mint creates `target/dam-break-perf/<YYYY-MM-DDTHH-MM-SSZ>/` with exclusive `create_dir`, bumps one Unix second on `AlreadyExists`, and never removes or writes into a pre-existing stamp. (22-01) | ✓ VERIFIED | `stamp.rs` `mint_exclusive_stamp`: `create_dir` only, 8-attempt fail-closed, no `remove_dir_all`. Unit tests cover create, one-second bump with marker preserved, and eight-occupied fail-closed. |
| 3 | Playground xtask is `playground.rs` plus `playground/*.rs` with no `playground/mod.rs`; files stay under 628 lines; `dam-break-bench` still defaults `--warmup`/`--steps` to 60/600. (22-01) | ✓ VERIFIED | Dispatcher 66 lines; children 40–616 lines; `playground_cli.rs` 622. `ls playground/mod.rs` is missing. `parse_counts_defaults_to_locked_warmup_and_steps` asserts 60/600. `run()` still dispatches `dam-break-bench` → `pair::run`. |
| 4 | Developer can run a thin `just` / xtask profile recipe that rebuilds a `[profile.profiling]` Dam Break binary (`inherits = "release"`, `debug = true`), captures samply 0.13.1 of the timed Rust loop, and writes `rust.json.gz` plus host/HEAD/compiler/command identity into a new stamp; profiled wall times are never the 3× number. (ROADMAP SC2 / PERF-PROFILE) | ✓ VERIFIED | Workspace `Cargo.toml` `[profile.profiling]` inherits release, `debug = true`, `strip = false`. No `[profile.release]`. `profile::run` verifies samply 0.13.1, `cargo build --profile profiling --bin dam-break-bench`, mints a stamp, `samply record --save-only --unstable-presymbolicate -o rust.json.gz -- <profiling-binary> --warmup N --steps M`. Identity JSON: `kind: samply_cpu`, `not_timing_authority: true`, `cargo_profile: profiling`, `samply_version: 0.13.1`, command argv, host/HEAD/compiler. CLI test asserts no `pair.json` and cargo never gets `--release`. Instant isolation of construction/warmup remains in `run_dam_break_bench`; identity sets `warmup_included_in_samples: true` because samply wraps the process (plan 04 discretion; not `--pid`). |
| 5 | Missing samply fails closed with install/error text rather than silently skipping; no placeholder `rust.json.gz`. (ROADMAP SC3 / D-09) | ✓ VERIFIED | `verify_samply` runs before cargo build and stamp mint. Spawn failure or non-`0.13.1` stdout returns `PlaygroundError` kind `samply` with `cargo install --locked samply --version 0.13.1`, `brew install samply`, `samply setup`; Display must not contain `skip`. CLI test `missing_samply_fails_closed_without_placeholder_gzip` uses nonexistent `LIQUIDFUN_XTASK_SAMPLY`, expects nonzero exit and empty evidence tree. |
| 6 | `cargo xtask package verify` still passes; default `--release` is unchanged; `liquidfun` gains no profiler, samply, dhat, CMake, or serde dependency. (ROADMAP SC3 / PERF-PROFILE isolation / 22-06) | ✓ VERIFIED | `crates/liquidfun/Cargo.toml` production `[dependencies]` is `bitflags` only. Independent `cargo tree -p liquidfun --edges normal --offline` is `liquidfun → bitflags v2.13.0` (no serde, samply, dhat, cmake, flate2). No `[profile.release]` debug override. `package.rs` still enforces `FORBIDDEN_PREFIXES`. Plan 06 recorded `package verified: 238 entries built and tested outside the repository`. |
| 7 | Developer can emit coarse parent-phase timers (`particle_prepare` / `particle_solve` / `rigid_solve`) on a separate Dam Break diagnostic path, and those timers are absent from the unprofiled gate process. (ROADMAP SC4 / PERF-TIMERS) | ✓ VERIFIED | Native `dam-break-timers` bin calls `run_dam_break_timers` → `SessionCore::advance_profiled` → `World::step_profiled`. `dam_break_bench.rs` / `bin/dam_break_bench.rs` call `session.advance(1)` / ordinary `World::step` only (source grep + `dam_break_bench_does_not_call_step_profiled`). `ProofSession` exposes `advance`, not `advance_profiled`. |
| 8 | A fake timers run writes `timers.json` into a new stamp with `not_timing_authority: true` and parent tokens `particle_prepare`, `particle_solve`, and `rigid_solve`; that stamp has no `pair.json`. `MAX_ADVANCE_STEPS` remains 4; timers loop one-step `advance_profiled`. (22-05) | ✓ VERIFIED | `timers::run` cargo-runs `--release --bin dam-break-timers`, validates `kind == step_profiled_parents` and boolean `not_timing_authority`, mints a **new** stamp, writes `timers.json` only. CLI test asserts the three parent tokens, no `pair.json`, no `rust.json.gz`. `session.rs` `MAX_ADVANCE_STEPS: u32 = 4`; `advance_profiled` always steps 1. Measured loop is `for _ in 0..measured_steps { advance_profiled() }`. |
| 9 | `just playground-dam-break-bench`, `playground-dam-break-profile`, and `playground-dam-break-timers` are one-line `cargo xtask playground …` aliases with no cmake, samply, `-g`, or `--cpp-debuginfo` flags. (D-05/D-06/D-07) | ✓ VERIFIED | `justfile` lines 146–153. `rg cmake\|samply\|-g` on `justfile` is empty. CLI tests assert the exact one-line recipe bodies. `--cpp-debuginfo` is not implemented (Rust-only default profile path; no new CMake preset). |
| 10 | `BENCHMARKING.md` states that the unprofiled pair also writes gitignored stamps under `target/dam-break-perf/` and does not paste wall-ms numbers; `docs/playground-dam-break-timing.md` and `reference/performance/manifest.toml` (`reviewed_reports = []`) are unmodified in this phase. (22-06 / D-14) | ✓ VERIFIED | `BENCHMARKING.md` Exploratory section names `pair.json`/`pair.md`, sibling profile/timer stamps as not the 3× number, and forbids committing `.json.gz`/`.trace` and copying into `manifest.toml`. No wall-ms digits pasted as a claim. `manifest.toml` still `reviewed_reports = []`. Last git history on the timing doc / manifest is pre-phase (`a0f9355` / Phase 12). `/target/` is gitignored; `git ls-files '*.json.gz' '*.trace'` is empty. |

**Score:** 10/10 truths verified

### Required Artifacts

gsd-tools `verify artifacts` passed 18/18 planned paths. Manual L2/L3/L4 below.

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `tools/xtask/src/playground.rs` | Dispatcher only | ✓ VERIFIED | 66 lines; `mod` declarations; `run()` match for bench/profile/timers; `PlaygroundError` re-export |
| `tools/xtask/src/playground/stamp.rs` | UTC stamp + exclusive mint | ✓ VERIFIED | Exports `format_utc_stamp`, `mint_exclusive_stamp`; 271 lines |
| `tools/xtask/src/playground/pair.rs` | Unprofiled pair persist | ✓ VERIFIED | 616 lines; `unprofiled_pair`; `configure_and_build_cpp`; persist after validate |
| `tools/xtask/src/playground/profile.rs` | Samply argv + spawn + identity | ✓ VERIFIED | 529 lines; `samply_record_argv`, `require_samply_0_13_1`, `not_timing_authority` |
| `tools/xtask/src/playground/timers.rs` | xtask timer persist | ✓ VERIFIED | 316 lines; writes `timers.json` |
| `tools/xtask/tests/playground_cli.rs` | Fake cmake/cargo/samply CLI | ✓ VERIFIED | 622 lines; 8 tests covering persist, clobber, missing samply, just aliases |
| `tools/xtask/tests/fixtures/fake_upstream_tool.rs` | Fake samply 0.13.1 + timers JSON | ✓ VERIFIED | `run_samply` writes `-o` bytes; `print_timer_sample` emits required parents |
| `justfile` | Three one-line aliases | ✓ VERIFIED | bench/profile/timers each one `cargo xtask` line |
| `Cargo.toml` | `[profile.profiling]` | ✓ VERIFIED | inherits release, debug true, strip false |
| `crates/liquidfun-wasm/src/bin/dam_break_timers.rs` | Native timer CLI | ✓ VERIFIED | 63 lines; `run_dam_break_timers` |
| `crates/liquidfun-wasm/src/dam_break_timers.rs` | Recipe + parent aggregation | ✓ VERIFIED | 251 lines; `particle_prepare` / `ALL` parents |
| `crates/liquidfun-wasm/src/bin/dam_break_bench.rs` | Unprofiled gate CLI | ✓ VERIFIED | `run_dam_break_bench` only; no `step_profiled` |
| `BENCHMARKING.md` | Gitignored stamp note | ✓ VERIFIED | `target/dam-break-perf`; not-the-3× wording |
| `crates/liquidfun/Cargo.toml` | bitflags-only production deps | ✓ VERIFIED | `[dependencies] bitflags` only |
| `reference/performance/manifest.toml` | Empty reviewed_reports | ✓ VERIFIED | `reviewed_reports = []` |

### Key Link Verification

gsd-tools reported three `Source file not found` false negatives because the `from` values are functions/JSON kinds, not paths. Manual wiring:

| From | To | Via | Status | Details |
| ---- | -- | --- | ------ | ------- |
| `playground.rs` | `pair.rs` | `pair::run` | ✓ WIRED | `dam-break-bench => pair::run(command_args)` |
| `stamp.rs` | `target/dam-break-perf` | exclusive `create_dir` | ✓ WIRED | `EVIDENCE_RELATIVE_DIR = "target/dam-break-perf"` |
| `pair.rs` | `stamp.rs` | `mint_exclusive_stamp` after validate | ✓ WIRED | persist after both samples + ratio |
| `pair.json` | `policy.json` | `timing_authority unprofiled_wall_clock` | ✓ WIRED | `pair_report_json` writes the token; `reference/performance/policy.json` declares the same authority |
| `playground.rs` | `profile.rs` | `mod profile;` | ✓ WIRED | plus `dam-break-profile => profile::run` |
| `samply_record_argv` | `target/profiling/dam-break-bench` | argv after `--` | ✓ WIRED | `profiling_dam_break_bench_bin` joins `target/profiling/` (+ `.exe` on Windows); honors `CARGO_TARGET_DIR` |
| `justfile` | `playground.rs` | `dam-break-profile` | ✓ WIRED | one-line alias |
| `profile::run` | `samply_record_argv` | `Command::new(samply_program).args(argv)` | ✓ WIRED | `LIQUIDFUN_XTASK_SAMPLY` else `"samply"`; `Command::output()` so profiling JSON is not printed as the pair table |
| `session.rs` | `World::step_profiled` | `advance_profiled` only | ✓ WIRED | `#[cfg(not(target_arch = "wasm32"))]` |
| `dam_break_bench.rs` | `World::step` | `run_dam_break_bench` / `advance(1)` | ✓ WIRED | no `step_profiled` |
| `package.rs` | `crates/liquidfun` | `FORBIDDEN_PREFIXES` | ✓ WIRED | isolation check still present |
| `justfile` | `cargo xtask playground` | three `playground-dam-break-` aliases | ✓ WIRED | |

### Data-Flow Trace (Level 4)

These artifacts persist command output rather than render a UI. Traced anyway to confirm they are not hollow.

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `pair.json` / `pair.md` | `BenchSample` wall_ms, compilers, identity | Parsed JSON from `--release` `dam-break-bench` and `oracle-release` C++ bench stdout; `host_identity` for git/os/arch | Fake CLI uses 300.0/1.0 walls and compilers; production path uses live bench JSON | ✓ FLOWING |
| `rust.json.gz` | samply `--save-only` output | `samply record -o <stamp>/rust.json.gz` | Fake samply writes nonempty bytes; production requires existing nonempty file before identity write | ✓ FLOWING |
| `profile-identity.json` | host, compiler, command | `host_identity` + `rustc --version` + exact samply argv | Fields populated from process/env, not empty literals | ✓ FLOWING |
| `timers.json` | parent `wall_ms` map | `dam-break-timers` stdout JSON merged with host identity | Fake cargo emits the six Phase 12 parent tokens; production binary sums `maybe_common_parent` durations | ✓ FLOWING |

### Behavioral Spot-Checks

Live Dam Break / real samply were not required. Checks are repo-local and fake-tool based.

| Behavior | Command / inspection | Result | Status |
| -------- | -------------------- | ------ | ------ |
| Isolation tree | `cargo tree -p liquidfun --edges normal --offline` | `liquidfun → bitflags v2.13.0` only | ✓ PASS |
| Three just aliases, no tool flags | `rg` + `include_str` tests on `justfile` | Exact one-line recipes; no cmake/samply | ✓ PASS |
| Named profiling profile, no release debug | `Cargo.toml` | `[profile.profiling]` present; no `[profile.release]` | ✓ PASS |
| Fake playground CLI | Orchestrator: `cargo test -p xtask --test playground_cli --offline -- --test-threads=1` | 8 passed (pair persist, profile persist, timers persist, missing samply fail-closed, stamp uniqueness, just aliases). Test bodies inspected; they assert the claimed files/fields, not just exit 0. | ✓ PASS |
| `liquidfun` doctests | Orchestrator: `cargo test -p liquidfun --offline` | 22 doctests passed | ✓ PASS |
| Physics kernels untouched | `git diff 20e9b73..HEAD -- crates/liquidfun/` | Only `particle_object.rs` +1/−1 doc backticks | ✓ PASS |
| No committed traces | `git ls-files '*.json.gz' '*.trace'` | empty | ✓ PASS |

Step 7b did not start a live oracle pair or samply session (hobby DoD).

### Requirements Coverage

Every PLAN-frontmatter ID is in REQUIREMENTS.md. REQUIREMENTS.md maps no extra IDs to Phase 22.

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PERF-PAIR | 22-01, 22-02, 22-06 | Re-run locked Dam Break Medium pair and persist dated unprofiled report under `target/dam-break-perf/<utc-stamp>/` | ✓ SATISFIED | Truths 1–3, 9–10; pair persist + exclusive stamp + just alias |
| PERF-PROFILE | 22-03, 22-04, 22-06 | Thin just/xtask `[profile.profiling]` samply 0.13.1 capture into a new stamp; profiled walls never the 3× number | ✓ SATISFIED | Truths 4–6, 9–10; fail-closed missing samply; isolation |
| PERF-TIMERS | 22-05, 22-06 | Coarse `step_profiled` parents on a separate Dam Break path; not inside the unprofiled gate process | ✓ SATISFIED | Truths 7–9; `dam-break-timers` vs `dam-break-bench` |
| PERF-AUDIT | — (Phase 23) | Named-function audit doc | n/a this phase | ROADMAP Phase 23; not a Phase 22 gap |
| PERF-HEAP | — (Phase 23) | Private dhat dump | n/a this phase | ROADMAP Phase 23; not a Phase 22 gap |

No orphaned Phase 22 requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `tools/xtask/src/playground/profile.rs` | identity `warmup_included_in_samples: true` | Samply wraps the whole `dam-break-bench` process, so warmup is in the CPU samples | ℹ️ Info | Intentional plan-04 discretion (`--pid` rejected). Instant still excludes construction/warmup from pair wall_ms. Identity discloses the wrap. Not a 3× leak. |
| `tools/xtask/src/playground/timers.rs` | `validate_timer_report` | Checks `kind` + `not_timing_authority` only; does not require parent tokens at the xtask layer | ℹ️ Info | Binary always emits `DiagnosticProfileParent::ALL`; CLI test asserts the three required tokens. Persist cannot succeed with a pair-shaped JSON. |
| `tools/xtask/tests/playground_cli.rs` | pair/profile/timer runs | Fake CLI uses `--warmup 0 --steps 1` | ℹ️ Info | Locked 60/600 defaults are unit-tested in `parse_counts`. Fake-tool runs stay short by design. |
| playground production modules | tests only | `unwrap`/`expect` | ℹ️ Info | Production paths use `?` / `PlaygroundError`. Matches review special-look. |

No TODO/FIXME/placeholder stubs in playground or timer production files. No `return []` hollow persist. No cmake/samply flags hidden in `just`.

### Human Verification Required

None. Live Dam Break pair, real samply symbolication, and Instruments are optional local proof (CONTEXT D-13), not definition of done. Independent AI review is already recorded and is not this verifier’s self-approval.

### Gaps Summary

No blocking gaps. Phase 22 goal is achieved in the codebase: dated unprofiled pair persist, fail-closed samply capture into a sibling stamp, and a separate `step_profiled` timer path, without changing physics or the `--release` gate binary.

Out of scope (later phases, not gaps): `docs/native-performance-audit.md` and dhat (`PERF-AUDIT` / `PERF-HEAP`, Phase 23); filling `manifest.toml`; refreshing `docs/playground-dam-break-timing.md`; the ≤ 3× gate (Phase 24).

### Confirmation-bias notes (non-blocking)

1. **Partial check that still meets the requirement:** xtask does not schema-validate every parent token before writing `timers.json`. The producer (`run_dam_break_timers` + `DiagnosticProfileParent::ALL`) and the CLI test close that loop.
2. **Tests that pass for the right reason:** the eight `playground_cli` tests assert file bytes/fields (`kind`, `timing_authority`, `not_timing_authority`, parent tokens, nonempty gzip, stamp uniqueness), not merely process exit 0.
3. **Error path covered:** missing samply is before stamp mint. Wrong samply version uses the same install text. Empty gzip after a successful-looking samply is `require_recorded_profile`. C++ zero wall is a bench error with no pair files.

---

_Verified: 2026-09-21T01:06:27Z_
_Verifier: Cursor Grok 4.6 (gsd-verifier), independent of the implementing agent_
