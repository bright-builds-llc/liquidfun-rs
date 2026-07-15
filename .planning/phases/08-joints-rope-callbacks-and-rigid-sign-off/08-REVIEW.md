---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
depth: standard
status: findings
files_reviewed: 95
counts:
  critical: 0
  warning: 2
  info: 0
  total: 2
generated_by: gsd-code-review
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T23:43:40Z
---

# Phase 8 Code Review

## Scope and verdict

Reviewed the requested 95-file Phase 8 surface at current HEAD `ab71723`
against base `bb31d90`. The review covered the public joint and rope APIs, all
typed joint solver paths and four-body gear scatter, contact hooks and
lifecycle ownership, protocol validation and result bounds, native and pinned
C++ execution, comparison policy, accepted fixtures, oracle CI, and the
published canonical-evidence claims.

The earlier generic-solver and non-stepping-corpus blockers remain resolved,
and the canonical workflow identities are internally consistent. Two warnings
remain: ordinary hook-limit failures can leave the world partially progressed
without a resume contract, and three advertised differential mutation actions
are exact no-ops.

## Findings

### CR-08-25-01 — Warning — A hook-limit error consumes pair work and can partially mutate the world

- **Evidence:** `ContactManager::find_new_contacts` collects all broad-phase
  pairs and then applies them one at a time with `?` at
  `crates/liquidfun/src/world/contact_manager.rs:148-163`. The underlying pair
  update consumes the move buffer before processing at
  `crates/liquidfun/src/collision/broad_phase.rs:259-266`.
  `add_pair` can then fail while recording the filter event at
  `crates/liquidfun/src/world/contact_manager.rs:371`, after earlier pairs have
  already inserted contacts and adjacency at lines 381-399. Existing contacts
  have an equivalent late-failure path: contact state and transitions are
  mutated before lifecycle and hook capacity checks at
  `crates/liquidfun/src/world/contact_manager.rs:193-202`, while command/event
  capacity can fail at `crates/liquidfun/src/world/step.rs:694-717`.
  `World::step` returns these ordinary errors at
  `crates/liquidfun/src/world/step.rs:1151-1162`; only a panic poisons the world.
- **Impact:** With an event limit of zero, the first filter event fails after
  the candidate pair buffer has been emptied. Retrying with normal limits does
  not recreate that contact because no proxy is buffered as moved. With
  multiple pairs or existing contacts, earlier contact insertions, manifold
  updates, transitions, and wake changes can remain committed even though the
  caller receives no `StepReport` or resumable progress token. The error is
  therefore recoverable-looking but can silently alter later physics.
- **Missing regression:**
  `crates/liquidfun/tests/hook_contract.rs:326-351` verifies only that an event
  overflow discards a queued command and unlocks the world. It does not compare
  contact topology/state before and after the failure or retry the rejected
  step against a clean one-shot world.
- **Correction:** Make pre-solve lifecycle work transactional or explicitly
  resumable. In particular, do not irreversibly consume pair candidates before
  all fallible filter/lifecycle work is accepted, and stage or roll back
  contact, adjacency, transition, wake, and refilter mutations on an ordinary
  limit error. Add zero/partial event- and command-capacity regressions that
  prove a retry produces the same contact state and report as a clean step.

### CR-08-25-02 — Warning — Three closed-corpus joint mutations do not change their declarations

- **Evidence:** In
  `protocol/fixtures/accepted/rigid-world-request.jsonl:1`, action
  `joint-dpm-mutate` sets the mouse target to the declaration's existing
  `(2.0, 1.0)`, `joint-coupled-mutate` sets the motor correction factor to its
  existing `0.5`, and `gear-mutate` sets the gear ratio to its existing `-1.0`.
  The fail-closed validator checks only that a mutation contains some nonzero
  value at
  `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation/phase8.rs:167-227`;
  it never compares the candidate value with the target joint declaration.
  The C++ gear coordinate observer also reconstructs with the declaration's
  original ratio at
  `tools/reference/src/rigid_world_phase8_execute.hpp:602-607`, rather than the
  live `b2GearJoint::GetRatio()` value used by the solver after `SetRatio`.
- **Impact:** The canonical green run proves that both adapters can accept
  these action records, but not that the mouse-target, motor-correction, or
  gear-ratio setters change runtime behavior. Both sides could ignore any of
  these setters and the accepted trace would still pass. A genuinely changed
  gear ratio would additionally be observed asymmetrically by the current
  native and C++ adapters.
- **Violated evidence claim:** Phase 8's corpus work explicitly requires a
  nonzero state/mutation before positive stepping, and `JOIN-05` requires
  focused differential coverage for every joint type. A nonzero no-op is not
  mutation evidence.
- **Correction:** Change all three actions to values distinct from their
  declarations, observe the changed property through post-step semantics, and
  make request validation reject no-op mutations by resolving the target joint
  and comparing the affected field. Read the live gear ratio in the C++
  coordinate observer. Add fixture tests that independently replace each
  changed action with its declaration value and require validation or evidence
  coverage to fail.

## Verification evidence

- `cargo fmt --all --check` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed: 185 library tests, every integration
  target, and 13 doctests.
- Refreshed GitHub Actions run `29379350740` completed successfully for exact
  commit `e0b5106559b3c0c37beb44e4ade45c3b7919b59d`; canonical Linux, sanitizer
  Linux, macOS, and Windows jobs succeeded after the review fixes.
- Both published identity records bind the documented run, commit, upstream
  revision, Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, and
  `phase8-v1` policy.
- No new `unsafe` block, `unwrap()`, placeholder implementation, or ignored
  error path was found in the reviewed production surface.

***

_Reviewer: gsd-code-review_
_Lifecycle: 8-2026-07-13T21-26-30_
