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

## lesson-never-self-bless-oracle-bits | 2026-07-13 18:54

1. Date: 2026-07-13 18:54 CDT
1. What went wrong: An exact-bit standalone-rope fixture was provisionally updated from the Rust implementation under test after a guessed expectation failed.
1. Preventive rule: Exact compatibility fixtures must come from the pinned upstream oracle or an independently derived source-faithful calculation, never from the implementation being tested.
1. Trigger signal to catch it earlier: If an exact expected value is copied from an assertion's actual output, stop and reproduce the case with the pinned oracle before editing the fixture.

## lesson-verify-terminal-presentation-by-scenario-family | 2026-07-22 22:07

1. Date: 2026-07-22 22:07 CDT
1. What went wrong: The testbed renderer repair was declared complete after proving one joint checkpoint and labeling an empty particle teardown, without verifying that Run still presented useful particle geometry after the full particle lifecycle completed.
1. Preventive rule: For scenario-driven visual fixes, exercise Run through completion in each affected scenario family and distinguish the latest semantic truth from the checkpoint chosen for diagnostic presentation.
1. Trigger signal to catch it earlier: A reviewed scenario appends teardown or destruction actions, or a visual verification proves only an intermediate checkpoint while users primarily invoke Run.

## lesson-write-uat-checkpoints-as-concrete-assertions | 2026-07-22 22:16

1. Date: 2026-07-22 22:16 CDT
1. What went wrong: A UAT checkpoint named broad diagnostic categories but did not tell the tester which scenarios, labels, values, or count relationships constituted a pass.
1. Preventive rule: Write UAT expectations as a short executable procedure with named fixtures and exact visible assertions wherever the implementation defines deterministic labels or values.
1. Trigger signal to catch it earlier: A checkpoint asks the tester to confirm that information is “shown,” “helpful,” or “works” without naming the control path and at least one concrete expected output.

## lesson-preserve-exact-evidence-semantics | 2026-07-26 21:20

1. Date: 2026-07-26 21:20 CDT
1. What went wrong: A recovery contract used path-name-only set hashes where content digests were required, labeled JSONPath as a JSON pointer, and represented request authority with a truncated convenience ID.
1. Preventive rule: Translate evidence-contract nouns literally: content-set digests hash deterministic path-plus-content-hash pairs, JSON pointers use RFC 6901 escaping, and request identity records every full authority field.
1. Trigger signal to catch it earlier: A field named `content`, `pointer`, `exact`, or `identity` is implemented through a shorter surrogate such as path names, display syntax, or a truncated digest.
