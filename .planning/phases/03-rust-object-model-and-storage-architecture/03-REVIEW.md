---
phase: 03-rust-object-model-and-storage-architecture
depth: standard
status: findings
files_reviewed: 20
findings:
  critical: 0
  warning: 2
  info: 1
  total: 3
reviewed_at: 2026-07-10
---

# Phase 3 Code Review

## Scope and method

Reviewed the 20 requested implementation, manifest, architecture, and black-box test files against API-01 through API-08, DOCS-02, and locked decisions D-01 through D-17. The review also used the repository guidance in `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the Bright Builds architecture, code-shape, testing, verification, and Rust standards. The pinned LiquidFun implementation was consulted where D-06 requires upstream-equivalent destruction order.

## Warning findings

### WR-01: Body cascade records use creation order instead of the pinned upstream list order

**Evidence:** `crates/liquidfun/src/world/object.rs:291` appends fixtures to the body's adjacency vector, while `crates/liquidfun/src/world/object.rs:435` through `crates/liquidfun/src/world/object.rs:449` clone and consume that vector from front to back. The public regression at `crates/liquidfun/tests/object_model.rs:74` through `crates/liquidfun/tests/object_model.rs:100` explicitly locks in first-created-fixture then second-created-fixture order. In the pinned oracle, fixtures are inserted at the head of the body list (`third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.cpp:188` through `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.cpp:190`) and `DestroyBody` walks that list from its head (`third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp:154` through `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp:171`), so destruction-listener occurrence order is newest fixture first. Joints have the same prepend-and-walk shape in `b2World.cpp:218` through `b2World.cpp:231` and `b2World.cpp:126` through `b2World.cpp:141`.

**Impact:** D-06 requires the pinned-upstream cascade order and API-03 describes the cascade as upstream-equivalent. With two or more fixtures or joints, owned destruction records and association cleanup are emitted in the opposite within-category order from the oracle. `ARCHITECTURE.md:137` through `ARCHITECTURE.md:143` currently documents the incorrect creation-order behavior as implemented compatibility evidence.

**Action:** Represent body adjacency in the same effective order as upstream (for example, prepend on creation or iterate the stored vectors in reverse during cascade), update snapshots consistently, and change the consumer regression to assert newest-first fixture and joint occurrences against a multi-object fixture.

### WR-02: Particle IDs are not intrinsically particle-system scoped and overlapping storage ranges can alias

**Evidence:** The complete handle identity contains only world, slot, and generation at `crates/liquidfun/src/identity.rs:61` through `crates/liquidfun/src/identity.rs:66`. `ParticleStorage` stores a `system` field at `crates/liquidfun/src/particle/storage.rs:121` through `crates/liquidfun/src/particle/storage.rs:126`, but after construction that field is never consulted. Resolution at `crates/liquidfun/src/particle/storage.rs:382` through `crates/liquidfun/src/particle/storage.rs:393` checks only the world and a caller-supplied slot range. The existing cross-system test at `crates/liquidfun/src/particle/storage/identity.rs:42` through `crates/liquidfun/src/particle/storage/identity.rs:53` passes because it manually assigns non-overlapping bases `0` and `4`; two storages for different systems with the same base and generation can produce equal IDs and resolve one system's ID to the other system's row. The related public owner check at `crates/liquidfun/src/world/object.rs:369` through `crates/liquidfun/src/world/object.rs:373` rejects a same-world group from another system as `HandleError::WrongWorld`, even though the handle's world is correct.

**Impact:** This bypasses D-12's checked world-and-particle-system scope and contradicts `ARCHITECTURE.md:187` through `ARCHITECTURE.md:195`. A future integration that accidentally overlaps ranges could silently resolve a cross-system particle ID rather than return `WrongParticleSystem`; today, the public construction path also reports a semantically false error for the analogous ownership mismatch.

**Action:** Make particle-system ownership an authoritative checked identity dimension, or centralize globally unique range allocation and validate the owning `ParticleSystemId` during every lookup. Add a regression with distinct systems and intentionally overlapping local slot/generation values, plus a public test that a group from another system returns an explicit owner/scope mismatch rather than `WrongWorld`.

## Info findings

### IN-01: Semantic diagnostic IDs silently become duplicates at exhaustion

**Evidence:** `crates/liquidfun/src/world/object.rs:257` through `crates/liquidfun/src/world/object.rs:260` returns the current diagnostic ID and advances with `saturating_add`. Once `u64::MAX` is reached, every later object receives the same value, although `DestructionRecord::diagnostic_id` is documented as a stable world-local semantic identity at `crates/liquidfun/src/world/object.rs:197` through `crates/liquidfun/src/world/object.rs:201`.

**Impact:** This is practically remote but silently violates semantic identity uniqueness and can make owned destruction evidence ambiguous. The rest of the identity design explicitly fails or retires on exhaustion instead of wrapping or saturating.

**Action:** Advance diagnostic IDs with `checked_add`, return a typed exhaustion error without inserting the object, and add a focused boundary test using a test-only near-maximum world fixture.

## Verification

- `cargo fmt --all -- --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed: 45 unit/property tests, 13 integration tests, and 6 compile-fail doctests.
- No production `unsafe`, `unsafe impl`, or `unwrap()` usage was found in the reviewed Rust files; dense `ParticleIndex` remains private and solver-visible ordering does not use hash iteration.

## Conclusion

The phase is not clean: the green test suite currently codifies one incorrect upstream destruction order and does not cover the particle-system alias case. The remaining reviewed contracts, including stale/cross-world arena validation, generation retirement, deferred command revalidation, RAII unlock and poison behavior, owned event multiplicity, association cleanup, and validate-then-commit permutation handling, had no additional critical or warning defects at standard depth.
