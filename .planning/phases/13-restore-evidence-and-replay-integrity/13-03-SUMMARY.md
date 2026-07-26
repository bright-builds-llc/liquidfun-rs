---
phase: 13-restore-evidence-and-replay-integrity
plan: '03'
subsystem: canonical-evidence-production
tags: [github-actions, immutable-artifact, d0-d1, legacy-projection, provenance]
requires:
  - phase: 13-restore-evidence-and-replay-integrity
    plan: '01'
    provides: Target-scoped witness provenance and Phase 13 evidence classes
  - phase: 13-restore-evidence-and-replay-integrity
    plan: '02'
    provides: Reviewed rigid-stack capture-schema diagnosis and legacy physics projection
provides:
  - Canonical Linux producer with exact source, upstream, and toolchain identity gates
  - Immutable staged evidence bundle and complete acquisition tuple
  - Diagnosis-selected repeated D0 identity and independent pinned-oracle D1 comparison
affects: [13-04-promotion, 13-05-final-acceptance]
tech-stack:
  added: []
  patterns:
    - Diagnosis-selected D0 and D1 projection with expanded diagnostics retained separately
    - Immutable run-qualified artifact acquisition by provider and internal digests
key-files:
  created:
    - .github/workflows/phase13-evidence-producer.yml
    - tools/xtask/src/phase13_evidence.rs
    - tools/xtask/src/phase13_evidence/bundle.rs
  modified:
    - crates/liquidfun/src/debug_draw/collector/layers.rs
    - crates/liquidfun-differential/src/runner/catalog.rs
    - crates/liquidfun-differential/tests/catalog_round_trip.rs
    - tools/xtask/tests/phase13_evidence_contract.rs
key-decisions:
  - "Preserve semantic broad-phase source order instead of hashing process-unique world handles."
  - "Select the reviewed legacy physics projection only after the typed capture-schema diagnosis succeeds."
  - "Exclude debug-capture records and their aggregate count from cross-engine physics parity while retaining and validating expanded capture evidence separately."
  - "Keep production and read-only validation in one exact-SHA workflow; tracked promotion remains unavailable."
requirements-completed: [FND-04, COMP-04, COMP-05, COMP-08, TEST-07, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-26T00:53:00Z
duration: 4h
completed: 2026-07-26
---

# Phase 13 Plan 03: Canonical Evidence Producer Summary

**Exact-SHA canonical Linux production now emits one immutable, independently checked D0/D1 evidence bundle without modifying tracked evidence**

## Accomplishments

- Added a typed producer and bundle validator that require a clean full producer SHA, the pinned upstream revision, canonical Linux tool identities, repeated witness bytes, repeated native D0 identity, and passing pinned-oracle D1.
- Added a dedicated immutable GitHub Actions producer with exact checkout, pinned actions, bounded retention, read-only post-production validation, and acquisition metadata publication.
- Corrected native broad-phase debug ordering by preserving the documented semantic observation order instead of hashing process-unique world identity.
- Wired Plan 13-02's diagnosis before authority selection so legacy physics D0/D1 is compared independently while expanded debug capture remains separate evidence.
- Published and validated one immutable canonical bundle with no tracked promotion side effect.

## Canonical Acquisition Tuple

- **Producer run ID:** `30181863142`
- **Artifact ID:** `8625804327`
- **Artifact name:** `phase13-staged-30181863142-56844ae4e6b9ead030789eb034b5416d3cec8bf6`
- **Provider digest:** `sha256:9fc150fe6e7346753f8781b17743f9b963a69e1a8ba3081aaec3bdd2e7d1b606`
- **Producer SHA (P):** `56844ae4e6b9ead030789eb034b5416d3cec8bf6`
- **Bundle SHA-256 (B):** `3442e362ae69c31a4b144a90ca7b9662463e0f7089da9a81401533548acf9ba8`
- **Workflow:** `https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/30181863142`

## Verification

- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- `cargo test -p xtask --test phase13_evidence_contract` passed 15/15.
- Focused catalog projection tests passed, including rejection of a body-count divergence.
- `actionlint .github/workflows/phase13-evidence-producer.yml` passed.
- Canonical producer run `30181863142` passed every job step and uploaded the immutable bundle.
- Local `HEAD`, `origin/main`, and P all matched after acquisition.

## Deviations and Auto-Fixes

- Corrected the reviewed catalog slug selected by the producer.
- Recognized Clang's canonical `x86_64-pc-linux-gnu` target spelling without relaxing Rust target validation.
- Removed process-unique world-handle hashing from native broad-phase primitive ordering and added a D0 byte-identity regression.
- Added bounded D1 divergence diagnostics after the original gate hid the first mismatching field.
- Corrected the producer to consult the reviewed capture-schema diagnosis before choosing its D0/D1 projection.
- Corrected stale workflow option names used by the immediate read-only bundle recheck.

## Promotion Boundary

Plan 13-03 only produced and uploaded immutable evidence. Plan 13-04 must acquire this exact tuple, establish a clean promotion base R, prepare the exact seven-path replacement diff, and obtain an independently identified human review acknowledgement before any tracked evidence replacement.
