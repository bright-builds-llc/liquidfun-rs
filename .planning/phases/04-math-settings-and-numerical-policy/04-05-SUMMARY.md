______________________________________________________________________

phase: 04-math-settings-and-numerical-policy
plan: "05"
subsystem: cpp-math-probes-and-build-identity
tags: [cpp, rust, ieee-754, build-provenance, differential-testing]
requires:

- phase: 04-math-settings-and-numerical-policy
  plan: "04"
  provides: Strict bounded native math-probe request, result, and witness contract
  provides:
- External C++ execution of the complete 39-case Phase 4 math-probe corpus
- Representation-preserving float bit transport across the JSONL boundary
- Strict 17-field Rust and C++ floating build identity with D1/D2/D3 classification
- Fail-closed canonical compiler, target, floating flag, and runtime witness validation
  affects: [phase-5-collision, differential-evidence, canonical-oracle-ci]
  tech-stack:
  added: []
  patterns: [closed C++ dispatch, memcpy bit transport, strict provenance identity, fail-closed evidence tiers]
  key-files:
  created:
  - tools/reference/src/math_probe.hpp
  - tools/reference/src/math_probe.cpp
    modified:
  - tools/reference/src/protocol_bits.cpp
  - tools/reference/src/protocol.cpp
  - tools/reference/src/main.cpp
  - tools/reference/CMakeLists.txt
  - tools/reference/src/build_identity.hpp.in
  - crates/liquidfun-test-protocol/src/provenance.rs
  - crates/liquidfun-test-protocol/src/trace.rs
  - crates/liquidfun-differential/src/rust_adapter.rs
  - crates/liquidfun-differential/tests/round_trip.rs
    key-decisions:
- "Mirror the closed Rust math-probe contract in external C++ and preserve every float payload through memcpy-based uint32_t transport."
- "Treat canonical D1 compiler, target, flags, and runtime witnesses as a fail-closed identity contract before numerical comparison."
- "Record supported but noncanonical local toolchains as D2 or D3 evidence that cannot promote canonical results."
  patterns-established:
- "C++ probe parity: identical operation IDs, ordered case IDs, exact-bit results, bounded decode, and reset epochs across debug and release profiles."
- "Floating identity: all 17 Phase 4 fields participate in strict validation and identity hashing, while unsupported canonical capabilities remain explicit evidence-tier boundaries."
  requirements-completed: [COLL-01, COLL-08]
  generated_by: gsd-execute-plan
  lifecycle_mode: yolo
  phase_lifecycle_id: 4-2026-07-11T04-16-20
  generated_at: 2026-07-11T06:32:06Z
  duration: 29 min
  completed: 2026-07-11

______________________________________________________________________

# Phase 4 Plan 05: Pinned C++ Probes and Floating Build Identity Summary

**The external C++ oracle now executes the complete native math-probe contract with exact IEEE-754 transport, while one strict Rust/C++ build identity gates canonical numerical evidence.**

## Performance

- **Duration:** 29 min
- **Completed:** 2026-07-11T06:32:06Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments

- Added exhaustive C++ dispatch for all 24 Phase 4 operations and 39 deterministic cases, including cancellation, halfway, overflow, underflow, and non-fused FMA witnesses.
- Replaced arithmetic float reconstruction with size-asserted `std::memcpy` transport and exact tests for signed zeros, subnormals, infinities, quiet NaNs, and signaling NaNs.
- Added strict duplicate-aware bounded request parsing, unknown-operation rejection, ordered typed results, and two-request reset epochs without contaminating protocol stdout.
- Added the same 17 floating build identity fields to validated Rust provenance, raw trace decode, native Rust construction, configured C++ construction, handshake serialization, and identity hashing.
- Enforced checked canonical floating options, forbidden fast/native tuning rejection, round-to-nearest and gradual-underflow runtime witnesses, and non-promotable D2/D3 classification.
- Exercised the exact C++ probe contract in both debug and release oracle profiles and confirmed the published Cargo package remains independent of C++ tooling.

## Task Commits

1. **Task 1: Implement bit-faithful C++ probe dispatch** - `0775455` (feat)
1. **Task 2: Enforce canonical flags and one-to-one build identity** - `9a8beed` (feat)
1. **Final verification: Exercise debug and release math probes** - `6ad424b` (test)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wire and test the new C++ translation unit through private reference build inputs**

