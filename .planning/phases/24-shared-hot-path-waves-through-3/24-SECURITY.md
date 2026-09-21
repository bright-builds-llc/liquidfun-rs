---
phase: "24"
slug: shared-hot-path-waves-through-3
status: verified
threats_total: 29
threats_closed: 29
threats_open: 0
accepted_risks: 1
transferred_risks: 0
unregistered_flags: 0
asvs_level: 1
block_on: high
security_enforcement: true
created: "2026-09-21"
verified: "2026-09-21"
---

# Phase 24 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

This audit verifies the 29 threats declared in the six Phase 24 `<threat_model>`
blocks. It does not invent new threats. ASVS Level 1; `block_on: high`.

State B: no prior `24-SECURITY.md`. This is a local physics kernel plus
developer evidence loop: no auth, sessions, or network parsers. Security
enforcement is enabled because `.planning/config.json` does not set
`workflow.security_enforcement` to `false`.

Implementation files were not modified.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Public `ParticleId` → private dense row | Compaction remaps rows; callers must not observe stale indices. | Generational handles, `ParticleIndex` rows, SoA lanes |
| Neighborhood snapshot → SoA lanes | Carried rows are valid only for the view used to build the neighborhood. | `pair_rows`, positions, flags |
| Previous `ParticleContact` IDs | Last-step public handles may be destroyed; lookup must fail closed. | `ParticleContactError` / `ParticleProxyError` |
| Unprofiled `pair.json` vs profile stamp | Only unprofiled walls are the 3× number. | `kind`, `timing_authority`, `rust_over_cpp_ratio` |
| Committed notes vs gitignored blobs | Docs may name stamp paths, never embed dumps. | Stamp names, unreviewed banners |
| Release `check_invariants` vs user-facing mutation | Only the solver hot path may follow C++ NDEBUG; create/mutate stays fail-closed. | `debug_assertions` vs `check_invariants()?` |
| Spot JSON vs Dam Break pair.json | Spot walls must never become the 3× number. | `native_scene_spot`, `not_timing_authority` |
| Published `liquidfun` graph | Playground/xtask/dhat must not leak into the crate. | `bitflags` only |
| Review acknowledgment | Implementer identity must not be recorded as independent review. | SUMMARY / absence of `24-REVIEW.md` |

---

## Threat Register

