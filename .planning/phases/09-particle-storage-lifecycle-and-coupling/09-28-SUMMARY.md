---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "28"
subsystem: phase9-exact-ref-recovery
tags: [particles, differential, oracle, sanitizer, exact-ref, recovery]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "27"
    provides: portable retained-rigid and semantic Phase 9 evidence validator
provides:
  - hermetic retained-process fixture independent of ambient oracle-debug output
  - Phase 09-only one-time authorization exception bound to one immutable recovery SHA
  - one successful exact-ref canonical and fail-fast sanitizer artifact pair for seven cases and 58 semantic bindings
affects: [phase-09-verification, exact-ref-promotion-review]
tech-stack:
  added: []
  patterns: [hermetic fake-oracle compile database, scoped one-shot dispatch authority, same-run exact-ref pairing]
key-files:
  created:
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-28-SUMMARY.md
  modified:
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-28-PLAN.md
    - .codex/tasks/todo.md
key-decisions:
  - "Bind the user's one-time recovery authorization only to Phase 09 Plan 28 and immutable SHA 22b31c0e1be8896df622b1decd58ba2853a60b04; global approval policy remains unchanged."
  - "Reject failed run 29625083184 and canonical artifact 8423580554 as authority and never mix either with replacement evidence."
  - "Accept only canonical and sanitizer artifacts from successful run 29652578231 after safe-archive and exact-ref validation with all three historical runs denied."
patterns-established:
  - "Recovery evidence remains non-promotable until one clean authorized SHA owns one successful same-run canonical/sanitizer pair."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T16:52:00Z
duration: 15h 16m
completed: 2026-07-18
---

# Phase 9 Plan 28: Sanitizer Evidence Recovery Summary

**A hermetic retained-process fixture and scoped one-shot authorization produced one successful, independently validated Phase 9 canonical/sanitizer evidence pair without compatibility promotion.**

## Performance

- **Duration:** 15h 16m, including the authorization pause
- **Started:** 2026-07-18T01:36:00Z
- **Completed:** 2026-07-18T16:52:00Z
- **Tasks:** 3
- **Files modified:** 4 tracked implementation and lifecycle files

## Accomplishments

- Replaced the retained-process regression's ambient `target/reference/oracle-debug/compile_commands.json` dependency with a deterministic fake-root compile database containing exactly `collision_probe.cpp`, `math_probe.cpp`, `protocol_bits.cpp`, and `rigid_world.cpp`.
- Proved the fake compile database has four unique debug units, a valid effective-command digest, and works while the real workspace debug compile database is unavailable.
- Recorded failed run `29625083184`, sanitizer job `88027761125`, old SHA `7ed430c497efbaa8585ee9ef3862be1abda29ef5`, and canonical-only artifact `8423580554` as rejected authority.
- Published the reviewed recovery base `1cebf2f8b2b5c7f002e182c6313490cb7b87a1dc`, then recorded the user's Phase 09-only exception at clean local/remote SHA `22b31c0e1be8896df622b1decd58ba2853a60b04`.
- Re-proved zero existing `workflow_dispatch` runs for the exception-authorized SHA and dispatched `oracle.yml` exactly once at `2026-07-18T16:45:02Z`.
- Watched replacement run `29652578231` to success, downloaded only its exact Phase 9 artifacts after safe archive preflight, and preserved the prior local evidence under `target/phase9-evidence/superseded/29583793056`.
- Validated seven cases, 58 semantic bindings, 22 ordered policies per case, retained Phase 8 rigid equality, every request/native/oracle/comparison/binding hash, passing logs, and canonical/sanitizer manifest equality.
- Left `COMPATIBILITY.md`, the artifact ledger, and all Phase 09 promotion claims unchanged.

## Task Commits

1. **Task 1: Repair the hermetic process fixture and publish one clean recovery target** - `2c49194`, `1cebf2f`
2. **Task 2: Apply the one-time Phase 09 recovery authorization exception** - `22b31c0`
3. **Task 3: Dispatch exactly once and validate fresh canonical and sanitizer artifacts** - external run and local ignored evidence; captured by the plan metadata commit

**Plan metadata:** committed after summary, state, roadmap, requirement, and verification updates.

## Replacement Authority

