---
phase: 22-observability-shell
reviewed: 2026-09-21T01:00:42Z
depth: standard
files_reviewed: 29
files_reviewed_list:
  - BENCHMARKING.md
  - Cargo.toml
  - crates/liquidfun-wasm/Cargo.toml
  - crates/liquidfun-wasm/src/bin/dam_break_bench.rs
  - crates/liquidfun-wasm/src/bin/dam_break_timers.rs
  - crates/liquidfun-wasm/src/dam_break_bench.rs
  - crates/liquidfun-wasm/src/dam_break_timers.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/scene/dam_break.rs
  - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
  - crates/liquidfun-wasm/src/scene/float_or_sink.rs
  - crates/liquidfun-wasm/src/scene/fountain.rs
  - crates/liquidfun-wasm/src/scene/jelly_drop.rs
  - crates/liquidfun-wasm/src/scene/water_wheel.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun/Cargo.toml
  - crates/liquidfun/src/world/particle_object.rs
  - justfile
  - tools/xtask/src/main.rs
  - tools/xtask/src/playground.rs
  - tools/xtask/src/playground/counts.rs
  - tools/xtask/src/playground/error.rs
  - tools/xtask/src/playground/identity.rs
  - tools/xtask/src/playground/pair.rs
  - tools/xtask/src/playground/profile.rs
  - tools/xtask/src/playground/stamp.rs
  - tools/xtask/src/playground/timers.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/playground_cli.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
digest: 52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06
review_digest: 52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06
diff_digest: b570cc042809ed8994e46a9cac22f7cc801ff47dd8228a23e7a69c4089cd2b02
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
diff_base: 20e9b73ae6a68e7cc2ed9506a81f7ae2a3ddfb84
local_head: 2e9aa01fa52136a327ab975bb9a383839b1371ea
---

# Phase 22: Code Review Report

**Reviewed:** 2026-09-21T01:00:42Z
**Depth:** standard
**Files Reviewed:** 29
**Status:** clean

This is independent AI review (Cursor Grok 4.6, `gsd-code-reviewer`), not
human approval. The implementing agent must not treat this file as
self-approval. Passing automated checks is not this acknowledgment.

## Summary

Reviewed Phase 22 Observability shell source from parent `20e9b73` (before
first `22-01` commit `f757472`) through HEAD `2e9aa01`. Scope was xtask
playground modules and CLI tests, native Dam Break bench/timer paths,
workspace `[profile.profiling]`, `just` aliases, `BENCHMARKING.md`, and
isolation of `liquidfun`.

All reviewed files meet the quality bar for this phase. No bugs, security
issues, or maintainability defects were found that would clobber stamps,
leak profiled/timer walls into `pair.json`, put `step_profiled` on the
unprofiled gate binary, add profiler/serde/CMake deps to `liquidfun`, hide
cmake/samply flags in `just`, skip a missing samply, `unwrap()` in
production, inject via a shell, or commit traces.

All reviewed files meet quality standards. No issues found.

## Special-look outcomes

### Stamp overwrite / clobber

`mint_exclusive_stamp` creates `target/dam-break-perf/<YYYY-MM-DDTHH-MM-SSZ>/`
with exclusive `create_dir`, bumps one Unix second on `AlreadyExists`, and
fails closed after 8 occupied candidates. It never `remove_dir_all`s or
writes into a pre-existing stamp. Pair, profile, and timer commands mint
after validation (pair/timers) or after samply version check (profile).
CLI tests prove a same-second second run keeps the first stamp bytes.

### Profiled timings vs pair.json / 3×

`pair.json` is `kind: unprofiled_pair` with
`timing_authority: unprofiled_wall_clock`. Profile writes `rust.json.gz` plus
`profile-identity.json` (`kind: samply_cpu`, `not_timing_authority: true`)
and no `pair.json`. Timers write `timers.json` (`kind: step_profiled_parents`,
`not_timing_authority: true`) and no `pair.json`. Profile spawn uses
`Command::output()`, so `[profile.profiling]` bench JSON is not printed as
the Markdown pair table.

### `step_profiled` on dam-break-bench