Categories use STRIDE initials: spoofing (`S`), tampering (`T`), repudiation (`R`),
information disclosure (`I`), denial of service (`D`), and elevation of privilege (`E`).

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-24-01-01 | T | `ParticleContactUpdate::generate` | mitigate | Previous public IDs go through `maybe_live_row` → `resolve_live`; public `ParticleContact` stores `ParticleId` only. | closed |
| T-24-01-02 | T | `validate_pairs` / `listener_effects` | mitigate | `ParticleContactError::MissingParticle` / `WrongParticleSystem`; SoA `.get`; never unwrap a row. | closed |
| T-24-01-03 | D | `Vec` pair/contact growth | mitigate | Contact collection uses `with_capacity` + `push`; later `try_reserve_exact` maps to a typed error; recipe stays 1920. | closed |
| T-24-01-04 | T | `crates/liquidfun` deps | mitigate | `bitflags` only; no serde/dhat/samply. | closed |
| T-24-01-05 | S | Faster sample via cheat | mitigate | Shared `proxy.rs`/`contact.rs` used by `update_particle_contacts`; Dam Break recipe not shrunk. | closed |
| T-24-02-01 | S | `pair.json` | mitigate | `kind unprofiled_pair`, locked 1920/60+600, exclusive stamp, failed stamps preserved. | closed |
| T-24-02-02 | S | samply duration | mitigate | Profile identity sets `not_timing_authority`; samply is not the 3× number. | closed |
| T-24-02-03 | T | `manifest.toml` | mitigate | `reviewed_reports = []`; no README “Rust is N×”. | closed |
| T-24-02-04 | D | stamp mint | mitigate | `mint_exclusive_stamp`; evidence dirs are not `remove_dir_all`'d. | closed |
| T-24-03-01 | T | `replace_solver_candidate` / `check_invariants` | mitigate | Solver-candidate assertions gated; mutation `check_invariants()?` stays fail-closed. | closed |
| T-24-03-02 | I | leftover ranking | mitigate | Sidecar-first D-07 names from the live 24-02 profile; no hunt-list paste. | closed |
| T-24-03-03 | D | `try_reserve` / contact `Vec` | mitigate | `try_reserve_exact` maps to `ParticleStorageError`; no unwrap. | closed |
| T-24-03-04 | T | `body_contact::particle_row` | mitigate | That leftover was not selected; body contacts still resolve through `maybe_live_row`. | closed |
| T-24-03-05 | S | Dam Break cheats | mitigate | Shared `liquidfun` only; `PARTICLE_COUNT = 48 * 40`. | closed |
| T-24-04-01 | S | PERF-GATE evidence | mitigate | Same-HEAD unprofiled pair.json ≤ 3 with locked recipe and `unprofiled_wall_clock`. | closed |
| T-24-04-02 | T | leftover kernels | mitigate | Stale IDs still `resolve_live`; typed contact/proxy errors; no unwrap on reserve. | closed |
| T-24-04-03 | T | `unsafe_code` / SIMD | mitigate | Workspace `forbid`; `std::simd`/`rayon` absent from `liquidfun`. | closed |
| T-24-04-04 | S | `manifest.toml` | mitigate | `reviewed_reports` remains `[]`. | closed |
| T-24-04-05 | D | unbounded leftover loop | accept | Finite hobby loop: each win must improve ratio; D-06 stops at ≤ 3. See accepted-risks log. | closed |
| T-24-05-01 | S | `scene-spot.json` | mitigate | `kind native_scene_spot`, `not_timing_authority true`, no `rust_over_cpp_ratio`. | closed |
| T-24-05-02 | D | Fountain emit unbounded time | mitigate | 180s per-scene wall timeout; fail closed on timeout. | closed |
| T-24-05-03 | T | scene construction | mitigate | Typed `SceneSpotError` (maps `SessionCore` failures); nonzero exit; no unwrap. | closed |
| T-24-05-04 | S | cheat via `World::step` without hooks | mitigate | `SessionCore::create` + `advance(1)` in `scene_spot.rs`. | closed |
| T-24-05-05 | T | `liquidfun` deps | mitigate | Spot tooling in `liquidfun-wasm` + xtask; `liquidfun` has no serde. | closed |
| T-24-06-01 | T | `liquidfun` supply chain | mitigate | `cargo tree -p liquidfun --edges normal` is bitflags-only. | closed |
| T-24-06-02 | S | public speed claim | mitigate | Empty `reviewed_reports`; no README N×; unreviewed banners. | closed |
| T-24-06-03 | E | self-approval | mitigate | SUMMARY forbids implementer REVIEW; no `24-REVIEW.md`. | closed |
| T-24-06-04 | T | `unsafe_code` | mitigate | Workspace lint remains `forbid`; SIMD/rayon grep empty. | closed |
| T-24-06-05 | I | profile blobs in git | mitigate | `git ls-files` has no `json.gz` / `trace` / `dhat-heap.json`. | closed |

---

## Threat Verification Evidence

