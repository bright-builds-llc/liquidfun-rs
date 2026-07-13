# Debug Session: Main CI Failures at 4ee1b282

## Status

- State: resolved
- Started: 2026-07-13 13:00 CDT
- Resolved: 2026-07-13 13:22 CDT
- Goal: find and fix
- Approved scope: rigid promotion authority regression test and canonical Clang floating-point capability configuration

## Symptoms

- Expected: `cargo test --workspace --all-features` passes on canonical Linux with Rust 1.97.0.
- Actual: `supervisor_rejects_local_d2_rigid_output_for_promotion` receives `Ok(())` instead of `RigidPromotionError::NonCanonicalAuthority`.
- Expected: canonical Linux Clang 22.1.8 configures the `oracle-debug` preset.
- Actual: CMake rejects the compiler because `-fdenormal-fp-math-fp32=ieee` fails its capability probe.

## Reproduction

- Cargo CI run `29270221126`, at `crates/liquidfun-differential/tests/rigid_world.rs:964`.
- Oracle CI run `29270221193`; every required FP option probe succeeds except `-fdenormal-fp-math-fp32=ieee`.
- The rigid promotion test passes locally on macOS because the live native identity is D2 there, confirming that its result depends on the host platform.
- AppleClang 21 also rejects `-fdenormal-fp-math-fp32=ieee`; LLVM's driver option table spells the distinct f32 frontend option `-fdenormal-fp-math-f32`, while the supported general option already applies its mode to both the general and f32 denormal settings.

## Working Hypotheses

1. The promotion test uses the current build identity as D2 test data even though canonical Linux/rustc 1.97.0 intentionally classifies that identity as D1.
1. The CMake requirement contains a nonexistent `fp32` driver-option spelling and redundantly requires a per-f32 flag after the general denormal mode has already selected IEEE semantics for all floating types.

## Confirmed Root Cause

1. `EmptyWorldAdapter::new` derives identity from compile-time host metadata. On canonical Linux with rustc 1.97.0, `classify_evidence_tier` correctly returns `D1Canonical`, so promotion correctly succeeds. The test's assumption that the live local identity is always D2 is invalid.
1. `tools/reference/CMakeLists.txt` requires `-fdenormal-fp-math-fp32=ieee`, but that is not a Clang driver option. `-fdenormal-fp-math=ieee` is supported and sets both the general and f32 compiler denormal modes; the oracle additionally reports an actual f32 gradual-underflow runtime witness.

## Resolution

- Reused the existing explicit, platform-independent D2 identity in `rust_adapter.rs`, asserted its tier, and exercised rigid promotion rejection there.
- Removed the host-dependent rigid-world test instead of duplicating provenance construction in the large integration suite.
- Removed the nonexistent, redundant `fp32` capability requirement and derived configured denormal mode from the supported general IEEE option.

## Verification

- Exact Ubuntu 24.04 / Clang 22.1.8 reproduction: direct `-fdenormal-fp-math-fp32=ieee` compilation fails as an unknown argument, matching the CMake probe; the general `-fdenormal-fp-math=ieee` probe succeeds.
- `cargo fmt --all -- --check` — passed.
- `cargo test -p liquidfun-differential --all-features --test rust_adapter` — all 8 tests passed, including the explicit D2 rigid-promotion rejection.
- `target/debug/xtask upstream configure --preset oracle-debug` — passed locally with AppleClang 21.0.0 and the independently recomputed adapter digest.
- `target/debug/xtask upstream build --preset oracle-debug` — passed; generated compile commands apply `-fdenormal-fp-math=ieee` and contain no nonexistent `fp32` option.
- `git diff --check` — passed.

## Residual Risk

- The release-only upstream compatibility and Cargo-only report-check fixes have not yet rerun on GitHub-hosted canonical Linux. Focused local and isolated-fixture checks cover both changes, but remote end-to-end confirmation remains pending until the orchestrator pushes.
- The root orchestrator owns the full Rust pre-commit sequence and resulting workflow recheck.

