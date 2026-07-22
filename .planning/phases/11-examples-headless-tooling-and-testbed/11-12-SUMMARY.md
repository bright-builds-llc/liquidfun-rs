---
phase: 11-examples-headless-tooling-and-testbed
plan: "12"
subsystem: cpp-catalog-oracle
tags: [catalog, cpp-oracle, jsonl, checkpoint, process-isolation, sanitizer]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "06"
    provides: Closed 43-scenario registry and exact resolved plans
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "10"
    provides: Strict resolved-run request and canonical checkpoint wire contracts
provides:
  - Strict duplicate-aware C++ decoding of exact resolved catalog bytes, hashes, identity, provenance, and bounds
  - Fresh-world upstream execution with private semantic pointer maps for rigid, rope, particle, and group actions
  - Canonical semantic checkpoint records, reset epochs, and long-lived process dispatch
affects: [phase11-oracle-comparison, phase11-headless-runner, phase11-evidence, phase11-testbed]
tech-stack:
  added: []
  patterns:
    - Parse and validate resolved JSONL before constructing native world state
    - Keep native pointers and particle rows private behind stable scenario IDs
key-files:
  created:
    - tools/reference/src/catalog_run.cpp
    - tools/reference/src/catalog_run_decode.cpp
    - tools/reference/src/catalog_run_session.cpp
    - tools/reference/src/catalog_checkpoint.cpp
  modified:
    - tools/reference/src/main.cpp
    - tools/reference/src/protocol.cpp
    - tools/reference/CMakeLists.txt
    - tools/reference/adapter-inputs.txt
    - tools/reference/tests/protocol_tests.cpp
key-decisions:
  - "Extend the existing long-lived oracle with one catalog_run_request dispatch rather than introduce a second executable or protocol loop."
  - "Destroy the request-local world and all private semantic maps before a successful result returns, and advance reset epochs only after bounded checkpoint construction succeeds."
  - "Split strict decoding, effectful upstream execution, and checkpoint encoding into separate behavior-digested units so each trust-boundary concern remains reviewable."
patterns-established:
  - "Catalog oracle boundary: duplicate-aware bounded JSONL decode, exact resolved-byte SHA-256, redundant identity equality, and provenance checks precede effects."
  - "Portable capture: protocol output contains stable semantic counts and canonical checkpoint identity; pointers, dense rows, durations, and renderer state remain absent."