| Evidence | Canonical | Sanitizer |
| --- | --- | --- |
| Run | `29652578231` | `29652578231` |
| Job | `88101300857` (`canonical-linux`) | `88101300845` (`sanitizer-linux`) |
| Artifact | `8431920189` | `8431922578` |
| Artifact name | `phase9-canonical-29652578231-22b31c0e1be8896df622b1decd58ba2853a60b04` | `phase9-sanitizer-29652578231-22b31c0e1be8896df622b1decd58ba2853a60b04` |
| Archive SHA-256 | `ea333de6ac32d64c1c5b4e80738275451f0e51994b7f78e70961597d48e77500` | `99fa817d3b891a8942709e4b4af2bd4fa0aedbde0fc4c19b398829f02128a6c6` |
| Archive size | 184704 bytes | 184706 bytes |
| Trace SHA-256 | `2400f9b5dc69c9b07510ff934b1f41a455cdac71f3a3d7c5b8a372bf588316a9` | `f1f7d6cd2b2d6730fd4548cdfc643e3be7347613fe082f295b90622afe08d6ea` |
| Manifest SHA-256 | `662b9514472c1d6d8186115577f43c5987870a2a24592156b46631f1c28b4a3e` | `662b9514472c1d6d8186115577f43c5987870a2a24592156b46631f1c28b4a3e` |
| Test result | 25 passed, 0 failed, 1 ignored | 25 passed, 0 failed, 1 ignored |

Both artifacts bind exact head `22b31c0e1be8896df622b1decd58ba2853a60b04`, upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`, Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, Linux x86_64, and `phase9-v1`.

The byte-identical manifests have semantic digest `671d16f1c7af0f948760b9cdc62b3ed1fefb7307889a46334230605365aefe80`. Every case reports retained outcome `match` with Phase 6, 7, and 8 policy digests `7f10df148852866fd20d11b8d27adcddc0ad463ac3d3d716a8946ca5c8f1c63a`, `fd772b2cf523a6d40bf978bc4d0da18a4564181a93e6b2bdeb8e4d40d5613311`, and `2843ca40bec5b1c680135664c58c12a8388a7a9e86ad77f8ef5a268f3f15a6bf`.

## Verification

- Focused retained-process and closed fake compile-database regressions passed.
- The exact ordered Rust gate passed before the recovery and exception-record commits.
- Debug/release canonical and fail-fast ASan/UBSan local corpora passed before publication.
- Provenance, inventory, cargo-deny, workflow, Markdown, schema-drift, upstream read-only, and diff gates passed before publication.
- Immediately before dispatch, local HEAD, remote `main`, and the exception-authorized SHA were equal; the tree was clean and the matching dispatch count was zero.
- Run `29652578231` completed successfully with exactly one successful required canonical job and one successful required sanitizer job.
- Each archive had 34 safe relative regular files, no symlinks, the exact manifest-derived bounded file set, and bytes matching the live GitHub API size and SHA-256.
- `cargo xtask phase9-evidence validate --mode exact-ref` passed with runs `29439515367`, `29583793056`, and `29625083184` denied.
- `cargo xtask provenance check` passed after exact-ref validation.
- `git diff --exit-code -- COMPATIBILITY.md reference/artifacts/manifest.toml` passed.

## Decisions Made

- The user's exception changes no global or later-phase approval rule. It authorizes only the one Plan 09-28 dispatch for the SHA containing the hermetic repair and exception record.
- Failed run `29625083184` remains historical. Its successful canonical job cannot be paired with any other run's sanitizer result.
- Live run, job, and artifact API data were snapshotted into ignored `target/phase9-evidence/run.json`; the validator recomputed archive and semantic content independently.
- This plan establishes review-ready exact-ref authority only. Compatibility promotion remains separate Plan 09-29 work.

## Deviations from Plan

None - the recovery plan executed exactly as amended by the user's scoped authorization exception.

## Issues Encountered

- Failed run `29625083184` had one successful canonical job but no Phase 9 sanitizer artifact. Sanitizer job `88027761125` failed because the retained-process fixture tried to copy an ambient debug compile database absent from the sanitizer job's cache/build topology.
- The recovery made the fixture hermetic, and the replacement run passed without retry or redispatch.

## Known Stubs

None.

## Authentication Gates

None.

## User Setup Required

None.

## Next Phase Readiness

- Exact-ref replacement evidence is ready for independent promotion review.
- Runs `29439515367`, `29583793056`, and `29625083184` remain denied as new authority.
- Plan 09-29 remains pending and was not executed.
- No compatibility or ledger file changed in Plan 09-28.

## Self-Check: PASSED

- The hermetic fixture, Plan 09-28 lifecycle files, and this summary exist.
- Task commits `2c49194`, `1cebf2f`, and `22b31c0` exist in repository history.
- Run `29652578231`, jobs `88101300857` and `88101300845`, and artifacts `8431920189` and `8431922578` were re-queried live and validated.
- The exact-ref validator and provenance check passed on the fresh canonical/sanitizer pair.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-18*
