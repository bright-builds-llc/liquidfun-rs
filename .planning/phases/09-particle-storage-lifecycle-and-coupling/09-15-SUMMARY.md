---
phase: 09-particle-storage-lifecycle-and-coupling
plan: "15"
subsystem: particle-oracle-evidence
tags: [particles, lifecycle, contacts, oracle, provenance]
requires:
  - phase: 01-upstream-lock-and-inventory
    provides: exact pinned LiquidFun revision and read-only source contract
  - phase: 02-reference-oracle-foundation
    provides: private C++ oracle build and fail-closed provenance conventions
provides:
  - pinned-C++ equal-quantized-expiration semantic order witness
  - pinned-C++ strict body-contact pruning tie witness
  - fail-closed exact generation provenance validation
affects: [09-06, 09-09, 09-13, particle-lifecycle, particle-contacts]
tech-stack:
  added: []
  patterns: [pre-implementation C++ witness, semantic IDs only, exact digest provenance]
key-files:
  created:
    - tools/reference/src/phase9_lifecycle_contact_witness.cpp
    - reference/artifacts/phase9/lifecycle-contact-witnesses.json
    - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
    - tools/xtask/src/provenance/phase9_witness.rs
    - .planning/phases/09-particle-storage-lifecycle-and-coupling/09-15-SUMMARY.md
  modified:
    - tools/reference/CMakeLists.txt
    - tools/reference/adapter-inputs.txt
    - tools/xtask/src/upstream.rs
    - tools/xtask/tests/upstream_cli.rs
    - tools/xtask/src/provenance.rs
    - reference/source-map.toml
key-decisions:
  - "Witness exact ties through the pinned public C++ engine before any Rust implementation can define the expected result."
  - "Bind the semantic output to upstream, adapter, probe-source, toolchain, command, and content identities, plus a byte-identical rerun."
  - "Keep the additional private build target behind a closed xtask allowlist."
requirements-completed: [PART-07, PART-08, PART-15]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T04:34:57Z
duration: 15min
completed: 2026-07-14
---

# Phase 9 Plan 15: Lifecycle and Contact Oracle Witnesses Summary

**Pinned C++ execution now fixes equal-expiration destruction order and strict body-contact pruning ties as semantic, digest-bound witnesses before Rust can implement either rule.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-07-15T04:20:18Z
- **Completed:** 2026-07-15T04:34:57Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Added a private pinned-C++ probe that exercises public LiquidFun lifecycle and body-contact behavior without calling Rust or reading Rust-generated expectations.
- Captured stable semantic IDs and outcomes for eight equal-quantized-expiration particles and six exact equal-weight fixture contacts.
- Bound the committed witness to the exact upstream revision, adapter and probe source digests, compiler and target identity, CMake preset and target, exact command arguments, generation timestamp, and witness SHA-256.
- Added fail-closed repository verification for the new witness pair and proved repeated generation produces byte-identical semantic output.

## Task Commits

1. **Task 1: Build the narrow pinned-C++ lifecycle/contact probe** - `522a6a2` (feat)
2. **Task 2: Generate and verify semantic witness provenance** - `a23b5f0` (test)

## Files Created/Modified

