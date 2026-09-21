---
phase: 24-shared-hot-path-waves-through-3
reviewed: 2026-09-21T21:49:46Z
depth: standard
files_reviewed: 34
files_reviewed_list:
  - BENCHMARKING.md
  - crates/liquidfun-wasm/Cargo.toml
  - crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
  - crates/liquidfun-wasm/src/lib.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/particle.rs
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/contact.rs
  - crates/liquidfun/src/particle/lifetime.rs
  - crates/liquidfun/src/particle/proxy.rs
  - crates/liquidfun/src/particle/solver/boundary.rs
  - crates/liquidfun/src/particle/solver/constraints.rs
  - crates/liquidfun/src/particle/storage/lifecycle.rs
  - crates/liquidfun/src/particle/storage/permutation.rs
  - crates/liquidfun/src/particle/storage/runtime.rs
  - crates/liquidfun/src/particle/view.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_coupling/executor/boundary_runtime.rs
  - crates/liquidfun/src/world/particle_lifecycle.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/tests/particle_contacts.rs
  - docs/native-performance-audit.md
  - docs/playground-dam-break-timing.md
  - docs/playground-scene-spot-check.md
  - justfile
  - tools/xtask/src/main.rs
  - tools/xtask/src/playground.rs
  - tools/xtask/src/playground/spot.rs
  - tools/xtask/tests/fixtures/fake_upstream_tool.rs
  - tools/xtask/tests/playground_cli.rs
  - tools/xtask/tests/playground_cli/spot.rs
findings:
  critical: 0
  warning: 1
  info: 1
  total: 2
status: issues_found
digest: c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d
review_digest: c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d
diff_digest: 2a43aa7a5caea721dfcaa3492c88dc8c52736f461ba8858c16ce9adccbaf138c
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
diff_base: e88a4432a5921f43388dd619a005b2147177b6bb
local_head: a9a6a1c69e0081818487e45a09d93f57b0767c21
---

# Phase 24: Code Review Report

**Reviewed:** 2026-09-21T21:49:46Z
**Depth:** standard
**Files Reviewed:** 34
**Status:** issues_found

This is independent AI review (Cursor Grok 4.6, `gsd-code-reviewer`), not
human approval. The implementing executor did not write this file and must not
treat it as self-approval. Passing automated checks, plan summaries, and
`24-SECURITY.md` are not this acknowledgment.

Guidance used: `AGENTS.md` Repo-Local Guidance (hobby scope; independent AI
review 2026-09-16), `AGENTS.bright-builds.md`, `standards-overrides.md`,
`standards/index.md`, `standards/core/code-shape.md` (628-line trigger),
`standards/languages/rust.md` (`foo.rs` plus `foo/`, no production `unwrap`),
`PROJECT-SCOPE.md`.

## Summary

Reviewed Phase 24 Shared hot-path waves through 3× from parent `e88a443`
(Phase 23 finalize) through HEAD `a9a6a1c`. File scope started from
`24-0{1-6}-SUMMARY.md` `key-files`, then the complete relevant
`git diff e88a443..HEAD` (planning artifacts excluded). Live gitignored
stamps under `target/dam-break-perf/` were inspected for exclusive minting,
`pair.json` ratios, and `scene-spot.json`.

Admitted waves match CONTEXT D-01..D-15 for the shared scalar path, public
`ParticleId` surface, empty `reviewed_reports`, and unprofiled Dam Break
Medium ≤ 3× on this host. One error-path transactional Warning remains, so
this is not a clean phase approval. This review does not authorize package
publication, tags, or filling `reference/performance/manifest.toml`.

## Warnings

### WR-01: Failed particle solver leaves bodies and particle systems desynchronized

**File:** `crates/liquidfun/src/world/particle_coupling.rs:31-57`
**Issue:** `run_particle_solver` clones `bodies` and `particle_groups` for
rollback, then runs lifecycle and `replace_with_empty`s both body and
particle-system arenas. On any error it writes the candidate arenas back
and restores only bodies and groups. `particle_systems` stays at the
post-lifecycle / partial-solver candidate. `World::step` restores
`backup_step_limit_state` (which still clones particle systems) only for
`StepError::LimitExceeded`. Other particle errors therefore leave
pre-lifecycle rigid bodies beside compacted or half-solved particle
systems.

`24-04-SUMMARY.md` records that skipping all `run_particle_solver` backups
failed `continuous_resume_does_not_repeat_particle_stages`, so this mixed
restore is a landed leftover, not an accidental omission. Happy-path Dam
Break and spot-checks do not exercise it. Mixed state after a failed
`World::step` can still break later queries or steps.

Related: `crates/liquidfun/src/world/particle_lifecycle.rs:18-53` now
assigns the in-place arena back before `lifecycle?`, so a mid-loop
lifecycle error also commits partial compaction (the previous clone
candidate was discarded on failure).

