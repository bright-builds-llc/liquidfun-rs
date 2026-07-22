---
phase: 11-examples-headless-tooling-and-testbed
plan: "22"
subsystem: compatibility-evidence
tags: [github-actions, artifact-safety, exact-ref, d1-authority, provenance]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "21"
    provides: Identity-last Phase 11 generation and isolated same-run CI jobs
provides:
  - One fresh canonical and sanitizer Phase 11 D1 authority pair from one dispatch and immutable SHA
  - Live run, job, artifact, archive, identity, and semantic digest evidence
  - Permanent deny bindings for the failed forensic run and its unaccepted artifacts
affects: [phase11-evidence-sign-off, phase11-verification, phase12-release-readiness]
tech-stack:
  added: []
  patterns:
    - Authority acquisition requires zero-before and exactly-one-after dispatch cardinality
    - Remote archives are inspected as bounded closed topologies before independent extraction
key-files:
  created:
    - target/phase11-evidence/run.json
    - target/phase11-evidence/phase11-canonical
    - target/phase11-evidence/phase11-sanitizer
  modified:
    - .github/workflows/oracle.yml
    - tools/xtask/tests/phase11_evidence_cli/workflow.rs
    - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
    - reference/artifacts/phase10/group-topology-witnesses.provenance.json
    - TESTING.md
key-decisions:
  - "Bind Phase 11 authority only to run 29927362730 at repair SHA 4ea1e1e65919619d8cd1155a5461c2cda16ab7b6."
  - "Permanently deny failed run 29899265024 and artifact IDs 8521315244 and 8521345417; metadata inspection never makes a failed artifact eligible for download or promotion."
  - "Keep Phase 11 dispatch closed by making all six Oracle jobs mutually exclusive for an explicit evidence phase."
patterns-established:
  - "Fresh exact-ref acquisition: live metadata, recomputed archive digests, safe preflight, isolated extraction, live requery, then semantic validation."