`dam_break_bench.rs` and `bin/dam_break_bench.rs` call `session.advance(1)` /
ordinary `World::step` only. `step_profiled` is confined to native-only
`SessionCore::advance_profiled` and the sibling `dam-break-timers` binary.
`ProofSession` does not export `advance_profiled`. `MAX_ADVANCE_STEPS`
remains 4; timers loop `advance_profiled()` once per measured step.

### `liquidfun` isolation

`crates/liquidfun/Cargo.toml` production deps remain `bitflags` only. Serde
lives on xtask. Samply is a host tool, not a crate. Scene/particle_object
diffs in this phase are lint-only (digit separators / doc backticks).

### `just` aliases

```
playground-dam-break-bench:
    cargo xtask playground dam-break-bench

playground-dam-break-profile:
    cargo xtask playground dam-break-profile

playground-dam-break-timers:
    cargo xtask playground dam-break-timers
```

No cmake, ninja, `-g`, samply, or `--cpp-debuginfo` flags in `just`.

### Missing samply

`verify_samply` runs before cargo build and stamp mint. Spawn failure or
non-`0.13.1` stdout returns `PlaygroundError` kind `samply` with install
text and no "skip" wording. Missing `LIQUIDFUN_XTASK_SAMPLY` CLI test
asserts nonzero exit and no `rust.json.gz`.

### `unwrap()` / command injection / traces / file length

Production playground and timer code uses `?` and `PlaygroundError`.
`unwrap`/`expect` appear only in tests. Tool programs are
`Command::new(path).args([...])`, not a shell string. `git ls-files` has no
`*.json.gz` / `*.trace`. `/target/` is gitignored. Longest reviewed sources:
`playground_cli.rs` 622, `pair.rs` 616, `session.rs` 560, `profile.rs` 529
(all under the 628-line trigger).

## Digest

Independently computed from current HEAD file bytes. Concatenate in this
listed order, then SHA-256:

```bash
cat \
  BENCHMARKING.md \
  Cargo.toml \
  crates/liquidfun-wasm/Cargo.toml \
  crates/liquidfun-wasm/src/bin/dam_break_bench.rs \
  crates/liquidfun-wasm/src/bin/dam_break_timers.rs \
  crates/liquidfun-wasm/src/dam_break_bench.rs \
  crates/liquidfun-wasm/src/dam_break_timers.rs \
  crates/liquidfun-wasm/src/lib.rs \
  crates/liquidfun-wasm/src/scene/dam_break.rs \
  crates/liquidfun-wasm/src/scene/dam_break/tests.rs \
  crates/liquidfun-wasm/src/scene/float_or_sink.rs \
  crates/liquidfun-wasm/src/scene/fountain.rs \
  crates/liquidfun-wasm/src/scene/jelly_drop.rs \
  crates/liquidfun-wasm/src/scene/water_wheel.rs \
  crates/liquidfun-wasm/src/session.rs \
  crates/liquidfun/Cargo.toml \
  crates/liquidfun/src/world/particle_object.rs \
  justfile \
  tools/xtask/src/main.rs \
  tools/xtask/src/playground.rs \
  tools/xtask/src/playground/counts.rs \
  tools/xtask/src/playground/error.rs \
  tools/xtask/src/playground/identity.rs \
  tools/xtask/src/playground/pair.rs \
  tools/xtask/src/playground/profile.rs \
  tools/xtask/src/playground/stamp.rs \
  tools/xtask/src/playground/timers.rs \
  tools/xtask/tests/fixtures/fake_upstream_tool.rs \
  tools/xtask/tests/playground_cli.rs \
  | shasum -a 256
```

`review_digest` / `digest`:

`52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`

Supporting (not the binding digest): SHA-256 of
`git diff 20e9b73..HEAD` over the Phase 22 implementation paths excluding
unchanged `dam_break_bench` / `liquidfun/Cargo.toml`:

`b570cc042809ed8994e46a9cac22f7cc801ff47dd8228a23e7a69c4089cd2b02`

This review does not authorize package publication, tags, or filling
`reference/performance/manifest.toml`. Profiled and timer walls are not the
3× number.

---

_Reviewed: 2026-09-21T01:00:42Z_
_Reviewer: Cursor Grok 4.6 (gsd-code-reviewer), AI reviewer, not a human_
_Depth: standard_
