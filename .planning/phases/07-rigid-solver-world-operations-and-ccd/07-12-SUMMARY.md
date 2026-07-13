---
phase: 07-rigid-solver-world-operations-and-ccd
plan: "12"
subsystem: differential-testing
tags: [rust, cpp, differential-testing, rigid-world, ccd, evidence-governance]

requires:
  - phase: 07-rigid-solver-world-operations-and-ccd
    plan: "11"
    provides: Closed Phase 7 adapters, comparison policy, first-divergence evidence, and setup-preserving minimization
provides:
  - One bounded checked-in request covering all nine retained Phase 6 and Phase 7 rigid witness families
  - Reproducible local D2 compare, replay, and sanitizer evidence plus exact two-run D0 determinism evidence
  - Raw-order compatible continuous-contact evidence without contaminating persistent warm-start impulses
  - Pre-write proof that local D2 results cannot stage, review, promote, or create canonical D1 artifacts
affects: [07-13, phase-7-ledger, differential-evidence, rigid-world-governance]

tech-stack:
  added: []
  patterns: [locked deterministic corpus, transient TOI solve evidence, pair-local semantic contact identity, pre-write authority validation]

key-files:
  created: []
  modified:
    - protocol/fixtures/accepted/rigid-world-request.jsonl
    - crates/liquidfun-differential/src/rigid_fixtures.rs
    - crates/liquidfun-differential/src/rigid_world.rs
    - crates/liquidfun-differential/src/rigid_evidence/base.rs
    - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
    - crates/liquidfun/src/world/body.rs
    - crates/liquidfun/src/world/step.rs
    - crates/liquidfun/src/world/step/continuous.rs
    - tools/reference/src/rigid_world.cpp
    - tools/reference/src/rigid_world_phase7_execute.hpp
    - tools/xtask/src/differential.rs
    - tools/xtask/tests/differential_cli.rs

key-decisions:
  - "Keep exactly one bounded checked-in request in registry order: the two retained Phase 6 families followed by all seven Phase 7 families, bound to the reviewed Phase 7 policy hash."
  - "Report continuous-contact solve evidence through a transient lane while leaving persistent warm-start impulse storage unchanged."
  - "Map private manager occurrence values to pair-local semantic generations and retain raw upstream callback ordering in emitted evidence."
  - "Apply Phase 7 numeric overrides once, while inheriting Phase 6 structural, material, manifold, and exact-field checks without reapplying overridden exact-bit comparisons."
  - "Recompute canonical authority and checkout, adapter, compile-command, policy, request, and evidence-tier identity before every fixture write boundary."

patterns-established:
  - "Successful local debug, replay, and sanitizer comparisons remain D2; D0 requires exactly two byte-identical runs from one build."
  - "Continuous budget exhaustion carries coherent committed transient solve evidence that can be resumed without repeating discrete integration."

requirements-completed: []
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T09:35:00Z

duration: 1h 15m
completed: 2026-07-13
---

# Phase 7 Plan 12: Deterministic Evidence Workflow and D2 Governance Summary

**All nine locked rigid-world witness families now compare and replay through the existing native/oracle path, while local D2 evidence is rejected before every canonical D1 mutation boundary.**

## Performance

- **Duration:** 1h 15m
- **Started:** 2026-07-13T08:20:00Z
- **Completed:** 2026-07-13T09:35:00Z
- **Tasks:** 2
- **Files modified:** 21

## Accomplishments

- Replaced the two-family request with one bounded, deterministic nine-family corpus in closed registry order, retaining the Phase 6 witnesses and adding force/configuration, multi-contact/warm-start, sleep/wake, CCD/sub-step, continuous-budget, query/ray, and origin-shift witnesses.
- Bound the request, fixture lifecycle, compare, replay, D0 determinism, and sanitizer paths to the reviewed Phase 7 policy and trace identities. Local debug, replay, and sanitizer results report D2; exactly two byte-identical same-build runs report D0.
- Reconciled native and pinned upstream behavior for body advance, TOI sub-stepping, continuous-work exhaustion, raw callback order, semantic contact generations, and transient continuous post-solve evidence.
- Prevented inherited Phase 6 exact-bit comparison from rejecting body-motion or contact-impulse values that already passed a registered Phase 7 numeric policy.
- Proved with real-binary tests that D2 stage, review, and promotion attempts fail before candidate, review, accepted-trace, or manifest mutation, including stale adapter and effective compile-command identities.