**Fix:** Take a pre-lifecycle `particle_systems` backup (or restore all
three arenas from the existing `StepLimitBackup` on every
`run_particle_solver` error), and keep lifecycle assignment fail-closed
until the loop finishes:

```rust
let backup_bodies = self.bodies.clone();
let backup_systems = self.particle_systems.clone();
let backup_groups = self.particle_groups.clone();
self.run_particle_lifecycle_step(configuration.time_step(), hook_run)?;
let mut candidate_bodies = self.bodies.replace_with_empty();
let mut candidate_systems = self.particle_systems.replace_with_empty();
let result = (|| { /* solver loop */ })();
self.bodies = candidate_bodies;
self.particle_systems = candidate_systems;
if result.is_err() {
    self.bodies = backup_bodies;
    self.particle_systems = backup_systems;
    self.particle_groups = backup_groups;
}
```

## Info

### IN-01: Stale “current 3× number” pointer in the Phase 23 audit section

**File:** `docs/native-performance-audit.md:67-70`
**Issue:** The Phase 23 timing-authority subsection still says “The current
3× number is the first leftover admission `pair.json` in the leftover
section.” First leftover admission is stamp `2026-09-21T16-07-02Z` at
`15.08956518243927`, which is not ≤ 3. The later **PERF-GATE leftover
close** section correctly copies `2.8769953439599707` from
`2026-09-21T20-36-30Z/pair.json` and says that is the 3× number.
**Fix:** Point the Phase 23 subsection at the leftover-close stamp (or say
the 3× number is in that later section) so the two statements cannot be
read as competing authorities.

---

## Special-look outcomes (D-01..D-15)

### D-01..D-03 — first wave shape, no public handle break

`Proxy` now stores private `ParticleIndex` rows. `ParticleNeighborhood`
keeps `pair_rows` aligned with public `ParticleNeighborPair` `ParticleId`s
(`pub(in crate::particle)` plus rustdoc `compile_fail`).
`ParticleContactUpdate::generate` / `validate_pairs` / `listener_effects`
no longer call `particle_ids().iter().position`. Previous public contacts
use `maybe_live_row` → `resolve_live` (generational map, not a `HashMap`
cache). `lib.rs` still `compile_fail`s `use liquidfun::ParticleIndex`.
`particle.rs` does not export `ParticleIndex`. Public `ParticleContact` /
`ParticleNeighborPair` remain `ParticleId` only.

### D-04..D-09 — wave admission, exclusive stamps, stop at ≤ 3, no SIMD

Fifty-nine exclusive stamps remain on disk (Phase 23 pair/profile/bundle/heap
plus Phase 24 pairs, samply siblings, and the scene-spot stamp). Failed
leftovers are separate stamps; `mint_exclusive_stamp` still
`create_dir`s and fails on `AlreadyExists`. Locked recipe in every
`pair.json` is 1920 particles, 60 warmup, 600 measured.

Live gate pair `target/dam-break-perf/2026-09-21T20-36-30Z/pair.json`:
`kind: unprofiled_pair`, `timing_authority: unprofiled_wall_clock`,
`git_head: d843ba30d0909cc3215108c4814c8b8e97db9c83`,
`rust.wall_ms: 607.814417`, `cpp.wall_ms: 211.267084`,
`rust_over_cpp_ratio: 2.8769953439599707`. Later same-kernel docs-HEAD
pairs `2026-09-21T20-37-12Z` (`2.9538763508693644`) and
`2026-09-21T20-38-50Z` (`2.956857456935513`) are also ≤ 3. No stamp
overwrite. `rg` of `crates/liquidfun` has no `std::simd`, `rayon`, or
`unsafe `. Workspace `unsafe_code = "forbid"` is unchanged.

### D-10 — shared path, no Dam Break cheat

Kernel edits are in shared `liquidfun` particle/rigid stepping.
`git diff e88a443..HEAD -- crates/liquidfun-wasm/src/scene/dam_break.rs`
is empty. 1920-particle Medium recipe is unchanged.

### D-11..D-12 — spot-check and same-cluster canary

Live `target/dam-break-perf/2026-09-21T21-10-50Z/scene-spot.json` is
`kind: native_scene_spot`, `not_timing_authority: true`, no
`rust_over_cpp_ratio`, no `pair.json`. Five scenes finished
`timed_out: false`. Fountain and Water Wheel grew 1→3200. Committed notes
match those walls. PERF-CANARY2 is an explicit same-cluster sentence; no
second profile stamp. `just playground-scene-spot` is a one-line
`cargo xtask playground scene-spot` printer.

### D-13..D-15 — gates, honesty, isolation

