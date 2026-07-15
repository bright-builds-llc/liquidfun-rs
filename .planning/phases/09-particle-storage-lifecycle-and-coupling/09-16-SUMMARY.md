---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "16"
subsystem: testing
tags: [phase9-v1, github-actions, oracle, sanitizer, evidence]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: 14
    provides: closed Phase 9 corpus and exact-ref canonical/sanitizer workflow
provides:
  - human-approved immutable Phase 9 evidence target
  - successful exact-ref canonical and sanitizer evidence run
  - downloaded identity-bound artifacts with recomputed content digests
affects: [09-17, phase9-evidence, compatibility-promotion]
tech-stack:
  added: []
  patterns:
    - human approval of an immutable commit before remote evidence dispatch
    - exact run, job, artifact, identity, and payload digest validation
key-files:
  created:
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-16-SUMMARY.md
  modified: []
key-decisions:
  - "Bind Phase 9 evidence only to the human-approved full SHA after proving local and remote main equality."
  - "Accept remote evidence only when both required jobs, both exact artifact names, API digests, identity fields, safe paths, and recomputed payload hashes agree."
patterns-established:
  - "Exact-ref dispatch: record dispatch time, then select exactly one workflow_dispatch run by workflow, branch, full SHA, and creation time."
  - "Evidence validation: treat GitHub artifact metadata and downloaded content identity as separate fail-closed checks."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T18:15:10Z
duration: 4min
completed: 2026-07-15
---

# Phase 9 Plan 16: Exact-Ref Remote Evidence Summary

**Human-approved commit `a87f84bbdbfe55fb732d74c481c4a4bda9eec958` produced successful canonical and sanitizer evidence whose exact GitHub artifacts and downloaded payload hashes validate independently.**

## Performance

- **Duration:** 4 min
- **Started:** 2026-07-15T18:11:12Z
- **Completed:** 2026-07-15T18:15:10Z
- **Tasks:** 3
- **Tracked files modified:** 1

## Accomplishments

- Proved the approved SHA matched clean local `main` and remote `origin/main` before dispatching exactly once.
- Selected one unambiguous post-dispatch run and watched all jobs to a successful conclusion.
- Required exactly one successful canonical job and exactly one successful sanitizer job for the approved head SHA.
- Downloaded the two exact, unexpired Phase 9 artifacts and validated every identity field, artifact-relative path, and trace/manifest digest.

## Published Target and Dispatch

- **Repository:** `bright-builds-llc/liquidfun-rs`
- **Branch:** `main`
- **Human-approved SHA:** `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Dispatch timestamp:** `2026-07-15T18:11:12Z`
- **Run ID:** `29439515367`
- **Run URL:** <https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29439515367>
- **Event / workflow:** `workflow_dispatch` / `Oracle CI`
- **Run head SHA:** `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Conclusion:** `success`

The exact-run selector found one run created at `2026-07-15T18:11:14Z` whose workflow, event, branch, head SHA, and creation time matched the recorded dispatch.

## Job Results

| Job | Job ID | Started | Completed | Conclusion |
| --- | ---: | --- | --- | --- |
| Canonical Linux oracle | `87434583083` | `2026-07-15T18:11:18Z` | `2026-07-15T18:13:06Z` | `success` |
| Scheduled fail-fast sanitizer and reset corpus | `87434583043` | `2026-07-15T18:11:18Z` | `2026-07-15T18:14:03Z` | `success` |

Exactly one job with each required name completed successfully. The run's Windows and macOS portability jobs also succeeded but are not Phase 9 artifact authorities.

## Artifact Results

### Canonical

