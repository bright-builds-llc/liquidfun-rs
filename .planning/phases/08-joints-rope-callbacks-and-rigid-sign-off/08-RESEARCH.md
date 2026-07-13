---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T22:04:00.000Z
phase: 08-joints-rope-callbacks-and-rigid-sign-off
requirements: [RIGD-11, JOIN-01, JOIN-02, JOIN-03, JOIN-04, JOIN-05]
---

# Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off - Research

## Research Objective

Determine how to plan a source-faithful Phase 8 without weakening the Phase 3 identity model, Phase 4 numerical/evidence policy, or Phase 6/7 rigid solver and protocol. The pinned source at commit `7f20402173fd143a3988c921bc384459c6a858f2` is the behavioral authority.

## Executive Recommendation

Build Phase 8 as nine dependency-ordered slices:

1. Joint public contract, shared private state, arena replacement, dependency graph, and lifecycle.
1. Revolute and prismatic joints, which establish limit/motor state and gear coordinates.
1. Distance, pulley, mouse, friction, and motor joints, which cover scalar constraints and origin-shifted world targets.
1. Wheel, weld, and rope joints, which add coupled spring/limit behavior.
1. Gear joints and source-faithful joint island ordering.
1. Standalone rope as a separate deep module.
1. Source-timed filter/pre-solve/destruction decisions and one owned lifecycle timeline.
1. Semantic reconstruction dump and bounded diagnostic records.
1. Closed Phase 8 protocol/oracle/comparator corpus, accumulated rigid regression, review, and truthful sign-off.

Do not plan eleven isolated public subsystems. The joint base, solver scratch, island ordering, dependency graph, validation/error model, snapshot vocabulary, and differential model must be shared, while per-kind constraint equations and observable state stay separate and exhaustively tested.

## Pinned Source Inventory

### Shared joint infrastructure

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Joints/b2Joint.h`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Joints/b2Joint.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Island.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.cpp`

The base joint stores type, linked body edges, active/collision state, island membership, and user data. Rust should keep only semantic state and explicit adjacency/dependency edges; intrusive pointers, allocator fields, island indices, and user-data pointers remain private or are omitted.

### Eleven joint kinds

| Joint | Pinned files | Planning-critical behavior |
| --- | --- | --- |
| Revolute | `b2RevoluteJoint.{h,cpp}` | angular coordinate/speed, lower/upper/equal limits, motor torque, 3x3 solve, warm-start limit state |
| Prismatic | `b2PrismaticJoint.{h,cpp}` | axis/perpendicular constraints, translation/speed, limit state, motor force, gear coordinate |
| Distance | `b2DistanceJoint.{h,cpp}` | fixed length, frequency/damping gamma and bias, soft vs rigid position behavior |
| Pulley | `b2PulleyJoint.{h,cpp}` | ground/local anchors, constant and ratio, two segment lengths, world-space ground anchors |
| Mouse | `b2MouseJoint.{h,cpp}` | world target, max force, frequency/damping, gamma/beta, target origin shifting, dump unsupported upstream |
| Gear | `b2GearJoint.{h,cpp}` | two revolute/prismatic dependencies, four bodies, coordinate constant, ratio, four combinations, dependency lifetime |
| Wheel | `b2WheelJoint.{h,cpp}` | point-to-line constraint, spring, translation/speed, motor torque, mixed scalar constraints |
| Weld | `b2WeldJoint.{h,cpp}` | rigid 3x3 or soft angular constraint, reference angle, frequency/damping |
| Friction | `b2FrictionJoint.{h,cpp}` | capped linear and angular friction impulses, max force/torque |
| Rope joint | `b2RopeJoint.{h,cpp}` | maximum length unilateral constraint and inactive/at-upper limit state |
| Motor | `b2MotorJoint.{h,cpp}` | linear/angular offsets, force/torque caps, correction factor, warm-started errors |

Every planner task that translates a joint must enumerate definition invariants, snapshots/accessors, setters and exact wake behavior, solver initialization, warm starting, velocity solve, position solve, reactions, dump record, origin shifting if applicable, focused tests, and differential observations.

### Standalone rope

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Rope/b2Rope.h`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Rope/b2Rope.cpp`

