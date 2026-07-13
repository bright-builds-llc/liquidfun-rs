---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T06:46:00-05:00
iteration: 1
findings_in_scope: 13
fixed: 13
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes

All 13 findings from `07-REVIEW.md` were fixed and committed atomically. WR-04 and the compiled portion of WR-11 share one commit because the contact-identity correction and the nine-family C++ protocol regression are one inseparable oracle contract change. A focused WR-11 follow-up aligns the Rust protocol regressions exposed by the final workspace test.

## Findings

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| CR-01 | Critical | `a87b954` | Validates aggregate mass and all derived body state before committing a custom-mass mutation; overflow regression proves the world remains unchanged. |
| WR-01 | Warning | `2ba8beb` | Preserves the pending CCD checkpoint until a matching resume succeeds, including failure/retry coverage. |
| WR-02 | Warning | `2fa60bb` | Requires Phase 7 observations declared by each witness family on both engine sides before comparison. |
| WR-03 | Warning | `8f14433` | Removes unconsumed policy entries and closes the registry to fields actually compared by the evidence pipeline. |
| WR-04 | Warning | `cc40a67` | Replaces pointer-only C++ contact identity with lifetime-safe manager occurrence tracking, preventing allocator-address ABA reuse. |
| WR-05 | Warning | `c0d7f93` | Rejects fixture child indices that do not exist for the declared shape topology. |
| WR-06 | Warning | `f53dfa1` | Compares exhaustive/filtered ray hits as multiplicity-preserving multisets, closest equal-minimum identities as sets, and termination by status/count only. |
| WR-07 | Warning | `b6663b4` | Rejects stale rigid request policy hashes in compare, minimize, and staging paths without rewriting accepted fixtures. |
| WR-08 | Warning | `e5f273d` | Executes real native/oracle candidate evaluation during rigid mismatch minimization and persists canonical minimized evidence and provenance. |
| WR-09 | Warning | `20298a7` | Requires a complete minimization result, exact source/reduced signature proof, canonical reduced request bytes, and recorded transformations before staging a minimized regression. |
| WR-10 | Warning | `9c9abd0` | Writes and fsyncs a unique sibling temporary candidate, atomically renames it, fsyncs the parent, and surfaces cleanup failures; interrupted-write retry is covered. |
| WR-11 | Warning | `cc40a67`, `598ed6f` | Updates the compiled C++ protocol self-test and Rust protocol regressions to cover all nine declared witness families and required checkpoints. |
| IN-01 | Info | `5e21f58` | Renames ambiguous contact fixture locals so workspace Clippy passes with warnings denied. |

## Verification

The final tree passed:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`
- `cargo test -p liquidfun-differential --test rigid_fixture_workflow --all-features` — 11 passed
- `cargo test -p liquidfun-differential --lib --all-features fixtures::rigid::tests::interrupted_candidate_publish_cleans_temporary_state_and_allows_retry -- --exact` — 1 passed
- `ctest --test-dir target/reference/oracle-debug --output-on-failure` — 1 passed
- Rigid-world compare and replay against the reviewed debug oracle — all nine witness families matched

Every Rust commit was preceded by the repository-required format, lint, build, and test sequence. The final diff was reviewed for unrelated changes; `.planning/config.json` remains the pre-existing user-owned modification and this report intentionally remains uncommitted for the review workflow.

## Residual Risk

No review finding was skipped. Expensive scheduled fuzzing, sanitizers, and randomized differential campaigns remain part of the project’s normal later evidence lanes, not blockers for this review-fix pass.
