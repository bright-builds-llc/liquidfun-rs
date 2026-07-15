---
phase: 09-particle-storage-lifecycle-and-coupling
status: findings
depth: standard
files_reviewed: 83
findings:
  critical: 2
  warning: 8
  info: 0
  total: 10
generated_at: 2026-07-15T19:17:44Z
---

# Phase 9 Code Review

## Scope and methodology

Reviewed the 83 existing, deduplicated source and evidence files declared by the Phase 9 summaries. The review traced particle identity and dense-row permutations, lifecycle and capacity transactions, contact/coupling state, force and query surfaces, Rust/C++ protocol execution, result validation, evidence workflow identity, and compatibility-ledger promotion.

The review applied `AGENTS.md`, `AGENTS.bright-builds.md`, the repository-local guidance, `standards-overrides.md`, and the Bright Builds architecture, code-shape, verification, testing, and Rust standards. It also checked the locked Phase 9 decisions, especially stable semantic identity, authoritative pending-delete state, requested destruction occurrences, closed branch evidence, and D1-only promotion.

## Critical findings

### CR-01 — Promoted differential evidence never compares Rust with the pinned oracle

**Locations:** `.github/workflows/oracle.yml:146-150`; `crates/liquidfun-differential/tests/phase9_corpus.rs:289-303`; `crates/liquidfun-differential/tests/phase9_corpus.rs:353-383`; `reference/compatibility.json:139,142,162,165`

The evidence workflow runs only the `phase9_corpus` test target. Its native test compares two Rust executions with each other, while its oracle-mode test compares C++ replay and debug/release results with each other. No test executes Rust and C++ for the same request and compares their semantic results, and the Phase 9 comparison-policy registry is not consumed. Nevertheless, four compatibility rows are marked `differentially_validated: evidenced` and `platform_validated: evidenced` from these artifacts.

This makes the central Phase 9 parity promotion unsupported: arbitrarily different Rust and C++ results can both pass the canonical and sanitizer jobs.

**Required fix:** add a Phase 9 differential comparator that executes the exact same request through `NativeRigidWorldExecutor` and the pinned process oracle, requires every closed policy path to be consumed, and fails on the first semantic mismatch. Regenerate canonical and sanitizer authority before restoring the four promoted claims.

### CR-02 — Closed-corpus branch coverage is declarative rather than executable

**Locations:** `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json:12-114`; `crates/liquidfun-differential/tests/phase9_corpus.rs:125-210`; `crates/liquidfun-differential/tests/phase9_corpus.rs:232-262`

The manifest declares lifetime ordering, zombie behavior, capacity eviction, listener and filter branches, particle/body contacts, strict-contact modes, dynamic/static coupling, callback control, and other witnesses. The executable request creates only four flag-zero particles, performs edits/force/statistics/basic queries/basic rays, marks one particle, compacts, and destroys both systems before retained rigid work. It does not step live particles or create live bodies beside them, so most declared lifecycle, contact, callback, and coupling branches cannot execute. The coverage test only set-compares branch-name strings and checks that each appears once; it never binds a case to request bytes, actions, checkpoints, or expected observations.

The successful D1 artifacts therefore prove that a list of labels is complete, not that the labeled behavior ran.

**Required fix:** make each case an executable scenario with branch-specific semantic observations, mechanically record which witness each action/checkpoint satisfies, and fail if a declared branch is not reached. Run every case in native, canonical, and sanitizer evidence, and bind the executed request/result digests into the artifact identity before promotion.

## Warning findings

### WR-01 — Particle lifecycle and coupling run on zero-time and continuation calls

**Locations:** `crates/liquidfun/src/world/step.rs:1351-1361`; `crates/liquidfun/src/world/step.rs:1362-1377`; pinned `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp:1019-1026`

`World::step` runs particle lifetime, zombie compaction, contact generation, and rigid coupling unconditionally. The rigid solve is correctly restricted to a fresh continuous step with `time_step > 0.0`, and the pinned source places particle simulation under the same `m_stepComplete && dt > 0` guard. Rust can therefore advance timestamps/lifetimes, destroy particles, regenerate contacts, or apply body reactions on zero-dt calls and repeat particle work on continuation calls for an incomplete continuous step.

**Required fix:** put both particle stages behind the same fresh-and-positive-dt condition as the discrete solve. Add zero-dt and forced continuous-continuation regressions that prove particle state changes exactly once.

### WR-02 — The public `ZOMBIE` flag and authoritative pending-delete state diverge

**Locations:** `crates/liquidfun/src/particle/definition.rs:20-22`; `crates/liquidfun/src/particle/storage.rs:859-869`; `crates/liquidfun/src/particle/storage.rs:945-973`; `crates/liquidfun/src/world/particle_lifecycle.rs:21-33`; `crates/liquidfun/src/world/particle_object.rs:601-610`

Creation stores supplied flags unchanged, but the lifecycle pass compacts only identities already in `PendingDelete`; it never converts a live row carrying `ParticleFlags::ZOMBIE` into pending state. Such a particle can survive indefinitely despite the public flag contract. In the other direction, `mark_particle_for_destruction` moves only identity state and does not set the zombie bit, so public flags omit the pending-delete fact. This violates the locked requirement that one authoritative state drive aggregate flags, views, and lifecycle behavior.

**Required fix:** centralize the zombie transition. Every explicit mark must set the flag and pending state atomically, and a lifecycle pass must first transition all live zombie-flag rows before one compaction transaction. Add regressions for both directions.

### WR-03 — Capacity eviction drops an existing destruction-listener occurrence

**Locations:** `crates/liquidfun/src/particle/lifetime.rs:430-446`; `crates/liquidfun/src/world/particle_object.rs:526-539`; `crates/liquidfun/src/world/particle_object.rs:549-553`