Standalone rope is not a joint and has no `World`, handle, contact, island, or body dependency. Its source order is compact enough to translate as a pure checked core: initialize vertices and inverse masses; integrate; run stretch/bend/stretch per iteration; reconstruct velocity. Fixed vertices use zero inverse mass. Preserve zero-step behavior, angle wrapping, and source expression grouping.

### Callbacks and destruction

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2ContactManager.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Contacts/b2Contact.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp`

The current Rust step runs contact-manager update before a later grouped hook pass. That is insufficient for pinned timing. Filtering belongs at pair admission and refilter. Begin/end and pre-solve belong inside each contact update. Post-solve belongs after each discrete or TOI constraint result. Destruction listener occurrences belong at their actual cascade positions.

## Current Rust Integration Seams

### Object model and lifecycle

- `crates/liquidfun/src/identity.rs` already defines the correct `JointId` identity.
- `crates/liquidfun/src/world/object.rs` has a placeholder `Joint`, `Arena<Joint, JointId>`, newest-first body adjacency, generic create/destroy seams, counts, snapshots, and centralized body cascades.
- The placeholder must be replaced without changing public handle identity or exposing arena coordinates.
- Gear requires forward dependency state on the gear and reverse dependent-gear lanes on both source joints. Prepare every multi-object mutation before committing any adjacency or arena change.
- Body and source-joint destruction must first collect the complete gear-dependent cascade so allocation failure or invalid topology cannot leave partial adjacency.

### Solver and ordering

- `crates/liquidfun/src/world/island.rs` already reserves `joint_ids`; populate it during DFS in source list/LIFO order.
- `crates/liquidfun/src/world/contact_solver.rs` and `world/step.rs` provide the staged island transaction and timing inputs.
- Add joint solver scratch alongside copied body positions/velocities. Constraint initialization can fail before commit; no earlier island result may leak on a later joint failure.
- Preserve upstream phase order: initialize contacts, initialize joints, solve joints then contacts for each velocity pass, solve contacts then joints for each position pass, then sleep evaluation.
- Do not add ordinary joints to TOI islands. Preserve Phase 7 continuous behavior unless a pinned source path explicitly requires joint state invalidation or wake propagation.

### Hook and event model

- `crates/liquidfun/src/world/step.rs` already has borrow-scoped contact views, narrow decisions, deferred commands, panic poisoning, and owned reports.
- Split fixture-pair filtering from touching-contact observation. A pair view needs semantic fixture/child/body data but no contact identity.
- Pre-solve needs current and previous manifold snapshots and per-update enable/disable. Inventory source setters before adding typed friction/restitution/tangent-speed controls.
- Replace independently assembled event vectors as sources of truth with one ordered lifecycle timeline; expose existing convenience slices as projections.
- Direct non-step destruction needs an owned report/result path capable of carrying end/destruction listener evidence in occurrence order.

### Diagnostics and differential harness

- `crates/liquidfun-differential/src/rigid_world/` owns bounded scenario models and native execution.
- `crates/liquidfun-differential/src/rigid_evidence/` owns declarations, observations, policy application, and promotion evidence.
- `tools/reference/src/rigid_world*` owns the pinned C++ decode/validate/execute/trace path.
- Extend the existing protocol rather than creating a joint-only harness. Add bounded typed joint declarations/actions and standalone-rope requests while retaining all Phase 6/7 families unchanged.
- Separate public semantic snapshots from harness-private occurrence ordinals and reconstruction indices.

## Recommended Internal Architecture

### Public modules

```text
crates/liquidfun/src/
  joint.rs                 # curated public re-exports and common tagged types
  joint/
    definition.rs         # checked per-kind definitions and JointDef
    snapshot.rs           # owned tagged snapshots and common accessors
  rope.rs                  # standalone rope public contract and pure core
  world/
    joint.rs               # World authority, lifecycle, checked mutations
    joint/
      solver.rs            # shared solver dispatcher/scratch contracts
      revolute.rs
      prismatic.rs
      distance.rs
      pulley.rs
      mouse.rs
      gear.rs
      wheel.rs
      weld.rs
      friction.rs
      rope.rs
      motor.rs
