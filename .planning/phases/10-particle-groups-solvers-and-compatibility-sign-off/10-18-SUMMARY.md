---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "18"
subsystem: particle-material-solvers
tags: [rust, particles, materials, contacts, source-parity]

requires:
  - phase: 10-02
    provides: Checked exact material coefficients on ParticleSystemDef
  - phase: 10-03
    provides: Complete source-ordered particle topology records
  - phase: 10-05
    provides: Authoritative particle/group membership and color lanes
  - phase: 10-06
    provides: Storage-owned tensile and depth scratch lanes
  - phase: 10-15
    provides: Scheduled solid-depth computation and rigid group cache timing
provides:
  - Exact distinct S07-S12 material kernels
  - Source-ordered body/particle contact, membership, depth, and color effects
  - Focused control, activation, zero-coefficient, mixed, and deterministic witnesses for every material pass
affects: [10-20, 10-22, 10-23, particle-solvers, particle-world-coupling, compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - One crate-private safe numerical kernel per materially distinct LiquidFun pass
    - Complete velocity and color candidates committed through ParticleStorage authority
    - Exact source-statement f32 grouping and stable contact traversal

key-files:
  created:
    - crates/liquidfun/src/particle/solver/material.rs
    - crates/liquidfun/src/particle/solver/material/tests.rs
  modified:
    - crates/liquidfun/src/particle/solver.rs
    - crates/liquidfun/src/particle/storage.rs

key-decisions:
  - "Keep S07-S12 separate even where passes share contact traversal so each public flag retains its pinned formula, gate, and independently testable effect."
  - "Use the authoritative storage contact, membership, weight, tensile, depth, velocity, and color lanes without introducing a second solver-state owner."
  - "Preserve the pinned color mixer through explicit signed arithmetic, uint8 wrapping conversion, and source-ordered contact mutation."

patterns-established:
  - "Material-pass evidence: every kernel has no-effect control, default activation, zero-coefficient suppression, mixed-input interaction, and bit-identical repeat coverage."
  - "Paired effects: repulsive, powder, tensile, and solid impulses update particle endpoints equally and oppositely in incumbent contact order."

requirements-completed: [PART-12, PART-13, TEST-01]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-20T15:42:37Z

duration: 58min
completed: 2026-07-20
---

# Phase 10 Plan 18: Exact Material Solver Passes Summary

**Six crate-private, source-ordered material kernels now reproduce pinned LiquidFun viscosity, repulsion, powder, tensile, solid-ejection, and color-mixing behavior through authoritative particle storage.**

## Performance

- **Duration:** 58 min
- **Started:** 2026-07-20T14:44:41Z
- **Completed:** 2026-07-20T15:42:37Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments

- Implemented S07 viscous transfer across body contacts first and particle contacts second with exact configured strength and particle inverse mass.
- Implemented S08 repulsion only across distinct group memberships and S09 powder scattering only above the pinned `1 - particleStride` weight boundary.
- Implemented S10 tensile as two ordered traversals over storage contacts with aligned weighted-normal scratch, exact pressure/normal strengths, and maximum variation cap.
- Implemented S11 cross-group solid ejection from the scheduled authoritative depth lane.
- Implemented S12 color mixing with the both-particles flag gate, integerized default strength, source-ordered repeated mixing, and exact four-channel wrapping behavior.
- Added seven focused tests covering every pass's control, activation, zero coefficient, mixed interaction, deterministic repeat, default coefficients, paired conservation, group gates, and exact manifest placement.

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement exact material interactions** - `27bdc5b` (feat)

## Files Created/Modified

- `crates/liquidfun/src/particle/solver/material.rs` - Exact crate-private S07-S12 kernels and small numerical helpers.
- `crates/liquidfun/src/particle/solver/material/tests.rs` - Co-located focused control, activation, interaction, and deterministic witnesses.
- `crates/liquidfun/src/particle/solver.rs` - Private material-module registration.
- `crates/liquidfun/src/particle/storage.rs` - Minimal validated whole-color candidate replacement under existing lane authority.

## Decisions Made

- Kept the six passes independent instead of fusing their shared contact walks. The distinct formulas and gates remain directly comparable to the pinned source and independently dispatchable.
- Used `ParticleStorage` views for all contact, membership, weight, tensile, depth, velocity, and color inputs. Numerical kernels build owned output candidates and never duplicate authoritative derived state.
- Mirrored `b2ParticleColor::MixColors` with signed widened multiplication, arithmetic shift, explicit low-byte conversion, and wrapping channel updates. This preserves both negative channel deltas and extreme-channel behavior without unsafe code.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added one validated storage-owned color commit seam**

- **Found during:** Task 1 implementation
- **Issue:** The optional color lane was intentionally private and exposed only an immutable view, so S12 could not commit an aligned color candidate without widening general mutable storage access.
- **Fix:** Added `replace_solver_colors`, which rejects mismatched or absent lanes before replacing the complete candidate.
- **Files modified:** `crates/liquidfun/src/particle/storage.rs`
- **Verification:** Both-particle gating, zero-strength control, exact default channel results, source-order interaction, and the complete ordered Rust gate passed.
- **Committed in:** `27bdc5b`

**2. [Rule 3 - Code Shape] Split the co-located witness module from production kernels**

- **Found during:** Task 1 simplification review
- **Issue:** Keeping all six implementations and their complete witness matrix in one physical file would exceed the repository's approximate 628-line refactor trigger.
- **Fix:** Kept the private `#[cfg(test)]` module co-located under `material` while moving its body to `material/tests.rs`; production remains 276 lines and the test module 613 lines.
- **Files modified:** `crates/liquidfun/src/particle/solver/material.rs`, `crates/liquidfun/src/particle/solver/material/tests.rs`
- **Verification:** No function exceeds the 161-line trigger, focused tests discover the private module, and all-target clippy passes with warnings denied.
- **Committed in:** `27bdc5b`

**Total deviations:** 2 auto-fixed (1 blocking, 1 code-shape)
**Impact on plan:** Both changes preserve the requested private surface and storage authority; no dependency, public API, unsafe/FFI, external I/O, or solver-order change was introduced.

## Issues Encountered

- Initial focused expectations used an inverse time step that made the default critical velocity twenty rather than one; the witness input was corrected to derive the intended unit critical velocity from the pinned radius formula.
- Clippy required explicit localized documentation of the two source-mandated color conversions. The allowances are limited to the float-to-int strength conversion and signed-to-byte wrapping delta.
- macOS executable policy scanning made the prescribed broad filtered and full all-target test runs slow, but both exact commands completed without interruption, narrowing, or skipped binaries.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-22 can adapt candidate world bodies to `MaterialBodyCoupling` and dispatch the independently witnessed S07-S12 kernels in the closed manifest order.
- Plan 10-23 can consume the complete per-pass material control/activation matrix for native coverage closure.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all four created or modified task files and this summary exist.
- Confirmed task commit `27bdc5b` exists.
- Confirmed no stub patterns, unsafe/FFI code, new dependencies, network endpoints, auth paths, file-access surfaces, or schema changes were introduced.
- Confirmed the exact focused filter, 7 selected material witnesses, and complete ordered Rust gate all pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-20*
