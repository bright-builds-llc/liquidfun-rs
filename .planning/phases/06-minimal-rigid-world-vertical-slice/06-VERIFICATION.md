---
phase: 06-minimal-rigid-world-vertical-slice
verified: 2026-07-12T07:45:18Z
status: gaps_found
score: "38/46 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T07:45:18Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - id: RIGD-01
    status: verified
  - id: RIGD-02
    status: blocked
  - id: RIGD-04
    status: blocked
must_haves:
  roadmap_success_criteria: 1/4
  plan_truths: 38/45
  repository_completion_gates: 0/1
  plan_artifacts: 36/36
gaps:
  - id: aggregate-mass-atomicity
    severity: critical
    requirements: [RIGD-02]
    plans: [06-03]
  - id: non-dynamic-contact-admission
    severity: warning
    requirements: [RIGD-04]
    plans: [06-04, 06-13]
  - id: ignored-step-parameters
    severity: warning
    requirements: [RIGD-04]
    plans: [06-08]
  - id: rigid-action-bound-mismatch
    severity: warning
    requirements: [RIGD-04]
    plans: [06-06, 06-10]
  - id: invalid-centered-inertia-boundary
    severity: warning
    requirements: [RIGD-02, RIGD-04]
    plans: [06-06, 06-10]
  - id: rigid-staging-not-integrated
    severity: warning
    requirements: [RIGD-04]
    plans: [06-12]
  - id: rigid-sanitizer-not-executed
    severity: warning
    requirements: [RIGD-04]
    plans: [06-13]
human_verification: []
evidence:
  focused_rust_suites: passed
  rigid_debug_compare: passed_d2
  rigid_release_compare: passed_d2
  rigid_replay: passed_d2
  rigid_determinism: passed_d0
  real_rigid_stage: rejected_exit_64
  review_findings_remaining: 7/7
---

# Phase 6: Minimal Rigid World Vertical Slice Verification Report

**Phase Goal:** Deliver the smallest complete native Rust rigid world that proves
object creation, destruction, contact lifecycle, and semantic differential
execution end to end.

**Verified:** 2026-07-12T07:45:18Z\
**Status:** gaps_found\
**Re-verification:** No — initial goal-backward verification after code review

## Verdict

Phase 6 does not yet achieve its goal. The fixed checked-in rigid timelines pass
native/C++ debug, release, replay, and two-run determinism, but seven reviewed
gaps remain in the actual code. One is a safe-public-API panic after partial
fixture/proxy mutation; the others break upstream contact admission, the shared
protocol/execution contract, the advertised regression-fixture path, or the
required sanitizer evidence.

Passing tests do not override these source-level contradictions. There are no
accepted verification overrides.

The verification was informed by the repository-local `AGENTS.md`,
`AGENTS.bright-builds.md`, `standards-overrides.md`, and the local architecture,
code-shape, testing, verification, and Rust standards. No substantive local
override applies.

## Goal Achievement

### Roadmap Success Criteria

| # | Observable truth | Status | Actual evidence |
| ---: | --- | --- | --- |
| 1 | Consumers can create, mutate, inspect, activate, deactivate, and destroy all three body types with stable typed identity. | VERIFIED | Public rigid-world, object-model, and definition suites pass. `World` retains world-scoped typed-handle validation and owned snapshots/destruction records. |
| 2 | Fixtures and sensors expose upstream-equivalent density, mass/inertia, material, filtering, and destruction behavior. | FAILED | Individual definitions are checked, but aggregate mass is computed infallibly after fixture/proxy/adjacency commit. Finite per-fixture values can overflow and reach an assertion, leaving partial mutation. |
| 3 | Contacts are created, persisted, filtered, updated, solved, and destroyed with upstream-equivalent manifold, material, warm-start, and sensor semantics. | FAILED | The supported static/dynamic witness works, but `pair_is_eligible` rejects only static/static. Static/kinematic and kinematic/kinematic pairs are incorrectly admitted, so contact creation is not upstream-equivalent. |
| 4 | Non-colliding and colliding steps pass through scenario, Rust adapter, C++ oracle, comparator, and regression-fixture paths. | FAILED | Compare/replay/D0 pass for the fixed request, but the real differential binary rejects `fixture stage --scenario rigid-world` with exit 64. Rust/C++ accept different action bounds, invalid centered inertia crosses the boundary, the native executor ignores authored step parameters, and the sanitizer lane never executes the rigid adapter. |

**Roadmap score:** 1/4

### Detailed Plan Truth Accounting

