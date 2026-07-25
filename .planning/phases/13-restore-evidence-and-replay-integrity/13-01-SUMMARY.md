---
phase: 13-restore-evidence-and-replay-integrity
plan: '01'
subsystem: provenance
tags: [cmake-file-api, depfiles, sha256, git-revision, evidence-schema]
requires:
  - phase: 09-contact-ordering-lifetimes-and-group-creation
    provides: Reviewed lifecycle/contact witness pair and native Rust consumers
  - phase: 11-compatibility-harness-public-observability-and-evidence
    provides: Artifact ledger, source map, and strict provenance entrypoint
provides:
  - Mechanically derived target-scoped Phase 9 witness materials closure
  - Repository-bound scoped provenance schema with edited-hash resistance
  - Phase 13 witness, replay, staging, and promotion evidence-class contracts
affects: [13-02-replay-integrity, 13-03-staging, 13-04-canonical-regeneration]
tech-stack:
  added: []
  patterns:
    - CMake File API codemodel plus compiler depfile closure
    - Length-prefixed typed material hashing
    - Repository revision binding for provenance inputs
key-files:
  created:
    - tools/reference/phase9-lifecycle-contact-witness.materials.json
    - tools/xtask/src/provenance/phase9_witness/materials.rs
    - tools/xtask/src/provenance/evidence_schema.rs
    - tools/xtask/tests/phase9_witness_provenance.rs
  modified:
    - tools/xtask/src/provenance/phase9_witness.rs
    - tools/reference/CMakeLists.txt
    - reference/artifacts/manifest.toml
    - reference/source-map.toml
key-decisions:
  - "Bind scoped materials to both their recomputed digest and a recorded repository revision so editing current hashes cannot self-bless changed inputs."
  - "Split protocol bit encoding into a dedicated CMake target so unrelated adapter sources are outside the Phase 9 witness link closure."
  - "Leave the reviewed schema-1 witness/provenance pair byte-identical and reject it explicitly until Plan 13-04 performs reviewed regeneration."
patterns-established:
  - "Target-scoped provenance: declarations are checked bidirectionally against independently derived build inputs before hashing."
  - "Evidence-class provenance: every Phase 13 record class names its pinned source, derivation, alteration, and notice contract."
