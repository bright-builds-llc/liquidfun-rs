---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-13T21:32:39.399Z
---

# Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off - Context

**Gathered:** 2026-07-13
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Finish the rigid-body surface by implementing all eleven pinned upstream joint types, the independent standalone rope model, source-timed contact filters/listeners/destruction notifications and supported pre-solve controls, semantic diagnostic reconstruction, and a closed broad rigid-body differential gate. Particle behavior and the complete public debug-draw/profile surface remain outside this phase.

</domain>

<decisions>
## Implementation Decisions

### Joint identity, definitions, and inspection

- **D-01:** Retain the established opaque world-scoped `JointId`. Add one public `joint` deep module with eleven checked definition types wrapped by a closed tagged `JointDef`: revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope joint, and motor.
- **D-02:** Create joints through granular `World` authority. Definitions own invariant-bearing configuration and semantic body/dependency IDs; world-space convenience constructors may derive canonical local anchors, axes, and reference values without exposing internal storage.
- **D-03:** Expose an owned tagged `JointSnapshot` with common kind/body/collision/anchor state and exhaustive type-specific configuration and runtime state. Compute reactions through an explicit inverse-timestep query rather than storing a context-free force or torque.
- **D-04:** Provide checked type-specific mutation methods for limits, motors, softness, targets, ratios, force/torque caps, offsets, correction, and rope-joint maximum length. Validate the complete request before wake, cached-impulse, adjacency, proxy, or contact effects; distinguish invalid handles, wrong kinds, invalid values, locked worlds, and poisoned worlds.
- **D-05:** Preserve each pinned setter's exact changed-only or unconditional wake behavior. Joint creation does not invent a wake transition. `collide_connected` is creation-time state, and creation/destruction performs source-equivalent contact refiltering.

### Joint dependencies, solving, and origin shifting

- **D-06:** Gear creation atomically validates two live same-world revolute/prismatic source joints, all derived bodies, and upstream topology constraints. Store forward and reverse dependency edges; never permit dangling gear dependencies.
- **D-07:** Destroy dependent gear joints first in deterministic newest-first occurrence order when a source joint or involved body is destroyed, then continue the ordinary source-joint/body cascade. Emit explicit dependency-cascade causes and complete owned snapshots as the safe-Rust strengthening of upstream's delete-gear-first precondition.
- **D-08:** Preserve newest-first world/body joint lanes and source-faithful DFS island traversal. Initialize contacts then joints, solve joints then contacts for velocity iterations, solve contacts then joints for position iterations, retain pinned early-exit behavior, and exclude joints from TOI solving where the selected revision does.
- **D-09:** Translate every joint solver with pinned scalar expression grouping, impulse caches, limit states, gamma/bias calculations, and gear four-body Jacobians. Use named per-observable Phase 4 policies; never add a joint-wide epsilon or reorder through hashes.
- **D-10:** Origin shifting updates only upstream world-space joint state, including pulley ground anchors and mouse targets, while preserving topology, local coordinates, ratios, caches, and semantic identity.

### Standalone rope

- **D-11:** Implement `rope::Rope` independently of `World`, `JointId`, and `RopeJointDef`, with checked owned vertices, fixed/dynamic inverse-mass semantics, gravity, damping, stretch stiffness, bend stiffness, borrow-scoped vertex inspection, checked stepping, and angle control.
- **D-12:** Preserve the pinned rope algorithm and order exactly: zero timestep is a no-op; integrate vertices in index order; each solver iteration runs stretch, bend, stretch; recompute velocities in index order; retain pinned angle wrapping and expression grouping. Invalid lengths, non-finite state, and invalid iteration counts fail transactionally.

### Filters, listeners, pre-solve, and destruction timing

