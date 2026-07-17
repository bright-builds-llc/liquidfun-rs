---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "23"
subsystem: phase9-exact-ref-evidence
tags: [github-actions, particles, differential, sanitizer, provenance]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "22"
    provides: fail-closed executable Phase 9 canonical and sanitizer evidence workflow
provides:
  - human-approved immutable Phase 9 evidence target
  - exact-ref successful canonical and sanitizer authority pair
  - downloaded identity-bound artifacts with recomputed payload hashes
affects: [09-24, phase-10, compatibility-promotion]
tech-stack:
  added: []
  patterns: [full-SHA approval gate, post-dispatch run cardinality, artifact identity revalidation]
key-files:
  created:
    - target/phase9-evidence/run.json
    - target/phase9-evidence/phase9-canonical/identity.json
    - target/phase9-evidence/phase9-sanitizer/identity.json
  modified: []
key-decisions:
  - "Accept only workflow run 29583793056 for human-approved full SHA b27fc14f6b29fb82ca815fa1effba71bae09d424; superseded run 29439515367 remains rejected."
  - "Treat downloaded canonical and sanitizer artifacts as evidence inputs only; Plan 09-23 makes no compatibility or ledger change."
patterns-established:
  - "Exact-ref evidence requires literal 40-character approval, clean local/remote equality, one later matching dispatch, unique successful authority jobs, and live API-bound artifact digests."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-17T13:33:47Z
duration: 22 min
completed: 2026-07-17
---

# Phase 9 Plan 23: Exact-Ref Canonical and Sanitizer Evidence Summary

**Human-approved commit `b27fc14f6b29fb82ca815fa1effba71bae09d424` now owns one successful canonical/sanitizer authority pair whose fresh artifacts prove all seven cases, 58 branches, 22 policies per case, and digest-bound native/C++ agreement.**

## Performance

- **Duration:** 22 min
- **Started:** 2026-07-17T13:11:44Z
- **Completed:** 2026-07-17T13:33:47Z
- **Tasks:** 3
- **Files modified:** 3 tracked planning files plus ignored downloaded evidence

## Accomplishments

- Published and received explicit human approval for immutable full SHA `b27fc14f6b29fb82ca815fa1effba71bae09d424` on clean local and remote `main`.
- Dispatched `Oracle CI` exactly once with `evidence_phase=phase9` at `2026-07-17T13:23:47Z` and selected exactly one later full-SHA run.
- Required one successful `Canonical Linux oracle` job and one successful `Scheduled fail-fast sanitizer and reset corpus` job.
- Downloaded both exact unexpired artifacts into fresh fixed directories and revalidated safe paths, identities, API digests, embedded hashes, passing logs, provenance, inventory, and read-only boundaries.
- Proved canonical and sanitizer manifests are byte-identical and contain seven distinct cases, exactly 58 distinct retained branches, every one of the 22 policy paths per case, and valid request/native/oracle/comparison SHA-256 values.
- Left `COMPATIBILITY.md`, protocol, scenarios, reference evidence, and compatibility inventory ledgers unchanged.

## Task Commits

Plan tasks were evidence and approval operations rather than code changes:

1. **Task 1: Publish a clean exact evidence target** - `b27fc14` (published immutable target)
1. **Task 2: Approve the exact published gap-fix SHA** - human approval recorded verbatim; no commit
1. **Task 3: Dispatch and validate new exact-ref artifacts** - run `29583793056`; no implementation commit

**Plan metadata:** committed after summary and state tracking verification.

## Exact Run and Artifact Evidence

- **Repository / branch:** `bright-builds-llc/liquidfun-rs` / `main`
- **Approved and executed SHA:** `b27fc14f6b29fb82ca815fa1effba71bae09d424`
- **Run:** [29583793056](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29583793056), `workflow_dispatch`, successful
- **Canonical job:** [87895618287](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29583793056/job/87895618287), exactly one successful `Canonical Linux oracle`
- **Sanitizer job:** [87895618338](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29583793056/job/87895618338), exactly one successful `Scheduled fail-fast sanitizer and reset corpus`
- **Canonical artifact:** `phase9-canonical-29583793056-b27fc14f6b29fb82ca815fa1effba71bae09d424`, artifact ID `8408156562`, API digest `sha256:faaf24c870826251f0dd1d507ba9c335269b78433ba1ce2ee0e1995336f0139a`
- **Sanitizer artifact:** `phase9-sanitizer-29583793056-b27fc14f6b29fb82ca815fa1effba71bae09d424`, artifact ID `8408174081`, API digest `sha256:f4b30cebed7b81a41282a33d45b81231485a2fa0c3a958c7b68a3ecbad086e7c`
- **Canonical trace:** `5c6805e0e998394947439bf6eb295526130cb4db81a67e0f560c6d6bc3f33545`
- **Sanitizer trace:** `4261dbe8993155dd7ab9e7992f90bef60e57762c36a66ea97b3bc9131804508a`
- **Shared byte-identical manifest:** `b7bb43a6ce083fe543bf6eb3f92b1b4f663d4bd6520dbb83c3a00072a3010a8b`
- **Identity:** run `29583793056`, upstream `7f20402173fd143a3988c921bc384459c6a858f2`, Rust `1.97.0`, CMake `4.3.3`, Ninja `1.13.2`, Clang `22.1.8`, target `x86_64-unknown-linux-gnu`, policy `phase9-v1`

Both traces contain `test result: ok. 7 passed; 0 failed; 1 ignored` and contain no `FAILED` marker. Both inventory logs verify 177 compatibility rows, both provenance logs bind the pinned upstream and reviewed Phase 9 witness, and both read-only logs prove a zero diff.

