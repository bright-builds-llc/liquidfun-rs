---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
fixed_at: 2026-07-21T19:36:55Z
review_path: .planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-REVIEW.md
iteration: 2
findings_in_scope: 6
fixed: 6
skipped: 0
status: all_fixed
---

# Phase 10: Code Review Fix Report

**Fixed at:** 2026-07-21T19:36:55Z
**Source review:** `.planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-REVIEW.md`
**Iteration:** 2

**Summary:**

- Findings in scope: 6
- Fixed: 6
- Skipped: 0
- Verification: each atomic commit passed `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. Focused regressions also passed before their corresponding commits.

## Fixed Issues

### WR-01: Rust permits destroying a particle system while Phase 10 groups are live

**Files modified:** `crates/liquidfun-differential/tests/phase10_protocol.rs`, `crates/liquidfun-differential/tests/phase10_protocol/lifecycle_validation.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase10.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs`
**Commit:** `00d8caa`
**Status:** fixed: requires human verification
**Applied fix:** Added a Phase 10 lifecycle query for live group ownership and reject particle-system destruction before mutating Phase 9 state whenever that system still owns a live group. Added request-decoder regressions for the invalid system-first ordering and valid group-first control.

### WR-02: Rust accepts Phase 10 group actions with arbitrary nonblank phase labels

**Files modified:** `crates/liquidfun-differential/tests/phase10_protocol/lifecycle_validation.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs`
**Commit:** `cca043b`
**Status:** fixed: requires human verification
**Applied fix:** Required every `ParticleGroup` action record to use the exact `phase10` label and return `CheckpointPhaseMismatch` otherwise. Added regression coverage for a mutated nonblank label plus the canonical control.

### WR-03: Phase 10 result binding uses whole-timeline identities instead of the inspection prefix

**Files modified:** `crates/liquidfun-differential/tests/phase10_protocol/lifecycle_validation.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10/prefix.rs`
**Commit:** `5f63b57`
**Status:** fixed: requires human verification
**Applied fix:** Replaced whole-timeline identity collection with a prefix lifecycle replay that tracks provenance, historical and live group ownership, particle ownership, and split membership uncertainty at each inspection. Added regressions rejecting future groups in early observations and joined-away or destroyed groups in later observations.

### WR-04: Phase 10 event payload identities and kind-specific shapes are not validated

**Files modified:** `crates/liquidfun-differential/tests/phase10_protocol/lifecycle_validation.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10/prefix.rs`
**Commit:** `5f63b57`
**Status:** fixed: requires human verification
**Applied fix:** Added closed per-kind event shape validation and bound group, particle, other-particle, and body identities to the applicable inspection prefix. Added focused negative regressions for each identity field family and for a wrong-kind extra field.

### WR-05: Body contacts do not prove that the fixture belongs to the reported body

**Files modified:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/ownership_tests.rs`
**Commit:** `e308b11`
**Status:** fixed: requires human verification
**Applied fix:** Validate each body contact against an authoritative live `(fixture_id, owner_body_id)` pair instead of independent identity membership. Added a focused ownership regression proving matching pairs are accepted and cross-wired live body/fixture pairs are rejected.

### WR-06: Generated JSON Schema accepts particle flag bit zero

**Files modified:** `crates/liquidfun-differential/tests/phase10_protocol/lifecycle_validation.rs`, `crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs`, `protocol/schemas/scenario-v1.schema.json`, `protocol/schemas/trace-v1.schema.json`
**Commit:** `10970f0`
**Status:** fixed
**Applied fix:** Added `multipleOf: 2` to the shared bounded particle-flag schema and regenerated both tracked schemas. Added schema/decoder parity coverage proving `0`, `2`, and the public mask are accepted while bit zero and one-over-mask values are rejected at both request and result boundaries.

## Verification Evidence

- Focused lifecycle and prefix-result regressions passed for WR-01 through WR-04.
- The focused fixture-owner unit regression passed for WR-05.
- `cargo test -p liquidfun-differential --test phase10_protocol lifecycle_validation::schemas_and_decoders_share_the_closed_particle_flag_domain` passed for WR-06.
- The exact full Rust gate passed before every commit: format, Clippy with warnings denied, all-target/all-feature build, and all-feature tests including doctests.
- Final scoped diff checks passed; planning and agent-runtime changes were excluded from every commit.

_Fixed: 2026-07-21T19:36:55Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
