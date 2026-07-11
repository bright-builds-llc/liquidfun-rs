---
phase: 03-rust-object-model-and-storage-architecture
depth: standard
status: findings
files_reviewed: 20
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
reviewed_at: 2026-07-10
---

# Phase 3 Code Review

## Scope and method

Re-reviewed the same 20 implementation, manifest, architecture, and black-box test files from the original Phase 3 review, plus fixes `9f39ab0`, `7cdbc11`, and `8af9ed8`. The review applied the repo-local `AGENTS.md` contract, `AGENTS.bright-builds.md`, the placeholder-only `standards-overrides.md`, and the Bright Builds architecture, code-shape, verification, testing, and Rust standards. The pinned LiquidFun source at submodule revision `7f20402173fd143a3988c921bc384459c6a858f2` was checked for body fixture/joint list insertion and destruction traversal order.

## Resolution of original findings

- **WR-01 resolved:** Body fixtures and joints are prepended on creation, so cascade records and the root body snapshot use pinned newest-first order. Multi-fixture and multi-joint unit and consumer tests also prove typed association cleanup follows that same record order.
- **WR-02 resolved:** `ParticleId` identity, equality, hashing, arena reconstruction, and lookup now include authoritative `ParticleSystemId` scope. The private storage regression intentionally overlaps local slot and generation ranges between two systems and rejects resolution both ways. Same-world group/system mismatch now returns `HandleError::WrongParticleSystem` rather than the semantically false `WrongWorld`.
- **IN-01 resolved:** Diagnostic allocation uses checked state, permits `u64::MAX` once, and returns `ArenaInsertError::DiagnosticIdExhausted` before a later arena insertion. The boundary test proves unique final IDs and unchanged object count on exhaustion.

## Warning findings

### WR-03: Particle-system cascades construct snapshots after mutating the state they claim to preserve

**Evidence:** `destroy_particle_system` clones the initial group and particle lists at `crates/liquidfun/src/world/object.rs:508` through `crates/liquidfun/src/world/object.rs:510`, but unlike `destroy_body` it does not construct and retain a root snapshot. It removes groups first at lines 513 through 520. `remove_particle_group` then removes the group from the system and clears every member particle's `maybe_group` at lines 661 through 674. The later particle records therefore snapshot `maybe_group: None` at lines 686 through 712, and `remove_particle_system` builds its root snapshot from the already-drained `removed.groups` and `removed.particles` vectors at lines 632 through 648. This contradicts the public `ObjectSnapshot::ParticleSystem` contract (“membership at the start of its destruction transaction”) and `ARCHITECTURE.md:149` through `ARCHITECTURE.md:151`, which says owned snapshots capture pre-invalidation adjacency, owner, and group state. The current cascade tests assert record order and invalidation but never inspect the particle or root-system snapshots.

**Impact:** Consumers receive incomplete semantic destruction evidence: a grouped particle appears ungrouped, and the root particle-system snapshot reports empty membership even when the system contained groups and particles. That breaks D-06's owned-snapshot requirement and can make differential traces or post-destruction diagnostics semantically wrong despite correct record and association-cleanup order.

**Action:** Capture the root system snapshot before any dependent mutation and pass it to root removal, as the body path already does. Preserve each particle's pre-cascade group association while retaining the required groups-then-particles record order, for example by capturing particle snapshots before group cleanup or by using a cascade-specific removal transaction. Add a consumer regression that checks the group record, grouped and ungrouped particle snapshots, root-system membership snapshot, and typed association cleanup order from the same multi-object cascade.

## Verification

- `cargo fmt --all -- --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed: 48 unit/property tests, 14 integration tests, and 6 compile-fail doctests.
- `git diff cd6da0d..HEAD --check` and `git diff --check` passed.
- No production `unsafe`, `unsafe impl`, or `unwrap()` usage was found in the reviewed Rust scope. `ParticleIndex`, lane buffers, raw identity construction, and storage coordinates remain private; solver-visible storage order uses vectors and explicit permutations rather than hash iteration.

## Conclusion

The three original review findings are fixed without regressions in their targeted behavior, and the required Rust gates are green. Phase 3 is still not clean because the particle-system cascade returns snapshots assembled after adjacency and group ownership have already been mutated. The remaining reviewed contracts—including stale, cross-world, cross-system, and generation handling; checked diagnostic exhaustion; bounded commands/events; RAII unlock and poison behavior; permutation atomicity; and public API curation—had no additional critical or warning defects at standard depth.
