---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "22"
subsystem: phase9-executable-differential-evidence
tags: [rust, cpp, particles, differential, sanitizer, github-actions]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "21"
    provides: same-request native-versus-C++ comparison and closed Phase 9 policy consumption
provides:
  - seven digest-bound cases with exact-once executable coverage of 58 retained Phase 9 branches
  - fail-closed canonical and sanitizer evidence runner with identity-last publication
  - exact-source sanitizer exception contract for the pinned upstream particle translation unit
affects: [09-23, 09-24, phase-10, oracle-ci]
tech-stack:
  added: []
  patterns: [semantic branch witnesses, identity-last evidence, effective compile-command scope testing]
key-files:
  created:
    - scripts/phase9-evidence.sh
    - tools/reference/tests/sanitizer_scope.cmake
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/
  modified:
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - tools/reference/src/rigid_world_phase9_execute.hpp
    - tools/reference/CMakeLists.txt
    - .github/workflows/oracle.yml
    - TESTING.md
key-decisions:
  - "Retain 58 executable Phase 9 branches and defer the first cross-engine stable-ID rotation witness to Phase 10, where public group behavior exposes it without a test-only seam."
  - "Permit `-fno-sanitize=shift-base` only for pinned upstream b2ParticleSystem.cpp under oracle-asan-ubsan, guarded by effective compile-command tests; keep all other sanitizer categories fail-fast."
  - "Create evidence identity only after passing corpus logs, complete branch/policy/digest validation, provenance, inventory, and read-only checks."
patterns-established:
  - "Every Phase 9 branch witness names an action, checkpoint, observation, and branch-specific semantic predicate over both validated engine results."
  - "Preset-specific compiler exceptions are verified against compile_commands.json for exact positive scope and negative non-leakage."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-17T12:26:31Z
duration: 9h 10m
completed: 2026-07-17
---

# Phase 9 Plan 22: Executable Differential Evidence Summary

**Seven same-request native/C++ cases now close all 58 retained Phase 9 branches with semantic observations, complete policy/digest manifests, and fail-closed canonical and sanitizer evidence.**

## Performance

- **Duration:** 9h 10m
- **Started:** 2026-07-17T03:16:42Z
- **Completed:** 2026-07-17T12:26:31Z
- **Tasks:** 3
- **Files modified:** 24

## Accomplishments

- Replaced label-only coverage with seven checked-in JSONL cases whose 58 retained branches are each reached exactly once by a branch-specific semantic witness.
- Executed identical digest-bound request bytes through native Rust and pinned C++, validated both results, required a `Match`, and recorded request/native/oracle/comparison SHA-256 values plus all 22 policy paths.
- Added truthful lifecycle and particle/body-contact observations, requested-versus-unrequested destruction evidence, and Phase 9-only checkpoint construction without weakening retained Phase 6-8 checkpoints.
- Added a Bash evidence runner whose `pipefail`, passing-log, manifest, provenance, inventory, and read-only gates must all succeed before `identity.json` can exist.
- Added an approved source-specific sanitizer exception for the pinned upstream negative neighbor shift and a compile-database CTest proving that it cannot leak to debug, release, another vendored source, or repository-authored code.
- Rebuilt debug, release, and ASan/UBSan oracles and produced clean local canonical and sanitizer manifests with seven cases, 58 unique branches, complete policies/digests, and identity-last evidence.

## Task Commits

Each task was committed atomically:

1. **Tasks 1-2: Execute every retained Phase 9 branch and emit the closed manifest** - `86666c1` (test)
1. **Task 3: Make canonical and sanitizer evidence publication fail closed** - `f86b9d7` (ci)
1. **Task 3 approved verification repair: Scope the pinned-upstream sanitizer exception** - `a29a024` (fix)

## Files Created/Modified

- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/` - Owns the seven digest-bound case files and 58-branch registry.
- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Executes both engines, evaluates semantic witnesses, proves exact closure, and emits the evidence manifest.
- `tools/reference/src/rigid_world_phase9_decode.hpp` - Validates Phase 9 contact inspection actions and standalone checkpoints.
- `tools/reference/src/rigid_world_phase9_execute.hpp` - Emits truthful lifecycle, particle-contact, body-contact, force, and checkpoint observations.
- `scripts/phase9-evidence.sh` - Preserves test status through logging and writes identity only after every evidence gate.
- `.github/workflows/oracle.yml` - Routes both Phase 9 evidence jobs through the fail-closed runner and checks sanitizer exception scope.
- `tools/reference/CMakeLists.txt` - Applies the approved shift-base exception to one pinned upstream translation unit in one preset.
- `tools/reference/tests/sanitizer_scope.cmake` - Rejects missing, broadened, or leaked sanitizer exceptions using effective compile commands.
- `TESTING.md` - Documents executable evidence, rejected prior artifacts, exact local commands, the approved exception, and residual risk.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` - Binds the byte-identical witness to the current reviewed adapter digest.

