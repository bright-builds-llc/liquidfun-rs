---
phase: 12-performance-portability-and-release-hardening
plan: "13"
subsystem: public-documentation-and-release-policy
tags: [rustdoc, safety, zero-unsafe, msrv, platform-support, release]
requires:
  - phase: 12-performance-portability-and-release-hardening
    plan: "10"
    provides: typed regression and coverage evidence contracts
  - phase: 12-performance-portability-and-release-hardening
    plan: "11"
    provides: exact package artifact and platform support policy
  - phase: 12-performance-portability-and-release-hardening
    plan: "18"
    provides: zero-waiver renderer isolation
provides:
  - Exhaustive public API navigation and zero-unsafe safety contract
  - Evidence-scoped README, contributor, and release documentation
  - Executable exact doc markers and stale-claim rejection
affects: [phase-12-release-audit, phase-12-attestation, publication]
tech-stack:
  added: []
  patterns:
    - construct-aware unsafe scan
    - evidence-scoped documentation markers
    - fail-closed publication policy
key-files:
  created:
    - SAFETY.md
    - RELEASE.md
    - crates/liquidfun/tests/public_api_documentation.rs
  modified:
    - README.md
    - CONTRIBUTING.md
    - crates/liquidfun/src/lib.rs
    - tools/xtask/src/docs.rs
    - tools/xtask/tests/docs_contract.rs
key-decisions:
  - Treat the compatibility ledger and release audit as the sole authority for broad public claims.
  - Scan constructible unsafe syntax after stripping comments and literals while checking the workspace lint separately.
  - Represent conditional Intel macOS support as fail-closed when native evidence is absent or stale.
patterns-established:
  - Public docs check current evidence markers and reject known stale maturity claims.
  - Release docs distinguish the frozen source candidate from the later attestation commit.
requirements-completed: [API-11, API-12, DOCS-01, DOCS-07, DOCS-08]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T00:26:06Z
duration: 50 min
completed: 2026-07-23
---

# Phase 12 Plan 13: Public API and Release Documentation Summary

Evidence-scoped public documentation now defines the crate's complete API surface, zero-unsafe contract, supported environments, contributor workflow, and exact-artifact release policy with executable stale-claim rejection.

## Performance

- **Duration:** 50 min
- **Started:** 2026-07-23T23:36:00Z
- **Completed:** 2026-07-24T00:26:06Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments

- Added crate-level API navigation, lifecycle semantics, callback constraints, ownership guidance, and a dedicated zero-unsafe safety policy.
- Added black-box public API documentation tests covering handle invalidation, callback unlock behavior, owned reports, association maps, particle buffers, and construct-aware unsafe syntax rejection.
- Replaced maturity-stale public guidance with an evidence-scoped README, contributor workflow, and release procedure tied to exact artifacts and fail-closed support claims.
- Extended the xtask documentation contract to require all Phase 12 public documents and reject obsolete parity, maturity, particle, testbed, audit, and performance claims.

## Task Commits

Each task was committed atomically:

1. **Task 1: Document the complete public API and zero-unsafe contract** - `5d0dad3` (feat)
2. **Task 2: Publish evidence-backed user, contributor, and release guidance** - `4eb0282` (docs)

## Files Created/Modified

- `crates/liquidfun/src/lib.rs` - Adds public API navigation and lifecycle/ownership documentation.
- `SAFETY.md` - Defines the safe-Rust-only production contract and validation policy.
- `crates/liquidfun/tests/public_api_documentation.rs` - Exercises documented public behavior and scans constructible unsafe syntax.
- `README.md` - Describes current capabilities, maturity, supported targets, and Cargo-only usage without unsupported claims.
- `CONTRIBUTING.md` - Documents bootstrap, ordered gates, evidence ownership, and regression-promotion expectations.
- `RELEASE.md` - Defines SemVer/MSRV policy, source freeze, exact artifact reuse, dry run, publication, and rollback rules.
- `tools/xtask/src/docs.rs` - Enforces current Phase 12 document markers and rejects stale public claims.
- `tools/xtask/tests/docs_contract.rs` - Covers acceptance, missing markers, and stale-claim rejection for the documentation contract.
- `UPSTREAM.md`, `ARCHITECTURE.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `TESTING.md`, `UPSTREAM-CORPUS.md`, and `docs/decisions/0001-oracle-selection.md` - Mechanically normalized with the repository-pinned Markdown formatter.

## Decisions Made

- The compatibility ledger and release audit are the only authority for broad compatibility or release claims; public prose cannot promote itself.
- The source audit strips comments and string, character, and raw-string literals before detecting constructible unsafe Rust syntax, while the workspace lint independently forbids unsafe code.
- Conditional Intel macOS support fails closed whenever fresh native evidence is missing or older than the documented 90-day window.
- Release attestation may be committed after the source candidate freezes, but the exact candidate `.crate` artifact must be reused unchanged.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Normalized seven pre-existing Markdown baseline files**

- **Found during:** Task 1 verification
- **Issue:** The exact repository Markdown check traversed a very large ignored `target/debug/deps` tree before reaching seven already-known source formatting failures.
- **Fix:** After explicit parent authorization, formatted only the seven named baseline files with mdformat 1.0.0 under Python 3.13, verified their changes were whitespace-only, and reran the global tracked-source check in an exact repository mirror that excluded generated and parser-owned trees.
- **Files modified:** `UPSTREAM.md`, `ARCHITECTURE.md`, `standards-overrides.md`, `THIRD_PARTY_NOTICES.md`, `TESTING.md`, `UPSTREAM-CORPUS.md`, `docs/decisions/0001-oracle-selection.md`
- **Commit:** `5d0dad3`

## Issues Encountered

- The root `just markdown-check` invocation spent more than ten minutes enumerating ignored build artifacts. After reproducing the behavior, verification used a source mirror excluding `.git`, `.planning`, `target`, `third_party`, and backup directories while retaining every repository-owned non-GSD Markdown source.

## Verification

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`
- mdformat 1.0.0 under Python 3.13 plus `just markdown-check` over the complete tracked source mirror
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`
- `RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace --all-features`
- `cargo test -p liquidfun --test public_api_documentation`
- `cargo test -p xtask --test docs_contract`
- `cargo xtask docs check`
- `cargo xtask inventory check-report`

All verification passed. The final docs contract covers 12 testing layers and the Phase 4 through Phase 8 and Phase 12 public document contracts; all 38 contract tests and all 5 public API documentation tests pass.

## Known Stubs

None. Placeholder terms in the docs checker and its tests are intentional negative fixtures, not product stubs.

## User Setup Required

None.

## Next Phase Readiness

- Release-audit and attestation plans can consume the documented exact-artifact and evidence-authority contracts.
- Public claims remain intentionally fail-closed until the compatibility ledger and release audit supply the required evidence.

## Self-Check: PASSED

- All three created implementation files and this summary exist.
- Task commits `5d0dad3` and `4eb0282` are present in repository history.
