---
phase: 06-minimal-rigid-world-vertical-slice
verified: 2026-07-12T17:43:27Z
status: gaps_found
score: "55/61 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 6-2026-07-12T02-22-53
generated_at: 2026-07-12T17:43:27Z
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
  roadmap_success_criteria: 2/4
  plan_truths: 55/60
  repository_completion_gates: 0/1
  plan_artifacts: 43/43
gaps:
  - id: implicit-aggregate-mass-atomicity
    severity: critical
    requirements: [RIGD-02]
    plans: [06-03, 06-14]
  - id: zero-centered-inertia-boundary
    severity: critical
    requirements: [RIGD-02, RIGD-04]
    plans: [06-01, 06-16]
  - id: rigid-fixture-checkout-provenance
    severity: critical
    requirements: [RIGD-04]
    plans: [06-12, 06-17]
human_verification: []
evidence:
  mandatory_rust_sequence: passed
  focused_gap_suites: passed
  cpp_protocol_debug_fresh_build: passed_d2
  rigid_debug_compare: passed_d2
  rigid_release_compare: passed_d2
  rigid_replay: passed_d2
  rigid_determinism: passed_d0
  rigid_sanitizer_protocol: passed_d2
  rigid_sanitizer_compare: passed_d2
  implicit_mass_transition_probe: reproduced_two_partial_mutation_panics
  zero_centered_inertia_probe: accepted_incorrectly
  review_findings_remaining: 3/3
---

# Phase 6: Minimal Rigid World Vertical Slice Verification Report

**Phase goal:** Deliver the smallest complete native Rust rigid world that proves object creation, destruction, contact lifecycle, and semantic differential execution end to end.

**Verified:** 2026-07-12T17:43:27Z
**Status:** `gaps_found`
**Re-verification:** Yes — after gap plans 06-14 through 06-18

## Verdict

Phase 6 is not complete. The five gap plans substantively close most of the original seven findings: contact admission, the fixed step tuple, the shared 128-action bound, real rigid staging dispatch, and actual sanitizer execution now have focused and end-to-end evidence. Aggregate fixture creation and explicit reset are transactional, and negative/non-finite centered inertia is rejected.

Three critical contradictions remain in actual code:

1. body-type changes and fixture destruction still route user-constructible aggregate mass failure through `expect` after mutation;
2. centered inertia equal to zero is accepted even though the pinned debug oracle asserts and release code divides by zero for positive origin inertia; and
3. rigid fixture staging derives D1 from the child handshake without recomputing the adapter-source and effective compile-command digests against the current checkout.

These are mechanically reproduced correctness and provenance failures, not human-review questions. Passing fixed-corpus comparisons cannot override them. The phase must not be marked complete and there are no accepted overrides.