## Follow-on: Canonical Clang Option Ordering

### Evidence

- Post-push Oracle run `29273063424` passed `Configure oracle-debug`, proving removal of the nonexistent `fp32` flag fixed the original configure failure.
- `Build oracle-debug` then failed every first-wave Box2D compile because Clang 22.1.8 reported `overriding '-ffp-model=precise' option with '-ffp-contract=off'` and upstream's `-Werror` promoted the warning.
- AppleClang 21 reproduces the same diagnostic with the exact required FP option sequence under `-Werror`.
- Removing only `-ffp-model=precise` makes the remaining required option sequence compile cleanly while retaining `-fno-fast-math`, explicit contraction disablement, signed zeros, NaN and infinity honoring, and IEEE denormal handling.

### Confirmed Root Cause

`-ffp-model=precise` is an umbrella option whose precise mode enables its own contraction policy. The immediately following, deliberately stricter `-ffp-contract=off` overrides that umbrella setting. The combination is semantically intentional but internally contradictory, and Clang 22 diagnoses it. Suppressing the warning would preserve a redundant conflict in the canonical command and weaken the usefulness of `-Werror`.

### Resolution

- Removed the redundant `-ffp-model=precise` umbrella option.
- Kept the explicit deterministic FP controls, especially `-ffp-contract=off`, rather than weakening contraction policy or suppressing the diagnostic.
- Continued reporting the validated FP model as `precise` only when every explicit required FP control is supported.

### Verification

- Exact Ubuntu 24.04 compile using the workflow checksum-pinned `llvm.sh` and Ubuntu Clang 22.1.8 (`++20260613092238+e80beda6e255...`) with `-Werror` and the remaining required FP option set — passed without an override diagnostic.
- Direct AppleClang 21 compile with `-Werror` and the remaining required FP option set — passed without an override diagnostic.
- `target/debug/xtask upstream configure --preset oracle-debug` — passed with the recomputed adapter digest.
- `target/debug/xtask upstream build --preset oracle-debug` — all 64 steps passed, including the previously failing first-wave Box2D objects.
- Debug `compile_commands.json` inspection — every reviewed probe command retains `-fno-fast-math -ffp-contract=off -fsigned-zeros -fhonor-nans -fhonor-infinities -fdenormal-fp-math=ieee` and contains no `-ffp-model=precise`.
- `target/debug/xtask upstream configure --preset oracle-release` and `target/debug/xtask upstream build --preset oracle-release` — passed all 64 steps.
- `target/debug/xtask differential compare --scenario math-probes --preset oracle-debug --session-profile one-shot` — all 39 ordered cases matched under `phase4-v1`.
- `git diff --check` — passed.

## Follow-on: Release-Only Upstream Assertion Variable

### Evidence

- Oracle run `29273593108` passed canonical configure, the complete debug build, and release configure after the FP option-ordering fix.
- The release build compiled 63 of 64 steps and failed only in the read-only upstream `b2DynamicTree.cpp:664`, where Clang 22.1.8 reported `freeCount` as `-Wunused-but-set-variable` under `-O3 -DNDEBUG`.
- `freeCount` is incremented while walking the free list and consumed only by `b2Assert(m_nodeCount + freeCount == m_nodeCapacity)`. The upstream release configuration compiles that assertion away, leaving the legacy validation loop's local write-only.

### Confirmed Root Cause

The pinned 2014 upstream source assumes its assertion-only validation bookkeeping is acceptable in release builds. Modern Clang diagnoses the resulting write-only local, and the upstream target's own `-Werror` promotes that compatibility warning. Repository-authored C++ does not have this warning and must retain strict `-Werror`.

### Resolution

