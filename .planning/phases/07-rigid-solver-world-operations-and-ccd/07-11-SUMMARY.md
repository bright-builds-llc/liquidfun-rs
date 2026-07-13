---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "11"
subsystem: differential-testing
tags: [rust, cpp, differential-testing, rigid-world, ccd, minimization]

requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "10"
    provides: Closed Phase 7 rigid-world protocol, witness registry, and per-observable policy
  - phase: 07-rigid-solver-world-operations-and-ccd
    plans: ["05", "07", "09"]
    provides: Sleeping, origin-shift, and resumable CCD production behavior
provides:
  - Symmetric native Rust and pinned C++ execution for the closed Phase 7 rigid-world action surface
  - Declaration-first Phase 7 comparison with exact, numeric, multiset, and set semantics
  - Stable first-divergence evidence carrying action, stage, entity, values, policy, and completion state
  - Rigid minimization that preserves the divergent operation and its exact setup prefix
affects: [07-12, phase-7-fixtures, differential-evidence, rigid-world-governance]

tech-stack:
  added: []
  patterns: [deep evidence modules, declaration-first comparison, semantic ray tie sets, setup-preserving reduction]

key-files:
  created:
    - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase7/context.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase7/observation.rs
    - crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs
    - crates/liquidfun-differential/tests/support/phase7_comparator.rs
    - tools/reference/src/rigid_world_phase7_execute.hpp
  modified:
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_evidence.rs
    - crates/liquidfun-differential/src/comparator.rs
    - crates/liquidfun-differential/src/minimizer.rs
    - tools/reference/src/rigid_world.cpp
    - tools/reference/src/rigid_world.hpp

key-decisions:
  - "Keep native Rust and pinned C++ dispatch source-ordered behind the same validated Phase 7 action/result contract; adapter output remains semantic and pointer-free."
  - "Compare query occurrences as multiplicity-preserving multisets and equal-minimum ray identities as sets only in the evidence layer; nonminimum ray hits retain declared order."
  - "Bind the failure signature to action, stage, entity, field, exact values and bits, policy identity, and completion context so replay and reduction cannot silently change classification."
  - "Protect the divergent action and its complete same-timeline setup prefix during reduction, including callback directives, configured budget, and transported float bits."

patterns-established:
  - "Phase 7 evidence uses a small root facade with deep declaration, inherited Phase 6, Phase 7 context, observation, and ray modules."
  - "Large diagnostic variants are boxed at enum boundaries while reports remain owned, serializable, and bounded."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T08:18:00Z

duration: 1h 4m
completed: 2026-07-13
---

# Phase 7 Plan 11: Differential Adapters and Evidence Workflows Summary

**Native Rust and the pinned C++ oracle now execute the same closed Phase 7 rigid-world operations, while declaration-first evidence preserves exact first-divergence identity through comparison and minimization.**

## Performance

- **Duration:** 1h 4m
- **Started:** 2026-07-13T07:14:00Z
- **Completed:** 2026-07-13T08:18:00Z
- **Tasks:** 2
- **Files modified:** 31

## Accomplishments

- Extended both adapters across granular body controls, wake-policy force and impulse application, checked configured stepping, query and ray directives, origin shifting, and semantic completion/partial-progress observations.
- Kept C++ execution within the pinned LiquidFun oracle and split decoding, validation, execution, and trace encoding into reviewable private headers without adding a runtime or published-crate dependency.
- Added fail-closed Phase 7 evidence dispatch. Ordered status and state remain ordered, queries compare as multiplicity-preserving multisets, equal-minimum ray identities compare as sets, and numeric ray fields use their named policy.
- Expanded mismatch evidence with action, stage, entity, exact bit and decimal values, field policy, profile identity, and surrounding completion state.
- Updated minimization to retain the exact divergent operation and its complete setup prefix, preventing reduction from changing directives, work budget, or transported float bits.

## Task Commits

Each task was committed atomically after its required verification passed:

