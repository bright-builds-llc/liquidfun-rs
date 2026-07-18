---
phase: 09-particle-storage-lifecycle-and-coupling
plan: 30
subsystem: differential-evidence
tags: [rust, schema-v4, proof-topology, differential-testing, asan, ubsan]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    provides: Phase 9 seven-case particle corpus, witness bindings, and paired evidence pipeline
provides:
  - Canonical case-local topology for six independently persisted proof roles
  - Pre-deduplication schema-v4 topology validation shared by local and exact-ref modes
  - Digest- and identity-recomputed corruption regressions for proof substitution and aliasing
  - Fresh byte-identical canonical and sanitizer evidence for 7 cases and 58 bindings
affects: [phase-09-verification, exact-ref-evidence, particle-compatibility]
tech-stack:
  added: []
  patterns:
    - Exact logical proof roles validated before filesystem reference collection
    - Explicit equality allowlist paired with required path inequality
key-files:
  created: []
  modified:
    - crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - tools/xtask/src/phase9_evidence.rs
    - tools/xtask/tests/phase9_evidence_cli.rs
key-decisions:
  - "Validate exact case-local schema-v4 proof paths before payload collection or exact-file-set deduplication."
  - "Permit path reuse only for replay with corresponding D0 and minimized/copied with corresponding first-divergence; required persisted pairs remain path-distinct."
patterns-established:
  - "Proof topology: normalize only for rejection diagnostics, then require the original path spelling to equal its exact canonical role path."
  - "Adversarial evidence tests: recompute payload digests, semantic manifest digest, and artifact identity so topology is the independent rejection boundary."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T21:16:31Z
duration: 23min
completed: 2026-07-18
---

# Phase 09 Plan 30: Proof-Role Topology Summary

**Schema-v4 evidence binds six independent proof roles to exact case-local paths before deduplication, backed by recomputed-integrity attacks and byte-identical canonical/sanitizer evidence.**

## Performance

- **Duration:** 23 min
- **Started:** 2026-07-18T20:53:33Z
- **Completed:** 2026-07-18T21:16:31Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments

- Enforced exact `cases/<case-id>/proofs/` paths for replay-native, replay-oracle, debug, release, minimized, and copied proof roles.
- Rejected baseline substitution, noncanonical spelling, traversal, absolute paths, wrong case IDs, and every forbidden required-pair alias before payload collection or deduplication.
- Preserved only the reviewed replay-to-D0 and minimized/copied-to-first-divergence reuse relationships.
- Generated and validated byte-identical schema-v4 canonical and sanitizer manifests with 7 cases, 58 semantic bindings, and 22 unique policies.
- Passed the ASan/UBSan protocol test, the complete evidence closure gate, and an ASVS L1 review with no high-severity findings.

## Task Commits

Each task was committed atomically:

1. **Task 1: Specify schema-v4 proof-role topology with adversarial regressions** - `f66889c` (test)
2. **Task 2: Generate and validate canonical schema-v4 proof paths** - `bddc8d6` (feat)
3. **Task 3: Rebuild paired local evidence and pass the complete closure gate** - `fa8cac7` (test, verification-only empty commit)

## Files Created/Modified

- `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs` - Defines exact logical roles, canonical paths, reuse allowlist, distinctness requirements, and shared topology validation.
- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Emits schema-v4 manifests and six canonical proof payload files per evidence-bearing case.
- `tools/xtask/src/phase9_evidence.rs` - Requires schema v4 and validates topology before collecting referenced payloads.
- `tools/xtask/tests/phase9_evidence_cli.rs` - Exercises direct and full-CLI topology corruption with recomputed digests and artifact identity.

## Evidence Closure

- Canonical manifest SHA-256: `74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c`
- Sanitizer manifest SHA-256: `74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c`
- Semantic manifest SHA-256: `a319f771c5d9e952b9389160bb3ad19ce487da43271e62568828ce2ae22a33aa`
- Profiles: byte-identical
- Cases: 7
- Semantic bindings: 58
- Unique policies: 22
- Upstream revision: `7f20402173fd143a3988c921bc384459c6a858f2`

The full planned command chain passed: focused CLI and corpus suites, debug/release oracle builds, canonical generation, ASan/UBSan build and protocol test, sanitizer generation, paired local validation with four denied historical run IDs, inventory, provenance, dependency policy, schema drift, workflow lint, Markdown, upstream read-only, diff checks, and the exact ordered Rust gate.

## ASVS L1 Review

No high-severity finding was identified.

- **Path traversal:** Proof roles require exact case-local canonical paths; absolute, drive-absolute, parent traversal, dot components, duplicate separators, backslashes, and wrong case IDs fail closed.
- **Symlink/archive escape:** Evidence roots and every descendant reject symlink components and require canonical target containment; archive metadata and paths are checked before extraction.
- **Resource bounds:** JSON inputs are limited to 16 MiB, case IDs and proof paths are bounded, protocol decoding uses reviewed harness limits, and the corpus cardinalities remain fixed.
- **JSONL validation:** Requests and results use strict typed decoders with newline-complete records, closed schemas, and hard errors.
- **Shell arguments:** Oracle presets and evidence commands use fixed reviewed arguments; manifest content is not interpolated into a shell command.
- **Identity/digest binding:** Payload SHA-256 values, semantic manifest SHA-256, complete artifact identity, retained rigid comparison, and cross-profile semantics are independently validated.

## Decisions Made

- Kept topology evaluation in the shared differential evidence module so local and exact-ref validation cannot diverge.
- Required exact original path spelling after normalized safety checks, preventing alternative spellings from becoming accepted aliases.
- Represented the verification-only third task with an empty atomic commit, preserving the four-file source boundary while retaining one commit per plan task.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The first Task 2 all-features gate completed after its output session detached, so the exact four-command gate was rerun to obtain an unambiguous successful exit immediately before commit.
- Local CMake 3.27.9 and Apple Clang 21.0.0 differ from the recorded canonical Linux tool identities; the planned local evidence identity remains explicit, and all debug, release, and sanitizer builds passed.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

G09-PROOF-TOPOLOGY is closed locally. The schema-v4 implementation and fresh paired evidence are ready for one new exact-ref authority run; no workflow dispatch, evidence promotion, push, or publication was performed.

## Self-Check: PASSED

- All four declared source files exist.
- Task commits `f66889c`, `bddc8d6`, and `fa8cac7` exist.
- Canonical and sanitizer manifests are schema v4, contain 7 cases and 58 bindings, cover 22 unique policies, and are byte-identical.
- No production physics, public API, CMake input, oracle workflow, global configuration, or pinned upstream file changed.