This verification applied the repo-local GSD workflow in `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the local architecture, code-shape, testing, verification, and Rust standards. The material rules were validate-before-mutate transitions, parse-at-boundary parity, semantic rather than raw-memory evidence, source-bound provenance, warning-denied Rust verification, and no Phase 7 scope expansion. No active standards override applies.

## Goal Achievement

### Roadmap success criteria

| # | Observable truth | Status | Current evidence |
| ---: | --- | --- | --- |
| 1 | Consumers can create, mutate, inspect, activate, deactivate, and destroy static, kinematic, and dynamic bodies with stable typed identity. | VERIFIED | Checked definitions, world-scoped handles, snapshots, activation, transforms, and destruction tests pass. Invalid handles remain no-effect. The tracked `REQUIREMENTS.md` checkbox/table still says RIGD-01 pending, but actual code and tests satisfy the requirement. |
| 2 | Fixtures and sensors expose upstream-equivalent density, mass/inertia, friction, restitution, filtering, and destruction behavior. | FAILED | Explicit create/reset overflow paths are fixed, but `set_body_type` and `destroy_fixture` can still panic after mutation on a supported density-edit sequence. Positive origin inertia with exactly zero centered inertia is also accepted at the public and protocol boundaries. |
| 3 | Contacts are created, persisted, filtered, updated, and destroyed with correct manifolds, material mixing, warm-start state, and sensor semantics. | VERIFIED within the Phase 6 bounded slice | Static/dynamic lifecycle, persistence, features, material, warm start, sensor, filtering, activation, and destruction suites pass. Static/kinematic and kinematic/kinematic overlaps now remain contact-free in Rust and declaration-first Rust/C++ evidence. Broader islands remain Phase 7. |
| 4 | Non-colliding and colliding world steps pass through scenario, Rust adapter, C++ oracle, comparator, and regression-fixture paths. | FAILED | Debug/release/replay/D0 and sanitizer compare pass, and the real fixture command now dispatches. However, the staging/promotion path can accept a stale canonical-looking child because it does not bind adapter and compile digests to this checkout. |

**Roadmap score:** 2/4

### Plan truth accounting

There are 60 declared plan truths across 06-01 through 06-18. Fifty-five are verified. Five are contradicted:

| Plan | Contradicted truth |
| --- | --- |
| 06-01 | “Invalid centered inertia” is not fully rejected: the equality boundary reaches `Ok` for positive origin inertia. |
| 06-03 | Type and destruction mass-reset triggers do not follow checked atomic behavior for aggregate overflow. |
| 06-17 | The real rigid stage performs identity checks but does not validate that the reported adapter and compile-command digests match the current checkout before its first write. Its replay/review/promotion D1 recheck has the same source-binding defect. |
| 06-18 | Formal re-verification cannot mark RIGD-02 and RIGD-04 verified while the three critical findings remain. |

All 43 unique plan artifacts named across the 18 plans exist and are non-empty. The repository completion gate fails because code review and this formal verifier both retain critical findings.

**Overall must-have score:** 55/61

## Original Seven-Gap Recheck

| Original gap | Status | Actual evidence |
| --- | --- | --- |
| `aggregate-mass-atomicity` | PARTIAL / superseded by critical gap | Plan 06-14 correctly makes positive-density fixture creation and explicit reset fallible and no-effect. Focused regressions pass. The same fallible aggregate is still converted to `expect` by implicit type/destruction reset paths. |
| `non-dynamic-contact-admission` | CLOSED | `pair_is_eligible` requires at least one dynamic body. Both overlapping non-dynamic combinations pass focused tests and exact declaration-first Rust/C++ witnesses. |
| `ignored-step-parameters` | CLOSED | Rust, schema, native defense, and C++ admit only `0x3c888889`, 8, 3. Alternate-lane tests fail closed. |
| `rigid-action-bound-mismatch` | CLOSED | Rust, schema, and C++ share 128; 128 is accepted and 129 rejected. |
| `invalid-centered-inertia-boundary` | PARTIAL / still open at equality | Negative and non-finite centered results are rejected in Rust and C++. Equality zero remains accepted and is unsafe for the pinned positive-origin-inertia branch. |
| `rigid-staging-not-integrated` | PARTIAL / provenance-critical | The real command now stages/replays/promotes and checks a reported D1 tier before effects. It does not recompute checkout adapter and compile digests, so its D1 claim is not source-bound. |
| `rigid-sanitizer-not-executed` | CLOSED | Fresh local ASan/UBSan CTest and rigid compare execute successfully, and the workflow orders both before the read-only assertion with bounded failure upload. Local authority remains D2. |

## Critical Gap Details

### 1. Implicit aggregate mass resets panic after partial mutation

`World::set_body_type` destroys contacts and mutates the body type before calling `reset_body_mass_after_validation`. `remove_fixture` destroys contacts/proxies, removes fixture storage and adjacency, then calls the same helper. That helper calls `prepare_body_mass_state(...).expect(...)` even though supported APIs can create an invalid aggregate at the moment of either implicit reset:

- static bodies may hold two individually finite high-density fixtures because their current mass is zero;
- dynamic bodies may attach zero-density fixtures and independently raise each density without an implicit reset.

A temporary ignored-target probe, removed after execution, used `f32::MAX / 4.0` circle densities and `catch_unwind`. It reproduced:

```text
set_body_type_panicked=true
body_type_after=Dynamic
destroy_fixture_panicked=true
destroyed_fixture_still_live=false
proxy_counts_before_after=3,2
```

Both panics originated at `crates/liquidfun/src/world/object.rs:1234` with `NonFiniteMass`. The caught unwind proves partial visible mutation: the type changed, and fixture/proxy state was removed.

Required closure:

- precompute the prospective aggregate and complete candidate body state before contact/type mutation;
- precompute the remaining-fixture aggregate before contact/proxy/storage/adjacency removal;
- return typed aggregate errors from body-type and fixture-destruction public operations;
- add no-effect regressions covering type, contacts, fixtures, proxies, adjacency, and mass bits.

### 2. Zero centered inertia reaches an unsafe pinned branch

`BodyMassData::new`, Rust protocol validation, and the C++ decoder reject only centered results below zero. A temporary probe confirmed:

```text
BodyMassData::new(1.0, Vec2::new(1.0, 0.0), 1.0).is_ok() == true
```

The pinned `b2Body::SetMassData` implementation enters its positive-origin-inertia branch, subtracts the parallel-axis term, asserts `m_I > 0.0f`, and then computes `1.0f / m_I`. Equality therefore aborts a debug oracle and produces an infinite inverse inertia in release.

Required closure:

- when origin inertia is positive, require centered inertia to be strictly positive in `BodyMassData::new`, Rust protocol validation, and C++ decode;
- continue allowing origin inertia zero through the pinned no-inertia branch;
- add matching public-domain, protocol-fixture, and C++ equality tests.

### 3. Rigid fixture D1 is not bound to the current checkout

The ordinary rigid compare path recomputes both:

- `upstream::adapter_source_digest(repository_root)`, and
- `effective_compile_command_sha256(repository_root, preset)`.

`stage_rigid_candidate` performs neither check. It validates the child handshake, its requested preset, semantic comparison, and the tier derived from the same self-reported identity, then calls `validate_rigid_promotion_authority` before writing. Replay checks the stored identity against candidate metadata generated from that same identity. The real-binary test fake explicitly emits arbitrary digest strings, including compile digest `"11".repeat(32)`, and the canonical-looking fixture test passes.

This means a stale canonical binary can stage and promote evidence while metadata records the current generator revision.

Required closure:

- share the ordinary rigid compare adapter/compile digest validator with rigid staging before any directory creation;
- revalidate the binding during review/promotion or immutably bind and verify the candidate's generator inputs;
- add stale-adapter and stale-compile-digest real-binary rejection tests proving no staging, accepted artifact, or manifest mutation.

## Requirement Accounting

| Requirement | Status | Evidence |
| --- | --- | --- |
| RIGD-01 | VERIFIED | All three body types use checked definitions and stable typed handles across create, inspect, mutate, activation, deactivation, and destroy. Focused body/object suites pass. `REQUIREMENTS.md` still records this as pending and should be reconciled only after the correctness gaps are fixed and verification is rerun. |
| RIGD-02 | BLOCKED | Fixture material/filter/sensor behavior is present, but implicit mass-reset public transitions can panic after partial mutation, and the zero-centered-inertia equality is accepted. |
| RIGD-04 | BLOCKED | Contact behavior and fixed differential execution pass, but malformed equality can reach the oracle and canonical fixture evidence is not bound to checkout source/compile identity. |

No additional Phase 6 requirement IDs are claimed by any plan.

## Automated Verification Evidence

The following commands passed on commit `d83b0e8b9cf1e31100d63ebea111a8db4809c3b6` before this report write.

### Mandatory Rust sequence

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

### Focused gap and workflow suites

1. `cargo test -p liquidfun-test-protocol rigid_world --all-features` — 18 passed.
2. `cargo test -p liquidfun-test-protocol --test fixtures --all-features` — 10 passed.
3. `cargo test -p liquidfun-differential --test rigid_world --all-features` — 13 passed.
4. `cargo test -p liquidfun-differential --test rigid_fixture_workflow --all-features` — 4 passed.
5. `cargo test -p xtask --test differential_cli --all-features` — 27 passed.
6. `cargo test -p xtask --test docs_contract --all-features` — 25 passed.

### Fresh C++ and differential evidence

1. Debug configure/build and `liquidfun-reference-protocol-tests`: CTest 1/1 passed.
2. `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot`: two families matched, D2.
3. Equivalent `oracle-release` compare: two families matched, D2.
4. Debug rigid replay: two families matched, D2.
5. Debug `verify-determinism --runs 2`: byte-identical native and oracle runs, D0.
6. Fresh `oracle-asan-ubsan` build, protocol CTest, and one-shot rigid compare under fail-fast ASan/UBSan: passed, D2.

Local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8, so none of these local runs is represented as D1 or platform evidence.

### Repository checks

1. `cargo xtask docs check`
2. `cargo xtask inventory check` — 177 rows.
3. `cargo xtask provenance check` — pinned oracle and one artifact record verified.
4. `cargo xtask package verify` — 58 entries verified outside the repository.
5. `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps`
6. GSD lifecycle validation — valid.
7. GSD phase completeness — all 18 plan summaries present.
8. `git diff --check`

## Lifecycle and Artifact Notes

- All 18 plans have matching summaries and their task commits are present in history.
- `.planning/STATE.md` correctly remains `status: verifying` and says formal verification is pending.
- `.planning/ROADMAP.md` currently labels Phase 6 completed, while this formal report is `gaps_found`; that completion marker must not be treated as authoritative until the critical gaps are fixed and this report passes.
- `.planning/REQUIREMENTS.md` marks RIGD-02 and RIGD-04 complete despite the remaining contradictions, while RIGD-01 is still pending despite passing actual verification. State/roadmap/requirement accounting needs reconciliation after the fixes, through GSD tooling rather than manual edits.
- Verification created no source changes, accepted artifacts, fixture candidates, or manifest mutations. The temporary diagnostic project was removed. The worktree was clean before this report was written.

## Residual Authority Limits

- D0 proves repeated bytes on this build; D2 proves supported local behavior. Neither grants canonical D1 or platform authority.
- The fake D1 test is useful control-flow evidence only. Until the fixture path binds its digests to the checkout, it is not proof that staged evidence came from current source.
- Phase 7 remains responsible for forces, configurable step/iterations, general islands, sleeping, CCD, queries, ray casts, origin shifting, and broad world configuration. Phase 8 retains joints and broad rigid sign-off.
- No human verification is needed to classify the remaining gaps.

***

_Verifier: gsd-verifier_
_Result: gaps_found_
