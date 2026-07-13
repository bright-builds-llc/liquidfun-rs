# Debug Session: Main CI Failures at 4ee1b282

## Status

- State: resolved
- Started: 2026-07-13 13:00 CDT
- Resolved: 2026-07-13 13:01 CDT
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

- The patched CMake configure has not yet run on the GitHub-hosted canonical Linux image. The original CI log plus an exact standalone Clang 22.1.8 flag reproduction prove the removed option was the sole failed capability check, but remote end-to-end confirmation remains pending until the orchestrator pushes.
- The root orchestrator owns the full Rust pre-commit sequence and resulting workflow recheck.