- **D-13:** Use exactly one borrowed synchronous decision hook per operation rather than persistent registration or multi-listener APIs. Passing a later hook replaces the effective decision maker; a no-op hook is unregistration, and any number of consumers may process the owned result afterward.
- **D-14:** Move collision filtering to pinned pair admission and flagged refilter points through a borrow-scoped semantic fixture-pair view. Invoke pre-solve immediately after that contact's update and begin/end emission, only for eligible non-sensor solid contacts, with current and previous semantic manifolds available.
- **D-15:** Keep supported pre-solve control narrow and typed: per-update enable/disable plus only inventory-proven validated material/tangent-speed controls. Never expose `&mut Manifold`, `&mut World`, reusable contact handles, or arbitrary contact mutation.
- **D-16:** Maintain one authoritative owned timeline appended at actual source-equivalent points: filter decision, begin/end, pre-solve, discrete and TOI post-solve impulses, then destruction evidence. Preserve order, multiplicity, and repeated CCD occurrences exactly; convenience slices are projections, never independently grouped reconstructions.
- **D-17:** Direct mutation/destruction reports preserve the pinned distinction between implicit listener notifications and explicit destruction. Body cascades record dependent gear/joint goodbye before invalidation, touching contact end occurrences, implicit fixture goodbye before invalidation, then root destruction; explicit joint/fixture destruction must not fabricate upstream `SayGoodbye` callbacks.
- **D-18:** Hook-requested commands remain bounded, typed, ordered, and deferred until unlock. Synchronous decision-hook panic restores the lock, discards unapplied commands, poisons coherent world operations, and resumes unwinding; post-completion consumers of owned reports cannot poison the world.

### Diagnostic reconstruction and rigid evidence

- **D-19:** Treat upstream dump output as diagnostic reconstruction, not persistence or a byte-format contract. Produce owned typed semantic records for gravity, bodies, fixture shapes/material/filter state, joint definitions/dependencies, reconstruction indices, and explicit unsupported cases; render deterministic human text only as a secondary view.
- **D-20:** Preserve reconstruction dependency order: bodies and fixtures first, non-gear joints next, gear joints last. Mouse-joint dump remains explicitly unsupported when the pinned source cannot faithfully reconstruct it; do not fabricate fields or claim round-trip support.
- **D-21:** Keep Phase 8 renderer-neutral. Do not store renderer callbacks or traits in `World`. Shared headless primitives, if required for focused diagnostics, use semantic owner IDs and no consumer-visible visitation order. Defer complete public debug drawing, timing profiles, particle drawing, and the `RIGD-10` completion claim.
- **D-22:** Extend checkpoint diagnostics with exact body, fixture, joint, contact, manifold-point, and proxy counts plus exact tree height/balance and a named float policy for tree quality. Exclude wall-clock profile timings from D0 and parity evidence.
- **D-23:** Retain every Phase 6/7 witness family unchanged and add fail-closed families for all eleven joints, gear combinations/dependencies/cascades, standalone rope, filter/listener/pre-solve timing, destruction order, collision suppression/refiltering, island order, waking, origin shift, reactions, and semantic reconstruction including unsupported cases.
- **D-24:** Define a closed `phase8-v1` policy registry with no wildcard or automatic widening. Compare identities, kinds, flags, counts, dependencies, branch/limit states, dump field presence, unsupported status, callback/destruction/solver order, and multiplicity exactly; use exact bits, ULP, absolute-relative, or dimensioned absolute policies only for named justified float paths.
- **D-25:** D0 requires byte-identical same-build traces with nondeterministic timing excluded. Only actual pinned Linux x86_64 Rust 1.97.0/Clang 22.1.8 D1 evidence may support the scoped claim “canonical scalar rigid-body and joint differential sign-off for the closed Phase 8 corpus.” D2 remains non-promotable supported-platform evidence and D3 remains diagnostic.
- **D-26:** Unknown witness families, joint kinds, dump fields, observations, missing declarations, or policy paths are harness failures. Without clean debug/release comparison, replay, D0, sanitizer, and actual D1 evidence, keep `RIGD-11` and `JOIN-05` pending and report only the narrower evidence achieved.

### the agent's Discretion

- Exact public/private type, module, method, error, event, diagnostic-record, and witness-family names within the locked contracts.
- Exact plan decomposition across joint families and shared solver infrastructure, provided every joint is independently fail-closed and gear/rope/callback/sign-off work remains explicit.
- Exact field-specific numerical thresholds, bounded capacities, and corpus sizes when justified from pinned-source analysis and canonical evidence.
- Whether supported material/tangent-speed pre-solve controls ship in the first callback slice, provided the pinned Phase 8 inventory explicitly proves or excludes each control and coverage remains fail-closed.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked architecture

