---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "13"
subsystem: particle-oracle
tags: [cpp, particles, jsonl, oracle, sanitizer, coupling]
requires:
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "12"
    provides: closed phase9-v1 request and semantic trace schema
  - phase: 09-particle-storage-lifecycle-and-coupling
    plan: "15"
    provides: pinned lifecycle/contact witnesses and exact provenance
provides:
  - strict bounded Phase 9 decode in the existing long-lived pinned C++ process
  - source-native particle execution with semantic IDs and protocol-only stdout
  - fixed/growable buffers, lifecycle, force, query, and rigid coupling oracle evidence
affects: [09-14, 09-16, particle-differential-evidence]
tech-stack:
  added: []
  patterns: [strict foreign-boundary decode, semantic handle mapping, additive legacy trace patch]
key-files:
  created:
    - tools/reference/src/rigid_world_phase9_decode.hpp
    - tools/reference/src/rigid_world_phase9_execute.hpp
    - crates/liquidfun-differential/tests/particle_oracle.rs
  modified:
    - tools/reference/src/rigid_world.cpp
    - tools/reference/src/rigid_world.hpp
    - tools/reference/src/rigid_world_decode.hpp
    - tools/reference/adapter-inputs.txt
key-decisions:
  - "Decode and validate Phase 9 before stripping its additive fields for the retained Phase 6-8 decoder, then merge source-native semantic records back by stable checkpoint ID."
  - "Represent particles with pinned b2ParticleHandle values internally while emitting only scenario IDs, exact float bits, semantic counts, and body snapshots."
  - "Preserve the closed phase9-v1 schema rather than inventing the plan-mentioned particle-iteration field that 09-12 does not declare."
patterns-established:
  - "Foreign requests are bounded and domain-checked before any pinned-world allocation or mutation."
  - "One process can execute repeated Phase 9 requests with reset epochs increasing exactly once per completed request."
requirements-completed: [PART-01, PART-02, PART-03, PART-04, PART-06, PART-07, PART-08, PART-14, PART-15, PART-16, PART-17]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T11:10:00Z
duration: 37 min
completed: 2026-07-15
---

# Phase 9 Plan 13: Pinned Particle Oracle Summary

**The existing pinned C++ JSONL process now validates and executes the closed Phase 9 particle surface, preserves semantic identity across source permutations, and records rigid particle/body coupling without leaking pointers or dense indices.**

## Performance

- **Duration:** 37 min
- **Started:** 2026-07-15T10:33:00Z
- **Completed:** 2026-07-15T11:10:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Verified the exact Plan 09-15 witness digest `08d41d25f3766b9bf4bef51fb10713b7f925c074399b9642ad5cb4ce933fc8e3` and pinned revision before generalized execution, then made that witness consumption an integration regression.
- Added strict Phase 9 declaration/action decoding for exact float bits, IDs, buffer capacity, closed flags, bounds, and Phase 10 exclusions before allocation.
- Executed both growable and caller-owned fixed buffer modes, multiple newest-first systems, stable particle handles, pause/resume, mutation, force/impulse, lifetime destruction, compaction, statistics, AABB queries, rays, and teardown in the pinned source.
- Mirrored scoped rigid bodies and fixtures into the same pinned world for static and off-center dynamic coupling, recording source body-contact counts and rigid linear/angular reactions.
- Proved two requests execute through one process with clean protocol-only stdout and reset epochs `1` then `2`; malformed Phase 10 topology fails hard on stderr.

## Task Commits

1. **Task 1: Decode bounded Phase 9 requests in the existing oracle** - `3f80bb0` (feat)
2. **Task 2: Execute source behavior and collect semantic traces** - `5abb2fd` (test)

## Files Created/Modified

- `tools/reference/src/rigid_world_phase9_decode.hpp` - Closed Phase 9 foreign-boundary validation and additive legacy stripping.
- `tools/reference/src/rigid_world_phase9_execute.hpp` - Pinned particle execution, stable-ID mapping, semantic collection, and scoped rigid coupling.
- `tools/reference/src/rigid_world.cpp` - Existing process dispatch and checkpoint patch integration.
- `tools/reference/src/rigid_world.hpp` - Phase 9 timeline payload carried by the existing request.
- `tools/reference/src/rigid_world_decode.hpp` - Phase 9 decode routing before retained Phase 6-8 decode.
- `tools/reference/adapter-inputs.txt` - New behavior-affecting sources included in build identity.
- `crates/liquidfun-differential/tests/particle_oracle.rs` - Witness, decode, hard-failure, action-family, coupling, and process-reset regressions.

