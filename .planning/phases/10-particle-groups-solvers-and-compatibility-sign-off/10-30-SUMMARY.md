---
phase: 10-particle-groups-solvers-and-compatibility-sign-off
plan: "30"
subsystem: particle-differential-evidence
tags: [rust, github-actions, sanitizers, evidence-generation, exact-ref]

requires:
  - phase: 10-29
    provides: Shared fail-closed local and exact-ref Phase 10 evidence validator
provides:
  - Identity-last local D2 canonical and fail-fast sanitizer evidence generation
  - One same-run Oracle CI authority pair with exact jobs, artifacts, toolchains, pins, and retention
  - Executable workflow contract plus concrete exact-ref acquisition and D0-D3 documentation
affects: [10-31, 10-32, phase10-authority-acquisition, phase10-compatibility-sign-off]

tech-stack:
  added: []
  patterns:
    - Generate bounded semantic payloads and manifests before writing the final identity
    - Bind pre-upload sentinel identities to post-upload live artifact metadata externally
    - Reuse existing canonical and sanitizer jobs while preserving earlier phase behavior

key-files:
  created:
    - scripts/phase10-evidence.sh
    - crates/liquidfun-differential/tests/phase10_corpus/evidence_output.rs
  modified:
    - .github/workflows/oracle.yml
    - justfile
    - TESTING.md
    - tools/xtask/src/phase10_evidence/authority.rs
    - tools/xtask/tests/phase10_evidence_cli.rs
    - crates/liquidfun-differential/tests/phase10_corpus.rs

key-decisions:
  - "Generate canonical and sanitizer evidence from the same sealed five-case corpus and write identity only after every semantic, provenance, inventory, and read-only check passes."
  - "Keep archived artifact_id at the zero pre-upload sentinel because GitHub assigns the live ID only after upload; exact-ref authority binds the archive to the independently captured nonzero API ID, name, digest, size, and timestamps."
  - "Extend the existing Oracle CI jobs with Phase 10-only conditions and dynamic Phase 10 authority names, leaving the Phase 9 step names, commands, and conditions unchanged."

patterns-established:
  - "Identity-last runner: clean a bounded target directory, emit payloads and manifest, validate content, hash the closed file set, then publish identity.json."
  - "Non-circular artifact authority: immutable archived bytes contain a zero pre-upload ID while run.json owns the externally assigned live ID and archive binding."

requirements-completed: [PART-18, TEST-01, TEST-02, TEST-04]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 10-2026-07-19T05-17-27
generated_at: 2026-07-21T12:36:07Z

duration: 37m
completed: 2026-07-21
---

# Phase 10 Plan 30: Local and Same-Run Evidence Production Summary

**Five-case Phase 10 evidence is now generated identity-last for local canonical and sanitizer validation, while one manual Oracle CI run can publish the exact same-SHA D1 authority pair consumed by exact-ref validation.**

## Performance

- **Duration:** 37m
- **Started:** 2026-07-21T11:59:00Z
- **Completed:** 2026-07-21T12:36:07Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments

- Added a fail-fast, symlink-aware Phase 10 runner that generates actual native, selected-oracle, comparison, replay, debug, release, deliberate-divergence, and inherited proof payloads; validates provenance, inventory, read-only state, content, and file digests; and writes identity last.
- Generated fresh local canonical and ASan/UBSan directories and validated the complete shared pair: 5 cases and 80 semantic leaves, explicitly classified as D2/D0 and non-promotable.
- Added thin `just` recipes for local canonical generation, sanitizer generation, and pair validation.
- Extended `Oracle CI` with a manual `phase10` choice that produces exact canonical and fail-fast sanitizer jobs and artifacts from one run/SHA on the locked Linux x86_64, Rust 1.97.0, Clang 22.1.8 stack.
- Added a workflow contract regression covering exact names, one runner invocation per mode, retention, least permissions, and full-SHA action pins, plus documented dispatch, exact-ref acquisition, and D3 promotion boundaries.

## Task Commits

Each task was committed atomically:

1. **Task 1: Produce and validate fresh local D2 artifacts** - `bc943c8` (feat)
1. **Task 2: Add one same-run canonical D1 workflow pair** - `9b74746` (ci)

