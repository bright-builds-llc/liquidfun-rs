---
phase: 11-examples-headless-tooling-and-testbed
plan: "20"
subsystem: compatibility-evidence
tags: [xtask, evidence, sha256, exact-ref, archives, provenance]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "19"
    provides: Sealed three-case corpus, complete scenario mappings, and inherited Phase 6-10 proofs
provides:
  - One shared semantic evaluator for local and exact-reference Phase 11 evidence
  - Identity-last local artifacts and same-run exact-reference authority validation
  - Bounded pre-extraction archive topology, digest, size, mode, and compression checks
affects: [phase11-evidence-generation, phase11-oracle-workflow, phase11-sign-off, phase12-release-readiness]
tech-stack:
  added: []
  patterns:
    - Exact-reference authority consumes one accepted typed content result and never reinterprets semantic records
    - Tracked local corpus validation is structurally identity-free and non-promotable
key-files:
  created:
    - tools/xtask/src/phase11_evidence.rs
    - tools/xtask/src/phase11_evidence/content.rs
    - tools/xtask/src/phase11_evidence/authority.rs
    - tools/xtask/tests/phase11_evidence_cli.rs
  modified:
    - tools/xtask/src/main.rs
    - tools/xtask/src/provenance/artifact.rs
    - reference/artifacts/manifest.toml
key-decisions:
  - "Recompute resolved, request, checkpoint, comparison, mapping, policy, and inherited-proof digest sets before accepting any local or exact-reference identity."
  - "Keep local tracked-corpus and generated-artifact paths explicitly non-promotable; only a fresh live same-run exact-reference pair can satisfy D1 authority."
  - "Register the Phase 11 evidence schema without adding a compatibility promotion record."
patterns-established:
  - "Content before authority: evaluate the closed corpus and all four D0 proof roles once, then layer identity and live-run checks over the accepted digest."
  - "Archive safety: inspect exact paths, modes, counts, bytes, compression, and collisions without extracting or trusting archive metadata."