Destroy-by-age capacity preparation returns a `ParticleCompactionOutcome` containing requested listener occurrences. Particle creation discards that outcome during both cloned preflight and real commit. Although the eviction call does not add a listener request, it preserves a victim's pre-existing `DESTRUCTION_LISTENER` flag, so compaction correctly discovers an occurrence and the creation path silently loses it. The pinned source performs the zombie solve synchronously during capacity eviction and invokes the requested listener.

**Required fix:** journal or return the committed eviction occurrence through a lifecycle receipt that the public integration can deliver exactly once. Add a full-system regression whose evicted oldest particle already carries the listener flag.

### WR-04 — Permutations publish zero weights beside retained contacts

**Locations:** `crates/liquidfun/src/particle/storage/permutation.rs:101-147`; `crates/liquidfun/src/particle/storage/permutation.rs:185-215`; `crates/liquidfun/src/particle/storage/permutation.rs:218-270`; `crates/liquidfun/src/particle/view.rs:73-76`

Permutation preparation initializes every candidate weight to zero and copies the primary rows without copying or recomputing weights. It separately remaps and retains surviving particle and body contacts, then commits both the zeroed weight lane and those contacts. A view immediately after contact-bearing compaction or rotation can therefore expose contacts whose published derived weights are all zero until a later contact refresh.

**Required fix:** recompute weights from the remapped contacts before committing the candidate, or clear the derived contacts if the operation intentionally invalidates them. Add a contact-bearing compaction/rotation regression that checks contact and weight coherence.

### WR-05 — Result validation accepts arbitrary Phase 9 particle observations

**Location:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:537-619`

For each particle action, result validation consumes one value of the broad `RigidWorldObservation::Particle` variant and performs no action-specific validation of the nested observation kind, system/particle IDs, ordering, lengths, ordinals, or state. A stale or fabricated particle payload can therefore pass the protocol's validated-result boundary as long as its outer enum tag and count are plausible.

**Required fix:** replay Phase 9 system/particle liveness while validating results and match every action to its required observation variant and semantic identity/order contract. Add mutation tests for wrong variants, unknown/stale IDs, reordered IDs, and inconsistent lengths.

### WR-06 — Rust and C++ disagree on mixed-state body identity

**Locations:** `crates/liquidfun-differential/src/rigid_world/phase9.rs:293-310`; `tools/reference/src/rigid_world_phase9_execute.hpp:311-338`

Rust emits all live semantic body IDs in insertion order for every `MixedState` observation. The C++ adapter hard-codes `body_ids` to an empty array even though it can enumerate live declarations for body snapshots. Any particle action performed while bodies are live already violates the exact `particle.coupling.identity` policy; CR-01 currently masks the mismatch.

**Required fix:** derive live C++ body IDs in the same explicit declaration/insertion order as Rust and add a mixed rigid/particle differential checkpoint.

### WR-07 — Request decoding omits Phase 9 reference and lifecycle validation

**Locations:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:154-163`; `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:397-467`; `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:494-497`; `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:780-861`

The rigid action validator tracks declared, created, and live bodies, fixtures, joints, and ropes. Phase 9 actions receive only scalar/vector/range-shape checks; create, destroy, inspect, pause, mark, compact, statistics, query, and ray IDs are not checked against declarations or live state. Duplicate creation, use-before-create, use-after-destroy, cross-system ranges, and unknown owners can decode as validated requests and fail later, potentially with different Rust and C++ adapter behavior.

**Required fix:** extend the decoder state machine with declared/created/live particle systems and particles plus particle ownership. Reject invalid order and references before either executor runs, with focused negative fixtures.

### WR-08 — The protocol rejects valid negative infinite lifetimes

**Locations:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:806-816`; `crates/liquidfun/src/particle/definition.rs:612-625`; `crates/liquidfun/src/particle/definition.rs:668-671`

The production API accepts every finite lifetime and defines all values at or below zero as infinite, preserving exact bits. Phase 9 request validation instead requires lifetime bits to be nonnegative. This excludes valid negative-infinite-lifetime behavior from the differential protocol and contradicts the locked lifetime semantics.

**Required fix:** validate lifetime as finite rather than nonnegative, then add negative-lifetime native/oracle witnesses covering infinite classification and age ordering.

## Positive observations

- Stable particle IDs are translated consistently at public contact and query boundaries.
- Permutation candidates validate before commit and remap index-bearing proxies, contacts, pair/triad references, group ranges, and expiration order transactionally.
- Force and impulse preparation validates the complete candidate range before mutation.
- Particle contact and coupling work uses cloned candidate systems/bodies before publishing effects, preserving coherence on recoverable failure or hook panic.
- The protocol retains closed-member decoding, bounded arrays/records, exact float-bit transport, separated protocol stdout and diagnostics stderr, and a pinned adapter-input source digest.
- Evidence artifacts bind exact run/head/toolchain/profile identities and recomputable payload hashes; these are strong foundations once the executed parity and branch witnesses are made authoritative.

## Verification

- `cargo test -p liquidfun-differential --test phase9_corpus` passed: 5 tests. The pass confirms the current checks are internally green while CR-01 and CR-02 remain possible.
- The review inspected the pinned upstream `b2World::Step`, particle creation/eviction, zombie solve, and destruction-listener paths for the parity findings above.
- The report diff was reviewed separately from the pre-existing `.planning/STATE.md` and `.planning/config.json` modifications; no source file was changed.

## Conclusion

Phase 9 has two critical evidence-authority defects and eight warning-level implementation/protocol defects. The existing compatibility promotion should not be treated as demonstrated Rust/LiquidFun parity until CR-01 and CR-02 are repaired and new canonical/sanitizer authority is generated. The runtime and protocol warnings should be fixed before Phase 9 is declared clean.
