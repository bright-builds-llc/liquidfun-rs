---
phase: 12-performance-portability-and-release-hardening
plan: "11"
subsystem: packaging-and-platform-policy
tags: [cargo-package, sha256, msrv, portability, d2-evidence]
requires: []
provides:
  - content-addressed reusable crate artifact identity and verification commands
  - exact Rust 1.92 consumer contract with Rust 1.97 native platform reuse
  - closed four-target durable support policy and fail-closed conditional Intel macOS tier
  - D2-only platform evidence classification with no fixture-promotion capability
affects: [phase-12-ci-fanout, release-packaging, platform-validation, msrv]
tech-stack:
  added: []
  patterns: [inspect-hash-extract exact bytes, strict serde contracts, expiring native evidence]
key-files:
  created:
    - tools/xtask/src/package/artifact.rs
    - reference/platform/support.json
    - reference/platform/schema.json
  modified:
    - tools/xtask/src/package.rs
    - tools/xtask/src/package/metadata.rs
    - tools/xtask/tests/package_cli.rs
key-decisions:
  - "Create the reusable crate archive once with Rust 1.97 and bind its exact bytes, source inventory, consumer metadata, and candidate commit in a SHA-256 identity."
  - "Reserve Rust 1.92 full artifact verification for canonical x86_64 Linux while native target lanes use Rust 1.97 and D2 evidence."
  - "Downgrade x86_64 macOS to unsupported whenever its named native-runner evidence is missing, future-dated, malformed, or older than 90 days."
patterns-established:
  - "Artifact trust: validate identity and SHA-256 before confined extraction, then build and test only the inspected bytes."
  - "Platform trust: closed target tiers and strict evidence expiry cannot claim D1 or promote fixtures."
requirements-completed: [FND-06, PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T19:43:27Z
duration: 18m
completed: 2026-07-23
---

# Phase 12 Plan 11: Reusable Package Artifact and Platform Policy Summary

**A single Cargo-produced crate can now be content-addressed, reused across native platform lanes, and verified against exact consumer metadata while platform evidence remains D2-only and Intel macOS support expires closed.**

## Performance

- **Duration:** 18m
- **Started:** 2026-07-23T19:25:34Z
- **Completed:** 2026-07-23T19:43:27Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added `package create-artifact` to create one Rust 1.97 `.crate` and record its SHA-256, byte size, package/version, Rust 1.92 contract, exact features/dependencies/source files, legal files, candidate commit, scalar mode, compiler class, and tolerance profile.
- Added `package verify-artifact --archive --identity --toolchain --target` to reject substitutions before extraction, confine extraction, revalidate the packaged manifest and legal/source inventories, and build/test the inspected bytes.
- Hardened ordinary package verification to require exactly Rust 1.92, the `default` and `differential-internals` features, an MIT license declaration, and the existing private/native dependency and source exclusions.
- Encoded durable Linux x86_64, Linux ARM64, macOS ARM64, and Windows x86_64 targets plus conditional macOS x86_64 with exact 90-day native evidence expiry and explicit unsupported downgrade.
- Prevented platform results from claiming D1 or carrying any fixture-promotion field while preserving strict scalar/compiler/tolerance classification.

## TDD Evidence

- **RED:** The first archive-substitution command test failed with `package/usage: expected verify` because artifact commands did not exist.
- **GREEN:** Twenty-two package CLI tests pass, including exact-byte native reuse and negative contracts for hash substitution, wrong Rust version, missing features, forbidden package content, D1 promotion, fixture promotion, and missing/stale conditional evidence.
- **REFACTOR:** Centralized strict artifact identity, archive inspection, platform policy, expiry, and build/test handling while reusing the established confined extraction and package allowlists.
- The plan prohibited committing a failing RED state, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Verify one reusable publishable artifact and strict platform policy** - `31ae6bb` (feat)

## Files Created/Modified

- `tools/xtask/src/package/artifact.rs` - Creates and verifies content-addressed package identities, exact archive bytes, toolchain/target policy, and expiring native evidence.
- `reference/platform/support.json` - Declares four durable targets, conditional Intel macOS, D2 authority, and fail-closed expiry behavior.
- `reference/platform/schema.json` - Publishes the closed machine-readable platform support schema without fixture-promotion capability.
- `tools/xtask/src/package.rs` - Routes artifact commands and selects Rust 1.92 only on canonical Linux for ordinary package verification.
- `tools/xtask/src/package/metadata.rs` - Enforces and returns normalized package identity, Rust version, license, features, and normal dependencies.
- `tools/xtask/tests/package_cli.rs` - Exercises success and rejection paths through the real command boundary with isolated package fixtures.

## Decisions Made

- Bound `candidate_commit` to the repository HEAD whenever verification runs in a Git checkout; isolated fixtures remain usable outside a checkout.
- Passed the reviewed target explicitly to both extracted-package build and test commands so downstream lanes cannot silently verify host defaults.
- Kept the checked-in Intel macOS evidence record null until sustainable native evidence exists, making its effective result unsupported rather than overstating coverage.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The repository already had the package SHA-256 dependency at workspace scope, so no new dependency was necessary.
- Local execution is macOS ARM64; the actual reusable artifact passed its native Rust 1.97 D2 round trip, while canonical Linux/MSRV command policy is covered by command tests and the explicit Rust 1.92 all-target/all-feature check.

## Known Stubs

None.

## Verification

- `cargo test -p xtask --test package_cli` - 22 passed.
- `cargo xtask package verify` - 171 entries built and tested from a confined extraction.
- `cargo xtask package create-artifact ...` - created one SHA-256-addressed crate from Rust 1.97.
- `cargo xtask package verify-artifact ... --toolchain 1.97.0 --target aarch64-apple-darwin` - exact artifact passed as D2-supported native evidence.
- `cargo +1.92.0 check -p liquidfun --all-targets --all-features` - passed.
- Package source-isolation scan found no `tools/`, `reference/`, `third_party/`, native source, or graphics entries.
- Exact ordered commit gate passed: `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features`.
- Focused `cargo clippy -p xtask --all-targets --all-features -- -D warnings` and `git diff --check` passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CI fan-out can create one candidate artifact, verify it once at the canonical MSRV lane, and reuse the same SHA-256 bytes in Rust 1.97 native platform lanes.
- Intel macOS remains explicitly downgraded until a native runner publishes a fresh, exactly 90-day evidence record.

## Self-Check: PASSED

- Confirmed the summary and every declared key file exist.
- Confirmed task commit `31ae6bb` exists.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.
