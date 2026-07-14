---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
reviewed: 2026-07-14T03:05:14Z
depth: standard
iteration: 1
review_kind: pre-evidence_implementation_and_corpus_review
diff_range: 5c332cdd09bd80f607d1070d864c28e6e2bb0a14..e6f8533be0e71e512b4e8871951932168b957a57
files_reviewed: 108
findings:
  blocking: 2
  warning: 0
  info: 0
  total: 2
status: blocked
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-14T03:05:14Z
---

# Phase 8 Code Review

## Scope and verdict

Reviewed the complete Phase 8 diff from `5c332cd` through `e6f8533`, all
thirteen completed plan summaries, D-01 through D-26, the pinned-joint solver
call chain, the native/C++ Phase 8 execution paths, and the complete accepted
`phase8-v1` action registry. The review was materially informed by `AGENTS.md`,
`AGENTS.bright-builds.md`, the placeholder-only `standards-overrides.md`, and
the managed architecture, code-shape, testing, verification, and Rust
standards.

Phase 8 is blocked from canonical evidence collection. The public definitions,
identity/lifecycle work, standalone rope, and much of the bounded protocol
surface are substantial, but the live world step does not execute the pinned
per-kind joint solvers and the accepted corpus does not exercise the behaviors
whose witness-family names claim solver, callback, and destruction coverage.

## Findings

### CR-08-14-01 — Blocking — Live islands use one generic two-body constraint instead of the pinned per-kind joint solvers

- **Status:** open
- **Evidence:** `crates/liquidfun/src/world/joint/solver.rs:10-15` gives every
  joint exactly two solver-body indices. `CommonConstraint` at lines 24-36
  stores only a center delta, one linear impulse, one angular impulse, an axis,
  and generic caps. Lines 141-153 wrap that same value in eleven differently
  named enum variants. Lines 174-219 apply one center-velocity equation to all
  kinds, and lines 222-255 apply one center-delta position equation. The island
  driver calls only these generic functions at
  `crates/liquidfun/src/world/contact_solver.rs:253-291`; it never dispatches to
  the per-family runtime initialization, warm-start, velocity, or position
  routines implemented under `world/joint/`.
- **Violated decisions/requirements:** D-08, D-09, `JOIN-01`, `JOIN-02`, and
  the project requirement that behavior match the pinned revision.
- **Impact:** Revolute anchors are treated as center locking; distance/pulley,
  motor, softness, limits, and reaction caches do not use their pinned
  equations; and gear cannot possibly apply its four-body A/B/C/D Jacobian
  through a two-index input. A successful `World::step` therefore does not
  establish the advertised eleven-joint behavioral parity.
- **Correction:** Replace the generic common constraint with exhaustive
  per-kind staged solver state that consumes each `JointRuntime`, supplies all
  required body lanes (including gear A/B/C/D), preserves pinned expression and
  warm-cache ordering, and commits caches only after the complete island
  candidate validates.
- **Regression command:** Add consumer-visible, independently pinned
  differential scenarios that step every joint kind through nontrivial
  limit/motor/softness/reaction states and all four gear combinations, then run
  `cargo test -p liquidfun --all-features` and the complete Phase 8 compare,
  replay, D0, and sanitizer commands.

### CR-08-14-02 — Blocking — The closed Phase 8 corpus names behavioral witnesses without executing them

- **Status:** open
- **Evidence:** `protocol/fixtures/accepted/rigid-world-request.jsonl:1` has zero
  `step` actions in all of these families: `joint_definitions_and_mutations`,
  `revolute_prismatic_limits_and_motors`,
  `distance_pulley_mouse_constraints`,
  `wheel_weld_friction_rope_motor_constraints`,
  `gear_dependencies_and_four_body_solver`,
  `mixed_joint_island_order_and_collision_suppression`,
  `contact_filter_listener_and_pre_solve_timing`, and
  `destruction_listener_and_dependency_cascades`. The gear family creates and
  destroys two source joints plus a gear but never solves them. The callback
  family installs filter/pre-solve directives but destroys both bodies without
  a contact update. The destruction family explicitly destroys one revolute
  joint and contains no dependent gear cascade. The comparator and C++ oracle
  can therefore agree while the named behavior is absent.
- **Violated decisions/requirements:** D-23, D-25, D-26, `RIGD-11`, `JOIN-01`,
  `JOIN-02`, `JOIN-04`, and `JOIN-05`.
- **Impact:** Debug/release/replay/D0/sanitizer success over the current corpus
  is not evidence for joint dynamics, four-body gear solving, callback timing,
  or dependency cascades. Publishing exact-commit artifacts from this corpus
  would overstate the achieved scope.
- **Correction:** Expand the typed accepted corpus and its Rust/C++ validation,
  executor, comparator, and regression tests with actual world steps and exact
  semantic observations for every named behavior. Retain all Phase 6/7
  families and keep the policy fail closed.
- **Regression command:** A machine check must report at least one behaviorally
  meaningful `step` and observation sequence for every step-dependent Phase 8
  family, followed by debug/release compare, replay, exactly two D0 runs, and
  fail-fast ASan/UBSan over the complete accumulated corpus.

## Remediation gate

No remediation was attempted. CR-08-14-02 necessarily changes
`protocol/fixtures/accepted/rigid-world-request.jsonl` and focused Rust/C++
tests, which are outside Plan 08-14 task 02's explicit remediation allowlist.
The plan says any required path outside that list is a hard replanning blocker.
CR-08-14-01 also needs new behavior-focused regression files outside that
allowlist before its large solver replacement can be considered safe.

Because the review is blocked, `oracle.yml` must not be updated to publish a
Phase 8 canonical/sanitizer matrix, and no workflow may be dispatched until a
revised plan closes both findings.

## Verification evidence

- Static action audit: all eight step-dependent Phase 8 witness families above
  reported zero `step` actions.
- Static solver call-chain audit: only the generic `world/joint/solver.rs`
  routines are reachable from the live discrete island driver.
- No new `unsafe` block was found in the reviewed Phase 8 production Rust
  surface.
- Ordered Rust gate passed with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-target`:
  `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`;
  `cargo build --all-targets --all-features`; and
  `cargo test --all-features` (176 library tests, all integration targets, and
  13 doctests passed).

***

_Reviewer: gsd-code-reviewer_
_Lifecycle: 8-2026-07-13T21-26-30_