requirements-completed: [FND-04, COMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 13-2026-07-25T18-25-02
generated_at: 2026-07-25T20:06:26Z
duration: 30min
completed: 2026-07-25
---

# Phase 13 Plan 01: Target-Scoped Witness Provenance Summary

**CMake-derived Phase 9 witness materials with repository-bound hashes and strict Phase 13 evidence-class provenance**

## Performance

- **Duration:** 30 min
- **Started:** 2026-07-25T19:35:52Z
- **Completed:** 2026-07-25T20:06:11Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments

- Declared 111+ target-affecting build rules, sources, headers, flags, definitions, include paths, link inputs, and preset values in a canonical typed manifest.
- Derived the actual target closure from CMake File API codemodel replies and compiler depfiles, rejecting unexpected and declared-only materials with exact diagnostics.
- Removed the Phase 9 witness target's dependency on the broad adapter library while preserving shared protocol bit encoding.
- Replaced aggregate adapter provenance validation with scoped manifest, material, probe, witness, toolchain, and repository-revision identities.
- Added exact Phase 13 witness, replay-evidence, staged-bundle, and promotion-receipt schemas with source, alteration, and notice requirements.
- Preserved both reviewed Phase 9 artifact files byte-for-byte; the unchanged legacy provenance now fails with the intended schema migration diagnostic.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define and resolve the target-scoped Phase 9 materials closure** - `d54a999` (feat)
2. **Task 2: Migrate Phase 9 provenance validation to the scoped identity** - `5136c13` (feat)

## Files Created/Modified

- `tools/reference/phase9-lifecycle-contact-witness.materials.json` - Canonical typed Phase 9 target-material inventory.
- `tools/xtask/src/provenance/phase9_witness/materials.rs` - Manifest parser, CMake/depfile derivation, bidirectional comparison, hashing, and Git revision binding.
- `tools/xtask/src/provenance/evidence_schema.rs` - Strict Phase 13 evidence-class contract validator.
- `tools/xtask/tests/phase9_witness_provenance.rs` - Closure, isolation, tamper, edited-hash, and missing-provenance regression tests.
- `tools/xtask/src/provenance/phase9_witness.rs` - Schema-2 scoped provenance enforcement.
- `tools/xtask/src/provenance/artifact.rs` - Phase 13 schema and materials source-map enforcement.
- `tools/reference/CMakeLists.txt` - Narrow Phase 9 link closure and target-specific build identity.
- `tools/reference/src/phase9_lifecycle_contact_witness.cpp` - Target-specific identity inputs without aggregate adapter provenance.
- `reference/artifacts/manifest.toml` - Phase 13 evidence-class definitions.
- `reference/source-map.toml` - Materials-manifest mapping and updated artifact-schema classification.

## Decisions Made

- Scoped identities include typed metadata and file bytes, while mechanical completeness comes from the independent CMake codemodel and depfile closure.
- Repository-owned material bytes must match the recorded Git revision; pinned upstream materials remain bound through the separately validated oracle revision.
- Canonical evidence remains immutable in this plan. Validation fails closed on the legacy schema until reviewed regeneration and promotion in Plan 13-04.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Isolated the Phase 9 link closure from the aggregate protocol library**

- **Found during:** Task 1
- **Issue:** The witness still linked the broad protocol library, so unrelated adapter sources remained target-affecting despite a scoped manifest.
- **Fix:** Extracted protocol bit encoding into a dedicated static target and supplied target-specific identity definitions directly.
- **Files modified:** `tools/reference/CMakeLists.txt`, `tools/reference/src/phase9_lifecycle_contact_witness.cpp`, `tools/reference/src/generate_build_identity.cmake.in`
- **Verification:** The Phase 9 CMake target configured and built successfully; changing an unrelated adapter file left the scoped digest unchanged.
- **Committed in:** `d54a999`

**2. [Rule 2 - Missing Critical] Added reusable enforcement for Phase 13 evidence schemas**

- **Found during:** Task 2
- **Issue:** Adding TOML declarations alone would not make missing source, alteration, or notice fields fail the production provenance entrypoint.
- **Fix:** Added a typed evidence-schema validator and connected it to artifact-manifest validation, including required materials source-map coverage.
- **Files modified:** `tools/xtask/src/provenance.rs`, `tools/xtask/src/provenance/artifact.rs`, `tools/xtask/src/provenance/evidence_schema.rs`
- **Verification:** Focused negative tests reject missing source paths, alteration summaries, and notice references; the production provenance command accepts the new ledger schema before reaching the intentionally legacy Phase 9 record.
- **Committed in:** `5136c13`

***

**Total deviations:** 2 auto-fixed (2 missing critical)
**Impact on plan:** Both changes were necessary to make the scoped identity and evidence schemas enforceable; no canonical artifact was regenerated and no unrelated feature scope was added.

## Issues Encountered

- The shared repository Cargo target cache stalled on a stale lock. Verification used an isolated temporary `CARGO_TARGET_DIR`; all required commands then completed successfully.
- Local CMake/AppleClang versions are D2 rather than canonical Linux D1 identities. The target build was verified locally, while the checked-in manifest records the canonical identity that Plan 13-04 will use for reviewed regeneration.

## Verification

- `cargo test -p xtask --test phase9_witness_provenance` - 2 passed.
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings` - passed.
- `cargo fmt --all` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo build --all-targets --all-features` - passed.
- `cargo test --all-features` - passed.
- Phase 9 CMake configure/build - passed.
- `cargo xtask provenance check` - intentionally fails on the untouched schema-1 pair with the exact scoped-schema migration diagnostic.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 13-02 and 13-03 can build replay and staging workflows on the strict evidence-class contract.
- Plan 13-04 must regenerate and review the Phase 9 schema-2 provenance record before the complete provenance check can pass.
- The existing reviewed Phase 9 witness and provenance bytes remain unchanged for controlled migration.

***

*Phase: 13-restore-evidence-and-replay-integrity*
*Completed: 2026-07-25*

## Self-Check: PASSED

- All created files exist.
- Task commits `d54a999` and `5136c13` exist.
- Canonical Phase 9 witness and provenance files remain unchanged.
