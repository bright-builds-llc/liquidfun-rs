---
status: all_fixed
findings_in_scope: 3
fixed: 3
skipped: 0
iteration: 1
---

# Phase 3 Code Review Fix Report

## Summary

All Phase 3 review findings in the `all` scope were fixed. Each finding was
implemented and committed atomically after the repository's required Rust
verification sequence passed.

## WR-01: Pinned upstream body cascade order

Status: fixed

Commit: `9f39ab0` (`fix(03): match upstream body cascade order`)

Changes:

- Body fixture and joint adjacency now prepends new objects, matching the pinned
  upstream head-insertion list behavior.
- Body cascade records and the body snapshot now preserve newest-first order
  within the joint and fixture categories.
- Unit and black-box regressions use multiple joints and fixtures and assert
  newest-first records, snapshots, survivor cleanup, and typed association
  cleanup order.
- `ARCHITECTURE.md` records the pinned upstream ordering rule and executable
  evidence for D-06.

## WR-02: Authoritative particle-system identity scope

Status: fixed

Commit: `7cdbc11` (`fix(03): scope particle identities by system`)

Changes:

- `ParticleId` identity now includes the complete owning particle-system scope
  in equality, hashing, debug-token input, arena reconstruction, and lookup.
- World object storage retains and validates particle-system scope, and private
  dense particle storage checks scope before local-slot or dense resolution.
- Same-world owner mismatches return the explicit
  `HandleError::WrongParticleSystem` variant instead of `WrongWorld`.
- Regressions prove distinct systems with intentionally overlapping local
  slot/generation ranges produce distinct IDs and reject cross-resolution.
- Unit and black-box tests cover arena scope validation and public group-owner
  mismatch behavior without exposing dense indices.
- Public crate documentation and `ARCHITECTURE.md` document the system-scoped
  identity contract and D-12 evidence.

## IN-01: Checked diagnostic identity exhaustion

Status: fixed

Commit: `8af9ed8` (`fix(03): report diagnostic identity exhaustion`)

Changes:

- World diagnostic identity allocation now advances with `checked_add` and
  records exhausted state explicitly instead of saturating.
- `u64::MAX` is issued at most once; later creation returns the typed
  `ArenaInsertError::DiagnosticIdExhausted` before arena insertion.
- A test-only near-maximum setup proves the final two IDs are unique and the
  failing creation leaves object count unchanged.
- Public crate documentation and `ARCHITECTURE.md` document the exhaustion
  behavior and executable evidence.

## Verification

Before each of the three commits, the following commands passed in this exact
order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Final test evidence after IN-01: 48 unit/property tests, 14 integration tests,
and 6 compile-fail doctests passed. `git diff cd6da0d..HEAD --check` also passed.

## Residual risk

No known review finding remains in scope. The changes remain within the Phase 3
object-model and private particle-storage architecture boundary: no solver work,
unsafe code, public dense indices, or unrelated behavior was introduced.
