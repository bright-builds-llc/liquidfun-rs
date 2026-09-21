---
phase: 24-shared-hot-path-waves-through-3
verified: 2026-09-21T21:47:56Z
status: passed
score: 9/9 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 24-2026-09-21T15-01-47
generated_at: 2026-09-21T21:48:30Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 24: Shared hot-path waves through 3× Verification Report

**Phase Goal:** A developer can land evidenced scalar fixes in shared `liquidfun` particle/rigid stepping until the unprofiled Dam Break Medium pair is ≤ 3× pinned C++, while other playground scenes stay usable and the scalar deterministic baseline is unchanged.
**Verified:** 2026-09-21T21:47:56Z
**Status:** passed
**Re-verification:** No — initial verification

**Provenance:** `24-CONTEXT.md`, all six `24-0N-PLAN.md` files, and all six `24-0N-SUMMARY.md` files share `lifecycle_mode: yolo` and `phase_lifecycle_id: 24-2026-09-21T15-01-47`. This report copies those fields. Independent AI review is a later separate-AI step and is not this verifier self-approving the phase.

## Goal Achievement

### Observable Truths

Merged from ROADMAP success criteria plus non-duplicative PLAN `must_haves`. Roadmap wording is kept where a plan restated the same contract.

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Developer can land a hot-path change only when evidence names a function or typed bottleneck with non-trivial profile share, the unprofiled Dam Break Medium ratio improves on the same host and `just playground-dam-break-bench` recipe, existing native tests still pass, and the scene particle count is not lowered; a physics mismatch is a failed candidate, never a faster sample. | ✓ VERIFIED | Wave 1 `particle_rows` then D-07 leftovers from live sidecars. New exclusive `pair.json` stamps; Phase 23 `2026-09-21T04-32-19Z` still on disk. Failed leftovers kept (for example `2026-09-21T16-58-44Z` ratio `6.022655530354479`, `2026-09-21T19-28-39Z` ratio `3.4454334638738593`). `PARTICLE_COUNT` remains `48 * 40`. `24-06-SUMMARY.md` records `cargo test -p liquidfun` exit 0. |
| 2 | Admitted fixes land in shared `liquidfun` particle/rigid stepping (neighborhood/proxy, contacts, particle–body coupling, pressure/damping/integrate, rigid contact solve) so other particle/rigid scenes can benefit; Dam Break-only, WASM-copy, or skipped-solver cheats do not satisfy this. | ✓ VERIFIED | Edits in `proxy.rs`, `contact.rs`, `storage/runtime.rs`, `body_contact.rs`, `solver/boundary.rs`, `world/particle_coupling.rs`. `World::step` still calls `run_particle_solver` → `update_particle_contacts`. Scene-spot uses `SessionCore::advance` → `hooks.on_advance` then `world.step`. `dam_break.rs` still `48 * 40`. No `capture_frame` / `step_profiled` in `scene_spot.rs`. |
| 3 | Developer can demonstrate native Dam Break Medium unprofiled wall time ≤ 3× pinned C++ on the same host under scalar `--release` vs `oracle-release` for the locked 60+600-step pair, recording host, git HEAD, compilers, both wall times, and ratio into a new dated evidence directory; `reference/performance/manifest.toml` stays empty. | ✓ VERIFIED | Copied from `pair.json` only. Gate pair `target/dam-break-perf/2026-09-21T20-38-50Z/pair.json`: `kind unprofiled_pair`, `timing_authority unprofiled_wall_clock`, particles `1920`, warmup `60`, measured `600`, `os macos`, `arch aarch64`, `cpu_brand Apple M4 Max`, `git_head 89d3456406c3c79ed500192bca9491c707033647`, rustc `rustc 1.97.0 (2d8144b78 2026-07-07)`, C++ `AppleClang 21.0.0.21000334`, `rust.wall_ms 631.861834`, `cpp.wall_ms 213.693708`, `rust_over_cpp_ratio 2.956857456935513`. Kernel-HEAD pair `2026-09-21T20-36-30Z`: `git_head d843ba30d0909cc3215108c4814c8b8e97db9c83`, `rust.wall_ms 607.814417`, `cpp.wall_ms 211.267084`, `rust_over_cpp_ratio 2.8769953439599707`. `reviewed_reports = []`. |
| 4 | The scalar deterministic compatibility baseline still holds: no default Rayon or SIMD, no `-ffast-math` / `-march=native` on the pair, no lifting `unsafe_code = "forbid"` to chase the canary. | ✓ VERIFIED | Workspace `unsafe_code = "forbid"`. `cargo tree -p liquidfun --edges normal` is `liquidfun` → `bitflags v2.13.0` only. No `std::simd` / `rayon` under `crates/liquidfun/src`. No `[profile.release]` debug override. Pair JSON compilers are ordinary rustc 1.97.0 vs AppleClang `oracle-release`. |
| 5 | Developer can spot-check Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel with native `--release` headless stepping after shared-path fixes, recording non-catastrophic completion. | ✓ VERIFIED | `just playground-scene-spot` → `cargo xtask playground scene-spot`. Live stamp `target/dam-break-perf/2026-09-21T21-10-50Z/scene-spot.json` (`kind native_scene_spot`, `not_timing_authority true`, no `rust_over_cpp_ratio`, no `pair.json`). All five scenes `timed_out false`. Fountain and Water Wheel `1 → 3200` particles. |
| 6 | After PERF-GATE, Dam Break and the spot-checks share the same dominant particle-contact cluster, or a second native canary is recorded. | ✓ VERIFIED | `docs/playground-scene-spot-check.md` PERF-CANARY2 paragraph: same dominant particle-contact cluster; Fountain/Water Wheel extra wall is emission to the 3200 cap, not a rigid-solve hang. No second profile stamp. |
| 7 | Neighborhood proxies carry dense `ParticleIndex` with a private `pair_rows` lane; `generate` / `validate_pairs` / `listener_effects` index SoA in O(1) or via `maybe_live_row`; public contacts stay `ParticleId`; destroyed previous IDs return `MissingParticle`. | ✓ VERIFIED | `Proxy.row: ParticleIndex`, `pair_rows`, `pair_rows()`. No `fn particle_rows`. `maybe_live_row` + `pub(in crate::particle) fn resolve_live`. `ParticleContact` / `ParticleNeighborPair` store `[ParticleId; 2]` only. `ParticleIndex` is not `pub use`d. Test `contact_generate_returns_missing_particle_when_previous_id_was_destroyed`. `lib.rs` compile_fail doctest `use liquidfun::ParticleIndex`. |
| 8 | Exclusive evidence stamps are minted without overwriting Phase 23 or gate stamps; sibling samply stamps are `not_timing_authority` and contain no `pair.json`. | ✓ VERIFIED | Phase 23 pair `2026-09-21T04-32-19Z` and bundle `2026-09-21T04-34-32Z` remain. Wave 1 pair `2026-09-21T15-50-46Z` ratio `21.00845179052245`; profile `2026-09-21T15-52-27Z` `kind samply_cpu`, `not_timing_authority true`, no `pair.json`. Failed leftover directories preserved. Spot stamp does not contain `pair.json`. |
| 9 | `liquidfun` stays bitflags-only and package-isolated; playground `just` aliases stay one-line xtask printers; README has no universal “Rust is N×” claim; the implementing agent does not self-approve independent review. | ✓ VERIFIED | `crates/liquidfun/Cargo.toml` production dep is `bitflags` only. `FORBIDDEN_PREFIXES` still in `tools/xtask/src/package.rs`. `24-06-SUMMARY.md` records `cargo xtask package verify` exit 0. `just` playground recipes are one-line `cargo xtask playground …` with no `samply`/`cmake`/`dhat`. `BENCHMARKING.md` points at `docs/playground-scene-spot-check.md` and forbids copying into `manifest.toml`. `git ls-files '*.json.gz' '*.trace' 'dhat-heap.json'` empty. No `24-REVIEW.md`. |