requirements-completed: [TEST-03, EXMP-01, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T14:25:22Z
duration: 25 min
completed: 2026-07-22
---

# Phase 11 Plan 22: Fresh Same-Run D1 Authority Summary

**One independently downloaded canonical/sanitizer pair from a single immutable Phase 11 dispatch now passes archive-safety, live-identity, locked-stack, and complete exact-reference semantic validation.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-07-22T14:01:07Z
- **Completed:** 2026-07-22T14:25:22Z
- **Tasks:** 1
- **Tracked files modified:** 5
- **Fresh ignored evidence outputs:** 4

## Accomplishments

- Repaired stale Phase 9/10 adapter provenance bindings and made every Oracle CI evidence-phase route mutually exclusive, preventing a Phase 11 dispatch from running legacy authority jobs.
- Proved the repair with automatic push run `29926988101`, whose canonical legacy job succeeded while the other five jobs skipped.
- Made exactly one Phase 11 dispatch at immutable SHA `4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`; run `29927362730` completed with exactly the two requested successful Phase 11 jobs and four skipped legacy jobs.
- Downloaded the two live artifacts independently, recomputed their sizes and SHA-256 digests, inspected their closed nine-entry archives before extraction, re-queried live metadata, and passed exact-reference validation with all failed-run identities denied.
- Recorded the immutable authority identities and reusable deny-bound validation procedure in `TESTING.md` without changing corpus or compatibility authority.

## TDD Evidence

- **RED:** `workflow::oracle_workflow_produces_one_same_run_phase11_pair` failed because an explicit Phase 11 dispatch still admitted legacy Oracle jobs.
- **GREEN:** The workflow contract now proves exact mutually exclusive job conditions, full-SHA actions, exactly two Phase 11 script calls, and exact artifact names; the complete focused suite passes 12/12.
- **Authority proof:** The successful one-shot run instantiated that contract: Phase 11 canonical and sanitizer succeeded, while all four legacy jobs skipped.

No intentionally failing state was committed.

## Task Commits

1. **Rule 1 repair: isolate Phase 11 authority dispatch and refresh provenance** - `4ea1e1e` (fix)
1. **Task 1: record the validated Phase 11 D1 authority set** - `32d43ca` (docs)

**Plan metadata:** committed separately with this summary.

## Authority Identities

- **Immutable implementation/repair SHA:** `4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`
- **Automatic push proof:** run `29926988101`, canonical job `88946587737` successful, five jobs skipped
- **Single authorized dispatch:** run `29927362730`, created `2026-07-22T14:11:14Z`, terminal success
- **Canonical job:** `88947879161`, successful
- **Sanitizer job:** `88947879108`, successful
- **Platform/toolchain:** Linux x86_64; Rust 1.97.0; CMake 4.3.3; Ninja 1.13.2; Clang 22.1.8
- **Upstream:** `7f20402127410689df8a9d380332e1aaf2615f56`
- **Protocol/generator:** `catalog-phase11-v1` / `phase11-evidence-v1`

### Accepted Canonical Artifact

- **ID:** `8532642627`
- **Name:** `phase11-canonical-29927362730-4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`
- **Size:** 12,974 bytes
- **Archive SHA-256:** `2bbf2dd14fdb3a8fbae119a150e0e2292dc36f6752c54ae3be466a23415c81e0`
- **Identity SHA-256:** `3a2fe2c222103ca60501e448d0256821a1b3c21301773649a79a51f92f776303`

### Accepted Sanitizer Artifact

- **ID:** `8532662842`
- **Name:** `phase11-sanitizer-29927362730-4ea1e1e65919619d8cd1155a5461c2cda16ab7b6`
- **Size:** 12,968 bytes
- **Archive SHA-256:** `745b2e7fdeb730f8b40f68aea7f0f776b93465ed39a30e7832dcdeeaaa46ac3d`
- **Identity SHA-256:** `4beb3be3c802b7360adff2d90df20f51c8d74719bacb523ebfad0f84b5fd7437`

Both artifacts have semantic SHA-256 `248b19ecdfa6f5cd202a5d6b07783c82e097927d78e4ad87f5a4fe4c772687eb` and identical eight-file content inventories apart from their deliberately distinct authority identities.

## Files Created/Modified

- `target/phase11-evidence/run.json` - Fresh live run, six-job, two-artifact, toolchain, digest, and deny-bound snapshot.
- `target/phase11-evidence/phase11-canonical` - Independently extracted canonical artifact with exact closed topology.
- `target/phase11-evidence/phase11-sanitizer` - Independently extracted sanitizer artifact with exact closed topology.
- `.github/workflows/oracle.yml` - Closed explicit evidence-phase routing across all six Oracle jobs.
- `tools/xtask/tests/phase11_evidence_cli/workflow.rs` - Exact job-section, condition, action-pin, script-call, and artifact-name contract coverage.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` - Current adapter content binding.
- `reference/artifacts/phase10/group-topology-witnesses.provenance.json` - Current adapter content binding.
- `TESTING.md` - Immutable failed/accepted run identities, archive evidence, and reusable exact-ref procedure.

## Decisions Made

- Bound D1 authority to the only Phase 11 dispatch at the repair SHA. A retry, rerun, relabel, mixed run, or artifact from another SHA cannot substitute for this pair.
- Treated failed run `29899265024` as forensic metadata only. Its canonical artifact `8521315244` and sanitizer artifact `8521345417` were never downloaded or promoted and remain explicit validator deny inputs.
- Required archive safety to pass before extraction, then extracted each artifact into its own fresh fixed directory without copying or combining files.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Closed Phase 11 dispatch routing and refreshed stale provenance bindings**

- **Found during:** Task 1 pre-dispatch forensic run
- **Issue:** Failed run `29899265024` admitted four legacy jobs during a Phase 11 dispatch; all four failed because Phase 9/10 provenance still named the superseded adapter content SHA-256.
- **Fix:** Added exact mutually exclusive job conditions, strengthened workflow regression tests, and updated only the two stale provenance adapter digests.
- **Files modified:** `.github/workflows/oracle.yml`, `tools/xtask/tests/phase11_evidence_cli/workflow.rs`, `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json`, `reference/artifacts/phase10/group-topology-witnesses.provenance.json`
- **Verification:** Focused workflow tests 12/12; `actionlint`; provenance check; complete ordered Rust gate; push run `29926988101`; one-shot run `29927362730`.
- **Committed in:** `4ea1e1e`

**Total deviations:** 1 auto-fixed Rule 1 bug. **Impact:** The repair narrowed dispatch execution and restored current-checkout provenance without widening evidence, corpus, or compatibility authority.

## Issues Encountered

- `just markdown-check` exposed six unrelated pre-existing unformatted Markdown files. `TESTING.md` itself was formatted with mdformat 1.0.0 and passes its focused formatting check; unrelated files were left untouched.
- The failed forensic run's artifact metadata was sufficient to establish permanent deny identities. Those archives were deliberately never downloaded.

## Security Verification

- Zero-before/exactly-one-after cardinality, clean immutable SHA equality, and normal fast-forward push constrained the effectful workflow boundary.
- Live run/job/artifact requery, distinct nonzero IDs, exact names, same SHA, success conclusions, finite expiry, recomputed sizes/digests, and deny IDs prevented stale or spoofed authority.
- Both archives passed normalized relative path, traversal, backslash, absolute-path, duplicate, case-fold collision, entry-type, depth, count, and uncompressed-size checks before extraction.
- The extracted directories stayed distinct; no artifact content crossed between them. No secrets, unbounded logs, screenshots, raw memory, or private indices entered the evidence set.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Verification

- `actionlint .github/workflows/oracle.yml` passed.
- `cargo test -p xtask --test phase11_evidence_cli` passed 12/12.
- `cargo xtask provenance check` passed against the initialized pinned submodule.
- Automatic push run `29926988101` succeeded with the intended closed job matrix.
- Single dispatch run `29927362730` succeeded with two requested jobs and four skipped jobs.
- Both downloaded archives passed API size/digest comparison, ZIP integrity, and complete safety preflight before extraction.
- Fresh live metadata requery matched `run.json` and confirmed exactly one dispatch at the exact SHA.
- Exact-ref validation passed for three cases with `--deny-run-id 29899265024`, `--deny-artifact-id 8521315244`, and `--deny-artifact-id 8521345417`.
- `mdformat --check TESTING.md` passed. Repository-wide Markdown verification is blocked only by the six unrelated pre-existing files named above.
- The exact ordered `cargo fmt --all`, all-target/all-feature deny-warnings Clippy, all-target/all-feature build, and all-feature test gate passed twice in isolated target directories.
- `git diff --check` passed and the scoped diff contained no corpus or compatibility ledger change.

## Requirements Status

Plan 11-22's `TEST-03`, `EXMP-01`, and `EXMP-03` mappings are complete. This plan acquired and validated the fresh D1 evidence input; it did not promote D3 compatibility authority.

## User Setup Required

None.

## Next Phase Readiness

- The fresh pair is ready for the separate reviewed no-clobber D3 promotion plan.
- Failed-run identities and the exact accepted authority set are durable and independently reproducible while GitHub retention remains active.
- No Plan 11-22 blocker remains.

## Known Stubs

None.

## Self-Check: PASSED

- Confirmed all five tracked files and all four fresh ignored evidence outputs exist.
- Confirmed commits `4ea1e1e` and `32d43ca` are present.
- Confirmed the modified-file stub scan returned no matches and no new security-relevant surface escaped the plan's threat model.
- Confirmed the summary contains no standalone body separator that could confuse frontmatter parsing.

***

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