```

Exact file splits are discretionary. Keep small per-kind code together until the file-length trigger is reached, but avoid returning to a monolithic `world/object.rs` or `contact_solver.rs`.

### Shared private contracts

- `JointRecord`: tagged state, bodies, `collide_connected`, island flag, body edges, reverse gear dependents.
- `JointSolverContext`: copied body solver indices/states, timestep ratio, warm-start flag, inverse timestep.
- `JointVelocityConstraint` / `JointPositionConstraint`: private tagged solver scratch or per-kind structs behind exhaustive dispatch.
- `JointMutationError`: invalid handle, wrong kind, invalid value, locked, poisoned, derived non-finite state.
- `JointCreationError`: body/dependency/topology validation, capacity, locked/poisoned, invalid definition.
- `JointDestructionReport`: owned listener/destruction timeline plus the ordinary destruction records.

Prefer exhaustive enum dispatch to trait objects: the set is pinned and closed for this phase, hot solver calls should remain monomorphic, and the compiler should force every new observation/dump/comparator branch to account for all kinds.

## Exact Behavioral Risks

### Wake and mutation differences

Setter wake behavior is not uniform. Some setters wake only when a value changes; others reset impulses or limit state. Plans must copy each source branch order and add changed/equal/invalid tests. Do not apply a generic “all joint mutations wake both bodies” policy.

### Gear lifetime and four-body state

Upstream documents a deletion precondition rather than protecting dependent gears. Safe Rust must strengthen this by cascading dependent gear destruction first. This deliberate difference needs public documentation, focused order tests, and oracle scenarios that explicitly delete gears before their sources so physics comparisons remain meaningful.

### Collision suppression

`collide_connected = false` affects `ShouldCollide` through body joint edges. Creating or destroying such a joint must refilter relevant contacts/proxies at the pinned boundary. Tests must cover an existing touching contact, a separated pair that later overlaps, and multiple joints between the same bodies.

### Solver order

Joint/contact order is physics-visible. Do not sort joint IDs, iterate hash maps, or group constraints by kind. Preserve world/body newest-first lanes, DFS push/pop behavior, and island encounter order. Add order-sensitive mixed-joint witnesses rather than only isolated pairs.

### Dump fidelity

Upstream dump text includes temporary indices and code-like formatting; mouse joint is not reconstructable there. Make typed semantic reconstruction authoritative. Preserve two-pass non-gear/gear dependency order and explicit unsupported status. Text formatting is diagnostic only.

### Callback timing and panic

Moving decisions into contact-manager update means hook panic can occur earlier than today. Keep RAII locking and poison behavior around synchronous decisions, discard deferred commands, and ensure owned consumers run only after coherent completion. No observer callback should hold `&mut World` or retain a contact view.

## Focused Verification Matrix

For each joint kind require:

- valid/default definition construction and every invalid field;
- cross-world, stale, wrong-kind, locked, and poisoned rejection without effects;
- create/destroy adjacency and `collide_connected` refilter behavior;
- complete snapshot and common/type-specific inspection;
- every setter's equal/changed/invalid wake and cache behavior;
- anchor, coordinate/speed, reaction force/torque, limit/motor/spring state;
- cold and warm starts, zero/positive timestep, configured iteration counts;
- source-ordered mixed-island behavior and atomic late failure;
- dump record presence/order/support status;
- semantic Rust/C++ differential witness.

Additional required matrices:

- Gear: revolute/revolute, revolute/prismatic, prismatic/revolute, prismatic/prismatic; positive, negative, and zero ratios as accepted by source; dependency destruction and four-body order.
- Rope: fixed/free vertices, zero timestep, multiple iterations, stretch/bend order, set-angle wrapping, invalid transactional input.
- Callbacks: admission/refilter filter decisions, begin/end/pre-solve order, sensor exclusion, disable-for-one-update, repeated CCD occurrences, post-solve order, explicit vs implicit destruction notifications, hook panic, command deferral.
- Diagnostic reconstruction: bodies/fixtures, each joint kind, non-gear-before-gear ordering, mouse unsupported status, semantic/text separation.

## Differential and Evidence Strategy

Create a closed `phase8-v1` registry that independently declares every required joint kind and observation. Unknown or missing kinds, actions, fields, dump records, policy paths, or witness families are harness errors.

Required witness families:

1. `joint_definitions_and_mutations`
1. `revolute_prismatic_limits_and_motors`
1. `distance_pulley_mouse_constraints`
1. `wheel_weld_friction_rope_motor_constraints`
1. `gear_dependencies_and_four_body_solver`
1. `mixed_joint_island_order_and_collision_suppression`
1. `standalone_rope_evolution`
1. `contact_filter_listener_and_pre_solve_timing`
1. `destruction_listener_and_dependency_cascades`
1. `diagnostic_reconstruction_and_dump_order`
1. all retained Phase 6/7 rigid families

Structural state, branch/limit states, identities, dependencies, counts, callback/destruction order, and multiplicity compare exactly. Exact transported configuration bits remain distinct from semantic computed floats. Use named ULP policies for bounded local calculations, absolute-relative policies for anchors/reactions/velocities/rope vertices, and dimensioned absolute residuals only where justified. Retain signed-zero and non-finite rules.

D0 must be byte-identical with timing excluded. Local macOS evidence is D2 only. Do not mark `RIGD-11` or `JOIN-05` complete until actual pinned canonical D1 debug/release comparison, replay, and sanitizer evidence pass the complete closed corpus.

## Plan Decomposition and Dependencies

| Slice | Depends on | Primary outputs |
| --- | --- | --- |
| Joint contract/lifecycle | Phase 7 | definitions, snapshots, checked world API, arena replacement, adjacency, collision suppression |
| Revolute/prismatic | contract | first limit/motor solvers and gear coordinate sources |
| Distance/pulley/mouse/friction/motor | contract | scalar and capped constraints, world targets, origin shifting |
| Wheel/weld/rope joint | earlier shared solver | coupled spring/limit solvers |
| Gear/island integration | revolute/prismatic plus lifecycle | dependencies, four-body solver, full joint traversal |
| Standalone rope | math policy | independent rope module and focused tests |
| Callback timeline | contact manager/step | source-timed decisions, reports, destruction timing |
| Diagnostics/protocol model | joint snapshots plus callbacks | semantic reconstruction and bounded Phase 8 schema |
| Native/C++ execution and comparator | all behavior slices | witnesses, policies, replay/D0/sanitizer/D1 gates and docs |

The planner may create more than nine plans to keep files and tests bounded. Avoid parallel waves when plans share `world/object.rs`, `world/island.rs`, `world/step.rs`, protocol model files, or C++ oracle translation units.

## Verification Commands

Every implementation commit must run, in order:

1. `cargo fmt --all`
1. `cargo clippy --all-targets --all-features -- -D warnings`
1. `cargo build --all-targets --all-features`
1. `cargo test --all-features`

When macOS provenance stalls generated integration binaries under the repository target directory, use a clean temporary `CARGO_TARGET_DIR` for the test execution and record the reason; do not skip tests.

Affected plans also need focused crate tests, `just markdown-check` for non-GSD Markdown, oracle debug/release builds, protocol tests, rigid compare/replay/D0 commands, sanitizer execution, inventory/provenance checks, workflow lint, package isolation, `git diff --check`, and a complete diff review.

## Planning Guardrails

- Do not claim `RIGD-10`; only bounded Phase 8 diagnostic reconstruction is in scope.
- Do not introduce persistent callback registration, multiple decision listeners, raw contact handles, `&mut World` hooks, trait-object joint solvers, or hash-visible solver order.
- Do not turn dump text into a persistence format or compare whitespace/locale as physics evidence.
- Do not widen numeric thresholds by joint kind, iteration count, or scenario horizon without a named source-derived policy.
- Do not weaken accumulated Phase 6/7 witnesses to make the expanded corpus pass.
- Keep complete public rustdoc and compatibility claims synchronized with verified behavior.

## RESEARCH COMPLETE

Phase 8 should extend the existing deep world and rigid evidence modules through checked tagged joint state, source-ordered exhaustive solver dispatch, a separate rope core, source-timed synchronous decisions plus owned lifecycle evidence, and a fail-closed semantic reconstruction/differential gate. The central planning risks are gear lifetime, per-setter wake behavior, collision refiltering, joint/contact solver order, callback insertion points, and evidence overclaiming.
