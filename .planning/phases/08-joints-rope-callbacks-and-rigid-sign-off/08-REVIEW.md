---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
reviewed: 2026-07-14T06:24:09Z
depth: standard
iteration: 2
review_kind: remediation_and_local_evidence_review
diff_range: 5c332cdd09bd80f607d1070d864c28e6e2bb0a14..dc809b9b34b1784a000cd61c9beabe6ff1ad369e
files_reviewed: 108
findings:
  blocking: 0
  warning: 0
  info: 0
  total: 0
status: passed
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

The initial review blocked Phase 8 from canonical evidence collection. It
found that the live world step did not execute the pinned per-kind joint
solvers and that the accepted corpus did not exercise the behaviors whose
witness-family names claimed solver, callback, and destruction coverage.

The remediation review resolves both findings. Plans 08-14 through 08-18
replaced the generic staging path with exhaustive typed runtime candidates and
live two- and four-body solver dispatch. Plans 08-19 through 08-21 replaced the
non-step-bearing corpus with validated step sequences executed by both native
Rust and the pinned C++ adapter. Plan 08-22 closed Phase 8 comparison and
replay policy, then passed the complete local debug, release, replay, D0, and
fail-fast sanitizer matrix. These results are local D2 evidence only; canonical
D1 publication remains pending Plan 08-23.

## Findings

### CR-08-14-01 — Blocking — Live islands use one generic two-body constraint instead of the pinned per-kind joint solvers

- **Status:** resolved
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

#### Resolution

Commits `a2f5d5b`, `e4949b9`, `1ea8c5b`, `a9f1558`, and `38d5951`
introduced the staged typed call graph, activated all eleven joint runtimes,
and supplied semantic A/B/C/D lanes for every revolute/prismatic gear-source
combination. The live island tests now cover exhaustive dispatch, late-failure
atomicity, complete warm-cache commit, and four-body alias/scatter behavior.
The post-remediation Rust gate passed 185 library tests, every integration
target, and 13 doctests. The complete accepted corpus also matched the pinned
oracle in debug, release, replay, and sanitizer configurations, with two
byte-identical D0 runs.

### CR-08-14-02 — Blocking — The closed Phase 8 corpus names behavioral witnesses without executing them

- **Status:** resolved
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

#### Resolution

Commit `83dc3bf` made the accepted request validator require meaningful step and
post-step observations for every step-dependent Phase 8 family, all eleven
joint variants, all four gear lanes, callback timing, destruction cascades,
rope evolution, reconstruction, and diagnostics. Commits `bf2d79c` and
`053651a` execute that same typed corpus in native Rust and the pinned C++
adapter. Commit `dc809b9` migrated staging/replay to the inherited Phase 8
comparator, locked the reviewed field-specific residual policies, and added
stable first-divergence mutations including lifecycle multiplicity and every
gear result lane. Debug and release comparison, debug replay, exactly one
`verify-determinism --runs 2`, and fail-fast ASan/UBSan protocol/comparison all
exited successfully across 19 required families.

## Remediation gate

The revised Plans 08-14 through 08-22 supplied the required implementation,
corpus, native adapter, C++ adapter, comparator, and local evidence paths.
CR-08-14-01 and CR-08-14-02 are resolved with zero blocking or open findings.
`oracle.yml` may now be prepared for the Plan 08-23 human-controlled exact-ref
checkpoint. This review does not authorize a workflow dispatch, push, or
compatibility claim.

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
  `cargo test --all-features` (185 library tests, all integration targets, and
  13 doctests passed).
- Fresh configure/build passed for `oracle-debug`, `oracle-release`, and
  `oracle-asan-ubsan` before their evidence was consumed.
- Debug protocol CTest passed; debug and release Phase 8 comparisons each
  matched all 19 required families under `phase8-v1`.
- Debug replay matched all 19 required families, and exactly one
  `verify-determinism --runs 2` command produced two byte-identical native and
  oracle-debug runs.
- Fail-fast ASan/UBSan protocol CTest and the complete 19-family sanitizer
  comparison passed with recovery disabled.
- The local evidence used CMake 3.27.9, Ninja 1.13.2, and Apple Clang 21.0.0;
  it is therefore recorded only as D2-supported evidence, not canonical D1.

***

_Reviewer: gsd-code-reviewer_
_Lifecycle: 8-2026-07-13T21-26-30_
