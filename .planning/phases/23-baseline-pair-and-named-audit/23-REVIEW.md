---
phase: 23-baseline-pair-and-named-audit
reviewed: 2026-09-21T05:21:07Z
depth: deep
files_reviewed: 26
files_reviewed_list:
  - BENCHMARKING.md
  - Cargo.lock
  - Cargo.toml
  - crates/liquidfun-wasm/Cargo.toml
  - crates/liquidfun-wasm/src/bin/dam_break_bench.rs
  - crates/liquidfun/Cargo.toml
  - docs/native-performance-audit.md
  - docs/playground-dam-break-timing.md
  - justfile
  - reference/performance/manifest.toml
  - tools/xtask/src/main.rs
  - tools/xtask/src/playground.rs
  - tools/xtask/src/playground/bundle.rs
  - tools/xtask/src/playground/bundle/ops.rs
  - tools/xtask/src/playground/heap.rs
  - tools/xtask/src/playground/pair.rs
  - tools/xtask/src/playground/stamp.rs
  - tools/xtask/src/playground/symbols.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/playground_cli.rs
  - tools/xtask/tests/playground_cli/bundle.rs
  - tools/xtask/tests/playground_cli/heap.rs
  - tools/xtask/tests/playground_cli/pair.rs
  - tools/xtask/tests/playground_cli/profile.rs
  - tools/xtask/tests/playground_cli/support.rs
  - tools/xtask/tests/playground_cli/timers.rs
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
digest: c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d
review_digest: c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d
diff_digest: 28113adb401439bc7d02abb6def57ea6a5e861db258c560924ae1c234d49d7c9
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
diff_base: 121d741d8c0bc706b1b3882b80da6914c79e4a82
local_head: 30ecd75a449ec579d161ccc1e43bfd852e57bb33
---

# Phase 23: Code Review Report

**Reviewed:** 2026-09-21T05:21:07Z
**Depth:** deep
**Files Reviewed:** 26
**Status:** clean

This is independent AI review (Cursor Grok 4.6, `gsd-code-reviewer`), not
human approval. The implementing agent must not treat this file as
self-approval. Passing automated checks and `23-VERIFICATION.md` are not this
acknowledgment. This file **is** the CONTEXT D-16 digest-bound phase
acknowledgment. It is not advisory.

Guidance used: `AGENTS.md` Repo-Local Guidance (hobby scope; independent AI
review 2026-09-16), `AGENTS.bright-builds.md`, `standards-overrides.md`,
`standards/index.md`, `standards/core/code-shape.md` (628-line trigger),
`standards/languages/rust.md` (`foo.rs` plus `foo/`, no production `unwrap`).

## Summary

Reviewed Phase 23 Baseline pair and named audit from parent `121d741` (origin/main
before Phase 23) through HEAD `30ecd75`. Scope was xtask audit-bundle / heap /
symbols / stamp plumbing, playground CLI tests, optional `dhat-heap` on
`liquidfun-wasm` `dam-break-bench`, `just` aliases, committed audit and timing
docs, `BENCHMARKING.md`, `Cargo.lock` `dhat` 0.3.3, unchanged
`[profile.profiling]`, empty `reference/performance/manifest.toml`, and live
gitignored stamps. Also inspected `crates/liquidfun/src` (no Phase 23 kernel
diff) and CONTEXT D-01..D-16. `23-VERIFICATION.md` remains `human_needed`
solely for D-16; this review is that missing step.

All reviewed files meet the quality bar for this phase. No bugs, security
issues, or maintainability defects were found that would clobber stamps, move
pair/profile sources, put `dhat` on `liquidfun` or the unprofiled `--release`
gate, quote samply/profiling/dhat clocks as the Dam Break ratio, invent
hot-function names missing from the live sidecar, commit blobs, fill
`reviewed_reports`, hide cmake/samply/dhat flags in `just`, `unwrap()` in
production, inject via a shell, or edit physics kernels.

All reviewed files meet quality standards. No issues found.

This independent AI review **approves Phase 23 implementation for D-16**. It
does not authorize package publication, tags, or filling
`reference/performance/manifest.toml`.

## Special-look outcomes

### Live SHA-bound stamps (D-01, D-04, D-06, D-11..D-13)

Inspected on disk under `target/dam-break-perf/`:

| Stamp | Role | Contents |
| --- | --- | --- |
| `2026-09-21T04-32-19Z` | unprofiled pair | `pair.json`, `pair.md` |
| `2026-09-21T04-32-56Z` | samply CPU | `rust.json.gz` (463913 B), `rust.json.syms.json` (191835 B), `profile-identity.json` |
| `2026-09-21T04-34-32Z` | audit bundle | copies of pair + gzip + sidecar plus `audit-bundle-identity.json` |
| `2026-09-21T04-47-09Z` | private dhat | `dhat-heap.json` (594964 B, `dhatFileVersion: 2`, `mode: rust-heap`, cmd `target/profiling/dam-break-bench --warmup 60 --steps 600`), `heap-identity.json` |