- `tools/reference/src/phase9_lifecycle_contact_witness.cpp` - Executes the two bounded pinned-C++ scenarios and emits semantic witness and provenance JSON.
- `tools/reference/CMakeLists.txt` - Registers the private witness executable with the existing oracle build identity and warning policy.
- `tools/reference/adapter-inputs.txt` - Includes the probe source in the aggregate adapter content identity.
- `tools/xtask/src/upstream.rs` - Allows only the default oracle and the registered Phase 9 witness target through the upstream build command.
- `tools/xtask/tests/upstream_cli.rs` - Covers acceptance of the witness target and rejection of an unregistered target.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.json` - Records equal-expiration and strict-pruning semantic outcomes.
- `reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json` - Records the exact generation and content identity.
- `tools/xtask/src/provenance/phase9_witness.rs` - Validates schema, semantics, hashes, source identity, and generation command fail-closed.
- `tools/xtask/src/provenance.rs` - Includes the Phase 9 witness in full and Cargo-only provenance checks.
- `reference/source-map.toml` - Maps both generated records to the pinned C++ probe and explicitly excludes Rust-derived expectations.

## Decisions Made

- Used only public pinned-engine behavior for the witness: equal lifetimes are processed through `SolveLifetimes` and `DestroyOldestParticle`, while contact ties are produced through fixture overlap and strict pruning.
- Kept machine-facing results pointer-free and limited to stable scenario IDs, ordered semantic IDs, and keep/remove outcomes.
- Preserved the established shared oracle build identity and bound the new translation unit independently through its exact source digest and aggregate adapter digest because it has target-specific compile definitions.
- Recorded the truthful local D2 identity, AppleClang 21.0.0 on `arm64-apple-darwin25.5.0`, rather than implying canonical Linux D1 evidence.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added closed witness-target selection to the xtask build command**

- **Found during:** Task 1 (Build the narrow pinned-C++ lifecycle/contact probe)
- **Issue:** The plan's exact verification command passed `--target phase9-lifecycle-contact-witness`, but the existing closed xtask interface accepted only `--preset` and rejected the command before CMake could build the probe.
- **Fix:** Added a two-target allowlist containing the existing default oracle and the Phase 9 witness, plus positive and negative CLI coverage.
- **Files modified:** `tools/xtask/src/upstream.rs`, `tools/xtask/tests/upstream_cli.rs`
- **Verification:** The focused CLI test changed from rejection to acceptance, an unregistered target remains rejected, the exact planned configure/build command passed, and the full ordered Rust gate passed.
- **Committed in:** `522a6a2`

**2. [Rule 2 - Missing Critical] Registered fail-closed validation for the new standalone witness pair**

- **Found during:** Task 2 (Generate and verify semantic witness provenance)
- **Issue:** The required `cargo xtask provenance check` did not know about the new standalone artifacts, so it could have passed without checking their schema, source identity, hashes, or generation command.
- **Fix:** Added a dedicated validator, invoked it from both provenance paths, and added explicit source-map records for the witness and its provenance.
- **Files modified:** `tools/xtask/src/provenance/phase9_witness.rs`, `tools/xtask/src/provenance.rs`, `reference/source-map.toml`
- **Verification:** `cargo xtask provenance check` validated the exact witness SHA-256 and pinned revision; the full ordered Rust gate also passed.
- **Committed in:** `a23b5f0`

**Total deviations:** 2 auto-fixed (1 blocking, 1 missing critical)
**Impact on plan:** Both changes were required to run the literal plan verification and make that verification meaningful. They keep the entrypoint and provenance surface closed without changing witness scope.

## Issues Encountered

- The TDD red state demonstrated that the planned `--target` command was rejected, but it was not committed because repository policy requires the complete ordered Rust gate to pass before every commit. The green implementation was committed atomically with Task 1.
- The first C++ build used C++20 `std::set::contains` under the repository's C++17 mode. Replacing it with `find` preserved behavior and the next build passed.
- The new probe's target-specific compile definition makes its effective compile identity distinct from the four existing reference result units. The established shared identity was left intact, while the new probe is bound separately by its exact source and aggregate adapter digests.

## Validation Evidence

- Exact build command passed: `cargo xtask upstream configure --preset oracle-debug && cargo xtask upstream build --preset oracle-debug --target phase9-lifecycle-contact-witness`.
- The exact generation command ran twice against the same output paths; the two semantic files compared byte-identically.
- Witness SHA-256: `08d41d25f3766b9bf4bef51fb10713b7f925c074399b9642ad5cb4ce933fc8e3`.
- `cargo xtask provenance check` verified the Phase 9 witness and pinned upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- `git diff --exit-code -- third_party/liquidfun` passed and the submodule remained at the exact pinned gitlink.
- The ordered Rust gate passed after each task: `cargo fmt --all`, warning-denied Clippy, all-target/all-feature build, and all-feature tests, including 185 library tests, all integration targets, and 13 doctests.
- Final implementation diff review and `git diff --check` passed with no simplification change needed.

## User Setup Required

None.

## Next Phase Readiness

- Plans 09-06, 09-09, and 09-13 can consume the committed equal-expiration and strict-pruning outcomes without deriving expectations from Rust.
- The witness has exact pinned-source and truthful local D2 provenance. Any promotion to canonical Linux D1 evidence remains a separate future run against the canonical toolchain.

## Self-Check: PASSED

- Task commits `522a6a2` and `a23b5f0` exist.
- Both generated files, probe source, validator, and source-map records exist.
- The semantic file is hash-bound, generation is reproducible, and the pinned submodule is clean.
- `.planning/STATE.md` and `.planning/ROADMAP.md` were not modified.
- No push was performed.

***

*Phase: 09-particle-storage-lifecycle-and-coupling*
*Completed: 2026-07-14*
