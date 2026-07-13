## lesson-use-explicit-ci-identities | 2026-07-13 13:38

1. Date: 2026-07-13 13:38 CDT
1. What went wrong: Evidence-tier and promotion-authority tests derived their fixture identity from the current host, so the same test represented D2 locally on macOS but D1 on canonical Linux.
1. Preventive rule: Tests for evidence classification or promotion authority must construct an explicit identity for the tier under test and assert that tier before exercising the behavior.
1. Trigger signal to catch it earlier: If a test obtains compiler, platform, adapter, or provenance identity from the running build while expecting a fixed evidence tier, replace it with an explicit fixture.

## lesson-validate-all-canonical-presets | 2026-07-13 13:38

1. Date: 2026-07-13 13:38 CDT
1. What went wrong: Floating-point flags that appeared acceptable locally were unsupported, internally conflicting under canonical Clang with `-Werror`, or exposed a release-only warning after assertions were disabled.
1. Preventive rule: Validate compiler-option changes with the exact canonical compiler and strict warnings in both debug and release presets; prefer a coherent set of explicit controls over redundant umbrella options.
1. Trigger signal to catch it earlier: If a CMake change adds, removes, reorders, or overrides compiler flags, or if behavior differs under `NDEBUG`, probe and build every canonical preset before pushing.

## lesson-preserve-ci-isolation-boundaries | 2026-07-13 13:38

1. Date: 2026-07-13 13:38 CDT
1. What went wrong: A Cargo-only documentation contract invoked the full inventory check, which correctly requires the native source checkout that Cargo CI deliberately omits.
1. Preventive rule: Keep checked-in ledger and report validation separate from live-source discovery, and test both the isolated Cargo-only path and the full native-source path.
1. Trigger signal to catch it earlier: If a Cargo-only test invokes xtask, inventory, provenance, packaging, or documentation commands, verify that every transitive input exists in a checkout without submodules.
