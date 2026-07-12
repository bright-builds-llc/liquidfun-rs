______________________________________________________________________

## generated_by: gsd-discuss-phase lifecycle_mode: yolo phase_lifecycle_id: 7-2026-07-12T23-36-17 generated_at: 2026-07-12T23:44:54.491Z

# Phase 7: Rigid Solver, World Operations, and CCD - Context

**Gathered:** 2026-07-12
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Complete the native scalar rigid-body step beyond Phase 6's fixed one-contact witness: expose checked force, impulse, velocity, damping, gravity, body-mode, and world-step controls; build source-ordered discrete islands with warm starting and sleeping; orchestrate bullet CCD, TOI, and sub-stepping; add world AABB queries, ray casts, force clearing, and origin shifting; and prove the required non-colliding, stacked, sleeping, fast-moving, filtered, and queried scenario families through bounded semantic differential evidence. Joints, standalone rope, broad rigid sign-off, and particle behavior remain later phases.

</domain>

<decisions>
## Implementation Decisions

### Body and world control contract

- **D-01:** Preserve the Phase 3/6 authority model: expose granular handle-oriented `World` methods for body forces, torques, linear and angular impulses, linear and angular velocity, damping, gravity scale, fixed rotation, sleeping permission, awake state, and bullet state. Do not add a public command DSL or borrow-scoped mutable body façade.
- **D-02:** Replace boolean-blind wake parameters with a small typed wake policy while preserving pinned no-effect behavior. Force, torque, and impulse operations on non-dynamic bodies, and preserve-sleep operations on sleeping bodies, return `Ok(())` without mutation rather than exposing a public no-op outcome taxonomy.
- **D-03:** Validate the complete handle and every finite/range invariant before effects. Invalid handles, non-finite values, negative damping, non-finite derived accumulations, and invalid fixed-rotation mass candidates return typed errors and leave body, contact, proxy, force, and sleep state unchanged.
- **D-04:** Preserve source-specific wake behavior: nonzero velocity setters wake while zero setters do not; forces and impulses wake only when requested; damping, gravity scale, bullet state, and world gravity changes do not wake; disabling sleep wakes; putting a body to sleep clears velocity, force, and torque; fixed rotation clears angular velocity and transactionally resets mass data.
- **D-05:** Keep world configuration owned and checked by `World`. Gravity and automatic-force-clearing state are explicit, automatic clearing defaults to enabled, and `clear_forces` remains available for multi-substep application loops. Successful step calls clear accumulators when enabled, including calls that stop with continuous work pending.

### Island solver and sleeping semantics

- **D-06:** Replace the fixed Phase 6 contact witness with private ephemeral source-faithful DFS islands. Seed from an explicit newest-first body-order lane, traverse newest-first contact adjacency with LIFO behavior, stop propagation through static bodies, and reserve the joint traversal lane for Phase 8. Do not substitute sorted connected components or hash-derived order.
- **D-07:** Parse timestep, velocity-iteration, and position-iteration inputs into checked invariant-bearing configuration before any step effects. Retain previous inverse timestep across zero-duration steps and compute the pinned warm-start ratio from the accepted current and previous timestep values.
- **D-08:** Stage the complete discrete-solver result across all islands before committing body motion, contact impulses, sleep timers/states, and proxy synchronization. A late numerical or capacity failure may not leave earlier islands partially solved; preflight every statically knowable bound before contact or solver mutation.
- **D-09:** Preserve the pinned phase order and scalar expression grouping: integrate dynamic forces and gravity, apply Padé damping, initialize constraints, optionally scale and apply warm-start impulses, run velocity passes, store impulses, clamp and integrate positions, run position passes, synchronize transforms/proxies, and then evaluate sleeping.
- **D-10:** When warm starting is disabled, initialize the current solve with zero impulses but still store newly solved impulses for later steps. Preserve manifold feature identity and manager occurrence order rather than canonicalizing solver constraints.
- **D-11:** Implement per-island sleeping with the pinned linear/angular thresholds, accumulated sleep time, allowed-sleep flag, position-convergence condition, and all-body transition. Wake propagation must follow pinned contact and mutation sources; activation alone must not invent a wake transition.

### CCD and sub-stepping contract

- **D-12:** Implement world-level continuous collision as a private source-faithful state machine over the existing Phase 5 TOI kernel. Keep candidate indices, cached flags, TOI counters, queue/storage details, and sweep bookkeeping private.
- **D-13:** Preserve manager-order TOI scanning and strict-less-than candidate selection, cached TOI reuse, sweep `alpha0` equalization, sensor and non-bullet dynamic-pair exclusions, rejected-contact rollback, accepted-body waking, bounded TOI island construction, no-warm-start TOI solve, displaced-cache invalidation, and the pinned strict substep-count guard.
- **D-14:** Surface only semantic step completion in the owned report: `Complete` or `ContinuousPending`. With sub-stepping enabled, stop after one accepted TOI event; a pending continuation skips the next discrete solve and resumes continuous work without requiring a public continuation token.
- **D-15:** Add a reviewed aggregate per-call continuous-work budget. Exhaustion returns typed partial-state evidence with a coherent resumable world rather than exposing internal candidate state or silently widening limits.
- **D-16:** Differential witnesses must prove anti-tunneling through body pose/velocity, awake and bullet state, ordered contact transitions, post-solve impulses, completion state, and barrier-side or signed-separation semantics across continuous on/off, bullet on/off, and sub-stepping variants.