## Task Commits

Each task was committed atomically after its required verification passed:

1. **Task 1: Wire the locked deterministic Phase 7 corpus into compare and replay** - `5e11d17` (`feat`)
2. **Task 2: Prove D2 cannot mutate canonical D1 evidence state** - `4b27042` (`test`)

## Files Created/Modified

- `protocol/fixtures/accepted/rigid-world-request.jsonl` - One bounded request containing all nine rigid witness families and the reviewed Phase 7 policy hash.
- `crates/liquidfun-differential/src/rigid_fixtures.rs` - Phase 7 policy, trace, profile, and fixture-lifecycle identity routing.
- `tools/xtask/src/differential.rs` - Closed Phase 7 compare, replay, D0, minimization, and evidence command paths.
- `crates/liquidfun-differential/src/rigid_world.rs` and `rigid_world/evidence.rs` - Pair-local semantic contact identity and source-compatible event collection.
- `crates/liquidfun-differential/src/rigid_evidence/base.rs` - Inherited structural comparison without reapplying Phase 7-overridden numeric fields.
- `crates/liquidfun/src/world/body.rs` - Pinned body-advance collapse of current sweep state to the advanced initial state.
- `crates/liquidfun/src/world/step.rs`, `step/continuous.rs`, and `continuous/event.rs` - Transient continuous solve evidence, coherent budget progress, and resume-time contact update/hooks.
- `tools/reference/src/rigid_world.cpp` and `rigid_world_phase7_execute.hpp` - Multi-contact declaration validation and bounded upstream sub-step execution for continuous-work evidence.
- `crates/liquidfun-differential/tests/rigid_fixture_workflow.rs` and `tools/xtask/tests/differential_cli.rs` - Locked-corpus and pre-write D2 authority coverage.
- `crates/liquidfun/tests/rigid_ccd.rs` - Symmetric multi-contact advance and transient TOI solve regressions.

## Decisions Made

- Kept corpus generation out of the workflow. The only accepted input is the reviewed checked-in JSONL record, and its family order and policy hash have a direct regression test.
- Kept TOI impulses transient. Continuous post-solve evidence is observable for comparison, but the persistent contact-manager impulse lanes remain the discrete warm-start authority.
- Resumed the upstream-equivalent contact discovery, update, and hook stages after a pending continuous step while continuing to skip already committed discrete integration and solving.
- Preserved raw upstream callback order instead of normalizing the C++ oracle. The native evidence adapter now interleaves manager transitions and matching pre-solve callbacks in the same semantic order.
- Used pair-local contact generations for protocol identity so unrelated manager allocation order cannot change semantic occurrence values.

## Test Evidence

- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` matched all 9 families as D2-supported native and oracle evidence.
- `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot` replayed all 9 families with the same D2 classification.
- `cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2` produced D0 after exactly two byte-identical native and oracle-debug runs.
- The fail-fast `oracle-asan-ubsan` configure, build, and compare path matched all 9 families as D2-supported evidence with no sanitizer finding. Local CMake 3.27.9 and Apple Clang 21 differed from canonical pins, so no D1 or platform-wide claim was made.
- The rigid fixture workflow passed 8/8; the xtask differential CLI suite passed 28/28; the differential rigid-world suite passed 21/21; and focused rigid CCD regressions passed 7/7.
- Same-signature minimization remains covered by the typed Phase 7 minimizer regression in the all-feature suite. The locked corpus itself matched, so there was no honest first divergence to minimize during the passing compare.
- GSD summary verification, both commit checks, all 3 artifact checks, and Phase 7 lifecycle validation passed. The textual key-link heuristic reported 0/2 because the closed xtask and fixture modules do not name each other by source path; the 8/8 real lifecycle and 28/28 CLI suites provide the executable authority links.
- Before each retained task commit, the exact ordered Rust gate passed with exit code 0:
  1. `cargo fmt --all`
  2. `cargo clippy --all-targets --all-features -- -D warnings`
  3. `cargo build --all-targets --all-features`
  4. `cargo test --all-features`

## Simplification Review

- Reused the existing request, runner, comparison, replay, minimization, fixture, and xtask paths. No alternate harness, generated corpus, dependency, runtime FFI path, or canonical write route was introduced.
- Added one transient continuous-solve lane rather than storing TOI impulses in the discrete warm-start state or inventing a second public event system.
- Kept inherited comparison behind one boolean override seam so Phase 7 reuses the complete Phase 6 structural order without duplicating its comparator.
- Limited C++ changes to the private oracle adapter and upstream sub-step controls needed to express the same bounded semantic work budget.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Corrected pinned body advance and continuous-step behavior exposed by the complete corpus**

- **Found during:** Task 1 cross-engine comparison
- **Issue:** The native advance path retained pre-advance current sweep state, and resume skipped upstream contact update/hooks; CCD and budget witnesses therefore diverged despite valid protocol inputs.
- **Fix:** Collapsed current sweep state after advance, restored resume-time contact update/hooks without repeating discrete solve, and added focused regressions.
- **Verification:** Focused CCD tests, all-feature tests, debug compare/replay/D0, and sanitizer compare passed.
- **Committed in:** `5e11d17`

**2. [Rule 3 - Blocking] Preserved continuous post-solve evidence without corrupting warm-start state**

- **Found during:** Task 1 raw callback comparison
- **Issue:** Accepted TOI events needed pre/post-solve evidence, but committing their impulses to ordinary contact storage would change upstream warm-start semantics.
- **Fix:** Carried staged solves through successful and budget-exhausted continuous progress as transient evidence only.
- **Verification:** The regression proves positive transient TOI impulses alongside exact-zero persistent impulse lanes; all differential lanes matched.
- **Committed in:** `5e11d17`

**3. [Rule 3 - Blocking] Removed the inherited exact-bit double check after Phase 7 numeric acceptance**

- **Found during:** Task 1 multi-contact comparison
- **Issue:** A Phase 7 motion value passed its registered absolute/relative policy, then failed when the inherited Phase 6 comparator reapplied exact bits.
- **Fix:** Retained inherited structural and non-overridden checks while skipping only Phase 7-overridden body-motion and contact-impulse numeric fields.
- **Verification:** A one-ULP comparator regression and the nine-family debug and sanitizer comparisons passed.
- **Committed in:** `5e11d17`

***

**Total deviations:** 3 auto-fixed blocking correctness issues.
**Impact on plan:** Each fix was necessary for honest native/oracle evidence from the planned corpus. No public foreign-language dependency, unbounded work, D1 authority, or parity claim was added.

## Issues Encountered

- Running two full test commands concurrently caused duplicate integration processes to contend for test resources. The duplicate was terminated, all processes were allowed to exit, and fresh single PTY-backed all-feature runs were polled to authoritative exit code 0 before either commit.
- The local sanitizer toolchain differs from canonical compiler and CMake pins. Provenance reported those differences and correctly kept the passing result at D2.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 07-13 can document the implemented contract and update the compatibility ledger from the executed D0/D2 evidence without claiming D1, D3, multi-platform, or complete rigid parity.
- The checked-in corpus, exact policy identity, raw first-divergence machinery, replay, minimizer regression, deterministic two-run lane, and sanitizer path are ready for final Phase 7 documentation checks.
- No blocker remains.

## Self-Check: PASSED

- Task commits `5e11d17` and `4b27042` exist, and all listed implementation and authority files exist.
- Debug compare, replay, exact two-run D0, sanitizer compare, focused authority suites, focused CCD and rigid-world suites, and both complete ordered Rust gates passed.
- Diff review found no generated corpus, canonical D1 mutation, runtime C++ dependency, persistent TOI impulse contamination, pointer evidence, unsafe code, new dependency, absolute-path artifact, or unsupported parity claim.
- The pre-existing `.planning/config.json` chain-state change remains unstaged and uncommitted.

***

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Completed: 2026-07-13*