- **Found during:** Task 1
- **Issue:** The planned probe implementation could not build or receive direct C++ regression coverage without updating the CMake source list, adapter-input manifest, and private protocol test target, which were not all listed in the task file set.
- **Fix:** Registered `math_probe.cpp`, bound it into adapter identity inputs, and added private C++ protocol tests alongside Rust-driven integration coverage.
- **Files modified:** `tools/reference/CMakeLists.txt`, `tools/reference/adapter-inputs.txt`, `tools/reference/tests/protocol_tests.cpp`
- **Verification:** Debug and release CMake builds, both CTest protocol suites, adapter-input verification, and Rust-driven probe tests pass.
- **Committed in:** `0775455`

**2. [Rule 1 - Bug] Suppress the harmless Clang option-override diagnostic only for repository-authored probe translation units**

- **Found during:** Task 2
- **Issue:** Clang reports `-ffp-contract=off` as overriding the contract setting implied by the required preceding `-ffp-model=precise`; strict `-Werror` converted the otherwise correct explicit canonical setting into a build failure.
- **Fix:** Added the narrow `-Wno-overriding-option` diagnostic suppression without removing either required floating option or weakening other repository-authored warnings.
- **Files modified:** `tools/reference/CMakeLists.txt`
- **Verification:** Effective compile commands contain the required supported flags, omit forbidden flags, and build repository-authored C++ successfully under strict warning denial.
- **Committed in:** `9a8beed`

**3. [Rule 3 - Blocking] Extend the real-oracle regression to the required release profile**

- **Found during:** Final verification
- **Issue:** The named math-probe integration test resolved only `oracle-debug`, leaving release-profile execution unasserted despite the plan requiring both profiles.
- **Fix:** Parameterized real-oracle resolution and ran the same 39-result contract and two-request reset assertions against debug and release presets.
- **Files modified:** `crates/liquidfun-differential/tests/round_trip.rs`
- **Verification:** `cpp_math_probe_matches_operation_contract` passes against both built presets and the exact ordered full Rust gate passes.
- **Committed in:** `6ad424b`

**Total deviations:** 3 auto-fixed (2 blocking, 1 bug). **Impact:** Private build wiring, one narrow diagnostic policy, and broader verification coverage only; no published C++ dependency or production physics scope was added.

## Issues Encountered

- Local Apple Clang 21 supports seven of the eight required canonical floating options but not `-fdenormal-fp-math-fp32=ieee`. The build therefore records `precise-supported-d2` and the missing capability explicitly, retains passing runtime rounding/gradual-underflow witnesses, and cannot promote the result to D1. Canonical Clang 22 configuration remains fail closed and was not weakened.
- The local D2 libm produced one signed-zero difference in a transform-composition angle outside the pinned exact witness set. The probe still preserves operation order and exact transport; the difference remains correctly classified as supported-platform policy evidence rather than canonical parity.

## Verification

- Both private C++ protocol test executables built and passed under `oracle-debug` and `oracle-release`.
- `cpp_protocol_bits_preserve_exceptional_classes` passed with exact `uint32_t` payload equality for exceptional IEEE classes.
- `cpp_math_probe_matches_operation_contract` passed for debug and release profiles with 39 ordered results and reset epochs 1 then 2.
- All named strict identity decode, native construction, missing-field, forbidden-flag, and non-promotion tests passed.
- `cargo xtask upstream verify` and debug/release configure/build workflows passed.
- Effective compile commands contain every locally supported required option and no forbidden fast-math or native-tuning option; the handshake carries the complete classified identity without secrets or environment values.
- `cargo package -p liquidfun --allow-dirty --list` passed and contains no C++, reference harness, protocol fixture, or private tooling input.
- Ordered Rust gate passed after every atomic implementation change: `cargo fmt --all`; strict all-target/all-feature Clippy; all-target/all-feature build; all-feature tests.
- `git diff --check` and task-scoped diff review passed.

## User Setup Required

None - local noncanonical toolchains are classified automatically, and canonical evidence remains confined to the pinned oracle lane.

## Next Phase Readiness

- Plan 04-06 can consume strict build identity and cross-language exact-bit probes when defining the remaining numerical-policy evidence and tolerances.
- Collision work can reuse the closed external probe and provenance boundary without adding C++ to published crates.

## Self-Check: PASSED

- Task commits `0775455`, `9a8beed`, and `6ad424b` exist in history.
- Both created C++ probe artifacts and all Rust/C++ identity integration files exist.
- Debug and release probe comparisons, C++ protocol tests, focused Rust tests, package isolation, effective-flag inspection, and the exact full Rust gate pass.
- D1 enforcement remains closed, while this machine's missing exact denormal flag is recorded as non-promotable D2 evidence.

______________________________________________________________________

_Phase: 04-math-settings-and-numerical-policy_
_Completed: 2026-07-11_