### World queries, ray casts, and origin shifting

- **D-17:** Expose borrow-scoped streaming visitors over semantic fixture identities and owned hit data. AABB queries use typed continue/terminate control. Ray casts use typed ignore, terminate, continue-without-clipping, and clip-to-checked-fraction directives instead of public magic float return values.
- **D-18:** Preserve upstream traversal behavior without promising callback order. World queries do not automatically apply fixture collision-filter masks; applications and differential scenarios may filter explicitly. Repeated multi-child fixture occurrences remain observable and must not be deduplicated.
- **D-19:** Keep canonicalization in evidence collectors, not production callbacks. All-continue and independently filtered hits compare as semantic multisets; termination compares count/status rather than first fixture identity; closest-hit comparison excludes equal-fraction ties or represents the tied result as a set.
- **D-20:** Origin shifting is a checked prepare/commit transition. Validate the full translated body transform/sweep and active broad-phase bounds before mutation, then subtract the origin from body world positions, sweep centers, tree AABBs, and later joint world anchors without rebuilding the tree or changing proxy topology, move buffers, contacts, filters, velocities, local coordinates, or normals.
- **D-21:** Query and shift evidence covers empty/full/explicitly filtered AABBs, duplicate child occurrences, continue/terminate/ignore/clip rays, nearest-hit and tie cases, invalid-directive no-effect failures, locked/non-finite/overflow shift rejection, and translation covariance of semantic hits, fractions, points, positions, and normals.

### Differential evidence and phase truthfulness

- **D-22:** Extend the existing bounded rigid-world protocol and evidence pipeline rather than creating a second harness or an unrestricted world DSL. Required families include force/impulse and configuration transitions, multi-contact stacks and island order, sleep/wake transitions, fast bullet/CCD cases, filtered world queries/rays, and origin-shift covariance.
- **D-23:** Compare identities, flags, counts, branch/completion state, lifecycle order, solver-visible order, and callback control structurally and exactly. Add closed per-observable Phase 7 numeric policies for force integration, damping, motion, impulses, TOI, ray fractions/points, and shifted positions; never introduce a global epsilon or iteration-scaled tolerance.
- **D-24:** Preserve the existing D0-D3 authority model, first-divergence reporting, replay, minimization, staging, review, and promotion rules. Local passing runs remain D2 and must not be described as canonical D1 or broad rigid parity.

### the agent's Discretion

- Exact public type, method, directive, configuration, report-status, and typed-error names within these contracts.
- Exact private module boundaries and scratch-buffer representations, provided `world` remains a cohesive deep module and file/function refactor triggers are respected.
- Exact reviewed capacity and continuous-work budgets, provided exhaustion is typed, bounded, deterministic, and covered by no-partial-mutation tests.
- Exact field-specific numerical thresholds, provided each is justified against the pinned source and the Phase 4 policy with focused differential evidence.

</decisions>

\<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and inherited contracts

- `.planning/ROADMAP.md` § Phase 7 — fixed goal, requirements, success criteria, research flags, and later-phase boundary.
- `.planning/REQUIREMENTS.md` § `RIGD-03`, `RIGD-05`, `RIGD-06`, `RIGD-07`, `RIGD-08`, and `RIGD-09` — Phase 7 acceptance requirements.
- `.planning/PROJECT.md` — native Rust, oracle isolation, deterministic ordering, safe API, numerical, testing, and truthfulness constraints.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — granular world authority, typed handles, transient contacts, deferred mutation, event ordering, and poisoning.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — source-ordered scalar math, field policies, non-finite rules, ordering, horizons, and D0-D3 authority.
- `.planning/phases/05-shapes-and-collision-foundation/05-CONTEXT.md` — immutable shapes, broad-phase/tree order, ray primitives, semantic manifolds, and TOI kernel.
- `.planning/phases/06-minimal-rigid-world-vertical-slice/06-CONTEXT.md` — checked body/fixture contract, contact lifecycle, fixed witness solver, rigid protocol, and exact Phase 7 deferrals.

### Pinned rigid solver, sleeping, CCD, and world operations

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Body.h` and `b2Body.cpp` — forces, impulses, velocity, damping, gravity scale, bullet, sleep, awake, fixed-rotation, and wake/no-op semantics.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2Island.h` and `b2Island.cpp` — force integration, damping, constraint phase order, sleeping, and TOI island solving.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.h` and `b2World.cpp` — step configuration, island traversal, force clearing, continuous state, TOI selection, queries, rays, and origin shifting.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2TimeStep.h` — timestep, inverse timestep, ratio, iteration, and warm-start input contract.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Contacts/b2ContactSolver.h` and `b2ContactSolver.cpp` — complete discrete and TOI constraint initialization, solving, and impulse persistence.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h` — AABB query and ray-cast callback meanings and mutation restrictions.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Collision/b2DynamicTree.h` and `b2DynamicTree.cpp` — traversal, clipping, termination, and in-place origin-shift behavior.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter06_Bodies.md`, `Chapter09_Contacts.md`, and `Chapter10_World.md` — consumer-visible force, sleep, contact, step, query, and world-operation semantics.