- `.planning/PROJECT.md` — Native-Rust, oracle-isolation, safety, determinism, and truthfulness constraints.
- `.planning/REQUIREMENTS.md` — `RIGD-11` and `JOIN-01` through `JOIN-05` acceptance requirements; `RIGD-10` remains incomplete.
- `.planning/ROADMAP.md` — Fixed Phase 8 boundary, success criteria, dependency on Phase 7, and research flags.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — `JointId`, destruction, hook, event, mutation, panic, and association contracts.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — Source ordering, numeric policy registry, collection semantics, and D0-D3 evidence authority.
- `.planning/phases/06-minimal-rigid-world-vertical-slice/06-CONTEXT.md` — Contact lifecycle, hook/report evolution, destruction timing, and bounded rigid timeline.
- `.planning/phases/07-rigid-solver-world-operations-and-ccd/07-CONTEXT.md` — Reserved joint traversal lane, solver/CCD ordering, origin shifting, and accumulated rigid evidence.

### Project evidence and public contract

- `ARCHITECTURE.md` — Current world, callback, solver, oracle, and renderer-independence boundaries.
- `COMPATIBILITY.md` — Inventory ownership and evidence-backed compatibility claims.
- `TESTING.md` — Differential, replay, determinism, sanitizer, and authority-tier rules.
- `UPSTREAM.md` — Pinned revision provenance and source-reference rules.

### Pinned upstream implementation oracle

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/Joints/` — Base joint contract plus all eleven joint definitions, setters, solvers, dumps, and reactions.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2World.cpp` — Creation/destruction, island traversal, dump order, origin shift, and world callback timing.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2ContactManager.cpp` — Pair admission, refilter, and contact update ordering.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h` — Filter, contact listener, destruction listener, and debug-draw interfaces.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Rope/` — Standalone rope state, step ordering, constraints, and angle behavior.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/liquidfun/src/identity.rs`: existing world-scoped `JointId` and typed handle invariants.
- `crates/liquidfun/src/world/object.rs`: placeholder joint arena, newest-first body adjacency, centralized validation/destruction, owned snapshots, counts, proxies, and origin-shift integration points.
- `crates/liquidfun/src/world/island.rs`: reserved joint lanes in ephemeral source-ordered islands.
- `crates/liquidfun/src/world/step.rs`: borrow-scoped `ContactView`, restricted decisions, deferred commands, poisoning, `StepReport`, and ordered lifecycle evidence.
- `crates/liquidfun-differential/src/rigid_world/` and `crates/liquidfun-differential/src/rigid_evidence/`: bounded Phase 6/7 protocol, comparison, replay, minimization, and evidence extension seams.
- `tools/reference/src/rigid_world*`: long-lived pinned C++ oracle decode, execution, validation, and trace seams.

### Established Patterns

- Candidate-first checked mutation with one commit after full validation.
- Opaque semantic identities and owned snapshots/events; no raw pointers, durable contact handles, or storage-order promises.
- Source-significant traversal and callback/destruction order preserved exactly; unordered queries canonicalized only in evidence collectors.
- Closed per-observable tolerance policies and fail-closed declarations; local runs remain D2.
- Deep cohesive modules with private solver/state machinery and thin public `World` authority methods.

### Integration Points

- Replace the placeholder `Joint` record and generic `create_joint` seam without changing `JointId` identity.
- Populate existing island `joint_ids` and reserved solver traversal while keeping continuous/TOI exclusions source-faithful.
- Move filter/pre-solve invocation into contact-manager update/admission points and make existing `StepReport` views projections of one timeline.
- Extend body-cascade destruction, origin shifting, diagnostics, semantic protocol models, native adapter, C++ oracle, comparator registry, fixtures, and CI evidence lanes.

</code_context>

<specifics>
## Specific Ideas

- Make safe Rust enforce gear dependency lifetime instead of reproducing upstream dangling-pointer preconditions.
- Keep the rope joint and standalone rope visibly separate in modules, types, and evidence.
- Treat callback timing as part of physics compatibility: the authoritative timeline is appended where effects occur, not reconstructed after stepping.
- Treat dump as a semantic reconstruction aid with explicit unsupported cases, never a persistence promise or whitespace-parity target.
- Keep the final claim deliberately scoped to the closed rigid corpus even after D1 passes; particles, cross-platform validation, performance, testbed, and release readiness remain separate work.

</specifics>

<deferred>
## Deferred Ideas

- Complete public renderer-neutral debug-draw primitives, timing profiles, particle drawing, and the `RIGD-10` completion claim remain with their later roadmap phase.

</deferred>

***

*Phase: 08-joints-rope-callbacks-and-rigid-sign-off*
*Context gathered: 2026-07-13*
