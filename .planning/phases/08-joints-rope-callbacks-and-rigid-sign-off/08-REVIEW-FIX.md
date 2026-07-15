---
status: all_fixed
findings_in_scope: 2
fixed: 2
skipped: 0
iteration: 1
generated_by: gsd-code-review-fix
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-15T02:14:17Z
---

# Phase 8 Code Review Fix Report

## Outcome

Both warning-severity findings from `08-REVIEW.md` were fixed in separate atomic commits. Local Rust, protocol, native differential, C++ oracle, sanitizer, replay, and determinism checks pass. Final refreshed canonical GitHub evidence binds the changed Phase 8 request fixture, both fixes, and the reviewed workspace-Clippy cleanup to exact code commit `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`.

## CR-08-25-01: Hook limit failures were not transactional

Status: fixed in `53a0b02263e06807eb7b7c93b9ba79025f531e36` (`fix(08): make hook limit failures transactional`).

The complete locked rigid step now snapshots and restores bodies, fixtures, joints, broad-phase state, contacts, continuous checkpoint state, and world configuration when an ordinary event or command `StepError::LimitExceeded` occurs. `ContinuousWorkLimitExceeded` retains its resumable partial-progress behavior.

RED evidence:

- The original event-limit retry lost buffered pair work and reported zero contacts instead of the clean world's two.
- The original command-limit retry exposed partial contact lifecycle mutation instead of matching a clean one-shot step.

GREEN evidence:

- `cargo test -p liquidfun --all-features --test hook_limit_transaction` passed two transaction and retry tests.
- Limits `0` and `1` are covered independently for events and commands.
- Tests assert immediate world diagnostics and object state after failure, then compare retry and clean one-shot lifecycle/report projections.
- `cargo test -p liquidfun --all-features --test hook_contract` passed all eight existing hook contract tests.

## CR-08-25-02: Corpus joint mutations were observational no-ops

Status: fixed in `e0b5106559b3c0c37beb44e4ade45c3b7919b59d` (`fix(08): require observable joint mutations`).

The request validator now resolves each mutation's target declaration and rejects every supported joint mutation whose value is unchanged. The three affected corpus actions now use distinct values: mouse target `(3, 2)`, motor correction factor `0.75`, and gear ratio `-2`. The C++ gear observer reads the live `b2GearJoint::GetRatio()` value instead of the declaration value.

RED evidence:

- Three independent regression tests changed the mouse target, motor correction factor, and gear ratio back to their declaration values; all three were initially accepted.

GREEN evidence:

- The three no-op regression tests now fail closed with `InvalidJointDefinition`.
- The closed mutation acceptance tables in the protocol and native executor use values distinct from their declarations and pass for every supported mutation kind.
- `cargo test -p liquidfun-test-protocol --all-features phase8` passed 18 tests.
- The focused differential suites passed: `rigid_world_phase8` (10), `phase8_comparator` (6), `rigid_world` (45), and `round_trip` (13).
- The updated request fixture SHA-256 is `a7f921abac1cbb488cce86a8ecfdb8faab4145f0784dbbce99b7fb89e65f43a8`.

## Verification

The mandatory Rust pre-commit sequence passed in order before each fix commit:

1. `cargo fmt --all`
1. `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo build --all-targets --all-features`
1. `cargo test --all-features`

Additional local Phase 8 evidence passed:

- Upstream verification, provenance check, and inventory check.
- Debug, release, and ASan/UBSan oracle configuration and builds.
- Debug and ASan/UBSan C++ protocol tests. Leak detection was omitted from the sanitizer test because the local macOS AddressSanitizer reports that leak detection is unsupported; address and undefined-behavior checks remained enabled and passed.
- Rigid-world debug and release comparisons matched all 19 required families.
- Debug replay matched all 19 required families.
- Two native and debug-oracle determinism runs were byte-identical.

The xtask fixed-evidence policy intentionally rejects an ASan/UBSan rigid-world compare session because reviewed fixed scenarios allow only the one-shot debug or release shapes. The sanitizer C++ protocol test passed independently.

## Canonical evidence refresh

The prior canonical GitHub run `29374708477` and commit `533c2ccf97b3921079baf7c339ddb4dad1a4038b` are superseded because they predate `53a0b02`, `e0b5106`, and the updated request fixture. Intermediate workflow-dispatch run `29379350740` at `e0b5106559b3c0c37beb44e4ade45c3b7919b59d` is also superseded because it predates the reviewed workspace-Clippy cleanup. Final successful workflow-dispatch run `29383445374` targets exact head `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`; its canonical Linux, sanitizer Linux, macOS, and Windows jobs all passed. Push-triggered Cargo CI run `29382964877` and Oracle CI run `29382964854` also passed at that exact head.

Exactly two unique unexpired artifacts were validated:

- `phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`
- `phase8-sanitizer-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`

Both identity records match run ID, head SHA, Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, upstream `7f20402173fd143a3988c921bc384459c6a858f2`, and policy `phase8-v1`; the job identities are `canonical-linux` and `sanitizer-linux` respectively.