### Existing Rust and evidence seams

- `crates/liquidfun/src/world.rs`, `world/object.rs`, `world/body.rs`, `world/step.rs`, and `world/contact_solver.rs` — Phase 6 world storage, checked mutations, reports, fixed step, and contact solver to deepen.
- `crates/liquidfun/src/collision/toi.rs` — existing checked source-ordered TOI kernel.
- `crates/liquidfun/src/collision/tree/traversal.rs` and `collision/tree.rs` — checked query/ray traversal and in-place tree-origin shifting.
- `crates/liquidfun-test-protocol/src/scenario/rigid_world.rs`, `scenario/rigid_world/`, and `tolerance/rigid_policy.rs` — bounded rigid scenario and closed numeric-policy extension points.
- `crates/liquidfun-differential/src/rigid_world.rs`, `rigid_evidence.rs`, `rigid_fixtures.rs`, `runner.rs`, and `comparator.rs` — Rust execution, first-divergence comparison, evidence lifecycle, and fixture workflows.
- `tools/reference/src/rigid_world.cpp`, `rigid_world.hpp`, and `oracle_adapter.cpp` — pinned C++ rigid adapter to extend without a parallel oracle path.
- `ARCHITECTURE.md`, `TESTING.md`, and `COMPATIBILITY.md` — current system boundaries, evidence placement, and truthful compatibility claims.
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` — required deep-module, control-flow, unit-test, and verification rules.

\</canonical_refs>

\<code_context>

## Existing Code Insights

### Reusable Assets

- `World` already owns typed arenas, explicit body/fixture/contact adjacency, newest-first contacts, broad-phase proxies, centralized validate-then-commit mutations, restricted hooks, owned reports, and deferred commands.
- `world/contact_solver.rs` already implements source-ordered contact constraint construction, warm-start impulses, velocity/position passes, clamped integration, and impulse persistence for the Phase 6 witness.
- Phase 5 already provides checked scalar TOI, dynamic-tree AABB/ray traversal, shape child ray casts, and in-place tree origin shifting.
- The rigid protocol, differential comparator, fixture lifecycle, C++ adapter, and xtask evidence commands already provide bounded exact-bit transport, D0 replay, D2 comparison, first divergence, and reviewed promotion.

### Established Patterns

- Production behavior stays safe native Rust in one published crate; private protocol/differential/C++ tooling depends inward and never shapes ordinary Cargo use.
- Public inputs become invariant-bearing values before effects, complete candidate state is prepared before mutation, and source-visible order never depends on hash iteration.
- Contacts remain transient, semantic identity replaces pointer/storage identity, and authority labels remain explicit.

### Integration Points

- Deepen the existing `world` module with body controls, checked step configuration, island scratch state, sleeping, and private continuous state rather than adding another dynamics crate or world model.
- Reuse the existing `collision::toi`, tree traversal, broad phase, contact manager, and contact solver; reserve explicit joint lanes in island and origin-shift logic for Phase 8.
- Extend the current rigid scenario/schema/policy/comparator/C++ adapter/xtask path with closed Phase 7 action and trace variants.

\</code_context>

<specifics>
## Specific Ideas

- Keep ignored upstream branches boring for consumers: a force applied to a static body is a successful no-op, while malformed or non-finite input is a typed no-effect error.
- Add an explicit newest-first body-order lane because the current arena's stable ascending-slot iteration is deterministic but does not reproduce the pinned linked-list seed order.
- Treat CCD as a private world state machine, not a priority queue and not a public continuation object; expose only whether continuous work remains.
- Keep callback order unspecified in the production API while making differential evidence deterministic through declared multiset/set policies and tie-aware witnesses.
- Prove origin shifting by translation covariance and preserved tree/proxy topology, not merely by comparing shifted coordinates.

</specifics>

<deferred>
## Deferred Ideas

- Joint traversal, joint constraints, and joint world-anchor shifting — Phase 8.
- Broad rigid sign-off, general randomized rigid corpora, and an unrestricted future-world action DSL — later rigid validation work.
- Persistent incremental island graphs — only after the source-faithful ephemeral baseline is complete and profiling proves reconstruction is material.
- Public canonical query collection helpers — only after consumer evidence justifies a second API distinct from streaming parity callbacks.
- Solver parallelism, SIMD, native CPU tuning, or alternate ordering — excluded from the canonical deterministic baseline and considered only after parity and profiling evidence.

</deferred>

______________________________________________________________________

*Phase: 07-rigid-solver-world-operations-and-ccd*
*Context gathered: 2026-07-12*