## Decisions Made

- The retained Phase 6-8 decoder remains the authority for rigid declarations and actions. Phase 9 additions are validated first, stripped only for that legacy decode, executed independently against the same pinned request data, and merged by stable checkpoint ID.
- Upstream dense indices remain ephemeral. Stable scenario IDs map internally through `b2ParticleHandle`, and every emitted record uses semantic IDs or exact scalar bits.
- Fixed buffers supply all reviewed external lanes for their declared lifetime. Growable buffers retain source-owned allocation while the declared initial capacity remains semantic evidence.
- The exact closed phase9-v1 protocol remains unchanged. Its rigid step carries no particle-iteration field, so this plan did not invent an undeclared Phase 10 control.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used the repository's actual upstream build commands**

- **Found during:** Task 1 RED setup
- **Issue:** The literal planned command `cargo xtask reference build --preset ...` does not exist in the checked-in xtask command registry.
- **Fix:** Used the closed, documented `cargo xtask upstream configure --preset ...` followed by `cargo xtask upstream build --preset ...` for debug, release, and ASan/UBSan.
- **Files modified:** None.
- **Verification:** All three presets configured and built successfully.
- **Committed in:** N/A (verification-only deviation)

**2. [Rule 3 - Blocking] Preserved retained rigid decode while adding Phase 9**

- **Found during:** Task 1 GREEN
- **Issue:** Phase 6-8 decoders reject additive fields and particle action wrappers before the new executor can run.
- **Fix:** Strictly validate the original Phase 9 timeline first, strip only its known additive members for legacy decode, and retain the original bounded timeline for pinned particle execution and semantic checkpoint merging.
- **Files modified:** `rigid_world_phase9_decode.hpp`, `rigid_world_decode.hpp`, `rigid_world.hpp`, `rigid_world.cpp`.
- **Verification:** Valid Phase 9 passes; unknown `particle_groups` remains present and fails as a hard harness error.
- **Committed in:** `3f80bb0`

### Deferred Plan Detail

**Configured particle-iteration count is not representable in phase9-v1**

- **Plan text:** Exercise multiple configured particle iterations.
- **Constraint:** The exact Plan 09-12 schema has no particle-iteration field, while this plan also requires exact schema consumption and explicit Phase 10 rejection.
- **Disposition:** Preserved phase9-v1 and the inherited pinned rigid-step particle iteration. Adding a new control belongs in a separately reviewed protocol revision.
- **Impact:** Every declared Phase 9 family is exercised; no undeclared field or Phase 10 behavior was admitted.

**Total deviations:** 2 auto-fixed blocking issues and 1 deferred non-representable plan detail. **Impact:** The implementation remains closed, source-native, and backward-compatible.

## Issues Encountered

- Task 1 RED failed as expected with `UnexpectedEof` and stderr `rigid timeline contains unknown member particle_systems`.
- Task 2 RED produced a semantic statistics record with `body_contact_count = 0`; mirroring scoped rigid declarations into the same pinned world made the coupling test GREEN with a nonzero contact count and rigid reaction.
- Writing two maximum-sized requests before draining stdout deadlocked on ordinary pipe backpressure. The long-lived regression now writes and drains one request at a time, matching supervisor discipline.

## Validation Evidence

- Exact witness/provenance validation passed against upstream `7f20402173fd143a3988c921bc384459c6a858f2`.
- `cargo test -p liquidfun-differential --test particle_oracle` passed all seven tests.
- `cargo test -p liquidfun-differential --test particle_oracle coupling` passed static and dynamic coupling cases.
- Debug, release, and ASan/UBSan reference presets configured and built successfully under local D2 AppleClang 21.0.0.
- Before each commit: `cargo fmt --all`, warning-denied all-target/all-feature Clippy, all-target/all-feature build, and all-feature tests passed in order.
- `git diff --exit-code -- third_party/liquidfun`, `git diff --check`, and stdout/stderr separation checks passed.

## User Setup Required

None.

## Next Phase Readiness

- Ready for 09-14 to assemble differential comparison evidence from the closed phase9-v1 semantic traces.
- The local builds are D2 supported evidence; canonical Linux D1 promotion remains a separate workflow.
- A future protocol revision is required before differential scenarios can vary particle iterations explicitly.

## Self-Check: PASSED

- Task commits `3f80bb0` and `5abb2fd` exist.
- All three key created files exist and are registered in the adapter content identity.
- The pinned submodule is unchanged and every requested build preset passes.
- `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-15*