- **Name:** `phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Artifact ID:** `8352859391`
- **Archive URL:** <https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352859391/zip>
- **GitHub API digest:** `sha256:f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea`
- **Expired:** `false`
- **Identity job / policy:** `canonical-linux` / `phase9-v1`
- **Trace:** `phase9-trace.log` — `3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64`
- **Manifest:** `phase9-manifest.json` — `36cfaad1f56505f8427408733e2231ad613984a4cb3eb3b8d757e7a14b2c38e0`

### Sanitizer

- **Name:** `phase9-sanitizer-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`
- **Artifact ID:** `8352881868`
- **Archive URL:** <https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352881868/zip>
- **GitHub API digest:** `sha256:95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f`
- **Expired:** `false`
- **Identity job / policy:** `sanitizer-linux` / `phase9-v1`
- **Trace:** `phase9-trace.log` — `ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070`
- **Manifest:** `phase9-manifest.json` — `0c89f0136eda6689118d3eaa909defb1d182d5723e7a64ea1e958396066dce15`

## Identity Validation

Both downloaded `identity.json` files matched:

- Run ID `29439515367` and approved head SHA `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`.
- Pinned upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- Target `x86_64-unknown-linux-gnu`.
- Rust `1.97.0`, Clang `22.1.8`, CMake `4.3.3`, and Ninja `1.13.2`.
- Policy `phase9-v1` and the expected job identity for the containing artifact.
- Safe relative trace and manifest paths with lowercase 64-hex SHA-256 values.

`shasum -a 256 -c` independently recomputed and passed both named files in both artifacts.

## Task Commits and Evidence

1. **Tasks 1-2: Publish and confirm the exact evidence target** — approved commit `a87f84b`.
1. **Task 3: Dispatch, watch, download, and validate exact-ref evidence** — GitHub Actions run `29439515367`.

## Files Created/Modified

- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-16-SUMMARY.md` - Records immutable approval, exact remote run, jobs, artifact API identities, and independently validated downloaded payload identities.
- `target/phase9-evidence/run.json` - Local ignored diagnostic binding repository, branch, dispatch time, run, head, artifact names, archive URLs, and API digests.
- `target/phase9-evidence/phase9-canonical/` - Downloaded exact canonical evidence.
- `target/phase9-evidence/phase9-sanitizer/` - Downloaded exact sanitizer evidence.

## Decisions Made

- The user's approval was copied verbatim and accepted only after it equaled both local HEAD and remote `main` with a clean tracked worktree.
- GitHub's API-provided archive SHA-256 values were recorded separately from the content digests embedded in each downloaded `identity.json`.
- The checked workflow and `TESTING.md` contract use the identity field `policy`; validation therefore required `policy == "phase9-v1"`.

## Deviations from Plan

None - the successful attempt followed the exact-ref dispatch and fail-closed evidence contract without tracked implementation changes or compatibility promotion.

## Issues Encountered

- Earlier runs `29426101690` and `29427603066` targeted superseded SHAs and failed before this approval. They were not reused; the successful run was selected only from records created after this attempt's dispatch timestamp and matching the newly approved SHA.
- The first pre-dispatch origin comparison did not recognize the SSH `git@github.com:OWNER/REPO.git` syntax. It failed before dispatch. Normalizing that URL to an exact owner/repository slug proved equality, after which the single authorized dispatch occurred.

## Verification

- `gh run watch 29439515367 --exit-status` completed with `success`.
- Run metadata validated exact workflow, event, head SHA, conclusion, and required job cardinality.
- GitHub's artifacts API validated exact names, non-expiration, and lowercase `sha256:` archive digests.
- Both exact artifacts downloaded by name; all identity fields and safe paths passed.
- Both trace and manifest hashes in both artifacts recomputed successfully.
- Local and remote `main` remained at the approved SHA throughout evidence validation.

## User Setup Required

None.

## Next Phase Readiness

- Plan 09-17 can independently revalidate the downloaded exact-ref evidence and make its bounded compatibility decision.
- This plan records evidence only; it does not itself promote compatibility or change implementation behavior.

## Self-Check: PASSED

- Run `29439515367` is successful and bound to approved SHA `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`.
- Exactly one required canonical and sanitizer job succeeded.
- Both exact artifacts are present locally, unexpired in the API, and pass identity and content-digest validation.
- `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/config.json` were not modified.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
