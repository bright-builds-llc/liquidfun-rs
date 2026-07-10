---
phase: 02-semantic-protocol-and-oracle-round-trip
fixed_at: "2026-07-10T13:09:44Z"
review_path: .planning/phases/02-semantic-protocol-and-oracle-round-trip/02-REVIEW.md
iteration: 2
findings_in_scope: 2
fixed: 2
skipped: 0
status: all_fixed
---

# Phase 2: Code Review Fix Report

**Fixed at:** 2026-07-10T13:09:44Z
**Source review:** `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-REVIEW.md`
**Iteration:** 2

**Summary:**

- Findings in scope: 2
- Fixed: 2
- Skipped: 0
- Prior iteration: all 11 iteration-1 fixes remain resolved

## Fixed Issues

### CR-01: Post-rename sync failure still rolls back an already committed artifact

**Files modified:** `crates/liquidfun-differential/src/fixtures/lifecycle.rs`, `crates/liquidfun-differential/src/fixtures/storage.rs`
**Commit:** `b1c9fe6`
**Applied fix:** Defined the manifest rename as the transaction's commit point and modeled every later manifest-directory durability or lock-cleanup failure as a committed warning. Promotion now receives `Err` only for pre-rename failures, so it cannot delete a destination referenced by the renamed manifest. Deterministic post-rename directory-sync fault injection proves the manifest and promoted destination both remain present; the earlier lock-cleanup fault test remains passing.

### WR-01: Failure bundles can record the wrong request and omit the validated build identity

**Files modified:** `crates/liquidfun-differential/src/main.rs`, `crates/liquidfun-differential/src/minimize_command.rs`, `crates/liquidfun-differential/src/runner.rs`, `crates/liquidfun-differential/tests/fixtures/fake_oracle.rs`, `crates/liquidfun-differential/tests/round_trip.rs`
**Commit:** `3d25243`
**Applied fix:** Added typed mismatch and harness run outcomes that carry the actual executed `ScenarioRequestRecord`, its canonical newline-complete JSONL, and the validated oracle session identity. Physics mismatches require that identity by type; harness outcomes retain it whenever the handshake completed. CLI reports, normal failure bundles, and minimization failure paths now persist this runner-owned context. Fake-oracle integration coverage proves second-request harness failures and physics mismatches have mutually consistent request IDs, canonical JSONL, reports, and session identities in both reuse and sanitizer profiles.

## Prior Iteration Disposition

All 11 findings fixed in iteration 1 remain resolved. Iteration 2 strengthens the transaction boundary behind C1 and the evidence fidelity behind W8 without reopening the other findings.

- C1: committed promotion integrity remains fixed; CR-01 additionally closes the post-rename directory-sync path.
- W1: deterministic CLI minimization remains active and tested.
- W2: bounded hashed reuse request identities remain active and tested.
- W3: native adapter source/toolchain identity remains bound and tested.
- W4: dirty fixture generator inputs remain rejected.
- W5: the shared C++ adapter input manifest remains authoritative.
- W6: empty checkpoint phases remain consistently rejected across Rust, schema fixtures, and C++.
- W7: the C++ stdin reader remains incrementally bounded.
- W8: bounded failure bundles remain persisted; WR-01 now binds them to the exact failed request and session.
- W9: scheduled and manual evidence runs remain non-cancelling.
- W10: concurrent output remains reconciled against the authoritative per-request budget.

Iteration-1 commits remain in history: `c8798cb`, `71eafb4`, `ce23bbc`, `f556b89`, `3f8da8f`, `cc5678b`, `03ef5a4`, `51d866d`, `25fbc16`, `45eac83`, and `ebc819f`.

## Verification

- Before each iteration-2 fix commit, in the required order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` — both sequences passed.
- Final full workspace pass: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --all-targets --all-features`, and `cargo test --workspace --all-features` — passed.
- Focused CR-01 tests `committed_manifest_survives_directory_sync_failure` and `committed_manifest_survives_lock_cleanup_failure` — passed.
- Full `liquidfun-differential` test suite — passed, including 10 round-trip CLI tests and 13 fixture workflow tests.
- Focused WR-01 test `cli_reuse_and_sanitizer_bundles_bind_the_second_request_and_session_identity` — passed for second-request harness failure and physics mismatch cases in both profiles.
- Aggregate repository contracts: `cargo xtask check` — passed.
- Native oracle CTest: debug and ASan/UBSan presets — passed.
- Real-oracle differential comparisons: debug reuse and ASan/UBSan sanitizer profiles — passed.
- `git diff --check` — passed.

## Skipped Issues

None.

## Environment Note

Local native verification used CMake 3.27.9 and Apple Clang 21.0.0. Both debug and ASan/UBSan verification passed, but these differ from the repository's canonical CMake 4.3.3 and Clang 22.1.8 identities; canonical-version coverage remains the responsibility of CI.

_Fixed: 2026-07-10T13:09:44Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 2_