- Added `-Wno-error=unused-but-set-variable` only for Release builds to the existing Clang compatibility options attached privately to the read-only `Box2D` target.
- Kept the warning enabled and visible; only its promotion to an error is disabled.
- Left repository-authored reference targets unchanged and did not modify the pinned upstream checkout.

### Verification

- Exact Ubuntu 24.04 compile using the workflow checksum-pinned `llvm.sh` and Ubuntu Clang 22.1.8 (`++20260613092238+e80beda6e255...`) with the complete release command for upstream `b2DynamicTree.cpp` — passed; the diagnostic remained visible as one warning.
- `target/debug/xtask upstream configure --preset oracle-release` — passed with the recomputed adapter digest.
- `target/debug/xtask upstream build --preset oracle-release` — passed; the changed target options rebuilt all 53 upstream Box2D objects, including the previously failing `b2DynamicTree.cpp`, then linked the complete oracle.
- Debug/release `compile_commands.json` scoping assertion — `-Wno-error=unused-but-set-variable` is present on upstream release `b2DynamicTree.cpp`, absent from its debug command, and absent from repository-authored `protocol.cpp` in both presets.
- `target/debug/xtask differential compare --scenario math-probes --preset oracle-release --session-profile one-shot` — all 39 ordered cases matched under `phase4-v1`.
- `git diff --check` — passed.

## Follow-on: Cargo-Only Compatibility Report Contract

### Evidence

- Cargo run `29273593160` passed the previously failing promotion tests and failed only `phase5_compatibility_report_matches_authoritative_ledger`.
- The test name promises report/ledger parity, but it invoked full `inventory check`, which additionally rescans the native source tree to prove the checked-in discovery snapshot is current.
- Cargo CI intentionally checks out without submodules. The missing `third_party/liquidfun/liquidfun/Box2D` tree therefore caused `inventory/discovery: missing allowlisted discovery root` before the report parity assertion could complete.
- The report is rendered entirely from the validated `reference/compatibility.json` ledger. The checked-in `reference/discovery.json` is still needed for schema, revision, and coverage validation, but report parity does not require rescanning native source bytes.

### Confirmed Root Cause

The docs contract accidentally selected the broader inventory validation boundary. Live discovery is correct and required for ordinary `inventory check`, but it is unrelated to proving that checked-in `COMPATIBILITY.md` exactly matches the authoritative validated ledgers. Conflating the two made a Cargo-only contract depend on the deliberately absent native oracle checkout.

### Resolution

- Added read-only `cargo xtask inventory check-report`, which validates compatibility and discovery ledger schemas, pinned revision, ledger coverage, and exact generated report bytes without live source discovery.
- Changed the docs contract test to invoke `check-report` so its behavior matches its name.
- Kept ordinary `inventory check` unchanged: it still performs live discovery before validating report bytes.
- Added command-level regression coverage that removes the synthetic fixture's entire `third_party` tree, proves `check-report` succeeds from validated ledgers, and proves full `check` still fails with `inventory/discovery`.

### Verification

- Focused `phase5_compatibility_report_matches_authoritative_ledger` — passed against the repository's real 177-row authoritative ledgers and report.
- Focused `report_check_uses_validated_ledgers_without_native_sources` — passed; the same fixture's full inventory check failed at the preserved discovery boundary as expected.
- Temporary isolated root containing copies of the repository's actual upstream lock, compatibility ledger, discovery ledger, and generated report, with no `third_party` path — `target/debug/xtask inventory check-report` passed for all 177 rows.
- Full `cargo test -p xtask --test inventory_cli` — 7 passed.
- Full `cargo test -p xtask --test docs_contract` — 29 passed.
- `cargo clippy -p xtask --all-targets --all-features -- -D warnings` — passed.
- `cargo xtask inventory check-report` — passed for 177 rows with the current submodule present.
- `cargo xtask inventory check` — passed for 177 rows, confirming live discovery validation remains intact.
- `cargo fmt --all -- --check` and `git diff --check` — passed.
