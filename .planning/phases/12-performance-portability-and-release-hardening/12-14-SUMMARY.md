---
phase: 12-performance-portability-and-release-hardening
plan: "14"
subsystem: release-evidence
tags: [compatibility, corpus, benchmarking, evidence, documentation]

requires:
  - phase: 12-performance-portability-and-release-hardening
    provides: controlled performance evidence workflow and immutable workflow identities
  - phase: 11-examples-headless-tooling-and-testbed
    provides: terminal semantic corpus authority
provides:
  - exact set-equality release dispositions for every compatibility identity
  - fail-closed compatibility, corpus, tolerance, manifest, and platform authority joins
  - deterministic public zero-gap compatibility and corpus reports
  - reproducible paired benchmarking methodology with immutable workload-only claim rules
affects: [release-audit, compatibility-claims, performance-claims, public-documentation]

tech-stack:
  added: []
  patterns:
    - ledger-derived identity cardinality with one explicit terminal release disposition per row
    - machine validation before generated public status
    - immutable workload-only performance claims with unprofiled timing authority

key-files:
  created:
    - BENCHMARKING.md
  modified:
    - COMPATIBILITY.md
    - UPSTREAM-CORPUS.md
    - reference/compatibility.json
    - tools/xtask/src/inventory.rs
    - tools/xtask/src/inventory/report.rs
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs

key-decisions:
  - "Keep eight evidence dimensions independent and add a separate exact-cardinality release-disposition join so terminal closure never rewrites evidence history."
  - "Treat D2 platform support, coverage, and performance as orthogonal authorities that can never promote D1 parity."
  - "Publish no performance number while the reviewed-report manifest is empty; future claims must bind one immutable report to one workload, size, host, build identity, and interval."

patterns-established:
  - "Release closure: derive the row set and count from the compatibility ledger, require ordered set equality, then validate every declared outcome against its authority."
  - "Benchmark claims: same-host paired measurements remain local and non-claiming until an immutable reviewed report is manifest-listed."

requirements-completed: [COMP-10, PERF-06, PLAT-06, DOCS-04, DOCS-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-24T02:23:36Z

duration: 29min
completed: 2026-07-23
---

# Phase 12 Plan 14: Compatibility and Benchmark Reporting Summary

**Ledger-derived release closure for all compatibility identities plus reproducible paired benchmarking rules that forbid unreviewed or generalized performance claims**

## Performance

- **Duration:** 29 min
- **Started:** 2026-07-24T01:55:05Z
- **Completed:** 2026-07-24T02:23:36Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments

- Added one explicit, ordered release disposition for every compatibility identity and validated exact set equality, corpus terminality, tolerance identities, machine-manifest schemas, D1/D2 boundaries, and single-commit parity evidence.
- Generated a compatibility report whose 181-row identity count is derived from the ledger and whose zero unexplained-gap status is reachable only after validation.
- Documented all 14 paired workloads, 32 sealed cases, five-run Student 95% analysis, calibrated practical floor, diagnostic profiling, optimization admission, reproduction workflow, and immutable workload-only claim contract.

## TDD Evidence

- **RED:** The omitted-release-identity test failed because the strict compatibility schema did not yet recognize release dispositions.
- **GREEN:** The release model and validator made omitted/duplicate joins, unexplained outcomes, nonterminal corpus items, mixed commits, empty rationales, coverage promotion, and D2 promotion fail closed while the complete fixture passed.
- **REFACTOR:** Report rendering consumes the validated readiness projection, and corpus source paths retain all terminal IDs rather than collapsing multiple source symbols.
- The plan prohibited a failing RED commit, so RED remained uncommitted.

## Task Commits

Each task was committed atomically:

1. **Task 1: Enforce zero-gap compatibility and truthful generated status** - `ad6be0f` (feat)
1. **Task 2: Document reproducible paired benchmarking and bounded claims** - `231e886` (docs)

## Files Created/Modified

- `BENCHMARKING.md` - Paired methodology, calibration, profiling, interpretation, reproduction, and claim boundaries.
- `COMPATIBILITY.md` - Deterministic public release-readiness projection and per-identity terminal outcomes.
- `UPSTREAM-CORPUS.md` - Refreshed deterministic corpus report required by the terminal closure check.
- `reference/compatibility.json` - Ordered one-to-one release disposition authority.
- `tools/xtask/src/inventory.rs` - Corpus loading and validated release-readiness wiring.
- `tools/xtask/src/inventory/report.rs` - Derived closure counts, authority boundary, and outcome report generation.
- `tools/xtask/src/inventory/validation.rs` - Set-equality joins and fail-closed release authority validation.
- `tools/xtask/tests/inventory_cli.rs` - Positive determinism and required negative closure cases.

## Decisions Made

- Preserved the existing evidence records and counts. Release outcomes are a separate reviewed projection, so a terminal corpus or architecture decision cannot masquerade as implementation or parity evidence.
- Used reviewed architecture and build documentation for native-Rust structural differences; no row was promoted from local command success.
- Kept the benchmark example explicitly hypothetical because `reference/performance/manifest.toml` contains no reviewed reports.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Refreshed the stale generated corpus report**

- **Found during:** Task 1 corpus closure verification
- **Issue:** `UPSTREAM-CORPUS.md` did not match the existing typed corpus generator even though its machine authority and generator code were unchanged by this plan.
- **Fix:** With parent authorization, regenerated the report only through `cargo xtask inventory corpus generate-report`, generated it a second time, and proved byte stability before closure verification.
- **Files modified:** `UPSTREAM-CORPUS.md`
- **Verification:** `cargo xtask inventory corpus check-closure` reports 388 terminal items and zero unresolved items.
- **Committed in:** `ad6be0f`

**Total deviations:** 1 auto-fixed (1 blocking generated-artifact issue)

**Impact on plan:** The repair was required for the plan's named corpus-closure command and did not change corpus authority or expand product behavior.

## Issues Encountered

- Repository-wide `mdformat --check .` traverses the excluded build tree before filtering it, so both pinned Markdown checks were slow but completed successfully.

## Known Stubs

None. The bracketed public-claim text in `BENCHMARKING.md` is an intentionally non-claiming template and explicitly cannot be published until populated from one reviewed immutable report.

## Threat Flags

None. The plan adds validation and documentation only; it introduces no endpoint, authentication path, file-write boundary, or schema at a runtime trust boundary.

## Verification

- `cargo test -p xtask --test inventory_cli` - 43 passed.
- `cargo xtask inventory check-report` - 181 derived compatibility rows verified.
- `cargo xtask inventory corpus check-closure` - 388 terminal corpus items and zero unresolved verified.
- Compatibility and corpus reports each generated twice with byte-identical results.
- Ordered set audit found 181 ledger identities, 181 unique release joins, and exact ordered equality.
- `cargo xtask docs check` and the required benchmarking token scan passed.
- Before each task commit, the exact ordered `cargo fmt --all`, Clippy, all-target build, all-feature test, and `just markdown-check` gates passed with `/tmp/liquidfun-phase12.OJRc0w` and one build job.
- `git diff --check` and both cached diff reviews passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The final release audit can consume one fail-closed compatibility/corpus closure definition and the bounded benchmark claim contract.
- No reviewed performance report exists yet, so public performance numbers remain correctly prohibited.

## Self-Check: PASSED

- Confirmed all eight task files and this summary exist.
- Confirmed task commits `ad6be0f` and `231e886` exist.
- Confirmed the summary contains exactly two YAML frontmatter delimiters.