requirements-completed: [EXMP-03, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T02:54:12Z
duration: 21 min
completed: 2026-07-21
---

# Phase 11 Plan 12: C++ Catalog Oracle Execution Summary

**The existing long-lived C++ oracle now validates exact resolved catalog requests, executes them in fresh upstream worlds through private semantic maps, and emits bounded canonical checkpoints with reset proof.**

## Performance

- **Duration:** 21 min
- **Started:** 2026-07-22T02:33:28Z
- **Completed:** 2026-07-22T02:54:12Z
- **Tasks:** 1
- **Files modified:** 14

## Accomplishments

- Added strict duplicate-aware catalog request decoding with exact resolved-byte SHA-256, redundant identity, version, provenance, finite-value, semantic-ID, collection, and action-order validation before effects.
- Added request-local upstream execution for catalog rigid bodies, fixtures, joints, ropes, particle systems, stable particle handles, and particle groups without exposing C++ pointers or dense rows as protocol identity.
- Added canonical checkpoint and catalog-run end records with deterministic semantic counts, logical checkpoint identity, output bounds, reset verification, and successful-request-only epoch advancement.
- Registered every behavior-affecting source in CMake build identity and `adapter-inputs.txt`, then verified both ordinary and ASan/UBSan protocol targets.

## TDD Evidence

- **RED:** `cmake --build target/reference/oracle-debug --target liquidfun-reference-protocol-tests` failed on `fatal error: 'catalog_run.hpp' file not found` after the catalog execution and malformed-reuse tests were added.
- **GREEN:** The same target built with strict warnings and `ctest --test-dir target/reference/oracle-debug --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'` passed 1/1.
- **REFACTOR:** The initial 772-line implementation was separated into a 38-line adapter, 250-line strict decoder, cohesive 563-line upstream session executor, and independent checkpoint encoder; debug and ASan/UBSan protocol tests still pass.

The intentionally failing RED state was not committed because repository policy requires all ordered Rust and C++ gates to pass before every commit.

## Task Commits

1. **Task 1: Extend the long-lived oracle for resolved catalog execution and capture** - `f1b291f` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `tools/reference/src/catalog_run.cpp` and `catalog_run.hpp` - Minimal reset-owning adapter surface.
- `tools/reference/src/catalog_run_decode.cpp` and `catalog_run_decode.hpp` - Duplicate-aware bounded request, resolved-byte, identity, provenance, and semantic entity validation.
- `tools/reference/src/catalog_run_session.cpp` and `catalog_run_session.hpp` - Fresh-world execution and private stable-ID-to-native-object maps.
- `tools/reference/src/catalog_checkpoint.cpp` and `catalog_checkpoint.hpp` - Canonical checkpoint and successful reset-end encoding.
- `tools/reference/src/main.cpp`, `protocol.cpp`, and `protocol.hpp` - Closed `catalog_run_request` dispatch in the existing stdin/stdout loop.
- `tools/reference/CMakeLists.txt` and `tools/reference/adapter-inputs.txt` - Native target, build-identity dependency, and adapter provenance registration.
- `tools/reference/tests/protocol_tests.cpp` - Valid rigid execution, schema parity, deterministic reuse, tamper, duplicate, oversized-input, private-identity exclusion, and reset recovery coverage.

## Decisions Made

- Reused the one existing oracle process and stdout writer so catalog execution inherits newline framing, protocol-only stdout, bounded input, and process supervision instead of duplicating those controls.
- Used upstream `b2ParticleHandle` only as private mutable lookup state. Protocol identity remains the resolved scenario ID even when upstream compaction changes dense particle indices.
- Counted debug-draw visits only as a stable structural observation in this boundary. Renderer primitives remain absent from the C++ checkpoint payload until a later comparison consumer requires their complete stable-key translation.
- Kept diagnostics as fixed bounded categories. Rejections never echo untrusted records, pointer values, stack traces, or private indices.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used the canonical adapter manifest path**

- **Found during:** Task 1 read-first and CMake integration
- **Issue:** The plan listed `tools/reference/src/adapter-inputs.txt`, but the repository's established digest authority is `tools/reference/adapter-inputs.txt` and CMake resolves that exact path.
- **Fix:** Updated the existing canonical manifest rather than creating a second source-local authority.
- **Files modified:** `tools/reference/adapter-inputs.txt`
- **Verification:** Both oracle-debug and oracle-asan-ubsan configurations accepted the recalculated adapter digest and built the protocol target.
- **Committed in:** `f1b291f`

**Total deviations:** 1 auto-fixed blocking path correction.
**Impact on plan:** The correction preserves the existing provenance authority and avoids a contradictory second manifest. No architecture or dependency scope changed.

## Issues Encountered

- The first configure attempt raced the in-progress adapter edits and retained the previous cached digest. Rerunning the reviewed xtask configure after all behavior inputs were registered produced the current digest and made subsequent CMake regeneration deterministic.
- The shared worktree contained four unrelated pre-existing edits. They remained unstaged and uncommitted by this plan.

## Security Verification

- Duplicate-aware framing and reviewed byte, depth, collection, action, checkpoint, iteration, and output bounds reject malformed input before effectful construction or excessive allocation.
- Exact resolved-byte SHA-256 and redundant identity/provenance equality prevent silent request substitution at the C++ boundary.
- Every request owns a fresh `b2World`; private object and particle-handle maps die with the request. Rejected requests do not advance reset epochs or poison later valid requests.
- Exceptions remain contained per request, stdout contains protocol records only, and stderr receives fixed bounded diagnostics without untrusted payloads.
- The focused protocol target passed under both oracle-debug and oracle-asan-ubsan. No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-12's `EXMP-03` and `EXMP-05` mappings are implemented at the C++ oracle boundary and retained in summary frontmatter. Their global requirement checkboxes remain intentionally unchanged until later Phase 11 comparison and end-to-end evidence plans prove the complete requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 11-13 can connect the session/comparison path to the C++ catalog stream without inventing a second process or schema.
- Later headless, regression, benchmark, and visual consumers can reuse the same exact resolved bytes and canonical checkpoint identities.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed all created catalog adapter, decoder, session, and checkpoint files exist and all behavior-affecting inputs are present in `tools/reference/adapter-inputs.txt`.
- Confirmed commit `f1b291f` exists and contains only Plan 11-12 C++ oracle, protocol, CMake, provenance, and test files.
- Confirmed oracle-debug protocol CTest passes 1/1 and oracle-asan-ubsan protocol CTest passes 1/1.
- Confirmed the exact ordered full-workspace `cargo fmt`, deny-warnings Clippy, build, test, and doctest gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-12`.
- Confirmed the four pre-existing fenced edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
