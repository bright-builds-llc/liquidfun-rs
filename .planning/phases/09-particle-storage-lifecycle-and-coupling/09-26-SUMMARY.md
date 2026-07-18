---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "26"
subsystem: phase9-executable-evidence
tags: [particles, differential, oracle, semantic-witnesses, determinism, sanitizers]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "25"
    provides: retained-first complete Phase 9 comparison and persistent particle-coupled rigid snapshots
provides:
  - closed typed action/checkpoint/observation/assertion bindings for all 58 reviewed Phase 9 branches
  - seven symmetric native/oracle cases with executable lifetime, contact, listener, filter, energy, and stuck evidence
  - result-digest replay, minimization, first-divergence, D0, and debug/release evidence
affects: [phase-09-verification, phase-10, compatibility-evidence]
tech-stack:
  added: []
  patterns: [closed semantic assertion registry, occurrence-index observation, result-digest evidence, symmetric callback capture]
key-files:
  created: []
  modified:
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs
    - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs
    - crates/liquidfun-differential/src/rigid_world/phase9.rs
    - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json
    - crates/liquidfun-differential/tests/phase9_corpus.rs
    - tools/reference/src/rigid_world_phase9_decode.hpp
    - tools/reference/src/rigid_world_phase9_execute.hpp
key-decisions:
  - "Represent every reviewed branch with a closed typed semantic assertion and exact action/checkpoint indices; free-form predicates and identity-only evidence are rejected."
  - "Capture listener occurrences symmetrically in native Rust and the pinned C++ oracle, then inspect them through a bounded occurrence index."
  - "Use immediate statistics for filter effects and late statistics for strict-contact and stuck evidence so each branch observes the named production behavior."
  - "Keep the contacted strict system near tangent to the retained dynamic fixture, preserving measurable body reaction while remaining inside retained comparison policy."
patterns-established:
  - "Evidence branches compare complete result digests or deliberate mismatch signatures rather than request/scenario identity."
  - "Phase-local result validation tracks absolute quantized expiry and reverse creation order for deterministic capacity victims."
