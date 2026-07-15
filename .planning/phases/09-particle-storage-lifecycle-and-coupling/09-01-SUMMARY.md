---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "01"
subsystem: particle-foundation
tags: [rust, particles, bitflags, validation, storage, permutation]
requires:
  - phase: 03-rust-object-model-and-storage-architecture
    provides: stable system-scoped particle identities and transactional dense-storage evidence
  - phase: 04-math-settings-and-numerical-policy
    provides: checked finite MKS math vocabulary and exact-bit preservation policy
provides:
  - checked public particle-system and particle construction definitions
  - exact retained particle flag bits and byte-exact particle colors
  - closed Phase 9 particle-state allocation, clear, permutation, and remap inventory
affects: [09-02, 09-03, 09-04, 09-05, 09-06, 09-07, 09-08, 09-09]
tech-stack:
  added: [bitflags 2.13.0]
  patterns: [checked definition builders, typed capacity policy, closed executable lane inventory]
key-files:
  created:
    - crates/liquidfun/src/particle/definition.rs
    - crates/liquidfun/src/particle/storage/lane_inventory.rs
    - crates/liquidfun/tests/particle_definitions.rs
  modified:
    - Cargo.toml
    - crates/liquidfun/Cargo.toml
    - crates/liquidfun/src/lib.rs
    - crates/liquidfun/src/particle.rs
    - crates/liquidfun/src/particle/storage.rs
key-decisions:
  - "Retain unknown particle flag bits because the pinned uint32 contract does not reject or normalize them."
  - "Carry typed user-association inputs in ParticleDef without storing Any, raw pointers, or a generic value in World."
  - "Keep declared fixed/growable capacity separate from allocator capacity and reject maxima that exceed a fixed lane limit."
patterns-established:
  - "Particle boundaries validate all floats and count relationships before any world mutation is possible."
  - "Every production particle state category selects one explicit allocation, clear, permutation, and remap disposition."
requirements-completed: [PART-01, PART-02, PART-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T04:33:12Z
duration: 30 min
completed: 2026-07-15
---

# Phase 9 Plan 01: Particle Definitions and Lane Inventory Summary

**Checked MKS particle contracts with exact flag/color representation and a source-cited 21-category state-permutation inventory.**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-15T04:03:00Z
- **Completed:** 2026-07-15T04:33:12Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments

- Exposed `ParticleSystemDef`, `ParticleDef`, `ParticleFlags`, `ParticleColor`, and typed construction errors through both `liquidfun::particle` and curated crate-root exports.
- Preserved pinned defaults and exact accepted bits while rejecting non-finite units, non-positive physical parameters, invalid iteration counts, and incompatible maximum/fixed-capacity combinations.
- Classified stable identity, required and lazy lanes, derived contacts/state, lifetimes, group ranges, and deferred pair/triad references with explicit source-cited permutation obligations.

## Task Commits

Each task was committed atomically:

1. **Lock the production definitions and validation boundary** - `a79c90a` (feat)
1. **Encode the complete Phase 9 lane and remap matrix** - `43c3bcd` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/definition.rs` - Public checked particle-system and particle definitions, flags, colors, capacities, and errors.
- `crates/liquidfun/src/particle/storage/lane_inventory.rs` - Closed source-cited inventory of 21 Phase 9 state and remap obligations.
- `crates/liquidfun/tests/particle_definitions.rs` - External-crate contract tests for exports, defaults, exact bits, builders, and rejection paths.
- `crates/liquidfun/src/particle.rs` - Public deep-module boundary with private storage internals.
- `crates/liquidfun/src/lib.rs` - Curated crate-root particle exports.
- `Cargo.toml`, `crates/liquidfun/Cargo.toml`, and `Cargo.lock` - Direct pinned `bitflags` dependency wiring.

## Decisions Made

- Unknown particle bits use `from_bits_retain`, matching the pinned unrestricted `uint32` storage policy rather than silently normalizing forward-compatible state.
- `ParticleDef<UserAssociation>` carries application-owned typed input only; `World` remains free of `Any`, raw pointers, or user-data generics, preserving the Phase 3 association boundary.
- Pairs and triads are inventoried only as permutation-safe references. Their topology generation and all solver use remain explicitly deferred to Phase 10.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added the required direct `bitflags` dependency**

- **Found during:** Task 1 (Lock the production definitions and validation boundary)
- **Issue:** `bitflags` 2.13 existed only transitively in `Cargo.lock`; the publishable `liquidfun` crate could not legally import it for the plan's exact public bitflag contract.
- **Fix:** Added the reviewed version to workspace dependencies and wired it directly into `liquidfun`.
- **Files modified:** `Cargo.toml`, `crates/liquidfun/Cargo.toml`, `Cargo.lock`
- **Verification:** Focused definition tests and the full ordered Rust gate pass.
- **Committed in:** `a79c90a`

***

**Total deviations:** 1 auto-fixed (1 blocking). **Impact:** Dependency declaration only; no behavioral scope expansion.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 09-02 can deepen the existing spike into authoritative production lanes using the checked definition and inventory vocabulary.
- The inventory keeps pair/triad generation and particle solver behavior outside Phase 9 Plan 01, with no dense row or allocator capacity exposed publicly.

## Self-Check: PASSED

- All key created files exist.
- Task commits `a79c90a` and `43c3bcd` are present.
- Focused tests, crate-root wiring check, and the complete ordered Rust gate pass.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
