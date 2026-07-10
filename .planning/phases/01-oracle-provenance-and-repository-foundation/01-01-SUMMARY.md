---
phase: 01-oracle-provenance-and-repository-foundation
plan: "01"
subsystem: provenance
tags: [liquidfun, git-submodule, provenance, licensing]

requires: []
provides:
  - Immutable official LiquidFun oracle gitlink and machine-readable identity lock
  - Accepted release-to-candidate oracle decision with Box2D ancestry evidence
  - Read-only upstream workflow, source-map schema, and third-party notice policy
affects: [01-02, 01-03, 01-04, oracle, inventory, licensing]

tech-stack:
  added: [official Google LiquidFun git submodule]
  patterns: [immutable upstream pins, wrapper-first compatibility, machine-readable provenance]

key-files:
  created:
    - .gitmodules
    - third_party/liquidfun
    - reference/upstream-lock.toml
    - reference/source-map.toml
    - docs/decisions/0001-oracle-selection.md
    - UPSTREAM.md
    - THIRD_PARTY_NOTICES.md
  modified: []

key-decisions:
  - "Use official commit 7f20402173fd143a3988c921bc384459c6a858f2 as the behavioral oracle because it preserves the v1.1.0 lineage while adding material native correctness and filtering fixes."
  - "Keep upstream read-only and prefer repository-owned wrapper compatibility; any unavoidable source patch requires an external hashed register."
  - "Require source-map provenance and alteration records for every future derived artifact; the root MIT license does not replace upstream duties."

patterns-established:
  - "Immutable oracle identity: gitlink, checkout, lock, ADR, and documentation must agree on full commit identities."
  - "Derived-material gate: local derivations require upstream revision/path, derivation kind, alteration summary, and notice class."

requirements-completed:
  - FND-01
  - FND-02
  - FND-04
  - DOCS-03
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T02:42:17Z

duration: 6 min
completed: 2026-07-10
---

# Phase 1 Plan 01: Oracle Selection and Provenance Summary

**Official LiquidFun commit `7f20402173fd143a3988c921bc384459c6a858f2` pinned with an auditable release delta, read-only workflow, and machine-readable provenance and notice policy**

## Performance

- **Duration:** 6 min
- **Started:** 2026-07-10T02:35:57Z
- **Completed:** 2026-07-10T02:42:17Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Pinned the complete official LiquidFun repository as a detached submodule and
  recorded the annotated `v1.1.0` tag object, peeled release commit, selected
  revision, submodule path, and `patch_set = "none"` in a TOML lock.
- Accepted the official post-release commit after documenting its material
  particle-group split fix, fixture/particle filtering and test, growable-buffer
  integration, build/warning corrections, and bindings/documentation changes.
- Preserved both pinned LiquidFun/Box2D notices verbatim and established a
  versioned source/alteration mapping contract before translated work begins.

## Task Commits

Each task was committed atomically:

1. **Task 1: Pin the selected official upstream commit** - `90fce91`
   (`chore`)
2. **Task 2: Record the oracle decision and maintainer workflow** - `94dd765`
   (`docs`)
3. **Task 3: Establish source mapping and notice policy** - `dc6415c`
   (`docs`)

## Files Created/Modified

- `.gitmodules` - Official repository URL and branchless submodule path.
- `third_party/liquidfun` - Gitlink pinned at the selected official commit.
- `reference/upstream-lock.toml` - Exact tag, release, revision, path, and patch
  identity.
- `reference/source-map.toml` - Versioned derivation and alteration schema with
  an explicitly non-derived bootstrap record.
- `docs/decisions/0001-oracle-selection.md` - Accepted release-to-candidate
  delta audit and oracle rationale.
- `UPSTREAM.md` - Initialize, verify, intentional update, build, patch, and
  license workflow.
- `THIRD_PARTY_NOTICES.md` - Preserved upstream notices and developer-only
  GoogleTest/freeglut package policy.

## Verification Evidence

- `git submodule status third_party/liquidfun` reported
  `7f20402173fd143a3988c921bc384459c6a858f2` with a clean status prefix.
- `git ls-tree HEAD third_party/liquidfun` and
  `git -C third_party/liquidfun rev-parse HEAD` both matched the lock revision.
- The annotated tag object resolved to
  `d15bcf1879144bf2a4c8ebcc73f6418186756fb2` and peeled to
  `f38db7c627c3dc5ec879d726e16fa5a12ad6e478`.
- Python `tomllib` checks parsed both TOML files and asserted all exact lock and
  required mapping values.
- An exact text comparison proved the two embedded notices match the pinned
  `liquidfun/Box2D/License.txt` and `liquidfun/NOTICE` files.
- Targeted `rg` checks covered the required identities, ancestry, delta
  classes, headings, ADR link, package policy, and alteration language.
- `mdformat --check` and `git diff --check` passed for the changed documentation.
- `git diff --name-only 4c4ecdf..HEAD` contained exactly the seven planned
  artifacts, and `git -C third_party/liquidfun status --short` was empty.

## Decisions Made

- Selected the official 2018 post-release commit, not the 2014 peeled release
  commit, because the bounded delta contains native fixes and a tested
  fixture/particle filtering behavior that belong in the canonical oracle.
- Kept all compatibility adaptations outside the submodule unless a future
  separately reviewed and hashed patch set becomes unavoidable.
- Classified the initial source-map entry as non-derived so the schema is
  exercised without falsely claiming that translated implementation exists.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for `01-02-PLAN.md` to build the Cargo-first workspace and prove
  consumer package isolation from the pinned upstream tree.
- The final licensing classification of future translated code remains a
  continuing legal-review caveat. This plan records the applicable notices and
  blocks unmapped or unclassified derived material; it does not make a final
  derivative-work legal conclusion.

## Self-Check: PASSED

- All seven key files and the pinned submodule exist.
- Three atomic `01-01` task commits are present in git history.
- Summary lifecycle metadata and completed requirements parse successfully.

***

_Phase: 01-oracle-provenance-and-repository-foundation_
_Completed: 2026-07-10_