requirements-completed: [API-09, API-10, PART-01, PART-02, PART-03, PART-04, PART-05, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T00:16:11Z
duration: 2h 38m
completed: 2026-07-18
---

# Phase 9 Plan 26: Executable Semantic Corpus Closure Summary

**Seven native/oracle requests now bind all 58 reviewed Phase 9 branches to exact typed observations, with real lifecycle/contact contrasts and result-based determinism evidence.**

## Performance

- **Duration:** 2h 38m
- **Started:** 2026-07-17T21:38:40Z
- **Completed:** 2026-07-18T00:16:11Z
- **Tasks:** 3
- **Files modified:** 19 implementation, fixture, oracle, and provenance files

## Accomplishments

- Replaced free-form corpus predicates with a closed, bounded `Phase9WitnessBinding` registry that identifies exact branch, action, checkpoint, observation kind, and typed semantic assertion.
- Added fail-closed validation and focused rejection coverage for duplicates, missing/extra branches, invalid indices, wrong observation kinds, generic evidence identity, zero collision energy, empty stuck candidates, unknown assertions, and oversized registries.
- Retained exactly seven JSONL cases and exactly 58 unique branch bindings while regenerating every request digest from the checked-in bytes.
- Proved finite expiration, infinite survival, equal-expiration eviction order, requested and unrequested destruction, strict-contact cardinalities, listener events, filter effects, positive finite collision energy, and nonempty stable stuck candidates.
- Captured particle-particle, particle-body, destruction, and contact-end occurrences symmetrically in native Rust and the pinned C++ oracle.
- Bound replay and D0 branches to repeated complete-result bytes/digests, minimization to preserved mismatch signatures, first-divergence to a deliberate copied-result mutation, and canonical debug/release to equal complete-result digests.
- Preserved all 22 Phase 9 comparison paths per case, retained-first rigid comparison, Phase 10 exclusions, compatibility authority, and the read-only upstream checkout.

## Task Commits

1. **Task 1: Define and reject non-semantic branch bindings** - `3083a81`
2. **Tasks 2-3: Execute all seven semantic cases and pass every native/oracle gate** - `71f8b5e`

**Plan metadata:** committed after summary and state tracking verification.

## Verification

- `cargo test -p liquidfun-test-protocol phase9`
- `cargo test -p liquidfun-differential --test phase9_corpus witness_binding -- --nocapture`
- `cargo test -p liquidfun-differential --test phase9_corpus` - 22 passed, 0 failed, 1 ignored regeneration tool
- `cargo test -p liquidfun-differential --test particle_oracle` - 13 passed, 0 failed
- Fresh `oracle-debug` and `oracle-release` configure/build
- Canonical complete corpus with debug/release result equality - 22 passed, 0 failed, 1 ignored
- Fresh `oracle-asan-ubsan` configure/build and protocol test target
- Fail-fast ASan/UBSan protocol CTest - 1 passed, 0 failed
- Complete fail-fast sanitizer corpus - 22 passed, 0 failed, 1 ignored
- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`, including 16 doctests
- `cargo xtask provenance check`
- `git diff --exit-code -- third_party/liquidfun`
- `git diff --check`

The exact Rust precommit sequence was rerun after the tracked provenance refresh and completed immediately before the implementation commit. Local oracle builds reported the existing noncanonical development identities: CMake 3.27.9 and Apple Clang 21 instead of canonical CMake 4.3.3 and Clang 22.1.8. No compatibility evidence was promoted from those local identities.

## Files Modified

- `crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs` - Defines the closed 58-branch registry, semantic assertions, bounded occurrence inspection, and fail-closed binding validation.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs` - Advances Phase 9 lifetime state on rigid steps.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs` - Validates absolute quantized expiry, creation-order eviction, occurrence inspection, and semantic observations.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs` - Includes occurrence inspection in lifecycle validation.
- `crates/liquidfun-differential/src/rigid_world.rs` - Stores the bounded Phase 9 occurrence stream.
- `crates/liquidfun-differential/src/rigid_world/phase8.rs` - Exercises production filter hooks for flag-gated Phase 9 contacts.
- `crates/liquidfun-differential/src/rigid_world/phase9.rs` - Captures step occurrences, exposes occurrence inspection, and filters expired handles from system observations.
- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json` - Binds exactly 58 branches to typed indexed assertions and fresh request digests.
- `crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/cases/*.jsonl` - Carries the regenerated seven-case request corpus.
- `crates/liquidfun-differential/tests/phase9_corpus.rs` - Generates, validates, executes, and semantically evaluates every binding plus result-based evidence contracts.
- `tools/reference/src/rigid_world_phase9_decode.hpp` - Decodes and lifecycle-validates bounded occurrence inspection.
- `tools/reference/src/rigid_world_phase9_execute.hpp` - Captures listener/filter/destruction occurrences and executes occurrence inspection.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` - Refreshes the reviewed adapter digest while preserving the witness payload digest.

## Decisions Made

- The manifest itself is the typed closure authority: every branch must deserialize into the closed assertion enum and pass the exact 58-branch validator before indexed evaluation.
- Contact evidence uses the real production filter/listener flags and callback streams; declaration bits alone never satisfy enabled/disabled branches.
- The unfiltered fixed C/D pair supplies particle-contact and listener evidence, while the flag-filtered growable pair supplies the zero-contact contrast.
- Non-contact corpus cases keep coupling outside query/ray ranges so their existing geometry remains stable; the contact case alone places coupling inside the retained fixture region.
- Evidence bindings execute repeated native/oracle results and deliberate copied mutations, rather than projecting identity fields from one result.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrected result-state lifetime and capacity-victim tracking**

- **Found during:** Task 2 lifetime and storage corpus execution
- **Issue:** Relative lifetime ordering and declaration order could select the wrong equal-expiration capacity victim after several steps.
- **Fix:** Tracked creation step and creation order, compared absolute quantized expiration, and removed expired particles during every rigid step.
- **Files modified:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs`, `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs`
- **Verification:** Native and oracle lifetime/storage cases pass complete Phase 9 comparison and the typed expiry/order assertions.
- **Committed in:** `71f8b5e`

**2. [Rule 1 - Bug] Closed the C++ occurrence lifecycle-validation gap**

- **Found during:** Task 2 first contact-case oracle execution
- **Issue:** `inspect_occurrence` passed structural decoding but was rejected by the independent lifecycle-state validator.
- **Fix:** Accepted the read-only bounded occurrence action in lifecycle validation without mutating live state.
- **Files modified:** `tools/reference/src/rigid_world_phase9_decode.hpp`
- **Verification:** Debug, release, and sanitizer corpora all execute occurrence inspection successfully.
- **Committed in:** `71f8b5e`

**3. [Rule 1 - Bug] Removed expired handles from native system observations**

- **Found during:** Task 2 finite-lifetime validation
- **Issue:** Native semantic ID storage retained an expired handle, so a post-step system inspection falsely reported the finite particle alive.
- **Fix:** Required a successful live world snapshot before including a mapped handle in `inspect_system`.
- **Files modified:** `crates/liquidfun-differential/src/rigid_world/phase9.rs`
- **Verification:** The finite particle is absent after its quantized expiry in both engines; infinite survival remains present.
- **Committed in:** `71f8b5e`

**4. [Rule 3 - Blocking] Refreshed stale reviewed adapter provenance**

- **Found during:** Task 3 provenance gate
- **Issue:** Reviewed C++ decoder/executor changes invalidated the lifecycle/contact witness adapter digest.
- **Fix:** Rebuilt and ran the checked-in witness generator with its recorded argv, preserving witness payload bytes while refreshing adapter digest and timestamp.
- **Files modified:** `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json`
- **Verification:** `cargo xtask provenance check` passes with witness SHA-256 `08d41d25f3766b9bf4bef51fb10713b7f925c074399b9642ad5cb4ce933fc8e3`.
- **Committed in:** `71f8b5e`

***

**Total deviations:** 4 auto-fixed (3 Rule 1 bugs, 1 Rule 3 blocker)
**Impact on plan:** All fixes were required for correct symmetric execution and evidence integrity; no Phase 10 or compatibility-authority scope was added.

## Issues Encountered

- A first contact design used transient fixtures whose destruction polluted the next retained rigid count ledger. The final case instead uses retained fixtures and a reviewed stuck threshold.
- Deep overlap with the retained dynamic fixture produced cross-engine rigid motion outside the retained numeric policy. A near-tangent contacted configuration preserved strict/listener/filter/body-reaction evidence and passed the complete retained comparator.
- Moving the coupling witness into query/ray geometry exposed unrelated fraction differences. The final generator places coupling in the contacted region only for the contact case and retains its distant position elsewhere.
- The full `cargo test --all-features` gate traversed many integration binaries and long-running compile-fail doctests twice, including the required post-provenance immediate precommit rerun; both completed successfully.

## Known Stubs

None.

## User Setup Required

None.

## Next Phase Readiness

- `G09-EXECUTABLE-COVERAGE` is closed with typed executable evidence for all 58 branches.
- Phase 9 now has local debug, release, and sanitizer proof without compatibility-authority promotion.
- Stable-ID rotation and all group/topology/pair/triad/source-area/solver behavior remain explicitly deferred to Phase 10.
- No blockers remain for Phase 09 verification or milestone audit.

## Self-Check: PASSED

- All five key implementation, manifest, oracle, and summary paths exist.
- Task commits `3083a81` and `71f8b5e` exist in repository history.
- Every focused, canonical, sanitizer, full Rust, provenance, upstream read-only, and diff gate passed.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-18*