`cmp` shows bundle `pair.json` / `pair.md` / `rust.json.gz` /
`rust.json.syms.json` byte-identical to the named source stamps. Bundle
identity: `kind: audit_bundle`, `git_head: 6d98531ac799987c209d3fd1e572e482fcab5da6`,
`source_pair_stamp: 2026-09-21T04-32-19Z`,
`source_profile_stamp: 2026-09-21T04-32-56Z`, `not_timing_authority: true`,
`timing_authority: unprofiled_wall_clock`, `worktree_dirty: true` (recorded;
physics HEAD still `6d98531…`, an ancestor of this review HEAD). Profile
identity: `kind: samply_cpu`, `samply_version: 0.13.1`,
`cargo_profile: profiling`, `not_timing_authority: true`. Heap identity:
`kind: dhat_heap`, `not_timing_authority: true`, `cargo_profile: profiling`,
`features: ["dhat-heap"]`, `source_stamp: 2026-09-21T04-34-32Z`,
`git_head: 8d9c6f2647317d3d99516e19c263eb4ac578b4b7` (docs commit after ranking;
kernel bytes unchanged from MEASURED_HEAD).

### Named functions vs live sidecar (D-02, D-07, D-08)

Confirmed in live `rust.json.syms.json` `string_table` (not hunt-list paste).
Gecko gzip has zero `particle_rows` strings; names come from the sidecar.

| Audit short name | Sidecar evidence |
| --- | --- |
| `liquidfun::particle::contact::particle_rows` | exact `particle_rows` (2 hits) |
| `slice_contains` / `check_invariants` | both present |
| `replace_solver_candidate` | `<ParticleStorage>::replace_solver_candidate` |
| `ParticleNeighborhood::from_view` | `<ParticleNeighborhood>::from_view::{closure#0}` (26 `from_view` hits) |
| `RawVecInner::finish_grow` | `alloc::raw_vec::RawVecInner<A>::finish_grow` |
| `Vec<ParticleContact>` collect | `SpecFromIterNested<liquidfun::particle::contact::ParticleContact, …>` |
| `recompute_weights` | present |
| `listener_effects` | present (22 hits) |
| `pressure::damping` | present |
| `ParticleContactUpdate::generate` | mangled `<ParticleContactUpdate>::generate::<…update_particle_contacts…>` |

C++ names appear only on shape-mismatch rows. **Not found** lists SIMD-first,
default Rayon, PGO, lifting `unsafe_code = "forbid"`, WASM-vs-C++, and the
Phase 12 sealed matrix. Approximate leaf-share percents were not independently
re-derived from gecko `stackTable`; names are sidecar-backed.

### `pair.json` is the only Dam Break number (D-03, D-14)

Live `pair.json`: `kind: unprofiled_pair`,
`timing_authority: unprofiled_wall_clock`, walls `71001.949958` /
`216.777375`, `rust_over_cpp_ratio: 327.53395024734476`,
`git_head: 6d98531…`. Audit and timing docs copy those pair fields exactly.
No samply duration, `[profile.profiling]` wall, `step_profiled`, or dhat
clock is quoted as the ratio. Profile/heap identities are
`not_timing_authority`.

### Copy-only exclusive bundle (D-04, D-05)

`mint_exclusive_stamp` still exclusive-`create_dir`s and never
`remove_dir_all`s. `copy_audit_bundle` validates kinds/HEADs/forbidden pair
keys (`samply` / `profile` / `not_timing_authority` / `cargo_profile`) and
nonempty gzip, then `fs::copy`s into a new stamp. CLI tests prove dest ≠
source names and source bytes unchanged; HEAD mismatch, missing gzip,
`../etc`, colon ISO stamps, and samply-tainted `pair.json` fail closed.
Stamp names are allowlisted `YYYY-MM-DDTHH-MM-SSZ`.

`just playground-dam-break-audit-bundle` / `playground-dam-break-heap` are
one-line `cargo xtask playground …` printers. No cmake, ninja, `-g`, samply,
or dhat flags in `just`. USAGE lists all five playground commands.

### dhat isolation (D-11, D-12, D-15)

`crates/liquidfun-wasm`: `default = []`, `dhat-heap = ["dep:dhat"]`,
`dhat` 0.3.3 optional. `#[global_allocator]` and `Profiler` are
`cfg(all(feature = "dhat-heap", not(target_arch = "wasm32")))`. Pair spawn is
`--release --bin dam-break-bench` with no `--features`. Heap spawn is
`--profile profiling --features dhat-heap` and never `--release`.
`crates/liquidfun/Cargo.toml` production deps remain `bitflags` only.
`cargo tree -p liquidfun --edges normal --locked` is bitflags-only; `dhat`
appears under `liquidfun-wasm` only with `--features dhat-heap`.
`Cargo.lock` pins `dhat` 0.3.3. `[profile.profiling]` still
`inherits = "release"`, `debug = true`, `strip = false`; no
`[profile.release]` debug override.

