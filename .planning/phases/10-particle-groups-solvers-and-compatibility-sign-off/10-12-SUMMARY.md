---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "12"
subsystem: particle-topology
tags: [rust, cpp-oracle, particle-pairs, particle-triads, voronoi, proptest]

requires:
  - phase: 10-03
    provides: Particle object identities and stable semantic view boundaries
  - phase: 10-05
    provides: Storage-owned group records, ranges, flags, and strengths
  - phase: 10-11
    provides: Bounded dense-order Voronoi generation with oriented nodes
provides:
  - Provenance-bound seven-case upstream group-topology decision artifact
  - Exact private pair and triad generation over checked dense endpoints
  - Probe-backed typed errors for zero-length pairs and missing necessary generators
  - Operation-scoped append-sort-deduplicate and preserve-order policies
affects: [10-14, 10-20, particle-groups, reactive-particles, spring-solver, elastic-solver]

tech-stack:
  added: []
  patterns:
    - Private out-of-process C++ decision witness with deterministic semantic JSON
    - Pure candidate generation before storage mutation
    - Stable lexicographic append deduplication that retains the first record

key-files:
  created:
    - tools/reference/src/phase10_group_topology_witness.cpp
    - tools/reference/src/phase10_group_topology_cases.cpp
    - tools/reference/src/phase10_group_topology_cases.hpp
    - reference/artifacts/phase10/group-topology-witnesses.json
    - reference/artifacts/phase10/group-topology-witnesses.provenance.json
    - crates/liquidfun/src/particle/topology/constraints.rs
    - crates/liquidfun/src/particle/topology/constraints/tests.rs
    - crates/liquidfun/src/particle/topology/constraints/tests/properties.rs
  modified:
    - tools/reference/CMakeLists.txt
    - tools/xtask/src/upstream.rs
    - tools/xtask/tests/upstream_cli.rs
    - crates/liquidfun/src/particle/topology.rs

key-decisions:
  - "Reject exact zero-length pair rest distances and Voronoi generation without any necessary generator through named typed errors, matching the mandatory pinned-oracle classifications."
  - "Preserve finite barrier-pair and degenerate-triad source behavior without a global epsilon."
  - "Keep constraint endpoints private and dense, translating them only through existing stable semantic views."
  - "Stable-sort and retain the first exact duplicate only for append operations; preserve/remap operations leave historical order untouched."

patterns-established:
  - "Oracle decision gate: finite-degenerate behavior is selected from checked-in pinned evidence before Rust semantics are implemented."
  - "Topology input gate: validate aligned lanes, range, finite geometry, group owner, group strength, and every contact endpoint before producing records."
  - "Constraint generation: preserve source contact/generator order and expression grouping, then apply an explicit operation policy."