**Score:** 9/9 truths verified

### Agent-performed simple UAT

Objective checkpoints only. Not independent AI review and not invented human approval.

| Checkpoint | Result | Verified by | Evidence |
| --- | --- | --- | --- |
| Gate `pair.json` `rust_over_cpp_ratio` ≤ 3 | pass | agent | `target/dam-break-perf/2026-09-21T20-38-50Z/pair.json` ratio `2.956857456935513`; kernel-HEAD `2026-09-21T20-36-30Z` ratio `2.8769953439599707` |
| Five scenes completed in spot JSON | pass | agent | `target/dam-break-perf/2026-09-21T21-10-50Z/scene-spot.json` five scenes, all `timed_out false` |
| `liquidfun` bitflags-only | pass | agent | `cargo tree -p liquidfun --edges normal` → `bitflags v2.13.0` only |
| Empty manifest | pass | agent | `reference/performance/manifest.toml` `reviewed_reports = []` |
| Native tests recorded | pass | agent | `24-06-SUMMARY.md` isolation table: `cargo test -p liquidfun` exit 0; Task 2 commits `5715c2b` / `ceda4cb` exist |

### Required Artifacts

gsd-tools `verify artifacts` on all six plans: 22/22 passed. Manual L1–L3 below.

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun/src/particle/proxy.rs` | `Proxy.row` plus private `pair_rows` | ✓ VERIFIED | 295 lines; `row: ParticleIndex`, `pair_rows()`, compile_fail, unit test |
| `crates/liquidfun/src/particle/contact.rs` | O(1) SoA indexing | ✓ VERIFIED | 394 lines; `pair_rows` / `maybe_live_row`; public `generate` plus stepping `generate_indexed` |
| `crates/liquidfun/src/particle/view.rs` | `maybe_live_row` | ✓ VERIFIED | `fn maybe_live_row` |
| `crates/liquidfun/src/particle/storage/lifecycle.rs` | crate-visible `resolve_live` | ✓ VERIFIED | `pub(in crate::particle) fn resolve_live` at line 408; `check_invariants` still present |
| `crates/liquidfun/tests/particle_contacts.rs` | `MissingParticle` on destroyed IDs | ✓ VERIFIED | `contact_generate_returns_missing_particle_when_previous_id_was_destroyed` |
| `docs/playground-dam-break-timing.md` | Unreviewed gate pair ≤ 3 | ✓ VERIFIED | Banner + kernel pair `2.8769953439599707` from `2026-09-21T20-36-30Z` |
| `docs/native-performance-audit.md` | Unreviewed leftover-until-gate notes | ✓ VERIFIED | Wave 1, leftover, PERF-GATE sections; samply `not_timing_authority` |
| `docs/playground-scene-spot-check.md` | Five-scene table + same-cluster | ✓ VERIFIED | Table matches spot JSON; PERF-CANARY2 paragraph |
| `crates/liquidfun-wasm/src/scene_spot.rs` | Native five-scene `SessionCore` stepper | ✓ VERIFIED | 219 lines; `SessionCore::create`; `.advance(1)` at 145 and 158 |
| `tools/xtask/src/playground/spot.rs` | Exclusive `scene-spot.json` persist | ✓ VERIFIED | 379 lines; `native_scene_spot`; `mint_exclusive_stamp`; rejects `rust_over_cpp_ratio` |
| `justfile` | One-line `playground-scene-spot` | ✓ VERIFIED | `cargo xtask playground scene-spot` |
| `BENCHMARKING.md` | Pointer to spot-check notes | ✓ VERIFIED | Names `playground-scene-spot-check.md`; forbids manifest copy |
| `Cargo.toml` | `unsafe_code = "forbid"` | ✓ VERIFIED | Workspace lint; `[profile.profiling]` only |
| `crates/liquidfun/Cargo.toml` | bitflags-only | ✓ VERIFIED | `[dependencies] bitflags`; no serde/dhat/samply |
| `reference/performance/manifest.toml` | Empty reviewed reports | ✓ VERIFIED | `reviewed_reports = []` |

### Key Link Verification

gsd-tools `verify key-links`: 12/13 tool-verified. The 24-05 `advance\\(1\\)` miss is a regex-escape false negative; manual grep finds `.advance(1)`.

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `proxy.rs` | `storage.rs` | `Proxy.row: ParticleIndex` | WIRED | Pattern `row: ParticleIndex` |
| `contact.rs` | `view.rs` | `maybe_live_row` | WIRED | Previous public IDs and listener flags |
| `particle_coupling.rs` | `contact.rs` | shared contact generate | WIRED | Hot path calls `ParticleContactUpdate::generate_indexed`; public `generate` remains for tests |
| `playground-dam-break-timing.md` | `target/dam-break-perf` | gate stamp path | WIRED | Cites `2026-09-21T20-36-30Z` |
| `native-performance-audit.md` | timing doc / stamps | unprofiled pair is 3× number | WIRED | Unprofiled `pair.json` sections; samply labeled `not_timing_authority` |
| `particle_coupling.rs` | `particle` | `update_particle_contacts` | WIRED | `World::step` → `run_particle_solver` |
| `scene_spot.rs` | `session.rs` | `SessionCore::create` + `advance(1)` | WIRED | Manual: create at 137; `.advance(1)` at 145/158; `on_advance` inside `SessionCore::advance` |
| `spot.rs` | `stamp.rs` | `mint_exclusive_stamp` | WIRED | Exclusive persist |
| `playground-scene-spot-check.md` | `scene.rs` | five SceneId names | WIRED | Fountain through Water Wheel; not Dam Break C++ |
| `BENCHMARKING.md` | `playground-scene-spot-check.md` | unreviewed pointer | WIRED | Exploratory local diagnosis |
| `package.rs` | `crates/liquidfun` | `FORBIDDEN_PREFIXES` | WIRED | Isolation still encoded |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `pair.json` gate | `rust_over_cpp_ratio`, `rust.wall_ms`, `cpp.wall_ms` | Unprofiled `just playground-dam-break-bench` persist | Yes — live walls `631.861834` / `213.693708` | ✓ FLOWING |
| `scene-spot.json` | `wall_ms`, `start_particles`, `end_particles` | `time_scene` Instant loop + `live_particle_count` snapshot | Yes — Fountain `1→3200`, `2421.112916` ms | ✓ FLOWING |
| `docs/playground-dam-break-timing.md` | documented ratio | Copied from `2026-09-21T20-36-30Z/pair.json` | Yes — `2.8769953439599707` | ✓ FLOWING |
| `docs/playground-scene-spot-check.md` | scene table | Copied from `2026-09-21T21-10-50Z/scene-spot.json` | Yes — matches JSON fields | ✓ FLOWING |

Docs are not UI components; they cite gitignored stamps rather than hardcoded empty tables.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Gate pair ≤ 3, locked recipe | Python assert on `2026-09-21T20-38-50Z/pair.json` | ratio `2.956857456935513`, 1920/60/600, `unprofiled_wall_clock` | ✓ PASS |
| Kernel-HEAD pair ≤ 3 | Python assert on `2026-09-21T20-36-30Z/pair.json` | ratio `2.8769953439599707` | ✓ PASS |
| Five-scene spot JSON | Python assert on `2026-09-21T21-10-50Z/scene-spot.json` | five names, `timed_out false`, no ratio | ✓ PASS |
| bitflags-only graph | `cargo tree -p liquidfun --edges normal` | `liquidfun` → `bitflags v2.13.0` | ✓ PASS |
| Full `cargo test -p liquidfun` this turn | skipped (keep verification fast) | Recorded pass in `24-06-SUMMARY.md` | ✓ PASS (recorded) |

### Requirements Coverage

All six IDs appear in PLAN frontmatter and in REQUIREMENTS.md Phase 24 mapping. No orphaned Phase 24 IDs. `PERF-NOTES` and `PERF-WASM` map to Phase 25 and are out of this phase.

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| PERF-ADMIT | 24-02, 24-03, 24-04 | Named-share admission + improving exclusive pair; failed physics/ratio is not a win | ✓ SATISFIED | Live sidecar leftovers; improving pairs; failed stamps kept |
| PERF-SHARED | 24-01, 24-03, 24-04 | Shared particle/rigid stepping, not Dam Break cheats | ✓ SATISFIED | Shared crate edits; `World::step` still solves; scene count 1920 |
| PERF-GATE | 24-04 | Unprofiled Dam Break Medium ≤ 3× with recorded identity; empty manifest | ✓ SATISFIED | `pair.json` ratios `2.956857456935513` and `2.8769953439599707`; `reviewed_reports = []` |
| PERF-BASELINE | 24-01, 24-04, 24-06 | Scalar safe baseline; bitflags-only; `unsafe_code` forbid | ✓ SATISFIED | cargo tree; workspace lint; no SIMD/Rayon |
| PERF-SPOT | 24-05 | Five-scene native `--release` headless spot-check | ✓ SATISFIED | `scene-spot.json` + committed notes table |
| PERF-CANARY2 | 24-05 | Same-cluster note or second canary | ✓ SATISFIED | Same-cluster paragraph; no second profile |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `docs/native-performance-audit.md` | 69–70 | Stale sentence: “The current 3× number is the first leftover admission `pair.json`” | ℹ️ Info | Later PERF-GATE section and `playground-dam-break-timing.md` cite the real ≤3 pair. Historical Wave 1/leftover sections still say PERF-GATE was not claimed, which was true at those stamps. Does not undo the gate. |
| `crates/liquidfun/src/particle/solver/manifest/witness_registry.rs` | 1 | `HashMap` | ℹ️ Info | Witness registry, not `generate` identity cache |
| `crates/liquidfun/src/particle/storage/validation.rs` | 1 | `HashMap` | ℹ️ Info | Validation helper, not contact hot path |
| `crates/liquidfun/src/world/particle_coupling.rs` | 187 | `generate_indexed` instead of public `generate` | ℹ️ Info | Later leftover; still shared `World::step`. Public `generate` remains for tests. |

User-facing `check_invariants()?` remains on `creation.rs` / `mutation.rs`. Solver-candidate path is `#[cfg(debug_assertions)]` only, matching D-08.