| Threat ID | Evidence |
|-----------|----------|
| T-24-01-01 | `crates/liquidfun/src/particle/contact.rs:21-26` public contact is `[ParticleId; 2]` only. `contact.rs:176-180` previous IDs use `maybe_live_row`. `crates/liquidfun/src/particle/view.rs:52-56` delegates to `resolve_live`. `crates/liquidfun/src/particle/storage/lifecycle.rs:408-427` generation-checks live rows and returns `StaleOrDestroyed` / `WrongParticleSystem`. `lib.rs:219-221` compile_fail keeps `ParticleIndex` off the public API. |
| T-24-01-02 | `contact.rs:12-16` `WrongParticleSystem` / `MissingParticle`. `contact.rs:102-103`, `122-123`, `142-143` system mismatch. `contact.rs:191-193`, `218-244`, `280-284`, `304-314` SoA `.get` / `maybe_live_row` fail closed. No `unwrap`/`expect` in non-test `contact.rs`. |
| T-24-01-03 | `contact.rs:214` `Vec::with_capacity` then `push`. `runtime.rs:248-250` `try_reserve_exact` maps to `ParticleStorageError::InvalidLaneBundle` (no unwrap). `crates/liquidfun-wasm/src/scene/dam_break.rs:20` `PARTICLE_COUNT = 48 * 40`. `dam_break_bench.rs:13` `EXPECTED_PARTICLE_COUNT = 1920`. |
| T-24-01-04 | `crates/liquidfun/Cargo.toml:19-20` production dep is `bitflags` only. Live `cargo tree -p liquidfun --edges normal --depth 1` prints `bitflags v2.13.0` only. |
| T-24-01-05 | `crates/liquidfun/src/world/particle_coupling.rs:173-190` `update_particle_contacts` uses `ParticleNeighborhood::from_view` and `ParticleContactUpdate::generate_indexed`, mapping `StepError::ParticleProxy` / `ParticleContact`. Dam Break scene file still 1920 particles. |
| T-24-02-01 | `tools/xtask/src/playground/pair.rs:185-190` rejects non-1920; `counts.rs:3-4,56-62` defaults 60/600; `pair.rs:303-304` writes `kind: unprofiled_pair` and `timing_authority: unprofiled_wall_clock`. `24-02-SUMMARY.md` admitted exclusive stamp `2026-09-21T15-50-46Z` ratio `21.008…` below 327.53. `24-04-SUMMARY.md` Failed leftovers table preserves reverted stamps. |
| T-24-02-02 | `tools/xtask/src/playground/profile.rs:155-157` `kind: samply_cpu`, `not_timing_authority: true`. `docs/playground-dam-break-timing.md:3` unreviewed banner; gate table uses pair walls, not samply ms. `docs/native-performance-audit.md` labels samply siblings `not_timing_authority`. |
| T-24-02-03 | `reference/performance/manifest.toml:4` `reviewed_reports = []`. `rg -n "Rust is .*×" README.md` empty. |
| T-24-02-04 | `tools/xtask/src/playground/stamp.rs:32-76` exclusive `create_dir`, fail-closed after eight collisions. `pair.rs:205`, `profile.rs:29`, `spot.rs:42` reuse it. Playground stamp mint does not call `remove_dir_all`. |
| T-24-03-01 | `runtime.rs:69-102` `replace_solver_candidate` runs `check_invariants()?` only under `#[cfg(debug_assertions)]`. `mutation.rs:118` and `:175` remain fail-closed `check_invariants()?`. |
| T-24-03-02 | `24-03-SUMMARY.md` ranked leftover table from sidecar `2026-09-21T15-52-27Z` in D-07 order; admitted `check_invariants` / `slice_contains`, not a Phase 23 hunt-list paste. |
| T-24-03-03 | `runtime.rs:247-250` typed `map_err` on `try_reserve_exact`. `proxy.rs` neighborhood preallocate uses `with_capacity` (no unwrap of `try_reserve`). |
| T-24-03-04 | `24-03-SUMMARY.md` selected the `check_invariants` leftover, not `particle_row` `.expect`. `body_contact.rs:309-312` still goes through `maybe_live_row` for generated contacts. |
| T-24-03-05 | Shared kernel only (`24-04-SUMMARY.md` isolation: empty `dam_break.rs` physics cheat diff). `PARTICLE_COUNT` remains `48 * 40`. |
| T-24-04-01 | `24-04-SUMMARY.md` same-HEAD pair `2026-09-21T20-37-12Z`: `unprofiled_pair`, 1920/60+600, `rust_over_cpp_ratio` `2.9538763508693644`. `docs/playground-dam-break-timing.md` records the unreviewed gate sample. |
| T-24-04-02 | Public generate path still `maybe_live_row` (`contact.rs:176-180`). Stepping maps `ParticleProxyError` / `ParticleContactError` (`particle_coupling.rs:184-190`). Indexed collection still `.get` → `MissingParticle` (`contact.rs:218-222`). `try_reserve_exact` typed (`runtime.rs:248-250`). |
| T-24-04-03 | Root `Cargo.toml:37` `unsafe_code = "forbid"`. `rg std::simd\|rayon crates/liquidfun` empty. |
| T-24-04-04 | `reference/performance/manifest.toml:4` still `reviewed_reports = []`. |
| T-24-04-05 | Accepted: leftover while-loop is finite because each admitted kernel must improve `rust_over_cpp_ratio` and D-06 stops at ≤ 3 (`24-04-SUMMARY.md` winning/failed stamp tables). Gate was not skipped. |
| T-24-05-01 | `tools/xtask/src/playground/spot.rs:16,174-176` writes `kind: native_scene_spot` and `not_timing_authority: true`. `spot.rs:189-193` rejects `rust_over_cpp_ratio`. |
| T-24-05-02 | `crates/liquidfun-wasm/src/scene_spot.rs:15` `SCENE_WALL_TIMEOUT = Duration::from_mins(3)` (180s). `:150-151` and `:163-164` return `SceneSpotError::TimedOut`. |
| T-24-05-03 | `scene_spot.rs:27-45` typed `SceneSpotError` (construction / step / timeout). `:137` maps `SessionCore::create` failures. `crates/liquidfun-wasm/src/bin/playground_scene_spot.rs:12-18` prints the error and exits 1. No `unwrap` in `scene_spot.rs` or the bin. |
| T-24-05-04 | `scene_spot.rs:136-158` `SessionCore::create(id)` then loop `advance(1)` for warmup and measured steps. |
| T-24-05-05 | Spot sources live under `crates/liquidfun-wasm` and `tools/xtask`. `crates/liquidfun/Cargo.toml` has no serde. `justfile:161-162` `playground-scene-spot` is a one-line `cargo xtask` printer. |
| T-24-06-01 | Live `cargo tree -p liquidfun --edges normal --depth 1` → `bitflags v2.13.0` only. |
| T-24-06-02 | Manifest empty; README has no N× claim; `docs/playground-dam-break-timing.md:3`, `docs/native-performance-audit.md:3`, `docs/playground-scene-spot-check.md:3` unreviewed banners. |
| T-24-06-03 | `24-06-SUMMARY.md` Independent review section: implementer does not write REVIEW acknowledgment. No `24-REVIEW.md` in the phase directory. |
| T-24-06-04 | Same as T-24-04-03: `unsafe_code = "forbid"`; SIMD/rayon grep empty. |
| T-24-06-05 | `git ls-files '*.json.gz' '*.trace' 'dhat-heap.json'` empty. |

---

## Summary Threat Flags

Plans 24-01 through 24-05 have no `## Threat Flags` section.

`24-06-SUMMARY.md` `## Threat Flags`: none. Task 1 added a docs pointer; Task 2 ran isolation gates. No new network, auth, file-access, or schema surface.

No unregistered flags.

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-24-01 | T-24-04-05 | Hobby-host leftover loop is finite: each admitted kernel must improve `rust_over_cpp_ratio`, failed stamps are preserved and reverted, and D-06 stops once the unprofiled pair is ≤ 3. The 3× gate was not skipped. | phase 24-04 PLAN accept disposition; recorded by gsd-security-auditor | 2026-09-21 |

No transferred risks.

---

## Unregistered Flags

None.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-21 | 29 | 29 | 0 | gsd-security-auditor (AI) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-21 (gsd-security-auditor). This is threat-mitigation verification, not independent phase review or package-release authorization.

`block_on: high` is satisfied: no open high-severity registered threats.
