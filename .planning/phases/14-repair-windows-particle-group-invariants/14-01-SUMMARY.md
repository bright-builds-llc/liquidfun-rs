---
phase: 14-repair-windows-particle-group-invariants
plan: "01"
subsystem: testing
tags: [rust, windows, particles, allocation, diagnostics]
requires:
  - phase: 13.1
    provides: Refactored particle storage and preserved audited public replay
provides:
  - Exact original/current Windows failure lineage and retained logs
  - Observed operation-14 join scratch allocation failure with valid source and candidate invariants
  - Successful-append versus genuine topology-rejection repair contract
affects: [14-02, 14-03, 14-04]
tech-stack:
  added: []
  patterns: [bounded test-only stage observation, independent public regression, separately gated probe cleanup]
key-files:
  created: [.planning/phases/14-repair-windows-particle-group-invariants/14-DIAGNOSIS.md]
  modified: []
key-decisions:
  - Preserve the existing clone-plan-commit boundary; no authoritative invariant violation was observed.
  - Repair solver_state zeroed_lane scratch reservation sizing rather than suppressing the mapper panic.
  - Treat the unchanged Windows expected-panic regression pass as rollback evidence, not successful append.
requirements-completed: []
requirements-addressed: [PART-03, PART-09, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T02:36:27Z
duration: 39min
completed: 2026-09-14
---

# Phase 14 Plan 01: Windows allocation diagnosis Summary

**Exact Windows tracing proves operation 14 requests an 8.59 GB scratch lane for ten particles, maps allocation failure to InvalidLaneBundle, and panics while source and observed candidate invariants remain valid.**

## Performance

- Duration: 39 minutes from the first retained diagnostic attempt, including host gate recovery.
- Started: 2026-09-14T01:57:20Z (first retained attempt).
- Completed: 2026-09-14T02:36:27Z.
- Tasks: 2/2.
- Files: nine paths in the temporary probe transaction; final implementation/workflow files restored exactly; one lasting diagnosis file.

## Accomplishments

- Preserved audited source `4b6b15ba562cedcad3f37e379d01be9e9cb2e949`, run 30070539790/job 89410277623, and current source `5aadb6c105b98ae09443f74e44c57f8ce7eae19d`, run 34777296141/job 103777632638, with complete distinct seeds/controls and log digests.
- Ran a separate cfg(test) lib replay matching every original initialization and mutation. In [diagnostic run 34799215304](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34799215304), Windows operation 14 reaches valid ten-row topology then fails an 8,589,934,588-byte allocation during optional join preparation. The raw `TryReserveError::AllocError` becomes `Storage(InvalidLaneBundle)` and the unchanged mapper panic.
- Distinguished the first failed resource preparation from a nonexistent demonstrated lane/identity/float corruption. The valid source has nine rows; the unpublished candidate has ten; all observed invariants are `Ok(())`.
- Preserved all diagnostic evidence, removed every temporary source/helper/test/CI step, verified original source byte equality and separately gated/published cleanup.

## Task Commits

1. Task 1 — preserve exact Windows failure generations: `a2759db3690d91868afcc74d862146ccc8b1507e` (`docs(14-01): preserve exact Windows particle-group failures`).
1. Task 2 — publish diagnostic observation: `f4ef73930a62aa884b820ce91ffe35a4608b2b8b` (`test(14-01): trace exact Windows group candidate and allocation stages`).
1. Task 2 — retain conclusions and remove probes: `49e382f17c42b25b9483e9c6134994dd2bc1c201` (`test(14-01): retain allocation diagnosis and remove temporary probes`).

Both diagnostic publication and cleanup used ordinary non-force pushes to verified `bright-builds-llc/liquidfun-rs:main` under AGENTS.md standing authorization. No workflow dispatch was needed. The orchestrator owns STATE/ROADMAP/requirements/todo updates and the final metadata commit.

## Verification

All three task commits passed the ordered local gate using immutable Linux ARM64 container image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8` with explicit Rust 1.97.0, offline registry and isolated Cargo target: fmt → Clippy with denied warnings → all-target/all-feature build → all-feature tests. Managed checker and diff checks passed with zero findings. Scoped diffs were reviewed. The root independently reviewed the frozen diagnostic commit without a high-signal probe defect.

| Gate | Passing test count | Retained gate.log SHA-256 |
| --- | --- | --- |
| Task 1 | 987 | `5aca6107a4d29d2ed1a6d5fdd4830752dff5894e6a253c2f476308ef39965689` |
| Probe publication | 988 | `392ad7b7793f7e76d5e9ac459ef97e05b0d5d3fb111210f8271cfef36488661e` |
| Probe cleanup | 987 | `cad4098f076402fa198e4c9f6492228f16985487a301c22b1d78da604dcb71e0` |

Task 1's integration inventory lists three tests; the exact original regression selects one and passes locally. The private diagnostic inventory lists its full name exactly once and selects one test; all 14 operations complete locally. After cleanup, the lib inventory returns to 409 tests without the removed name, and the original public regression again passes with one selected/two filtered. No zero-test filtered invocation is counted as evidence.

At the diagnostic SHA, Linux job 103838288742 and macOS job 103838288721 pass their diagnostic, separate public-regression and full default-feature steps. Windows job 103838288703 preserves a failing diagnostic (0 passed/1 failed/409 filtered), a passing separate original regression (existing expected final panic plus rollback assertion), and a failed full lib suite (409 passed/1 failed). That lib failure prevents later integration targets in the full-suite step. None of these diagnostic results is called repaired Windows acceptance or canonical parity.

## Files and Evidence

- `14-DIAGNOSIS.md` contains exact source/run/job/toolchain/runner identities, full original and newer inputs, replay mapping, raw allocation error, storage values, backtrace interpretation, digests and repair contract.
- `target/phase14-diagnostics/5aadb6c105b98ae09443f74e44c57f8ce7eae19d/attempt-20260914T015720Z-historical/` preserves historical/current logs and metadata.
- `target/phase14-diagnostics/8b4eb5656dc69306225c900e6c44cb56b108f749/attempt-20260914-task1/` preserves local inventory/public regression/gate.
- `target/phase14-diagnostics/a2759db3690d91868afcc74d862146ccc8b1507e/attempt-20260914-probe01/` preserves exact probe patch, original source copies, local trace, failed/corrected compile logs, gate, and all remote diagnostic job logs/metadata.
- `target/phase14-diagnostics/f4ef73930a62aa884b820ce91ffe35a4608b2b8b/attempt-20260914-cleanup01/` preserves restored-path inventory, public regression and cleanup gate.

The seven original source/workflow paths are byte-identical to their pre-probe versions, and the temporary replay is absent. The root's independent CI Markdown plugin fix is preserved. No production debug API or new security-relevant runtime surface remains.

## Decisions Made

The allocation request uses the declared maximum `i32::MAX` where the bounded scratch candidate needs ten elements. Plan 02 must explicitly own this demonstrated solver-state sizing seam and its tests. D-03 does not justify widening the transaction: existing candidate isolation already preserves the live source.

The repaired original append must return its existing target, grow nine particles to ten and report `Applied { created: 0, lifecycle: 0 }`; any error is insufficient. Existing 64 widely spaced ELASTIC-position topology rejection remains exactly `InvalidParticleGroupTopology` with no effect. Resource-sizing and successful-operation assertions reject mapper-only suppression.

## Deviations from Plan

1. **[Rule 3 — Blocking] Local macOS startup gate recovery.** The root diagnosed pre-main executable delays and supplied a pinned local Linux container gate. No security settings were changed; real macOS and Windows CI remained required and were obtained for the diagnostic SHA. The root separately committed planning and CI formatter environment recovery before task execution.
1. **[Rule 2 — Missing critical observation] Narrow allocator seam.** The orchestrator amended Plan 01 before editing to include `particle/storage/solver_state.rs`, allowing observation of the raw reservation error and exact requested layout. The seam was test-only and removed with the probe.
1. **[Rule 1 — Bug] Diagnostic Clippy correction.** An initial no-effect underscore binding in the allocator observer failed Clippy. The corrected observer borrows the original reservation result before unchanged mapping; the failed and corrected compile logs remain retained. No production result, order or allocation size changed.

## Issues Encountered

The raw evidence disproves the plan's suspected invalid-state premise: there is a resource preparation failure, with no observed invalid storage predicate. The diagnosis records that distinction instead of inventing corrupted state. Linux quality also exposed pre-existing private differential test lints in `tests/round_trip/evidence.rs` at lines 61 and 113; the root owns separate recovery, not this plan.

## Next Phase Readiness

Plan 02 can proceed after its ownership/action amendment to the observed scratch allocation seam. The full historical/public regression correction, complete private transaction snapshots, typed outcomes and final supported-platform acceptance remain future plan work. Phase 14 itself is not complete. STATE and ROADMAP are intentionally left to the orchestrator to avoid shared-file conflicts.

## Self-Check: PASSED

The summary and diagnosis exist; all three exact task commits resolve; all seven original source/workflow files match retained pre-probe content; the temporary replay is absent. No implementation stubs remain. The orchestrator retains ownership of the metadata commit and shared planning updates.