requirements-completed: [TEST-03, EXMP-01, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T05:50:18Z
duration: 28 min
completed: 2026-07-22
---

# Phase 11 Plan 20: Fail-Closed Evidence Validator Summary

**One shared semantic evaluator now recomputes the sealed Phase 11 corpus before bounded local or same-run exact-reference authority can be accepted.**

## Performance

- **Duration:** 28 min
- **Started:** 2026-07-22T05:22:00Z
- **Completed:** 2026-07-22T05:50:18Z
- **Tasks:** 1
- **Files modified:** 14

## Accomplishments

- Added `phase11-evidence validate --mode local|exact-ref`, identity-last content validation, and a bounded record renderer for the fixed evidence-generation workflow.
- Recomputed exact resolved bytes, encoded requests, action/checkpoint schedules, comparison contracts, complete scenario mappings, closed numeric policies, and inherited Phase 6-10 proof hashes for all three representative cases.
- Required debug, release, replay, and sanitizer records to carry identical strict semantic content while rejecting pixels, frame data, wall-clock durations, private IDs, unknown policies, omitted leaves, and unknown fields.
- Enforced one fresh `workflow_dispatch` run and immutable SHA with distinct successful canonical and sanitizer jobs/artifacts on Linux x86_64, Rust 1.97.0, and Clang 22.1.8.
- Added pre-extraction archive checks for normalized paths, traversal, links/devices, collisions, entry/file/byte/compression limits, exact live sizes/digests/timestamps, mixed runs, zero live IDs, and historical denysets.

## TDD Evidence

- **RED:** The focused CLI suite failed 3/3 because `phase11-evidence` did not exist.
- **GREEN:** The completed focused suite passes 9/9 across local source/generated paths, semantic drift, partial/extra/symlink content, valid exact authority, mixed SHA, zero live IDs, denysets, and archive topology.
- **REFACTOR:** Split typed models, corpus hashing/closure, paths, content evaluation, and authority into cohesive modules; all production evidence files remain within the repository's 300-500 line guidance.

The intentionally failing RED state was not committed because repository policy requires every commit to follow a completely passing ordered Rust gate.

## Task Commits

Each task was committed atomically:

1. **Rule 3 prerequisite: restore strict workspace Clippy** - `2bf4550` (fix)
2. **Task 1: Implement shared semantic and authority validation** - `1789618` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `tools/xtask/src/phase11_evidence.rs` - Closed CLI, options, typed identity, identity inventory, and mode routing.
- `tools/xtask/src/phase11_evidence/content.rs` - Single local/exact content evaluator and fixed D0 record renderer.
- `tools/xtask/src/phase11_evidence/content/corpus.rs` - Exact corpus, run, checkpoint, mapping, policy, leaf, and inherited-proof recomputation.
- `tools/xtask/src/phase11_evidence/content/model.rs` - Strict deny-unknown-fields artifact models.
- `tools/xtask/src/phase11_evidence/authority.rs` - Live same-run D1 authority and bounded archive inspection.
- `tools/xtask/src/phase11_evidence/paths.rs` - Confined regular-file, digest, topology, and size helpers.
- `tools/xtask/tests/phase11_evidence_cli.rs` - Local, malformed, partial, private, and symlink CLI contracts.
- `tools/xtask/tests/phase11_evidence_cli/exact.rs` - Exact-reference live authority, denyset, mixed-run, and archive contracts.
- `tools/xtask/tests/phase11_evidence_cli/support.rs` - Identity-last test artifact construction.
- `reference/artifacts/manifest.toml` - Phase 11 evidence schema registration only; no compatibility promotion.
- `tools/xtask/src/provenance/artifact.rs` - Strict parser validation for the registered schema.

## Decisions Made

- Kept the tracked corpus admissible only in local identity-free mode so the required developer command remains useful without creating promotable authority.
- Made generated artifacts copy the sealed manifest/case bytes and bind four strict proof-role records to one independently recomputed semantic digest.
- Preserved the pre-upload `artifact_id = 0` inside identity-last artifact content while requiring nonzero IDs in independently captured live GitHub artifact metadata.
- Avoided duplicating Phase 10 files mechanically by separating Phase 11's typed corpus evaluator from the small authority and path layers it conceptually follows.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored strict workspace Clippy before evidence validation**

- **Found during:** Task 1 focused deny-warnings Clippy
- **Issue:** Existing committed xtask code manually reimplemented `Option::unwrap_or`, and a Phase 10 test helper model triggered the strict field-name lint.
- **Fix:** Used `unwrap_or(1)` and renamed the private `Case` helper to `EvidenceCase` without behavior changes.
- **Files modified:** `tools/xtask/src/differential.rs`, `tools/xtask/tests/phase10_evidence_cli/support.rs`
- **Verification:** Focused xtask all-target/all-feature deny-warnings Clippy and the complete ordered Rust gate pass.
- **Committed in:** `2bf4550`

**Total deviations:** 1 auto-fixed (1 Rule 3 blocking). **Impact:** The minimal lint-only repair was required to satisfy the repository's mandatory verification boundary and did not expand evidence behavior.

## Issues Encountered

- A supplemental bare `cargo test -p xtask` invocation did not build the external differential executable required by two legacy CLI tests. This package-only discovery limitation is not an authoritative repository gate: the required ordered full build/test gate and the focused Phase 11 suite pass. No test was weakened or skipped.
- `cargo xtask check` advances through inventory, package, and protocol validation before reaching the known TESTING table contract owned by Plan 11-21. Direct provenance validation parses the new schema and then stops on the preserved fenced dirty Phase 9 witness hash. Neither is part of Plan 11-20's scoped acceptance command.

## Security Verification

- Strict parsing rejects unknown fields, open policies, missing leaves, UI/timing/private identifiers, malformed paths, and partial proof roles before identity or authority checks.
- Exact-reference authority requires one immutable run/SHA and one distinct successful canonical/sanitizer pair with fresh independently captured live metadata.
- Archive bytes are inspected without extraction and rejected on traversal, absolute paths, backslashes, links/devices, collisions, unknown entries, excessive depth/count/size/compression, digest drift, or stale metadata.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-20's `TEST-03`, `EXMP-01`, and `EXMP-03` mappings are implemented and recorded in summary frontmatter. Global requirement checkboxes remain unchanged until the remaining Phase 11 evidence generation and sign-off plans close the complete scopes.

## User Setup Required

None - local validation is Cargo-only, and exact-reference acquisition remains owned by the following workflow plan.

## Next Phase Readiness

- Plan 11-21 can generate identity-last local and canonical artifacts against one closed validator and use `render-records` plus `validate-content` without reimplementing semantic hashing.
- No blocker remains for Plan 11-21.

## Self-Check: PASSED

- Confirmed the four primary source/test artifacts exist and commits `2bf4550` and `1789618` are present.
- Confirmed focused xtask deny-warnings Clippy, 9/9 Phase 11 CLI tests, and the exact required local validator command pass.
- Confirmed the exact ordered `cargo fmt --all`, all-target/all-feature deny-warnings Clippy, all-target/all-feature build, and all-feature test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-20` before each commit.
- Confirmed `reference/compatibility.json` is unchanged and the four fenced pre-existing edits remain unstaged and uncommitted.

***

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