## Files Created/Modified

- `scripts/phase10-evidence.sh` - Runs the closed corpus, fail-fast checks, file hashing, and identity-last publication for canonical or sanitizer mode.
- `crates/liquidfun-differential/tests/phase10_corpus/evidence_output.rs` - Serializes actual bounded proof payloads and the sealed Phase 10 manifest.
- `crates/liquidfun-differential/tests/phase10_corpus.rs` - Selects the requested reviewed oracle preset and activates evidence capture for the fixed corpus.
- `justfile` - Exposes thin local Phase 10 generation and validation commands.
- `.github/workflows/oracle.yml` - Adds the manual Phase 10 same-run canonical/sanitizer pair and exact artifact uploads.
- `tools/xtask/src/phase10_evidence/authority.rs` - Validates the immutable pre-upload artifact-ID sentinel against externally captured live artifact authority.
- `tools/xtask/tests/phase10_evidence_cli.rs` - Locks the workflow contract and rejects post-upload IDs asserted inside archived identities.
- `tools/xtask/tests/phase10_evidence_cli/exact.rs` - Builds realistic sentinel-bearing exact-ref fixtures.
- `TESTING.md` - Documents D0-D3, local non-promotion, dispatch, exact names, exact-ref acquisition, and external artifact binding.
- `.mdformat.toml` and five repository-owned Markdown files - Repair the pre-existing aggregate Markdown baseline without formatting managed/generated surfaces.
- Phase 9 and Phase 10 provenance records - Refresh the adapter digest after previously committed adapter changes.

## Decisions Made

- Actual corpus execution owns evidence serialization; the shell runner remains an orchestration boundary and never fabricates semantic physics payloads.
- A local identity deliberately uses run/artifact ID zero and local labels, making D2 output structurally unable to pass D1 exact-ref authority.
- A CI archive also retains artifact ID zero because no post-upload GitHub artifact ID exists while immutable identity bytes are being built. The exact-ref validator instead proves the downloaded archive against live API ID, name, digest, size, timestamps, and same-run metadata in `run.json`.
- Phase 10 reuses the existing canonical and sanitizer jobs. Phase 9 conditions and commands remain intact, while Phase 10-only job display names satisfy the exact validator contract.

## Threat Model Outcomes

- **T-10-30-01 Tampering:** Fixed modes, target containment, symlink rejection, `pipefail`, closed payload sets, recomputed digests, and identity-last publication reject partial or substituted evidence.
- **T-10-30-02 Spoofing:** Locked toolchain checks, exact job/artifact names, same run/SHA identity, live API binding, and full-SHA action pins constrain D1 authority.
- **T-10-30-03 Repudiation:** Explicit local/exact identity modes and D0-D3 documentation preserve evidence-tier meaning.
- **T-10-30-04 Denial of service:** The fixed five-case corpus, existing 45-minute job timeouts, manual/scheduled expensive lane, and finite retention keep work bounded.
- **T-10-30-05 Information disclosure:** Repository-level `contents: read` permissions and fixed commands avoid secrets and unbounded environment output.
- **T-10-30-06 Elevation of privilege:** Fixed workflow arguments and fully pinned actions prevent contributor-controlled executable/action references.
- **T-10-30-07 Partial publication:** Any test, sanitizer, provenance, inventory, read-only, content, or hashing failure exits before identity or upload.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added actual corpus evidence output and reviewed-preset selection**

- **Found during:** Task 1 (Produce and validate fresh local D2 artifacts)
- **Issue:** The existing Phase 10 corpus neither emitted the Plan 29 artifact contract nor selected the sanitizer oracle, so the planned runner could not generate or validate real artifacts.
- **Fix:** Added a bounded evidence-output module and mode-selected reviewed oracle execution to the existing corpus test.
- **Files modified:** `crates/liquidfun-differential/tests/phase10_corpus.rs`, `crates/liquidfun-differential/tests/phase10_corpus/evidence_output.rs`
- **Verification:** Fresh canonical and ASan/UBSan generation both passed; local pair validation reported 5 cases and 80 semantic leaves.
- **Committed in:** `bc943c8`

**2. [Rule 3 - Blocking] Refreshed stale adapter provenance digests**