requirements-completed: [PART-11, TEST-01, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-19T23:49:26Z

duration: 1h 36m
completed: 2026-07-19
---

# Phase 10 Plan 12: Exact Pair and Triad Topology Summary

**Pinned upstream evidence now drives exact private pair and triad generation with checked dense endpoints, source rest state, and operation-scoped stable deduplication.**

## Performance

- **Duration:** 1h 36m
- **Started:** 2026-07-19T22:13:02Z
- **Completed:** 2026-07-19T23:49:26Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Added a strict private C++ witness covering split metadata, zero-length pairs, degenerate triads, barrier pairs, no-necessary-generator Voronoi input, and empty/singleton rigid groups at pinned revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- Checked in byte-reproducible semantic evidence and provenance bound to exact source, compiler, preset, argv, and result hashes.
- Added pure pair generation with exact source gates, contact-order emission, combined contact flags, current rest distance, and minimum endpoint-group strength.
- Added dense-order Voronoi triad generation with source orientation, distance gates, minimum strength, centroid-relative offsets, and exact `ka`/`kb`/`kc`/`s` expression grouping.
- Added explicit append and preserve policies; append uses stable lexicographic sorting and retains the first duplicate record, while preserve leaves historical order and rest data untouched.
- Added structural branch coverage and three 128-case properties for endpoint bounds, deterministic replay, and exact duplicate uniqueness.

## Task Commits

Each task was committed atomically:

1. **Task 1: Probe split metadata and every listed finite-degenerate case** - `4d35afe` (feat)
2. **Task 2: Implement pair and triad generation** - `32866a7` (feat)

## Files Created/Modified

- `tools/reference/CMakeLists.txt` - Registered the strict private Phase 10 witness and provenance source hash.
- `tools/reference/src/phase10_group_topology_witness.cpp` - CLI and provenance shell for deterministic semantic artifacts.
- `tools/reference/src/phase10_group_topology_cases.cpp` - Seven real pinned-upstream decision probes.
- `tools/reference/src/phase10_group_topology_cases.hpp` - Narrow case-runner boundary.
- `reference/artifacts/phase10/group-topology-witnesses.json` - Exact per-case inputs, outcomes, classifications, and Rust decisions.
- `reference/artifacts/phase10/group-topology-witnesses.provenance.json` - Pinned upstream, source, compiler, preset, argv, and result identity.
- `tools/xtask/src/upstream.rs` - Registered the private witness target with the existing upstream build allowlist.
- `tools/xtask/tests/upstream_cli.rs` - Guarded the new target's CLI acceptance.
- `crates/liquidfun/src/particle/topology.rs` - Registered the private constraint module and particle-visible Voronoi error/limit types.
- `crates/liquidfun/src/particle/topology/constraints.rs` - Checked pair/triad generation, exact rest calculations, and explicit record policies.
- `crates/liquidfun/src/particle/topology/constraints/tests.rs` - Structural source-branch, orientation, coefficient, degenerate, and policy evidence.
- `crates/liquidfun/src/particle/topology/constraints/tests/properties.rs` - 128-case endpoint, determinism, and duplicate-uniqueness properties.

## Decisions Made

- Applied the artifact's `typed_error` classification exactly to zero-length pair rest distances and Voronoi input with no necessary generator.
- Applied the artifact's `preserve_source_behavior` classification to finite barrier pairs and collinear/degenerate triads; no global epsilon was introduced.
- Kept group metadata as a checked private per-particle projection carrying owner, flags, and strength, so foreign group ownership fails before any candidate is produced.
- Kept generated endpoints as private dense indices and reused existing semantic views for stable-ID translation.
- Separated contact-order generation from append finalization so source emission order and stable sorted storage behavior are independently testable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Registered the new private witness target with xtask**

- **Found during:** Task 1 (Probe split metadata and every listed finite-degenerate case)
- **Issue:** The plan's exact `cargo xtask upstream build --target phase10-group-topology-witness` command was rejected by the closed target allowlist.
- **Fix:** Added the target to the existing allowlist and one focused CLI acceptance regression.
- **Files modified:** `tools/xtask/src/upstream.rs`, `tools/xtask/tests/upstream_cli.rs`
- **Verification:** Focused xtask CLI test, exact witness build, and the complete ordered Rust gate passed.
- **Committed in:** `4d35afe`

**2. [Rule 2 - Missing Critical] Split the witness into bounded real C++ modules**

- **Found during:** Task 1 (Probe split metadata and every listed finite-degenerate case)
- **Issue:** Keeping seven real oracle cases in the CLI shell would exceed the repository's source-shape guidance and obscure the provenance boundary.
- **Fix:** Kept CLI/provenance orchestration in a 127-line witness file and moved real case execution behind an 8-line header into a 595-line source file; the CMake provenance hash covers all three files.
- **Files modified:** `tools/reference/CMakeLists.txt`, `tools/reference/src/phase10_group_topology_witness.cpp`, `tools/reference/src/phase10_group_topology_cases.cpp`, `tools/reference/src/phase10_group_topology_cases.hpp`
- **Verification:** Strict C++ build, two-run semantic `cmp`, source hash, and artifact hash checks passed.
- **Committed in:** `4d35afe`

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both changes were narrow private seams required for the exact planned command and repository code-shape/provenance constraints; no public API or runtime dependency changed.

## Issues Encountered

- The vendored JSON package exposes `nlohmann/json.hpp` but not `json_fwd.hpp`; the witness header uses the available reviewed header.
- The deliberate probe-only `private` visibility shim triggered Clang's strict keyword-macro warning; a narrow compiler diagnostic scope suppresses only that warning around the shim.
- The plan's exact focused Rust filter traversed unrelated integration binaries; it completed successfully, while the subsequent private rerun used `--lib` as the equivalent focused lane.
- Shared macOS process-start contention delayed freshly linked test binaries; every process was preserved and completed successfully.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 10-14 and 10-20 can consume exact probe decisions and complete pair/triad rest records without reopening D-12/D-13.
- Create, join, and reactive workflows can use append-sort-first-duplicate behavior; split, compaction, flag-change, and ordinary remap workflows can preserve historical order.
- No blockers remain.

## Self-Check: PASSED

- Confirmed all twelve created or modified task files exist.
- Confirmed task commits `4d35afe` and `32866a7` exist.
- Confirmed the checked-in semantic result hash matches provenance and the exact seven-case ID/decision query passes.
- Confirmed no known stub patterns or unplanned trust-boundary surfaces were introduced.
- Confirmed exact witness configure/build/replay checks, focused topology tests, three 128-case properties, and both mandatory ordered Rust gates pass.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-19*