## Digest-Bound Executed Cases

| Case | Request SHA-256 | Native result SHA-256 | Oracle result SHA-256 | Comparison SHA-256 |
| --- | --- | --- | --- | --- |
| `storage-systems-and-permutations` | `bdc12fdb6cc4903250056c46deb0967e03c5fa09dccb7abf7b5e6239f48139fc` | `ee69306d43942b754f555d7da11904a7426eda1152bfac7b4c7065e59bec29ff` | `4e162571272e508120da72ce6bc8e43869df600bc261c3c1ca7ea483ad836801` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `lifetime-zombie-and-eviction` | `a043268d18dcb3413da9e2c12fb9f498d7c183ac05e5af29affa75510e149317` | `3cc4f3f585f86f4042eb37e7c523722837b8dc8abe1d311eaadcd83c8777657f` | `806c960136fd2662ffe894be78fcea271c696a72025ad1cf2b072f9753345b5f` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `contacts-listeners-filters-and-coupling` | `d2a3436885de5b1f7d78f4580d3644ca74dcb6a326f3131ac3932ac135e59780` | `eb30d7cbf38a6724f0ef6ced24f94921034488a0775c6780fe584666034646d9` | `f2d36d50b3ad11bfc4aaad61031f6997a8b6591085d1522c6545f70a7ff9d165` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `forces-impulses-and-statistics` | `a11256e22311ac52a16ff5e369190543f5a84cf50c303b89f5a7a8e812241243` | `fceeb8cffc5d5af12764075a1860427141dd086c0d3c539b7d6e3dd6af37ae93` | `c4a03700558cf357ea894dd7e0f1f5f5969d6e46cf002a8721a7bf03e505c9d3` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `aabb-query-control-and-culling` | `47a380b63d7a778cf3387a3146b7d990e9ab90f4165123d33e7377efd53c1870` | `a1c26f0ed23b84b1cac70c8cc462d9ed4846bb1dbdd2cdd5f743893ebca2e44b` | `8f56c29c3e887235c121bea084d45dd32646513598ef284cbab10148316997db` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `ray-control-and-culling` | `641e4dd21994d2374ac921ed6bf59ba537993895d4cbd6f59ea104033d1521c7` | `a0edb7ef385322d0af2e30fb9e2357c2d165736bb5d571391840396f8723fb1a` | `b2d6294662f2986184dd4f58833c4ec6a3df4a130809657d4ed026e8ee75d4b7` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |
| `closed-evidence-contract` | `74df5b6c5462fcb21e528ba16d7e6471418ad1b303179d38d353e9251a144d98` | `80f4f7b2f69746691abf50066bae12afabd9924e13377fe9d7f5d94ce958c958` | `e3e6ff696d2cc6f1587a25e788949f007cf80a89ad999e1795bdb0ddc0254aa2` | `356ec35dea44c8f3cdcccdf7070a34ca2c83ca8a1cd38ba0c7a0f1fc3d57eedb` |

Each request digest was recomputed from the checked-in fixture. Both manifests bind the same ordered branch witnesses and digest set.

## Files Created/Modified

- `target/phase9-evidence/run.json` - Records approval, dispatch boundary, run, jobs, artifact URLs, and live API digests.
- `target/phase9-evidence/phase9-canonical/` - Fresh downloaded canonical identity, logs, and manifest.
- `target/phase9-evidence/phase9-sanitizer/` - Fresh downloaded sanitizer identity, logs, and manifest.
- `.planning/phases/09-particle-storage-lifecycle-and-coupling/09-23-SUMMARY.md` - Records exact-ref evidence and authority limits.
- `.planning/STATE.md` and `.planning/ROADMAP.md` - Advance GSD plan tracking without changing compatibility claims.

## Decisions Made

- The literal human approval applies only to full SHA `b27fc14f6b29fb82ca815fa1effba71bae09d424`; changed, short, dirty, or mismatched refs remain unauthorized.
- Run `29583793056` is the only accepted post-dispatch run. Rejected run `29439515367` and its artifacts remain explicitly superseded.
- Empty read-only logs are successful evidence because `git diff --exit-code` produced no output and returned zero.
- Plan 09-24 retains sole authority for any compatibility promotion. This plan records evidence but does not mutate a compatibility or ledger file.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The fixed target directory still contained the rejected 2026-07-15 run. It was preserved under `target/phase9-evidence-superseded-29439515367`, and the accepted run was downloaded into a newly created fixed directory.
- The first local aggregate validation used GNU `find -printf`, which is unavailable on macOS. The check was rewritten with portable `find -exec basename`, then the complete validator passed without changing the evidence.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 09-24 can independently inspect the exact run, jobs, artifact API digests, identities, logs, manifests, and recomputed hashes before any promotion decision.
- Phase 9 remains unpromoted until Plan 09-24 completes its explicit compatibility review and ledger transaction.
- Cross-engine stable-ID rotation remains deferred to Phase 10 as approved in Plan 09-22.

## Self-Check: PASSED

- The approved full SHA remained equal to clean local HEAD and remote `main` immediately before dispatch.
- `gh run watch --exit-status` returned success for run `29583793056`.
- Live run and artifact APIs prove exact workflow/event/ref identity, unique successful authority jobs, exact artifact names, unexpired status, and recorded archive digests.
- Artifact paths are regular and safe; identities and recomputed trace/manifest hashes match.
- Both logs passed; both manifests are byte-identical and contain seven cases, 58 distinct branches, 22 distinct policy paths per case, and valid request/native/oracle/comparison digests.
- `git diff --exit-code -- protocol scenarios reference COMPATIBILITY.md` passed, and the worktree was clean before planning metadata.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-17*