| Plan | Score | Status | Gap affecting the plan contract |
| --- | ---: | --- | --- |
| 06-01 checked definitions | 3/3 | VERIFIED | Direct body/fixture definitions and `BodyMassData` validation are substantive and tested. |
| 06-02 handle-oriented world | 3/3 | VERIFIED | Typed create/inspect/mutate/destroy paths and invalid-handle atomicity are present. |
| 06-03 proxy and fixture side effects | 3/4 | FAILED | Aggregate fixture mass/inertia is not fallible or validate-before-commit. |
| 06-04 contact manager | 4/5 | FAILED | Non-dynamic/non-dynamic contact admission differs from pinned `ShouldCollide`. |
| 06-05 bounded solver | 4/4 | VERIFIED | Fixed one-contact solve, feature-based warm start, sensor bypass, order, and unsupported-topology preflight are present and tested. |
| 06-06 rigid protocol model | 2/3 | FAILED | `SetCustomMassData` does not validate centered inertia at the protocol boundary. |
| 06-07 policy and schemas | 3/3 | VERIFIED | Closed policy/schema rendering is substantive and byte-stable; the cross-adapter bound mismatch is accounted under 06-10. |
| 06-08 native adapter | 1/2 | FAILED | `RigidWorldAction::Step { .. }` discards timestep and iteration fields and always uses defaults. |
| 06-09 comparator and supervision | 4/4 | VERIFIED | Declaration-first comparison, stable divergence identity, ordering, and harness/physics classification are present for accepted results. |
| 06-10 C++ adapter | 2/3 | FAILED | C++ action bounds disagree with Rust, and invalid centered inertia is not rejected before `b2World` execution. |
| 06-11 build identity | 3/3 | VERIFIED | Rigid sources and translation-unit command identity are wired into the reviewed adapter identity. |
| 06-12 evidence workflows | 3/4 | FAILED | Compare/replay/D0 work, but the advertised rigid fixture staging path is rejected by the real runner and the D1 authority helper is not integrated into production staging/promotion. |
| 06-13 documentation/sign-off | 3/4 | FAILED | The ledger marks the contact-manager subsystem differentially validated although the fixed corpus omits the non-dynamic admission branch. |

**Plan truth score:** 38/45

The additional repository completion gate also fails: the scheduled ASan/UBSan
job builds `rigid_world.cpp` but executes only `empty-world` scenarios.

**Overall must-have score:** 38/46

## Required Artifact and Wiring Verification

All 36 unique artifacts declared by the plan frontmatter exist and are non-empty.
The generic GSD artifact helper reported zero parsed artifacts because these
plans use plain-string artifact entries, so existence and wiring were checked
manually and through focused execution.

| Artifact/link cluster | Existence | Wiring status | Evidence |
| --- | --- | --- | --- |
| Public body, fixture, world, proxy, contact, and solver modules | PRESENT | PARTIAL | Consumer and focused tests pass, but fixture creation commits at `world/object.rs:490-506` before infallible aggregate mass at `world/body.rs:406-436`. |
| Broad phase to contact manager | PRESENT | INCORRECT | `world/contact_manager.rs:386-389` rejects only static/static rather than every pair with no dynamic body. |
| Typed rigid request to native execution | PRESENT | INCORRECT | `liquidfun-differential/src/rigid_world.rs:402-405` discards all `Step` fields and uses `StepLimits::default()`. |
| Rust protocol/schema to C++ decoder | PRESENT | INCORRECT | Rust permits 128 actions at `validation.rs:30`; C++ permits 64 at `rigid_world_decode.hpp:395-400` and `617-619`. |
| Custom mass request to both engines | PRESENT | INCORRECT | Rust validation at `validation.rs:380-390` checks mass/center/inertia independently but not centered inertia; the invalid state can reach native rejection or the C++ assertion path. |
| Xtask rigid stage to differential binary/storage authority | PRESENT | NOT WIRED | Xtask advertises `rigid-world`, while the real runner rejects every non-`empty-world` scenario at `main.rs:252-253`. `validate_rigid_promotion_authority` has no production caller. |
| Oracle sanitizer workflow to rigid C++ adapter | PRESENT | NOT WIRED | `.github/workflows/oracle.yml:180-183` runs only `empty-world` under ASan/UBSan. |
| Compatibility ledger to executed evidence | PRESENT | OVERCLAIMED | `subsystem.contacts-and-filtering` and `b2ContactManager.h` are marked differentially validated despite the omitted non-dynamic admission witness. |

## Gap Details

### 1. Aggregate mass can panic after partial mutation

`World::create_fixture` inserts the fixture, creates broad-phase entries, and
links body adjacency before calling mass reset. `BodyState::reset_mass_data`
then performs unchecked `f32` additions/division/subtraction and asserts that
centered inertia remains positive. Individually valid fixture mass data can
overflow in aggregate, producing infinity/NaN and a panic after the new
topology is already visible.

This violates RIGD-02 and the repository's validate-before-commit requirement.
The same infallible aggregation is reachable through explicit mass reset.

Required closure:

- make source-ordered aggregate mass calculation fallible;
- validate every aggregate operation before body or fixture mutation;
- precompute post-create aggregate before inserting a positive-density fixture;
- add create/reset regression tests proving fixture count, proxy count,
  adjacency, and body mass remain unchanged on rejection.

### 2. Contact admission accepts two non-dynamic bodies

The pinned body predicate rejects a pair unless at least one body is dynamic.
Current Rust code rejects only static/static, so overlapping static/kinematic
and kinematic/kinematic fixtures can create contacts. The fixed corpus keeps
these declarations separated and therefore cannot detect the mismatch.