## Decisions Made

- Reduced the executable registry from 59 to 58 branches after explicit approval to retain native stable-ID rotation tests and defer truthful cross-engine rotation to Phase 10 rather than introduce a hidden seam.
- Kept force/impulse observations before the particle step so legitimate derived upstream weight behavior remains visible instead of being normalized away.
- Preserved the pinned upstream tree byte-for-byte and confined the approved exception to `b2ParticleSystem.cpp` under `oracle-asan-ubsan`; ASan, shift-exponent, all other UBSan categories, and fail-fast recovery remain enabled.
- Classified local AppleClang/CMake results as non-promotable evidence. Plans 09-23 and 09-24 still own exact canonical remote authority and any compatibility promotion.

## Deviations from Plan

### Approved Architectural Decision

**1. [Rule 4 - Architectural] Scoped a pinned-upstream shift-base sanitizer exception**

- **Found during:** Task 3 sanitizer verification
- **Issue:** Pinned upstream `computeRelativeTag` shifts the unavoidable contact-neighbor offset `x = -1` at `b2ParticleSystem.cpp:352`, called from line 1841, so every stepped particle sanitizer corpus halted before semantic execution.
- **Fix:** After explicit user approval, applied `-fno-sanitize=shift-base` only to that exact source under `oracle-asan-ubsan` and added positive/negative effective compile-command regression coverage.
- **Files modified:** `tools/reference/CMakeLists.txt`, `tools/reference/tests/sanitizer_scope.cmake`, `.github/workflows/oracle.yml`, `TESTING.md`
- **Verification:** Sanitizer compile DB contains exactly one exception; debug/release contain zero; ASan/UBSan fail-fast flags remain; protocol CTest and the complete sanitizer corpus pass.
- **Committed in:** `a29a024`

### Auto-fixed Issues

**2. [Rule 1 - Bug] Preserved actual derived particle weights**

- **Found during:** Task 1 cross-engine case execution
- **Issue:** Inspecting force/impulse state after stepping mixed the intended observation with legitimate derived contact-weight behavior.
- **Fix:** Moved force/impulse inspection before the step and retained upstream `GetWeightBuffer` semantics.
- **Files modified:** Phase 9 case fixtures and corpus assertions
- **Verification:** Debug/release canonical comparison, exact branch predicates, and sanitizer comparison all pass without normalization.
- **Committed in:** `86666c1`

**3. [Rule 3 - Blocking] Refreshed exact witness provenance after adapter changes**

- **Found during:** Task 3 provenance verification
- **Issue:** C++ adapter and build-input changes invalidated the reviewed adapter digest.
- **Fix:** Rebuilt the exact witness target and regenerated provenance.
- **Files modified:** `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json`
- **Verification:** Witness JSON remained byte-identical at `08d41d25f3766b9bf4bef51fb10713b7f925c074399b9642ad5cb4ce933fc8e3`; `cargo xtask provenance check` passes.
- **Committed in:** `f86b9d7`, `a29a024`

**Total deviations:** 3 resolved (1 approved architectural decision, 1 bug, 1 blocking provenance repair).
**Impact on plan:** Executable evidence and fail-closed verification are stronger; no production API, upstream source, Phase 10 behavior, or compatibility claim was widened.

## Issues Encountered

- A stale local debug/release oracle initially produced a malformed record after a temporary schema experiment was reverted; exact preset rebuilds restored the committed protocol.
- The first sanitizer run correctly blocked identity creation on the pinned upstream negative shift. The approved narrow exception and its compile-command contract resolved the blocker without hiding any repository-authored sanitizer result.
- Regenerated lifecycle/contact witness payload bytes never changed; only reviewed adapter digests and timestamps changed in provenance.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- CR-02, G09-EXECUTABLE-COVERAGE, and G09-EVIDENCE-PIPELINE have local executable closure with passing canonical and sanitizer dry runs.
- Plan 09-23 can request a fresh exact-SHA canonical/sanitizer workflow run; no prior failed artifact or local result is promotion authority.
- Cross-engine stable-ID rotation remains explicitly deferred to Phase 10, and particle groups, pairs, triads, solver families, and source-area behavior remain out of Phase 9 scope.

## Self-Check: PASSED

- All three task commits exist and the summary names every intentional created/modified evidence surface.
- The exact Rust precommit sequence passed format, warning-denied Clippy, all-target/all-feature build, all-feature tests, and 16 doctests before the final implementation commit.
- Debug and release scope tests prove zero sanitizer-exception leakage; sanitizer scope and protocol CTests pass.
- Both evidence modes pass seven cases, 58 unique branches, 22 policy paths per case, complete request/result/comparison digests, provenance, inventory, and read-only checks before identity creation.
- `actionlint`, Bash syntax, `just markdown-check`, upstream read-only, and diff checks pass.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-17*