`replace_solver_candidate` / permutation `check_invariants` are
`debug_assertions` only; create/mutate still call `check_invariants()?`.
`crates/liquidfun/Cargo.toml` production deps remain `bitflags` only
(unchanged vs `e88a443`). No `Cargo.lock` / `liquidfun` dependency growth.
`reference/performance/manifest.toml` is `reviewed_reports = []`.
`git ls-files` has no `*.json.gz`, `*.trace`, or `dhat-heap.json`.
`BENCHMARKING.md` points at unreviewed spot-check notes and forbids
copying them into the manifest. README has no “Rust is N×” claim.
samply / `[profile.profiling]` / `step_profiled` / dhat are labeled
`not_timing_authority` and are not the 3× number.

New modules use `foo.rs` plus `foo/` (`scene_spot.rs`, `spot.rs`). No new
`mod.rs`. Touched production files stay under the 628-line trigger
(longest reviewed: `lifecycle.rs` 620). Production hot-path code uses
`?` / `Option`; `unwrap`/`expect` in the reviewed files are tests,
pre-existing invariant `expect`s, or rustdoc examples.

### CONTEXT match vs approval

D-01..D-15 implementation and the unprofiled `pair.json` 3× evidence are
truthful. WR-01 is a real failed-step consistency defect, so this review
does **not** grant a clean Phase 24 approval. Fix WR-01 (and optionally
IN-01) before treating the phase as review-clean. This file is not
package-publication authority.

## Digest

Independently computed from current HEAD file bytes (`git show HEAD:<path>`
concatenated in this listed order), then SHA-256:

```bash
{
  git show HEAD:BENCHMARKING.md
  git show HEAD:crates/liquidfun-wasm/Cargo.toml
  git show HEAD:crates/liquidfun-wasm/src/bin/playground_scene_spot.rs
  git show HEAD:crates/liquidfun-wasm/src/lib.rs
  git show HEAD:crates/liquidfun-wasm/src/scene_spot.rs
  git show HEAD:crates/liquidfun-wasm/src/session.rs
  git show HEAD:crates/liquidfun/src/arena.rs
  git show HEAD:crates/liquidfun/src/lib.rs
  git show HEAD:crates/liquidfun/src/particle.rs
  git show HEAD:crates/liquidfun/src/particle/body_contact.rs
  git show HEAD:crates/liquidfun/src/particle/contact.rs
  git show HEAD:crates/liquidfun/src/particle/lifetime.rs
  git show HEAD:crates/liquidfun/src/particle/proxy.rs
  git show HEAD:crates/liquidfun/src/particle/solver/boundary.rs
  git show HEAD:crates/liquidfun/src/particle/solver/constraints.rs
  git show HEAD:crates/liquidfun/src/particle/storage/lifecycle.rs
  git show HEAD:crates/liquidfun/src/particle/storage/permutation.rs
  git show HEAD:crates/liquidfun/src/particle/storage/runtime.rs
  git show HEAD:crates/liquidfun/src/particle/view.rs
  git show HEAD:crates/liquidfun/src/world/particle_coupling.rs
  git show HEAD:crates/liquidfun/src/world/particle_coupling/executor/boundary_runtime.rs
  git show HEAD:crates/liquidfun/src/world/particle_lifecycle.rs
  git show HEAD:crates/liquidfun/src/world/particle_object.rs
  git show HEAD:crates/liquidfun/tests/particle_contacts.rs
  git show HEAD:docs/native-performance-audit.md
  git show HEAD:docs/playground-dam-break-timing.md
  git show HEAD:docs/playground-scene-spot-check.md
  git show HEAD:justfile
  git show HEAD:tools/xtask/src/main.rs
  git show HEAD:tools/xtask/src/playground.rs
  git show HEAD:tools/xtask/src/playground/spot.rs
  git show HEAD:tools/xtask/tests/fixtures/fake_upstream_tool.rs
  git show HEAD:tools/xtask/tests/playground_cli.rs
  git show HEAD:tools/xtask/tests/playground_cli/spot.rs
} | shasum -a 256
```

`review_digest` / `digest`:

`c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d`

Supporting (not the binding digest): SHA-256 of
`git diff e88a4432a5921f43388dd619a005b2147177b6bb..HEAD -- <same paths>`:

`2a43aa7a5caea721dfcaa3492c88dc8c52736f461ba8858c16ce9adccbaf138c`

This review does not authorize package publication, tags, or filling
`reference/performance/manifest.toml`. Samply duration, profiling walls,
`step_profiled`, and dhat dumps are not the 3× number.

---

_Reviewed: 2026-09-21T21:49:46Z_
_Reviewer: Cursor Grok 4.6 (gsd-code-reviewer), AI reviewer, not a human_
_Depth: standard_
_implementing_or_fixing_executor: no_