- **Found during:** Task 1 provenance verification
- **Issue:** Earlier Phase 10 adapter changes left the checked Phase 9 and Phase 10 provenance records on an obsolete adapter SHA-256, so `cargo xtask provenance check` failed before evidence generation.
- **Fix:** Recomputed and updated only the two adapter-content digest fields.
- **Files modified:** `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json`, `reference/artifacts/phase10/group-topology-witnesses.provenance.json`
- **Verification:** Provenance checks passed inside both canonical and sanitizer evidence runs.
- **Committed in:** `bc943c8`

**3. [Rule 3 - Blocking] Repaired the repository Markdown baseline**

- **Found during:** Task 1 aggregate Markdown verification
- **Issue:** `just markdown-check` failed on five repository-owned documents plus managed/generated Markdown that must not be bulk-formatted.
- **Fix:** Formatted the five repository-owned documents with mdformat 1.0.0 under Python 3.13 and excluded managed/generated `AGENTS.md`, `CLAUDE.md`, and `COMPATIBILITY.md`.
- **Files modified:** `.mdformat.toml`, `ARCHITECTURE.md`, `UPSTREAM.md`, `THIRD_PARTY_NOTICES.md`, `standards-overrides.md`, `docs/decisions/0001-oracle-selection.md`
- **Verification:** Aggregate `just markdown-check` passed before both task commits.
- **Committed in:** `bc943c8`

**4. [Rule 1 - Correctness] Removed impossible circular artifact-ID self-binding**

- **Found during:** Task 2 workflow wiring
- **Issue:** GitHub assigns an artifact ID only after upload, but Plan 29 required that post-upload ID inside the immutable pre-upload identity. No workflow could satisfy that circular contract.
- **Fix:** Reserved embedded artifact ID zero as a pre-upload sentinel and retained authoritative binding through independently captured live ID, exact name, API/archive digest, byte size, timestamps, archive bytes, and run metadata.
- **Files modified:** `tools/xtask/src/phase10_evidence/authority.rs`, `tools/xtask/tests/phase10_evidence_cli.rs`, `tools/xtask/tests/phase10_evidence_cli/exact.rs`, `TESTING.md`
- **Verification:** The full 11-test adversarial validator suite accepts a valid sentinel pair and rejects a nonzero ID asserted inside an archive; the workflow contract and full Rust gate pass.
- **Committed in:** `9b74746`

**Total deviations:** 4 auto-fixed (1 correctness, 3 blocking)
**Impact on plan:** Every change was required to make the planned evidence path executable, non-circular, and repository-gated; compatibility authority and Phase 9 behavior remain unchanged.

## Issues Encountered

- The first local sanitizer attempt used a stale native sanitizer binary that predated the Phase 10 declarations. Reconfiguring and rebuilding `oracle-asan-ubsan` resolved it; the exact planned generation command then passed without source changes.
- The local CMake 3.27.9 and AppleClang 21 tool identities correctly remained D2 and emitted expected noncanonical warnings; no local output was relabeled as D1.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 10-31 can push the reviewed commits, dispatch `evidence_phase=phase10` once, capture the two exact same-run jobs/artifacts, and validate their live metadata and archive bytes without changing the evidence schema.
- The local corpus, sanitizer behavior, workflow contract, and exact-ref sentinel model are fully tested. No implementation blocker remains.

## Self-Check: PASSED

- Confirmed Task 1 commit `bc943c8` and Task 2 commit `9b74746` exist and are atomic.
- Confirmed fresh local canonical and sanitizer output validates as 5 cases and 80 semantic leaves in local mode.
- Confirmed `actionlint`, the workflow contract test, the complete 11-test adversarial validator suite, and aggregate Markdown checks pass.
- Confirmed the plan key-link verifier passes 1/1 for both workflow jobs invoking the shared runner.
- Confirmed each task commit was preceded by its own exact ordered Rust gate: format, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests.
- Confirmed `.planning/config.json`, `.planning/agent-history.json`, and `.planning/current-agent-id.txt` were not staged or committed.

*Phase: 10-particle-groups-solvers-and-compatibility-sign-off*
*Completed: 2026-07-21*