1. **Task 1: Implement symmetric Rust and C++ Phase 7 adapters** - `404d2ea` (`feat`)
2. **Task 2: Compare Phase 7 evidence without erasing first divergence** - `f00d1d8` (`feat`)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase7.rs` - Native mapping from closed Phase 7 actions to checked world APIs and semantic observations.
- `tools/reference/src/rigid_world_phase7_execute.hpp` - Pinned C++ execution of the same Phase 7 action surface.
- `tools/reference/src/rigid_world_action_decode.hpp` and `rigid_world_validate.hpp` - Bounded duplicate-aware decode and fail-closed validation helpers.
- `crates/liquidfun-differential/src/rigid_evidence.rs` - Public comparison facade and complete stable mismatch evidence.
- `crates/liquidfun-differential/src/rigid_evidence/base.rs` - Inherited Phase 6 comparison preserved in its original first-divergence order.
- `crates/liquidfun-differential/src/rigid_evidence/declaration.rs` - Per-engine request/result declaration validation before physics comparison.
- `crates/liquidfun-differential/src/rigid_evidence/phase7.rs` - Closed Phase 7 action, checkpoint, observation, status, and field-policy dispatch.
- `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs` - Equal-minimum set identity and separately policy-checked hit numerics.
- `crates/liquidfun-differential/src/comparator.rs` - Reusable multiset and set comparison helpers.
- `crates/liquidfun-differential/src/minimizer.rs` - Setup-prefix preservation keyed by the exact target action.
- `crates/liquidfun-differential/tests/support/phase7_comparator.rs` - Focused multiplicity, tie-set, diagnostic-context, and reduction regressions.

## Decisions Made

- Preserved source order in both production adapters. Canonicalization is restricted to the two evidence paths explicitly classified by the Phase 7 policy: query multisets and equal-minimum ray identity sets.
- Used exact transported `FloatBits` to classify equal-minimum ray ties. The comparator converts to `f32` only to choose the numerical minimum, then groups exact tie payloads without direct float equality.
- Boxed large declaration and physics mismatch enum variants. This keeps the public owned reports intact while avoiding oversized `Result` and outcome values across the fail-closed comparator call graph.
- Kept the failure digest stricter than its human-readable report: values, bits, stage, entity, profile, and completion context all participate, so a reducer cannot preserve only the semantic path while changing the actual failure.

## Test Evidence

- The focused Phase 7 comparator filter passed 4/4 tests; the minimization filter passed 1/1.
- The complete differential rigid-world target passed 20/20 tests, including native and oracle Phase 7 execution, declaration-first failures, query multiplicity, ray tie sets, full diagnostic context, and setup-preserving reduction.
- `cargo xtask upstream configure --preset oracle-debug` and `cargo xtask upstream build --preset oracle-debug` passed against the pinned checkout; the `liquidfun-reference-protocol` CTest passed.
- Before both retained task commits, the exact ordered Rust gate passed with exit code 0:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`
- GSD artifact verification passed 3/3. The textual key-link heuristic reported 0/1 because Rust and C++ adapters deliberately do not reference each other by source path; the shared typed protocol, matched dispatch tests, and 20-test differential target provide the executable link.

## Simplification Review

- Split the former 1,000-line rigid evidence module into a public facade and deep modules for inherited comparison, declaration validation, Phase 7 context, observations, and ray semantics. This keeps one cohesive API without fragmenting the crate or duplicating policy lookup.
- Split the C++ adapter into named private decode, validate, execute, and trace headers rather than embedding more substantial behavior in the adapter loop.
- Reused the protocol-owned action, result, completion, policy, and exact-bit types. No dependency, unsafe code, alternate physics kernel, public C++ boundary, or generic normalization framework was introduced.
- Preserved one first-divergence builder and one declaration gate instead of maintaining parallel Phase 6 and Phase 7 report formats.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Split growing Rust and C++ adapter/evidence files along domain boundaries**

- **Found during:** Tasks 1 and 2 implementation and strict Clippy verification
- **Issue:** Extending the existing monolithic adapter and evidence files made the closed dispatch hard to review and exceeded repository code-shape limits.
- **Fix:** Extracted private Rust deep modules plus private C++ decode, validation, execution, and trace headers while retaining the planned public entrypoints.
- **Verification:** Focused and full targets, C++ build/CTest, Clippy, build, and tests all passed.
- **Committed in:** `404d2ea`, `f00d1d8`

**2. [Rule 3 - Blocking] Tightened large report and crate-wide warning-denied boundaries**

- **Found during:** Task 2 strict Clippy gate
- **Issue:** Complete first-divergence reports made outcome/error variants oversized, and recompiling the expanded differential crate exposed stale wildcard, pass-by-value, and long-dispatch lint failures in adjacent adapter/test helpers.
- **Fix:** Boxed large enum variants, replaced wildcard imports, borrowed replay byte buffers, made a test helper associated, consumed the closed terminal kind explicitly, and added narrow reasoned allowances where a closed dispatch is clearer than artificial fragmentation.
- **Verification:** Focused package Clippy and the exact workspace Clippy command both passed with `-D warnings`; all behavior tests remained green.
- **Committed in:** `f00d1d8`

***

**Total deviations:** 2 auto-fixed blocking issues.
**Impact on plan:** The changes reduce module and value-size risk while preserving the exact planned adapter, evidence, and minimization behavior. No physics, protocol, or promotion scope was widened.

### Process adjustment: RED evidence was not committed

- Repository policy requires the complete ordered Rust gate before every commit. The intentionally failing comparator and minimization RED filters exited 101 and were retained as execution evidence, but only verified GREEN task states were committed.

## Issues Encountered

- Focused Clippy initially reported 32 warnings-as-errors after the expanded evidence types and extracted modules were compiled together. Boxing the large variants eliminated the cascading large-error warnings; explicit imports and small boundary cleanups resolved the remaining six without weakening warning policy.
- The GSD key-link checker only recognizes literal target-path references and therefore cannot establish semantic symmetry between independent Rust and C++ adapters. The shared protocol types and cross-engine execution tests are the authoritative evidence.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-12 can persist and replay Phase 7 evidence with complete action/stage/entity/value/policy/completion diagnostics and a reducer that protects the exact divergent setup.
- Native and oracle outputs remain bounded, semantic, reset-verified, and governed by existing D0-D3 authority; this plan does not promote local D2 evidence.
- No blocker remains.

## Self-Check: PASSED

- Task commits `404d2ea` and `f00d1d8` exist, all listed key files exist, and the summary represents the 31-file cumulative implementation/test diff.
- Focused comparator/minimization tests, the full rigid target, C++ configure/build/CTest, and the exact ordered workspace gate all passed.
- Diff review found no pointer or storage-layout evidence, runtime C++ dependency, broad canonicalization, failure-classification weakening, unbounded work, unsafe code, new dependency, or unrelated production physics change.
- The pre-existing `.planning/config.json` chain-state change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*