Required closure:

- match the pinned no-dynamic-body predicate;
- add focused static/kinematic and kinematic/kinematic overlap tests;
- add at least one declaration-first oracle witness before restoring the
  contact-manager differential claim.

### 3. Native execution ignores validated step parameters

The request accepts positive timestep bits and iteration counts up to 255, and
the C++ adapter executes those values. The native adapter matches
`RigidWorldAction::Step { .. }` and always uses the default fixed step. A request
can therefore be accepted while the two engines execute different authored
inputs.

Required closure: either pass the tuple into the native step/solver or reject
every tuple except the deliberately fixed Phase 6 values in Rust validation,
generated schemas, and C++ decoding, with boundary tests.

### 4. Rust and C++ action bounds disagree

The Rust authoritative boundary permits 128 actions; C++ rejects more than 64.
Requests containing 65-128 otherwise valid actions can pass native validation
and fail only as an oracle harness error.

Required closure: establish one contract value used by Rust validation,
generated schema, and C++ decode checks; test the accepted maximum and
maximum-plus-one through the real oracle boundary.

### 5. Invalid centered custom inertia crosses the protocol boundary

Protocol validation accepts positive mass, finite center, and nonnegative
origin inertia independently. It does not require
`inertia - mass * dot(center, center)` to be finite and nonnegative. The native
engine later rejects this via `BodyMassData`; the C++ path can reach a pinned
assertion. This is malformed input and should be rejected as such before either
engine has effects.

Required closure: perform the same source-ordered centered-inertia validation
in Rust and C++ decoders and cover it in schema/runtime/C++ boundary fixtures.

### 6. Rigid staging is advertised but rejected by the real runner

Focused xtask tests use a fake child and prove only the delegated argument
shape. A real invocation produced:

```text
differential command failed: usage: liquidfun-differential fixture stage --scenario empty-world ...
exit_status=64
```

The standalone `validate_rigid_promotion_authority` helper is exercised by unit
tests but not called from a production staging or promotion path.

Required closure: implement a real rigid stage/replay transaction that validates
the request/result/build identity and comparison before any candidate write,
calls the D1 authority guard, and reuses the existing confined lifecycle
storage. Add a real-binary end-to-end test for canonical acceptance and D2
rejection.

### 7. Sanitizers do not execute the Phase 6 C++ surface

The scheduled sanitizer job compiles the rigid adapter but runs only the
empty-world one-shot/reuse corpus. Rigid decode, pointer-to-semantic-ID maps,
contact bookkeeping, destruction, and trace encoding are not executed under
ASan/UBSan.

Required closure: run CTest/reference protocol tests under the sanitizer preset
and a fail-fast `rigid-world` sanitizer comparison before the read-only check.

## Requirement Accounting

| Requirement | Status | Verification evidence |
| --- | --- | --- |
| RIGD-01 | SATISFIED | Checked definitions and public typed-handle operations cover creation, mutation, inspection, activation/deactivation, and destruction for static, kinematic, and dynamic bodies. Focused body/object suites pass, and no contradictory review finding affects this body identity contract. |
| RIGD-02 | BLOCKED | Fixture/sensor/material/filter/destruction paths largely exist, but aggregate mass/inertia can panic after partial mutation. Protocol custom mass also accepts invalid centered inertia before engine execution. Upstream-equivalent safe behavior is not established. |
| RIGD-04 | BLOCKED | The fixed static/dynamic contact witness passes, but non-dynamic contact admission is wrong; authored step parameters are ignored by native execution; Rust/C++ request contracts differ; staging is broken; and rigid sanitizer execution is absent. |

No additional Phase 6 requirement IDs are orphaned: ROADMAP and all thirteen
PLAN frontmatter blocks consistently claim only RIGD-01, RIGD-02, and RIGD-04.

## Automated Verification Evidence

The following non-destructive checks passed on the current tree:

1. `cargo test -p liquidfun --all-features`
1. `cargo test -p liquidfun-test-protocol --all-features`
1. `cargo test -p liquidfun-differential --all-features --test rigid_world`
1. `cargo test -p xtask --test differential_cli`
1. `git diff --check`
1. Debug and release rigid-world comparison: two required families matched as D2.
1. Debug rigid-world replay: two required families matched as D2.
1. Debug two-run determinism: byte-identical native/oracle D0.

The real rigid fixture-stage probe failed as described above. No tracked source
or generated evidence file was changed by verification. The existing unrelated
`.planning/config.json` worktree change was preserved.

## Residual Evidence Notes

- Local oracle runs used CMake 3.27.9 and Apple Clang 21, so they are correctly
  D2 rather than canonical D1/platform evidence.
- This report does not request human verification: every blocking gap is
  mechanically visible in source or reproduced by the real command surface.
- The fixed timeline passes remain useful evidence, but they are insufficient
  to close a goal whose accepted input and lifecycle surfaces are internally
  inconsistent.

***

_Verifier: gsd-verifier_\
_Result: gaps_found_
