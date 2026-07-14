---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
plan: "23"
subsystem: canonical-evidence-publication
tags: [github-actions, canonical-evidence, sanitizers, windows, provenance]
requires:
  - phase: 08-22
    provides: reviewed Phase 8 implementation, closed comparator policy, and exact-identity Oracle workflow readiness
provides:
  - exact successful canonical and sanitizer evidence for commit 533c2ccf97b3921079baf7c339ddb4dad1a4038b
  - cross-platform-stable evidence checkout and upstream Clang compatibility policy
  - two unique unexpired SHA-scoped artifacts with validated toolchain, upstream, and phase8-v1 identities
affects: [08-24, canonical-evidence, windows-portability, rigid-sign-off]
tech-stack:
  added: []
  patterns: [exact-ref workflow dispatch, LF-normalized evidence checkout, upstream-only warning compatibility]
key-files:
  created:
    - .gitattributes
    - .planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-23-SUMMARY.md
  modified:
    - .github/workflows/oracle.yml
    - tools/reference/CMakeLists.txt
    - tools/xtask/tests/docs_contract.rs
    - tools/xtask/tests/upstream_cli.rs
key-decisions:
  - "Reject the first dispatch despite successful canonical and sanitizer jobs because the overall workflow conclusion was failure."
  - "Fix Windows evidence checkout and the read-only upstream warning at their platform boundaries instead of weakening provenance, inventory, or warning policy."
  - "Consume only run 29374708477 and its two exact SHA-scoped artifacts after every run, job, artifact, and identity check passed."