No TODO/FIXME/placeholder stubs in `contact.rs`, `proxy.rs`, `scene_spot.rs`, or `spot.rs`.

### Human Verification Required

None. Pair ratio, spot completion, isolation, and empty manifest are objective on-disk checks. Independent AI review remains a **separate** later reviewer and is not recorded as this verifier approving the phase.

### Gaps Summary

No blocking gaps. Phase 24 goal holds: evidenced scalar shared-path waves closed the unprofiled Dam Break Medium pair to ≤ 3× pinned C++ on this host, five playground scenes completed headless without hang/timeout, and the scalar deterministic baseline is unchanged.

Informational notes (do not fail the goal):

1. Last exclusive pair `git_head` is `89d3456406c3c79ed500192bca9491c707033647` (`docs(24-04): complete leftover waves through 3x plan`). Later 24-05/24-06 commits add scene-spot tooling, a BENCHMARKING pointer, and a clippy wrap-drop (`5715c2b`) that returns `()` from always-Ok helpers. That is not a new leftover kernel and does not rewrite Dam Break particle count.
1. `docs/native-performance-audit.md` still has one leftover-era sentence calling the first leftover pair “the current 3× number”; the PERF-GATE section below it is the authoritative ≤3 record.
1. Independent AI review is still required after this report, by a different agent, per owner policy 2026-09-16 and `24-06-SUMMARY.md`.

---

_Verified: 2026-09-21T21:47:56Z_
_Verifier: Claude (gsd-verifier)_