Sidecar-first `classify_profile_symbols` uses
`Path::with_extension("syms.json")` on `rust.json.gz` →
`rust.json.syms.json`, then other `*syms*`, then gzip JSON. Needles include
`alloc::`, `RawVec`, `to_vec`, `core::clone::`; bare `clone` is not a needle.
Skip paths: `Heap skipped: samply showed no allocator/Vec time` vs
`Heap skipped: could not read symbols`. Live dump ran after needle match
(`alloc::`, `__rdl_alloc`, `Vec`, `RawVec`, `to_vec`, `GlobalAlloc`,
`core::alloc::`, `core::clone::`). Fake cargo writes `LIQUIDFUN_DHAT_HEAP_FILE`
on `dhat-heap` **before** the profiling-build arm.

### No committed blobs / empty manifest / no kernel edits (D-09, D-10, D-15)

`git ls-files` has no `*.json.gz`, `*.trace`, or `dhat-heap.json`.
`.gitignore` has `/target/`. `reference/performance/manifest.toml`
`reviewed_reports = []`. `git diff 121d741..HEAD -- crates/liquidfun/src` is
empty. `playground_cli.rs` is a 14-line `#[path]` dispatcher; no
`playground_cli/mod.rs` or `main.rs`; longest child `support.rs` is 304 lines
(cap 628). Production playground/heap/bundle/symbols/stamp/wasm-bin use `?`
and `PlaygroundError`; `unwrap`/`expect` appear only in tests. Tool programs
are `Command::new(path).args([...])`, not a shell string.

### CONTEXT D-16 / verification

`23-VERIFICATION.md` (`status: human_needed`, score 12/12) correctly refuses
to treat itself, passing checks, or the previous advisory `23-REVIEW.md`
(`not_d16_phase_approval: true`) as D-16. This rewrite is the independent
acknowledgment bound to the digests below, reviewer identity Cursor Grok 4.6
(`gsd-code-reviewer`), and review time `2026-09-21T05:21:07Z`.
`implementing_or_fixing_executor: no`.

## Digest

Independently computed from current HEAD file bytes (`git show HEAD:<path>`
concatenated in this listed order), then SHA-256:

```bash
{
  git show HEAD:BENCHMARKING.md
  git show HEAD:Cargo.lock
  git show HEAD:Cargo.toml
  git show HEAD:crates/liquidfun-wasm/Cargo.toml
  git show HEAD:crates/liquidfun-wasm/src/bin/dam_break_bench.rs
  git show HEAD:crates/liquidfun/Cargo.toml
  git show HEAD:docs/native-performance-audit.md
  git show HEAD:docs/playground-dam-break-timing.md
  git show HEAD:justfile
  git show HEAD:reference/performance/manifest.toml
  git show HEAD:tools/xtask/src/main.rs
  git show HEAD:tools/xtask/src/playground.rs
  git show HEAD:tools/xtask/src/playground/bundle.rs
  git show HEAD:tools/xtask/src/playground/bundle/ops.rs
  git show HEAD:tools/xtask/src/playground/heap.rs
  git show HEAD:tools/xtask/src/playground/pair.rs
  git show HEAD:tools/xtask/src/playground/stamp.rs
  git show HEAD:tools/xtask/src/playground/symbols.rs
  git show HEAD:tools/xtask/tests/fixtures/fake_upstream_tool.rs
  git show HEAD:tools/xtask/tests/playground_cli.rs
  git show HEAD:tools/xtask/tests/playground_cli/bundle.rs
  git show HEAD:tools/xtask/tests/playground_cli/heap.rs
  git show HEAD:tools/xtask/tests/playground_cli/pair.rs
  git show HEAD:tools/xtask/tests/playground_cli/profile.rs
  git show HEAD:tools/xtask/tests/playground_cli/support.rs
  git show HEAD:tools/xtask/tests/playground_cli/timers.rs
} | shasum -a 256
```

`review_digest` / `digest`:

`c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`

Supporting (not the binding digest): SHA-256 of
`git diff 121d741d8c0bc706b1b3882b80da6914c79e4a82..HEAD -- <same paths>`:

`28113adb401439bc7d02abb6def57ea6a5e861db258c560924ae1c234d49d7c9`

This review does not authorize package publication, tags, or filling
`reference/performance/manifest.toml`. Samply duration, profiling walls,
`step_profiled`, and dhat dumps are not the 3× number.

---

_Reviewed: 2026-09-21T05:21:07Z_
_Reviewer: Cursor Grok 4.6 (gsd-code-reviewer), AI reviewer, not a human_
_Depth: deep_
_D-16 independent phase acknowledgment: yes_