requirements-completed: [RIGD-11, JOIN-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T23:03:00Z
duration: 20min
completed: 2026-07-14
---

# Phase 8 Plan 23: Publish Reviewed Ref and Prove Canonical Evidence Summary

**The exact reviewed Phase 8 ref now has successful canonical, sanitizer, macOS, and Windows evidence, with two validated SHA-scoped artifacts ready for the final documentation sign-off.**

## Performance

- **Duration:** 20 min
- **Started:** 2026-07-14T22:43:00Z
- **Completed:** 2026-07-14T23:03:00Z
- **Tasks:** 1
- **Files modified:** 5

## Accomplishments

- Integrated the concurrent managed Bright Builds Rules update without rewriting the 56 reviewed Phase 8 commits, ran the complete pre-commit gate, and published one clean `main` head.
- Dispatched Oracle CI against exact reviewed commits and refused to treat successful required jobs as sufficient when the first workflow had an overall failure.
- Reproduced the Windows provenance mismatch byte-for-byte as LF-to-CRLF conversion, added LF checkout policy, and locked it with a cross-platform evidence contract.
- Scoped the modern Clang deprecated-copy exemption only to the read-only 2014 upstream target while retaining warning denial for repository-authored C++.
- Made the Windows PowerShell step fail immediately on native command errors and added a workflow contract for command ordering.
- Validated successful workflow-dispatch run `29374708477`, exact head `533c2ccf97b3921079baf7c339ddb4dad1a4038b`, both required jobs, exactly two unexpired artifacts, and both identity JSON records.

## Task Commits

1. **Publish the reviewed commit and integrate the concurrent managed-rules update** - `16205bc` (merge)
1. **Repair deterministic Windows Oracle portability** - `533c2cc` (fix)

## Files Created/Modified

- `.gitattributes` - Forces repository text to retain LF bytes on Windows evidence checkouts.
- `.github/workflows/oracle.yml` - Makes Windows native-command failures stop the verify/build step immediately.
- `tools/reference/CMakeLists.txt` - Keeps one modern deprecated-copy warning nonfatal only for the read-only upstream target.
- `tools/xtask/tests/upstream_cli.rs` - Locks LF checkout and upstream warning-compatibility contracts.
- `tools/xtask/tests/docs_contract.rs` - Locks Windows PowerShell fail-fast ordering before the first Cargo command.

## Decisions Made

- Required the complete workflow conclusion to be successful even though Plan 23's two evidence-producing jobs had passed in the first dispatch.
- Treated checkout byte stability and legacy-upstream warning compatibility as separate platform defects with narrow independent fixes.
- Used the second workflow-dispatch run only after its head SHA, event, conclusion, unique jobs, artifact set, and identities all matched.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Repaired Windows evidence byte stability**

- **Found during:** First exact-ref workflow dispatch `29374175882`
- **Issue:** Windows checkout conversion changed the reviewed trace SHA from `7f72f928...` to `240d0a18...` and made generated inventory bytes stale.
- **Fix:** Added repository-wide `text=auto eol=lf` policy and a regression contract for the evidence path.
- **Verification:** The reported Windows hash was reproduced exactly from CRLF bytes; local provenance and inventory checks passed; the corrected Windows job passed.
- **Committed in:** `533c2cc`

**2. [Rule 3 - Blocking] Isolated a modern Clang warning in the pinned upstream target**

- **Found during:** First exact-ref Windows build
- **Issue:** Clang-cl diagnosed the upstream `b2ParticleColor` implicit copy constructor under the upstream `/WX` policy.
- **Fix:** Added `-Wno-error=deprecated-copy-with-user-provided-copy` only to the read-only `Box2D` target and retained visible warnings plus repository warning denial.
- **Verification:** Local wrapper build passed and the corrected Windows Visual Studio/clang-cl job completed successfully.
- **Committed in:** `533c2cc`

**3. [Rule 2 - Missing Critical Functionality] Made the Windows PowerShell step fail fast**

- **Found during:** First exact-ref Windows logs
- **Issue:** Native Cargo failures did not stop the step, obscuring the first failure with later configure/build output.
- **Fix:** Enabled both PowerShell and native-command error preferences before tool execution and locked their order in a workflow test.
- **Verification:** The focused workflow contract and full Rust test suite passed; the corrected Windows job passed.
- **Committed in:** `533c2cc`

**Total deviations:** 3 auto-fixed (2 blocking, 1 missing critical functionality). **Impact:** The fixes make evidence portable and failures sharper without changing physics, tolerances, canonical identity, or maturity policy.

## Issues Encountered

- The push-triggered Oracle run for `16205bc` occupied the workflow concurrency group before the first manual dispatch. It was allowed to finish, but only the later `workflow_dispatch` run was considered for Plan 23.
- The first manual run failed solely because Windows portability failed; canonical Linux and sanitizer evidence were not consumed from that failed run.

## Validation Evidence

- Successful run: `https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29374708477`, event `workflow_dispatch`, conclusion `success`, exact head `533c2ccf97b3921079baf7c339ddb4dad1a4038b`.
- Unique required jobs: `Canonical Linux oracle` and `Scheduled fail-fast sanitizer and reset corpus`, both successful; Windows and macOS portability also passed.
- Exact artifacts: `phase8-canonical-29374708477-533c2ccf97b3921079baf7c339ddb4dad1a4038b` and `phase8-sanitizer-29374708477-533c2ccf97b3921079baf7c339ddb4dad1a4038b`, both unexpired and unique.
- Both identity records match run ID, head SHA, Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, upstream `7f20402173fd143a3988c921bc384459c6a858f2`, and policy `phase8-v1`.
- Regression tests observed red before the fix and green afterward for LF checkout, upstream warning compatibility, and Windows native-command fail-fast behavior.
- Local provenance, inventory, Oracle debug configure/build, Markdown, ordered Rust, and diff checks passed before publication.

## User Setup Required

None.

## Next Phase Readiness

- Plan 08-24 may consume only run `29374708477` and its exact validated canonical and sanitizer identities.
- The evidence remains scoped to the closed Phase 8 scalar rigid-body and joint corpus; broader parity and release claims remain pending.

## Self-Check: PASSED

- The successful run targets the exact remote `main` head and has a successful overall conclusion.
- Exactly two expected unexpired artifacts exist and both identity files pass the Plan 08-23 `jq` predicates.
- Repository state remained clean after artifact download under ignored `target/`.
- No evidence from run `29374175882` was used for the final sign-off.

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Completed: 2026-07-14*
