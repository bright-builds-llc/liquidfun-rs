# Phase 10: Particle Groups, Solvers, and Compatibility Sign-Off - Research

<user-constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Particle-group public contract

- **D-01:** Introduce one cohesive public particle-group module with owned, typed, fixed-order creation recipes rather than a nullable C++-shaped definition. Represent filled shapes, stroke shapes, and explicit positions as invariant-bearing source types whose evaluation order is fixed and documented.
- **D-02:** Model the destination separately as either a new group or an explicit append target carrying a live same-system `ParticleGroupId`. Do not conflate the existing-group target with the particle sources or permit contradictory source combinations.
- **D-03:** Preserve the pinned group definition semantics for particle flags, group flags, transform, linear velocity, angular velocity, color, strength, stride, lifetime, user association, and source sampling while using owned safe Rust data and no borrowed raw buffers.
- **D-04:** Expose a borrow-scoped `ParticleGroupView`-style contract with the stable group ID, public flags, transform, center, linear and angular velocity, mass, inertia, member count, stable member `ParticleId` values, and aligned depth values where applicable. Keep dense row numbers, internal flags, cached statistics, and mutable storage private.
- **D-05:** Treat every group creation as a complete validate-then-commit transaction. Sampling, capacity, handle, allocation, topology, or invariant failure creates no particles or group identity, emits no lifecycle occurrence, and leaves every lane and cache unchanged.
- **D-06:** Joining preserves group A's identity and every particle identity, rotates the dense ranges in the pinned order, unions the required flags, generates only the source-equivalent cross-group connections, and invalidates group B only after the transaction commits.
- **D-07:** Splitting preserves the original group identity for the pinned first longest connected component, allocates new group IDs for later components in source component order, preserves every particle ID, and reproduces the pinned group and particle ordering without public index churn.
- **D-08:** Destroying a group's particles uses the established zombie and deferred-compaction lifecycle. Invalidate the group after its last member is removed unless the pinned can-be-empty flag retains it; retained empty groups remain valid and inspectable with source-equivalent zero-valued aggregate state.

### Topology and group mutation ordering

- **D-09:** Keep `ParticleStorage` as the single state authority. Add pure private topology-planning kernels beneath it, but commit group ranges, dense permutations, depth, rigid caches, pairs, triads, particle flags, and group metadata through one storage-owned source-order mutation candidate.
- **D-10:** Do not introduce a separately mutable topology graph or persist public stable IDs inside solver-significant topology records. Internal topology may use private dense references while every public and protocol boundary translates them to checked stable semantic IDs.
- **D-11:** Encode each pinned operation explicitly instead of using a generic “recompute topology” fallback. Ordinary buffer rotations remap existing pairs and triads without sorting or rebuilding historical rest values.
- **D-12:** Generate Voronoi seeds in current dense order and consume nodes in the pinned row-major order. Apply the upstream connection filters, distance tests, group-flag gates, and edge-case handling before producing pair or triad candidates.
- **D-13:** When topology generation appends new pairs or triads, preserve the pinned orientation, stable ordering, duplicate policy, strength, rest distance, and triad coefficients. Stable-sort and retain the first duplicate only at the exact source operations that do so.
- **D-14:** Joining rotates first and generates only cross-boundary constraints; splitting retargets surviving historical records to the resulting groups rather than regenerating their rest state; reactive regeneration clears the reactive flag only after pair and triad updates complete.
- **D-15:** Solid depth and rigid-group state are invalidated, recomputed, and advanced only at their pinned points. A failed group mutation may not expose partially recomputed depth, center, mass, inertia, transform, or rigid velocity.

### Pinned solver graph and flag behavior

- **D-16:** Define one private, closed, versioned `phase10-pass-graph-v1` manifest derived from the pinned `b2ParticleSystem::Solve` call graph. The manifest owns pass IDs, gates, multiplicity, and order; unknown, missing, duplicated, or reordered passes are failures.
- **D-17:** Preserve the outer ordering around sub-iterations: lifetime solving, zombie compaction, and all-flags refresh occur at their pinned points before the pause gate; paused systems skip the solver without fabricating group, contact, topology, or lifecycle changes.
- **D-18:** Within each particle sub-iteration, preserve the pinned order for contact and body-contact refresh, weight, conditional depth and reactive-topology updates, force and flag-driven passes, gravity, pressure and damping families, elastic and spring constraints, velocity limiting, rigid damping, barrier and collision response, rigid motion, wall enforcement, and final position integration. Research must transcribe the exact source graph and guards into the manifest before implementation begins.
- **D-19:** Implement every unflagged baseline pass and every public flag-driven behavior as a named cohesive kernel around the existing authoritative storage. Do not collapse materially different flags behind one generic approximate behavior.
- **D-20:** Cover water, wall, spring, elastic, viscous, powder, tensile, color mixing, barrier, static pressure, reactive, and repulsive behavior, plus the solid and rigid group flags and every interaction that changes pass admission or equations.
- **D-21:** Preserve interaction rules explicitly, including powder and tensile pressure suppression, static-pressure extra damping, reactive pair and triad regeneration and clearing, spring-pair and elastic-triad constraints, color mixing's both-particles gate, cross-group repulsion, barrier/wall behavior, and solid/rigid group effects.
- **D-22:** Keep zero-valued water a first-class compatibility leaf even though it has no bit to test. Keep Phase 9-owned zombie, destruction-listener, contact-filter, and contact-listener flags in the Phase 10 closure ledger without reassigning their implementation ownership.
- **D-23:** Preserve source-significant particle-system, group, particle, contact, pair, triad, and solver order. Never use hash iteration, default parallelism, broad canonicalization, fast-math, or a global tolerance to hide ordering or numerical divergence.
- **D-24:** Validate public mutations and solver inputs before effects. Non-finite values, invalid handles, wrong systems, invalid ranges, locked or poisoned worlds, capacity failures, and topology failures remain typed and transactional under the Phase 9 contracts.

### Testing, differential evidence, and sign-off

- **D-25:** Extend the existing long-lived Phase 9 rigid-world protocol, native adapter, C++ oracle, comparator, replay, and evidence pipeline. Do not create a parallel particle-group or solver harness.
- **D-26:** Give every particle flag, zero-valued water behavior, unflagged solver pass, group flag, group mutation, topology operation, and inherited lifecycle, buffer, contact, query, and callback path an individual closed ledger leaf with explicit implementation, test, witness, policy, and evidence references.
- **D-27:** Native tests may expose private test-only pass IDs to compare exact pass admission, multiplicity, and order against `phase10-pass-graph-v1`; those IDs are not public API and do not substitute for semantic differential evidence.
- **D-28:** For every flag or pass, include a control witness where the branch is inactive and an activation witness proving its semantic effect. Add bounded interaction witnesses wherever a single-flag case cannot prove the pinned branch or ordering behavior.
- **D-29:** Compare structural fields, stable IDs, flags, membership, counts, branch states, pass traces, order, and multiplicity exactly. Assign exact-bit, ULP, absolute-relative, or dimensioned-absolute policies only to named numeric paths with fixed horizons and source/evidence justification.
- **D-30:** TEST-01, TEST-02, and TEST-04 close through explicit leaf-to-test mappings: focused pure-kernel unit tests, public world/group/particle integration workflows, and reproducible property models for permutations, connectivity, topology, handles, geometry, queries, and world operation sequences.
- **D-31:** Retain all Phase 6 through 9 witness families and evidence authority unchanged. Unknown Phase 10 leaves, pass IDs, observations, policies, flags, group behaviors, or missing declarations are harness failures.
- **D-32:** D0 requires byte-identical same-build traces with nondeterministic timing excluded. Only actual pinned Linux x86_64 Rust 1.97.0/Clang 22.1.8 D1 evidence may promote Phase 10 leaves. D2 remains non-promotable supported-platform evidence and D3 remains diagnostic.
- **D-33:** Promote compatibility rows only from a complete, current, same-run authority set after debug, release, replay, D0, sanitizer, exact-ref, schema, provenance, and deterministic-report checks pass. Partial evidence may improve implementation or unit-test states but cannot sign off parity.
- **D-34:** Phase completion requires every Phase 10 leaf to have an explicit supported, documented-difference, or intentionally-unsupported outcome. Do not claim complete particle parity, examples/testbed maturity, performance, broad platform support, or v1 release readiness beyond the exact evidence achieved.

### the agent's Discretion

- Exact public and private type, module, method, error, view, recipe, transaction-candidate, pass-ID, witness-family, and ledger-leaf names within the locked contracts.
- Exact decomposition across plans and cohesive child modules, provided `ParticleStorage` remains the single authority and source operations remain independently auditable.
- Exact bounded corpus sizes, property-case counts, source-derived capacity bounds, and named numerical thresholds when justified by pinned-source analysis and canonical evidence.
- Whether test-only pass tracing is compiled under `cfg(test)` or an unpublished tooling feature, provided it cannot enter the published public API or become the sole differential oracle.

### Deferred Ideas (OUT OF SCOPE)

- Upstream example/testbed accounting, the renderer-neutral scenario catalog, headless controls, debug drawing, and optional visualization — Phase 11.
- Performance budgets, profiling-led optimization, fuzzing breadth, Miri/sanitizer expansion, broad platform evidence, coverage policy, release documentation, packaging, and v1 audit — Phase 12.
- Generic allocator traits, GPU storage, unsafe raw-buffer interoperability, SIMD, parallel stepping, and alternate precision modes — only after measured need and separate compatibility/safety decisions.
</user-constraints>

<phase-requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| PART-09 | Consumers can create particle groups from shapes, strokes, explicit positions, or existing groups and inspect group state. | The public recipe, view, sampling-order, and transaction patterns below define the required API and source semantics. [VERIFIED: `.planning/REQUIREMENTS.md`] |
| PART-10 | Group lifecycle, join, split, connectivity, empty retention, solid depth, rigid motion, and contiguous membership preserve upstream semantics. | The mutation algorithms, group-state authority, cache points, and tests below transcribe the pinned implementation. [VERIFIED: pinned `b2ParticleSystem.cpp` and `b2ParticleGroup.cpp`] |
| PART-11 | Voronoi topology, pairs, triads, and reactive regeneration are upstream-equivalent. | The topology section records seed/node order, filters, coefficients, stable sorting, deduplication, and historical-state rules. [VERIFIED: pinned `b2VoronoiDiagram.cpp` and `b2ParticleSystem.cpp::UpdatePairsAndTriads`] |
| PART-12 | Baseline collision, gravity, pressure, damping, rigid damping, extra damping, force, velocity limiting, and lifetime passes run in pinned order. | The closed pass graph below transcribes the outer and per-sub-iteration call graph. [VERIFIED: pinned `b2ParticleSystem.cpp::Solve`] |
| PART-13 | Every public particle behavior matches the selected upstream behavior. | The kernel inventory and control/activation validation map cover water, wall, spring, elastic, viscous, powder, tensile, barrier, static pressure, reactive, repulsive, and color mixing. [VERIFIED: `.planning/REQUIREMENTS.md`; pinned `b2ParticleSystem.cpp`] |
| PART-18 | Every flag, pass, group behavior, and inherited particle path is individually represented and signed off. | The proposed closed Phase 10 ledger extends the existing Phase 9 manifest/evidence pipeline and retains Phase 6-9 proof families. [VERIFIED: Phase 9 manifest, comparator, evidence script, and `reference/compatibility.json`] |
| TEST-01 | Focused unit tests cover pure math, geometry, ordering, identity, and solver kernels. | The validation architecture assigns each pure sampler/topology/solver operation a focused native test. [VERIFIED: `.planning/REQUIREMENTS.md`; `standards/core/testing.md`] |
| TEST-02 | Supported public workflows have integration tests. | The validation map covers public group creation, view, mutation, solver, callback, and lifecycle workflows. [VERIFIED: `.planning/REQUIREMENTS.md`; existing `crates/liquidfun/tests/`] |
| TEST-04 | Property tests cover permutation/group invariants and reproducible operation sequences. | The validation map extends current `proptest` storage models with group/topology and transaction models. [VERIFIED: `Cargo.toml`; existing particle permutation tests] |
</phase-requirements>

## Summary

Phase 10 is one integrated state-machine phase, not a group feature followed by a solver feature. The pinned solver consumes group ranges, depth, pairs, triads, aggregate flags, rigid statistics, contacts, forces, fixture sweeps, and body impulses in one exact order. The current Phase 9 `run_particle_contact_prefix` already refreshes contacts and applies a body-only pressure/damping subset, so Phase 10 must replace that prefix with the complete storage-owned solve transaction; appending the missing passes would execute pressure and damping twice and in the wrong place. [VERIFIED: `crates/liquidfun/src/world/particle_coupling.rs`; pinned `b2ParticleSystem.cpp::Solve`]

The correct architecture is a deep `particle` module with `ParticleStorage` as the sole solver-significant authority. Pure functions plan sampling, connectivity, Voronoi output, rotations, split components, and individual solver deltas; a storage mutation candidate validates every affected lane and commits once. The world group arena should remain the stable identity/lifecycle shell, while flags, ranges, strength, transform, depth, and statistics live in the system storage so the existing cloned-system step transaction includes them automatically. [VERIFIED: CONTEXT D-09; `world/object.rs`; `world/particle_coupling.rs`]

Compatibility closure requires a Phase 10 extension of the Phase 9 long-lived rigid-world harness, not a new executable or schema family. Add closed group/topology/solver declarations and observations, a closed `phase10-pass-graph-v1`, control-plus-activation witnesses, named numeric policies, same-run debug/release/replay/D0/sanitizer evidence, and exact-ref validation before changing compatibility rows. [VERIFIED: CONTEXT D-25 through D-33; `phase9_corpus.rs`; `scripts/phase9-evidence.sh`; `.github/workflows/oracle.yml`]

**Primary recommendation:** Implement Phase 10 in dependency waves—closed inventories and data model first, transactional group/topology operations second, the exact solver graph third, and semantic evidence/promotion last—while replacing the Phase 9 contact prefix only when the complete pass transaction is ready. [VERIFIED: dependency analysis of current storage/step seams and pinned solve graph]

## Project Constraints (from AGENTS.md)

- Keep production physics native Rust; C++ remains a private read-only oracle and normal Cargo builds must not require the submodule or C++ toolchain. [VERIFIED: `AGENTS.md`; `.planning/PROJECT.md`]
- Preserve one deep module and a functional-core/imperative-shell boundary; do not create parallel state authorities or mirror upstream source directories with shallow crates. [VERIFIED: `AGENTS.bright-builds.md`; `standards/core/architecture.md`]
- Safe Rust is the default; any future `unsafe` must be narrow, justified, carry a `SAFETY:` invariant, and receive focused tests. No Phase 10 design requires `unsafe`. [VERIFIED: `AGENTS.md`; source audit]
- Preserve deterministic source order; do not use hash iteration, default parallelism, fast math, or silent error handling. [VERIFIED: `AGENTS.md`; CONTEXT D-23]
- Public APIs require concise documentation, typed errors, stable identity, borrow-scoped views, and no raw pointers or dense storage details. [VERIFIED: `AGENTS.md`; Phase 3 context]
- Rust modules use `foo.rs` plus `foo/`, optional values use `maybe_` where helpful, `unwrap()` is forbidden, and unit tests use one concern with Arrange/Act/Assert. [VERIFIED: `AGENTS.md`; `standards/languages/rust.md`; `standards/core/testing.md`]
- Before a Rust commit, run `cargo fmt --all`, Clippy with all targets/features and warnings denied, build all targets/features, then all-feature tests. `.planning/**` must not be run through mdformat. [VERIFIED: `AGENTS.md` Repo-Local Guidance]
- Standalone `---` delimiters are reserved for opening/closing YAML frontmatter and must not be used as body separators in GSD Markdown. [VERIFIED: `AGENTS.md`]

## Standard Stack

### Core

| Component | Version | Purpose | Why Standard |
| --- | --- | --- | --- |
| Rust | 1.97.0 repository pin; publishable MSRV 1.92.0 | Native group, topology, and solver implementation | Already pinned and locally active; no language/toolchain change belongs in this phase. [VERIFIED: `rust-toolchain.toml`; `cargo metadata`; local `rustc --version`] |
| `liquidfun` | workspace `0.0.0` | Published native engine | Existing storage, IDs, views, contacts, queries, and transactional world step are the correct authority to deepen. [VERIFIED: `crates/liquidfun/Cargo.toml`; source audit] |
| `bitflags` | 2.13.0 | Particle and group public bit sets | Already the production representation for particle flags; add a separate public group-flag type while keeping internal group bits private. [VERIFIED: workspace `Cargo.toml`; `particle/definition.rs`; pinned group flags] |

### Supporting

| Component | Version | Purpose | When to Use |
| --- | --- | --- | --- |
| `proptest` | 1.11.0 | Reproducible group/topology/permutation properties | Dev-only for operation-sequence, connectivity, stable-ID, transaction, and Voronoi invariants. [VERIFIED: workspace `Cargo.toml`; `crates/liquidfun/Cargo.toml`] |
| `serde` / `serde_json` | 1.0.228 / 1.0.150 | Closed protocol and evidence records | Private protocol/differential/xtask crates only; do not add them to `liquidfun`. [VERIFIED: workspace manifests and `cargo metadata`] |
| `thiserror` | 2.0.18 | Closed library/tool error enums | Reuse in private harness crates; production `liquidfun` currently uses standard error traits and does not need a new dependency solely for Phase 10. [VERIFIED: workspace manifests; `particle/definition.rs`] |
| Pinned C++ oracle | revision `7f20402173fd143a3988c921bc384459c6a858f2` | Semantic reference and exact witness generation | Only through the existing CMake/Ninja long-lived oracle and evidence jobs. [VERIFIED: `reference/upstream.lock.json`; `UPSTREAM.md`] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
| --- | --- | --- |
| Storage-owned group table | Put mutable group state in the world arena | The existing step clones particle systems and bodies, not the group arena; solver caches in the arena would escape transaction rollback and duplicate storage authority. Do not use it. [VERIFIED: `world/particle_coupling.rs`; CONTEXT D-09] |
| Pure operation-specific planners | Generic topology graph/rebuild | Rebuilding destroys historical spring/elastic rest state and changes operation-specific sort/dedup behavior. Do not use it. [VERIFIED: pinned join/split/rotate/update functions; CONTEXT D-11 through D-14] |
| Existing ordered `Vec` lanes | Hash maps/sets for topology or pass admission | Hash iteration would alter solver-visible ordering and violates the locked determinism policy. Do not use it. [VERIFIED: CONTEXT D-23] |
| Extend rigid-world Phase 9 protocol | Separate particle-solver harness | A second harness would split retained evidence, replay, provenance, exact-ref, and unknown-field enforcement. Do not use it. [VERIFIED: CONTEXT D-25; current harness audit] |

**Installation:** No new production or development package is required for the recommended implementation. [VERIFIED: current workspace dependency graph and required algorithms]

**Version verification:** The versions above come from current workspace manifests and `cargo metadata`; the active compiler is Rust 1.97.0. This Rust phase has no npm packages to verify. [VERIFIED: local commands on 2026-07-19]

## Architecture Patterns

### Recommended Project Structure

```text
crates/liquidfun/src/
├── particle.rs
├── particle/
│   ├── group.rs                 # public recipes, flags, views, typed errors
│   ├── storage.rs               # sole state authority and commit boundary
│   ├── storage/
│   │   ├── group.rs             # metadata, ranges, caches, validation
│   │   ├── lanes.rs             # aligned solver lanes and full triad state
│   │   ├── mutation.rs          # operation-specific mutation candidates
│   │   └── permutation.rs       # rotations/remaps with property tests
│   ├── topology.rs              # private facade
│   ├── topology/
│   │   ├── connectivity.rs      # split components and pinned tie behavior
│   │   ├── voronoi.rs           # bounded source-order diagram
│   │   └── constraints.rs       # pair/triad generation/filter/sort/dedup
│   ├── solver.rs                # phase10-pass-graph-v1 and dispatcher
│   └── solver/
│       ├── material.rs          # viscous/repulsive/powder/tensile/solid/color
│       ├── pressure.rs          # weight/static/dynamic pressure and damping
│       ├── constraints.rs       # elastic/spring/limit
│       └── boundary.rs          # rigid damping/barrier/collision/rigid/wall
└── world/
    ├── particle_object.rs       # public validation and identity shell
    └── particle_coupling.rs     # replace prefix with complete step transaction

crates/liquidfun-test-protocol/src/schema/rigid_world/
└── phase10.rs                   # closed additions to the existing record family
```

This is a recommended decomposition, not a requirement to create every file immediately; split only when cohesive units approach the repository's 300-500-line review threshold. [VERIFIED: `standards/core/code-shape.md`; source-size guidance]

### Pattern 1: Stable Identity Shell, Storage-Owned Physics State

`ParticleGroupId` remains allocated by the world arena, but the arena object should retain only identity ownership, diagnostic identity, and owning-system association. The corresponding `ParticleStorage` group row owns public flags, internal flags, contiguous range, strength, transform, user association, depth, and cached statistics. Public access resolves the world ID, verifies same-system ownership, and borrows a storage view. [VERIFIED: current placeholder `ParticleGroup`; existing `ParticleSystem.storage`; CONTEXT D-04 and D-09]

This avoids two mutable copies and ensures the existing candidate-system clone covers every solver-significant group mutation. [VERIFIED: `World::run_particle_contact_prefix` clone/commit design]

### Pattern 2: Validate, Plan, Validate Candidate, Commit Once

```rust
// Source pattern: current Phase 9 storage/world transaction boundaries plus CONTEXT D-05.
fn create_group(
    &mut self,
    system: ParticleSystemId,
    recipe: ParticleGroupRecipe,
) -> Result<ParticleGroupId, ParticleGroupError> {
    let checked = self.validate_group_recipe(system, recipe)?;
    let sampled = self.plan_group_samples(&checked)?;
    let candidate = self.plan_group_creation(system, checked, sampled)?;
    candidate.validate_complete()?;
    self.commit_group_creation(candidate)
}
```

Sampling must not allocate a public ID or mutate lifecycle journals. Capacity, non-finite values, wrong-system append targets, checked grid sizes, handle availability, optional-lane allocation, topology, and full candidate invariants are resolved before commit. [VERIFIED: CONTEXT D-05/D-24; current Phase 9 creation contract]

### Pattern 3: Operation-Specific Topology Transactions

Create separate private candidates for create, join, split, zombie compaction, reactive regeneration, group-flag changes, and ordinary rotation. Each candidate carries the exact required permutation, range updates, pair/triad remaps or additions, depth invalidations, aggregate-flag changes, and lifecycle effects. [VERIFIED: pinned operations have different algorithms and side effects]

Ordinary rotations remap historical constraints without sorting. Create/join/reactive generation appends candidates, then stable-sorts and keeps the first exact duplicate at the pinned `UpdatePairsAndTriads` call. Split retargets historical pair/triad dense references and retains their rest values rather than regenerating them. [VERIFIED: pinned `RotateBuffer`, `JoinParticleGroups`, `CreateParticleGroupsFromParticleList`, and `UpdatePairsAndTriads`]

### Pattern 4: Closed Pass Manifest Drives Dispatch and Tests

The private manifest should be data, not duplicated match-arm folklore: ordered pass ID, outer/per-substep scope, exact gate, multiplicity, required scratch lanes, and kernel function. The dispatcher walks it once; a validator rejects an unknown, omitted, duplicated, or reordered declaration. Test-only tracing records admitted pass IDs and iteration ordinals from this same manifest. [VERIFIED: CONTEXT D-16/D-27]

Do not expose pass IDs in public Rust API or use them as C++ semantic evidence. The C++ side proves effects through state/contact/group/topology observations; exact native trace checks prove the Rust dispatcher matches its closed manifest. [VERIFIED: CONTEXT D-27/D-28]

### Exact `phase10-pass-graph-v1`

The following outer order occurs once per nonempty system solve. [VERIFIED: pinned `b2ParticleSystem.cpp:2973-3095`]

| Order | Pass ID recommendation | Gate | Multiplicity | Source |
| --- | --- | --- | --- | --- |
| O01 | `lifetime` | expiration lane exists | Once per step | [VERIFIED: `SolveLifetimes`] |
| O02 | `zombie_compaction` | aggregate particle flags include zombie | Once per step | [VERIFIED: `SolveZombie`] |
| O03 | `refresh_particle_flags` | aggregate particle flags dirty | Once per step | [VERIFIED: `UpdateAllParticleFlags`] |
| O04 | `refresh_group_flags` | aggregate group flags dirty | Once per step | [VERIFIED: `UpdateAllGroupFlags`] |
| O05 | `pause_gate` | always after O01-O04 | Once per step; terminates if paused | [VERIFIED: `b2ParticleSystem.cpp::Solve`] |

An empty system returns before all five outer entries; a paused nonempty system performs O01-O04 and then performs no contact, topology, timestamp, solver, or integration work. [VERIFIED: pinned `b2ParticleSystem.cpp::Solve`]

The following exact order repeats for each configured particle iteration after timestamp increment and source-equivalent substep derivation (`dt / particleIterations`, `inv_dt * particleIterations`). [VERIFIED: pinned `b2ParticleSystem.cpp:2999-3093`]

| Order | Pass ID recommendation | Gate | Core effect |
| --- | --- | --- | --- |
| S01 | `particle_contacts` | Always | Refresh particle contacts in source order. [VERIFIED: `UpdateContacts(false)`] |
| S02 | `body_contacts` | Always | Refresh fixture contacts and stuck state. [VERIFIED: `UpdateBodyContacts`] |
| S03 | `weight` | Always | Zero weights then accumulate particle-contact weights. [VERIFIED: `ComputeWeight`] |
| S04 | `solid_depth` | any group needs depth | Recompute scheduled solid depth before solid solve. [VERIFIED: `ComputeDepth`] |
| S05 | `reactive_topology` | aggregate reactive flag | Append reactive pairs/triads then clear reactive only after success. [VERIFIED: `UpdatePairsAndTriadsWithReactiveParticles`] |
| S06 | `force` | pending system force | Apply force lane and clear the pending-force marker. [VERIFIED: `SolveForce`] |
| S07 | `viscous` | aggregate viscous flag | Body-contact and particle-contact viscosity. [VERIFIED: `SolveViscous`] |
| S08 | `repulsive` | aggregate repulsive flag | Repel contacts across distinct group memberships. [VERIFIED: `SolveRepulsive`] |
| S09 | `powder` | aggregate powder flag | Apply high-weight scattering impulse. [VERIFIED: `SolvePowder`] |
| S10 | `tensile` | aggregate tensile flag | Accumulate weighted normals then apply surface tension. [VERIFIED: `SolveTensile`] |
| S11 | `solid` | aggregate solid group flag | Eject across different groups using depth. [VERIFIED: `SolveSolid`] |
| S12 | `color_mixing` | aggregate color-mixing flag | Mix only when both contact particles individually carry the flag. [VERIFIED: `SolveColorMixing`; programmer guide] |
| S13 | `gravity` | Always | Add scaled world gravity. [VERIFIED: `SolveGravity`] |
| S14 | `static_pressure` | aggregate static-pressure flag | Run configured relaxation iterations. [VERIFIED: `SolveStaticPressure`] |
| S15 | `pressure` | Always | Compute dynamic/static pressure and couple particle/body and particle/particle contacts. [VERIFIED: `SolvePressure`] |
| S16 | `damping` | Always | Apply body and particle contact damping. [VERIFIED: `SolveDamping`] |
| S17 | `extra_damping` | aggregate extra-damping flags (pinned: static pressure) | Apply extra body-contact damping. [VERIFIED: `k_extraDampingFlags`; `SolveExtraDamping`] |
| S18 | `elastic` | aggregate elastic flag | Enforce triad rest state using predicted positions. [VERIFIED: `SolveElastic`] |
| S19 | `spring` | aggregate spring flag | Enforce pair rest distances using predicted positions. [VERIFIED: `SolveSpring`] |
| S20 | `limit_velocity` | Always | Clamp to diameter/substep critical velocity. [VERIFIED: `LimitVelocity`; programmer guide Maximum Velocity] |
| S21 | `rigid_damping` | aggregate rigid group flag | Couple rigid groups with bodies and other groups. [VERIFIED: `SolveRigidDamping`] |
| S22 | `barrier` | aggregate barrier flag | Prevent third-particle crossing and preserve follow-up force semantics. [VERIFIED: `SolveBarrier`] |
| S23 | `collision` | Always | Sweep particles against fixtures and apply body reaction. [VERIFIED: `SolveCollision`] |
| S24 | `rigid` | aggregate rigid group flag | Advance group transform and write member velocities. [VERIFIED: `SolveRigid`] |
| S25 | `wall` | aggregate wall flag | Zero wall-particle velocities after collision/rigid work. [VERIFIED: `SolveWall`] |
| S26 | `integrate` | Always | Update every position only at substep end. [VERIFIED: final loop in `b2ParticleSystem.cpp::Solve`] |

### Group Sampling and Mutation Algorithms

- **Filled shapes:** compute the local union AABB, snap both axes with `floor(lower / stride) * stride`, visit `y` outer then `x` inner while each coordinate is strictly below its upper bound, test union membership, then transform accepted points. A filled multi-shape source is sampled as a union once, not as independent shapes that duplicate overlaps. [VERIFIED: pinned `CreateParticlesFillShapeForGroup` and `CompositeShape`]
- **Stroke shapes:** edge/chain sampling uses the group stride or default particle stride; `positionOnEdge` carries across consecutive chain children, so resetting it per edge changes samples at vertices. [VERIFIED: pinned `CreateParticlesStrokeShapeForGroup`]
- **Explicit positions:** preserve input order and transform each input point. [VERIFIED: pinned `CreateParticleGroup` explicit-position loop]
- **Initial velocity:** `linearVelocity + cross(angularVelocity, worldPosition - group.position)` in source expression order. [VERIFIED: pinned `CreateParticleForGroup`]
- **Append target:** plan a temporary new group, then execute the exact join transaction into the validated target, but expose only one atomic commit and return the target ID. [VERIFIED: pinned create-then-join semantics; CONTEXT D-02/D-05]
- **Join:** rotate B to the system end, rotate A immediately before B, refresh contacts, add only cross-threshold pairs/triads, assign B members to A, union flags, extend A, and invalidate B last. [VERIFIED: pinned `JoinParticleGroups`]
- **Split:** scan same-group contacts in contact order, union by longer linked list with ties retaining list A, select the first longest source component, merge zombies into it, retain the original ID for that component, and create later group IDs in component-source order. Preserve public particle IDs through the equivalent row permutation and retarget historical pairs/triads rather than cloning public identity. [VERIFIED: pinned split helper sequence; CONTEXT D-07]
- **Empty groups:** upstream explicitly tests creating a can-be-empty zero-particle group, joining into it, destroying its particles, stepping, and retaining the group with zero members. Non-retained groups disappear after deferred zombie compaction. [VERIFIED: pinned `CreateEmptyParticleGroupWithNoShape` and `DestroyParticleGroup` tests]

### Topology Exactness

Pair eligibility is spring or barrier, with connection additionally allowed for wall/spring/elastic particles or particles in a rigid group. Pairs are emitted from contact order, store the current rest distance and minimum endpoint-group strength, then are stable-sorted lexicographically and exact duplicates retain the first record. [VERIFIED: pinned `ParticleCanBeConnected`, `UpdatePairsAndTriads`, and pair comparator]

Triads are attempted only when elastic flags are present. Eligible generators are added in current dense order; the Voronoi grid is seeded in generator order, propagates neighbors left/down/right/up, preserves the current generator on equal squared distance, and emits two oriented nodes per cell in row-major order. Accepted triads store source orientation, centroid-relative rest offsets `pa/pb/pc`, coefficients `ka/kb/kc/s`, and minimum group strength before stable sort/first-duplicate retention. [VERIFIED: pinned `b2VoronoiDiagram.cpp`; pinned triad callback in `UpdatePairsAndTriads`]

The current Rust `ParticleTriad` has only indices, flags, and strength, so Phase 10 must add `pa`, `pb`, `pc`, `ka`, `kb`, `kc`, and `s` and include them in permutation validation and semantic views. [VERIFIED: `particle/storage/lanes.rs`; pinned `b2ParticleTriad`]

### Solver State and Coefficients

`ParticleSystemDef` must add checked source-default coefficients for pressure, elastic, spring, viscous, surface-tension pressure/normal, repulsive, powder, ejection, static-pressure strength/relaxation, and color mixing. Current Rust already has damping and static-pressure iterations but not these coefficients. [VERIFIED: current `particle/definition.rs`; pinned `b2ParticleSystemDef` defaults]

Aligned optional scratch/state must include static pressure, tensile accumulation, and depth; group metadata must include public/internal flags, range, strength, transform, and timestamped mass/center/velocity/inertia/angular-velocity cache. Optional allocation must remain deterministic and candidate-owned. [VERIFIED: pinned system/group buffers; current optional-lane pattern]

### Anti-Patterns to Avoid

- **Appending passes after `run_particle_contact_prefix`:** it would retain the Phase 9 body-only pressure/damping approximation and execute the complete pressure/damping family out of order. Replace the prefix as a unit. [VERIFIED: current coupling source; exact pass graph]
- **Testing water with `flags.contains(WATER)`:** water is zero bits, and containment of an empty bitset is not an activation test. Treat water as an explicit control/ledger leaf and absence-of-other-behavior baseline. [VERIFIED: pinned `b2_waterParticle = 0`; `ParticleFlags::WATER`]
- **Caching group state in the arena:** the step transaction does not clone that arena. Keep solver state under `ParticleSystem.storage`. [VERIFIED: current world transaction]
- **Sorting every topology mutation:** rotations and split retargeting preserve historical order/rest state; sort only in source operations that append candidates. [VERIFIED: pinned topology operations]
- **Regenerating topology after split:** this changes spring distances and elastic triad rest coefficients. Retarget records. [VERIFIED: pinned split implementation]
- **Canonicalizing semantic order in evidence:** IDs, groups, particles, contacts, pairs, triads, pass order, and occurrence order are comparison data, not unordered sets. [VERIFIED: CONTEXT D-23/D-29]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
| --- | --- | --- | --- |
| Stable IDs and invalidation | New group/particle handle scheme | Existing `Arena`, `ParticleGroupId`, `ParticleId`, and storage ID lanes | Phase 3/9 semantics and protocol translation already depend on them. [VERIFIED: `identity.rs`; current storage] |
| World rollback | Ad hoc inverse operations | Existing clone/validate/commit world-step transaction | Group caches, body impulses, contacts, and solver lanes must commit together. [VERIFIED: `world/particle_coupling.rs`] |
| Topology graph | Mutable graph library | Ordered dense lanes plus pure operation-specific planners | Solver-visible order and historical rest state are compatibility state. [VERIFIED: pinned topology code] |
| Geometry sampling | Generic tessellator/raster library | Source-transcribed fill/stroke samplers | Generic samplers differ at snapping, bounds, union overlap, chain carry, and boundary tests. [VERIFIED: pinned creation functions] |
| Voronoi | General computational-geometry crate | Small bounded source-transcribed grid algorithm | The upstream algorithm's queue order, tie behavior, and row-major nodes determine triad identity/order. [VERIFIED: pinned `b2VoronoiDiagram.cpp`] |
| Pass scheduler | ECS/job graph | Closed ordered static manifest | Reordering or parallelism changes numeric and semantic results. [VERIFIED: CONTEXT D-16/D-23] |
| Numeric comparator | One epsilon | Existing closed path-policy registry extended with named Phase 10 paths | Different fields require exact, ULP, absolute-relative, or dimensioned policies. [VERIFIED: Phase 6-9 comparator design; CONTEXT D-29] |
| Evidence pipeline | New scripts/executables | Phase 9 protocol, oracle, comparator, corpus, evidence validator, and workflow pattern | Existing authority and exact-ref invariants must remain one chain. [VERIFIED: Phase 9 evidence source audit] |

**Key insight:** Most apparently generic infrastructure here—sampling, Voronoi, graph connectivity, pass scheduling, and numeric comparison—contains observable upstream ordering or numerical policy. Reusing the repository's identity/transaction/evidence machinery is correct; substituting generic algorithm libraries is not. [VERIFIED: pinned algorithm and current architecture comparison]

## Common Pitfalls

### Pitfall 1: Phase 9 Prefix Survives as a Hidden Duplicate Solver

**What goes wrong:** Body pressure/damping runs before the full graph and then runs again inside Phase 10. [VERIFIED: current prefix and pinned graph]

**Why it happens:** The current method name sounds like contact preparation, but it also mutates particle and body velocities. [VERIFIED: `apply_body_contact_pressure_and_damping`]

**How to avoid:** First extract/reuse contact refresh, filter/listener, and fixture-source behavior; then replace the caller with a complete full-system candidate transaction. Add a native pass trace proving one admission of S15 and S16 per particle iteration. [VERIFIED: feasible current seams; CONTEXT D-27]

**Warning signs:** Existing `PRESSURE_STRENGTH` remains in `world/particle_coupling.rs`, or tests pass only when pressure/damping are disabled. [VERIFIED: current source]

### Pitfall 2: Public and Internal Group Flags Become One Type

**What goes wrong:** `willBeDestroyed` and `needsUpdateDepth` leak through views/protocol, or callers can set them. [VERIFIED: pinned internal flag mask]

**How to avoid:** Use a public `ParticleGroupFlags` containing solid, rigid, and can-be-empty, plus a private internal state bitset. Aggregate gates combine them only internally. [VERIFIED: pinned `b2ParticleGroupFlag`; CONTEXT D-04]

### Pitfall 3: Aggregate Flags Are Refreshed at the Wrong Time

**What goes wrong:** Removed flags still admit passes, new flags miss passes, or paused systems fabricate work. [VERIFIED: pinned dirty-aggregate design]

**How to avoid:** Flag additions may update aggregates eagerly, removals mark dirty, and the authoritative full refresh stays before the pause gate. Reactive clearing and solid-depth scheduling occur only at their source commit points. [VERIFIED: pinned `SetParticleFlags`, `SetGroupFlags`, `UpdateAll*Flags`, and solve order]

### Pitfall 4: Split Looks Correct but Changes Stable Semantics

**What goes wrong:** Components have the right membership but the wrong surviving ID, order, group metadata, particle IDs, or constraint rest state. [VERIFIED: pinned split tie/order behavior; locked Rust identity decisions]

**How to avoid:** Property-test first-longest ties, contact-order union, intermingled other groups, zombies, pair/triad retargeting, and identity preservation. Add exact structural differential observations before numeric solver witnesses. [VERIFIED: pinned `SplitParticleGroupInterminglingWithOtherGroups` test; CONTEXT D-07/D-14]

### Pitfall 5: Unbounded Sampling or Voronoi Allocation

**What goes wrong:** Tiny stride, huge finite geometry, or a sparse extreme AABB produces excessive loops/grid cells or integer overflow before capacity failure. [VERIFIED: source algorithms derive loops/grid dimensions directly from geometry and radius]

**How to avoid:** Preflight checked axis counts, products, queue bounds, particle capacity, and protocol limits using source-derived bounds; reject transactionally with typed errors before materialization. Preserve source order within accepted bounds. [VERIFIED: CONTEXT D-24; current bounded harness pattern]

**Warning signs:** `as usize` conversions precede range checks, or sampling allocates before comparing with effective capacity. [VERIFIED: Rust integer-conversion risk applied to source formulas]

### Pitfall 6: Degenerate Solver Denominators Are “Fixed” Globally

**What goes wrong:** Adding blanket epsilon guards changes source branch behavior in spring, elastic, barrier, rigid, or collision kernels; omitting all validation can instead create NaNs. [VERIFIED: pinned kernels contain source-specific divisions and branch conditions]

**How to avoid:** Create a focused control/degenerate witness for every denominator, preserve source behavior where the oracle is defined, and reject only invalid public/topology states through named typed invariants. Never add a global epsilon. [VERIFIED: Phase 4 numerical policy; CONTEXT D-24/D-29]

### Pitfall 7: Color Mixing Uses Floating-Point Blending

**What goes wrong:** Results differ because upstream converts `128 * strength` to an integer and uses signed channel deltas with byte wrapping behavior. [VERIFIED: pinned `SolveColorMixing` and `b2ParticleColor::MixColors`; upstream Color tests]

**How to avoid:** Implement and unit-test the exact integer channel operation, including no-mix, full-mix, negative, and overflow cases. Require both particles' individual flags. [VERIFIED: pinned Color tests and programmer guide]

### Pitfall 8: Collision Pass Lacks Previous Body Transform

**What goes wrong:** First-particle-iteration sweeps against the current fixture transform only and misses source-equivalent moving-body collision behavior. [VERIFIED: pinned `SolveCollision` uses `body->m_xf0` on iteration zero]

**How to avoid:** Pass the authoritative previous/current body transforms into the solver candidate and distinguish iteration zero from later sub-iterations. Test moving circle and polygon fixtures plus body reaction. [VERIFIED: pinned collision callback]

### Pitfall 9: Evidence Is Broad but Not Closed

**What goes wrong:** A “mixed particle demo” exercises many flags without proving which branch caused an effect, or a new observation silently bypasses policy validation. [VERIFIED: risk addressed explicitly by CONTEXT D-26/D-28/D-31]

**How to avoid:** Maintain an exact leaf set and require per-leaf control/activation bindings, explicit observation kind, test, policy, and same-run artifact. Reject unknown/missing/duplicate leaves and paths. [VERIFIED: Phase 9 closed registry implementation]

## Code Examples

### Source-Ordered Stable Deduplication

```rust
// Source: pinned UpdatePairsAndTriads behavior.
candidates.sort_by_key(|pair| (pair.indices[0], pair.indices[1]));
candidates.dedup_by(|right, left| right.indices == left.indices);
```

Rust slice sorting must be the stable variant here; retaining the first equal record is observable because its strength/rest state comes from first insertion order. The final implementation should use checked private dense indices and a named helper rather than copying this sketch. [VERIFIED: pinned pair/triad sort-unique behavior; Rust standard stable sort semantics]

### Water Admission

```rust
// Source: pinned b2_waterParticle = 0 and CONTEXT D-22.
let is_water_control = flags.is_empty();
```

Use this only for a closed witness/control classification, not as a general claim that every zero-bit particle has no other state. Solver admission remains based on the exact aggregate gates in the manifest. [VERIFIED: pinned zero flag and pass graph]

### Test-Only Pass Trace

```rust
// Source pattern: CONTEXT D-16/D-27.
#[cfg(any(test, feature = "differential-internals"))]
trace.record(PassTraceEntry {
    pass: descriptor.id,
    particle_iteration,
});
```

If an unpublished feature is used, keep its types crate-private and ensure the default published API/doc surface does not expose them. [VERIFIED: current `differential-internals` feature; CONTEXT D-27]

## State of the Art

| Current/Old Approach | Phase 10 Approach | Impact |
| --- | --- | --- |
| Placeholder world `ParticleGroup` plus storage membership lanes | Stable arena identity shell plus storage-owned group metadata/caches | Makes group mutation inspectable and rollback-safe without duplicate authority. [VERIFIED: current `world/object.rs`; CONTEXT D-09] |
| Phase 9 contact refresh with body-only pressure/damping | Complete exact manifest-driven particle solve | Avoids duplicate/wrong-order coupling and closes PART-12/PART-13. [VERIFIED: current prefix; pinned solve graph] |
| Triads store only indices/flags/strength | Full upstream rest offsets and coefficients | Enables elastic constraints and rotation/split preservation. [VERIFIED: current `lanes.rs`; pinned `b2ParticleTriad`] |
| Phase 9 schema rejects group/pair/triad/solver members | Closed Phase 10 extension of the same rigid-world family | Preserves retained proof families while adding new leaves. [VERIFIED: Phase 9 fixture `forbidden_phase10_members`; CONTEXT D-25] |
| Phase 9 evidence has canonical/sanitizer modes and exact content validation | Phase 10 repeats that authority shape with expanded same-run checks | Compatibility promotion remains provenance-bound and fail-closed. [VERIFIED: `phase9-evidence.sh`; `phase9_evidence.rs`; workflow] |

**Deprecated/outdated for this phase:**

- `run_particle_contact_prefix` as a solver boundary: retain reusable contact/callback logic but replace the partial-solver orchestration. [VERIFIED: architecture comparison]
- The Phase 9 prohibition on group/pair/triad/solver protocol members: preserve it for `phase9-v1`; add a new Phase 10 profile rather than weakening old schema validation. [VERIFIED: Phase 9 manifest and closed schema]
- Compatibility rows that leave particle groups/solver source areas `not_evidenced`: replace only after complete D1 promotion evidence exists. [VERIFIED: `reference/compatibility.json`; CONTEXT D-33]

## Planner-Oriented Dependency Decomposition

Use fine-grained plans with these dependency waves. Plans within a wave may run in parallel only when they own disjoint files and do not invent competing storage records. [VERIFIED: architectural dependency analysis]

1. **Wave 1 — closed contracts:** transcribe and test `phase10-pass-graph-v1`; create the Phase 10 compatibility leaf registry; expand checked system coefficients; define public group recipes/flags/views/errors; define full internal pair/triad/group record shapes. These contracts unblock all later implementation. [VERIFIED: current omissions and locked decisions]
2. **Wave 2 — storage foundation:** add storage-owned group table, aligned scratch lanes, group-range/cache validation, and operation-specific candidate primitives; extend permutation/compaction property models. Do not expose public mutations until candidate invariants are complete. [VERIFIED: D-05/D-09 dependencies]
3. **Wave 3 — group sampling/API:** implement fill-union, stroke-chain, explicit-position samplers; atomic creation/append; borrow-scoped views; destruction/can-be-empty behavior; lifecycle callbacks. [VERIFIED: pinned creation and group tests]
4. **Wave 4 — topology/mutations:** implement bounded Voronoi; pair/triad generation; ordinary rotation; join; split; reactive regeneration; solid-depth scheduling. Join/split depend on the full record/permutation work. [VERIFIED: pinned topology dependency graph]
5. **Wave 5 — early/material kernels:** contact/body refresh, weight, depth, reactive, force, viscous, repulsive, powder, tensile, solid, color mixing, gravity. Each gets focused kernel tests and pass admission tests. [VERIFIED: S01-S13]
6. **Wave 6 — pressure/constraint kernels:** static pressure, dynamic pressure, damping, extra damping, elastic, spring, and velocity limiting. These depend on topology and full coefficient/scratch lanes. [VERIFIED: S14-S20]
7. **Wave 7 — boundary/rigid kernels:** rigid statistics/damping, barrier, fixture collision with previous transforms, rigid transform advancement, wall, and final integration. These are the most world-coupled kernels and depend on body candidate plumbing. [VERIFIED: S21-S26]
8. **Wave 8 — orchestration replacement:** replace Phase 9 partial prefix with the full transaction, retain Phase 9 callback/lifecycle semantics, validate pause/empty/iteration behavior, and run the complete native regression suite. [VERIFIED: integration dependency]
9. **Wave 9 — protocol and differential corpus:** extend strict schema/decode/execute/native adapter/comparator; add control/activation and interaction witnesses; retain all Phase 6-9 assertions; add D0/replay/debug/release/sanitizer content validators. Protocol type work can begin earlier, but fixture sealing waits for stable semantics. [VERIFIED: Phase 9 harness design]
10. **Wave 10 — exact evidence and sign-off:** generate local D2 diagnostics, run canonical and sanitizer CI on the same commit, exact-ref validate artifact IDs/digests/provenance, update compatibility/inventory/docs from that authority, and run all final gates. [VERIFIED: Phase 9 promotion pattern; CONTEXT D-32/D-33]

Do not combine Wave 5-7 into one enormous “implement solvers” plan. Each named kernel needs an independently reviewable implementation/test/witness trail, but avoid one-file-per-pass fragmentation by grouping related kernels in deep modules. [VERIFIED: D-19/D-26; repository code-shape standard]

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
| --- | --- | --- | --- |
| — | No `[ASSUMED]` claims are used. All recommendations derive from locked context, current repository code/configuration, or the pinned upstream source/tests. | All | No user confirmation is required for a research assumption. |

## Open Questions (RESOLVED)

The planning uncertainty is resolved through mandatory evidence gates; no probe outcome is invented in this research document.

1. **Phase 10 numeric thresholds**
   - Resolution: Plan 10-27 creates the closed per-field policy registry without choosing unsupported thresholds. Plan 10-28 must collect bounded D2 native/oracle distributions over identical request bytes, combine those distributions with source scale, and lock each named exact-bit, ULP, absolute-relative, or dimensioned-absolute threshold before corpus sealing. Boundary and one-over-boundary mutation tests are mandatory, and a threshold may not be loosened to obtain a match. D1 later validates the already-locked policy rather than selecting it. [PLANNED: 10-27 Task 1; 10-28 Task 2]

2. **Split-created group metadata**
   - Resolution: Plan 10-12 Task 1 must build and run `phase10-group-topology-witness` against the pinned upstream revision before topology implementation. The provenance-bound artifact records later-component flags, strength, transform, user-data behavior, range/member order, and zero-valued statistics. Plan 10-14 must consume that exact result and cannot use defaults inferred from Rust or source inspection alone. [PLANNED: 10-12 Task 1; 10-14 Task 1]

3. **Finite-degenerate topology/solver behavior**
   - Resolution: The same mandatory Plan 10-12 witness must execute every listed case: zero-length pair, degenerate triad, barrier pair, no-necessary-generator Voronoi input, empty rigid group, and one-particle rigid group. Each case records exact inputs/outcomes and one explicit downstream decision, `preserve_source_behavior` or `typed_error`. Plans 10-12 and 10-20 must implement those classifications verbatim; Plan 10-21 consumes the rigid-group classifications. A missing, mismatched, or incomplete probe artifact blocks execution. No global epsilon is permitted. [PLANNED: 10-12 Task 1/2; 10-20 Task 1; 10-21 Task 1]

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
| --- | --- | --- | --- | --- |
| Rust/Cargo | Native implementation/tests | ✓ | 1.97.0 | None needed. [VERIFIED: local commands] |
| CMake | Private oracle configure | ✓ | 3.27.9 local | Existing preset floor supports local D2; canonical CI pin remains 4.3.3. [VERIFIED: local command; `.planning/research/STACK.md`] |
| Ninja | Private oracle builds | ✓ | 1.13.2 | None needed. [VERIFIED: local command] |
| Clang | Oracle compiler | ✓ | Apple Clang 21.0.0 | Local evidence is D2; pinned Linux Clang 22.1.8 CI is required for D1. [VERIFIED: local command; CONTEXT D-32] |
| Node | GSD/validation helper scripts | ✓ | 24.13.0 | None needed. [VERIFIED: local command] |
| `just` | Repository command facade | ✓ | 1.48.0 | Direct Cargo/CMake commands remain visible fallback; canonical stack recommends 1.55.1. [VERIFIED: local command; Justfile architecture] |
| `cargo llvm-cov` | Not required for Phase 10 closure | ✓ | 0.8.5 | Coverage policy is deferred to Phase 12. [VERIFIED: local command; CONTEXT Deferred Ideas] |
| `cargo deny` | Dependency/license checks | ✓ | 0.20.2 | None needed. [VERIFIED: local command] |
| `cargo nextest` | Optional test runner | ✗ | — | Use `cargo test`; nextest is not required by Phase 10 acceptance. [VERIFIED: local command absence; repo verification rules] |
| Canonical GitHub Actions runner | D1 promotion and sanitizer evidence | Remote only | Linux x86_64 Rust 1.97.0 / Clang 22.1.8 policy | Local D2 can diagnose but cannot promote. [VERIFIED: CONTEXT D-32; workflow design] |

**Missing dependencies with no fallback:** None for implementation and local validation. Canonical D1 promotion necessarily depends on the existing remote CI authority and cannot be replaced by this macOS host. [VERIFIED: environment audit and authority policy]

**Missing dependencies with fallback:** `cargo nextest` is absent; use the required `cargo test` commands. [VERIFIED: environment audit]

## Validation Architecture

This section is included because the Phase 10 research request explicitly requires it even though `.planning/config.json` currently has `workflow.nyquist_validation: false`. [VERIFIED: `.planning/config.json`; task scope]

### Test Framework

| Property | Value |
| --- | --- |
| Framework | Built-in Rust test harness on Rust 1.97.0 plus `proptest` 1.11.0; pinned C++ GoogleTest/CTest for upstream intent and oracle validation. [VERIFIED: manifests; upstream CMake/tests] |
| Config file | Workspace `Cargo.toml`, `rust-toolchain.toml`, `CMakePresets.json`, and existing oracle workflow. [VERIFIED: repository files] |
| Quick run command | `cargo test -p liquidfun --all-features <focused_test_target>` [VERIFIED: existing test-target layout] |
| Differential quick command | `cargo test -p liquidfun-differential --test phase10_corpus -- --nocapture` after Wave 9 creates it. [VERIFIED: Phase 9 pattern] |
| Full suite command | `cargo test --all-features` [VERIFIED: `AGENTS.md`] |
| Pre-commit sequence | `cargo fmt --all`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features` [VERIFIED: `AGENTS.md`] |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
| --- | --- | --- | --- | --- |
| PART-09 | Recipes, fixed sampling order, atomic creation/append, group view | Unit + integration | `cargo test -p liquidfun --all-features --test particle_groups` | ❌ Wave 0/3 |
| PART-10 | Join/split/empty/depth/rigid/membership/identity | Unit + integration + property | `cargo test -p liquidfun --all-features --test particle_group_mutation` | ❌ Wave 0/4 |
| PART-11 | Voronoi/pair/triad/reactive exact structure | Unit + property + differential | `cargo test -p liquidfun --all-features --test particle_topology` | ❌ Wave 0/4 |
| PART-12 | Baseline pass order/effects | Unit + integration + differential | `cargo test -p liquidfun --all-features --test particle_solver_baseline` | ❌ Wave 0/5-8 |
| PART-13 | Flag controls, activations, interactions | Unit + integration + differential | `cargo test -p liquidfun --all-features --test particle_solver_flags` | ❌ Wave 0/5-8 |
| PART-18 | Closed ledger, retained evidence, D0/D1 promotion | Harness + CI | `cargo test -p liquidfun-differential --test phase10_corpus` | ❌ Wave 0/9 |
| TEST-01 | One-concern pure kernel tests | Unit | `cargo test -p liquidfun --all-features particle::` | Existing framework; new modules needed |
| TEST-02 | Public group/world workflows | Integration | `cargo test -p liquidfun --all-features --test particle_groups --test particle_group_mutation --test particle_solver_baseline --test particle_solver_flags` | ❌ Wave 0 |
| TEST-04 | Group/permutation/topology/sequence invariants | Property | `cargo test -p liquidfun --all-features --test particle_group_properties` | ❌ Wave 0 |

### Required Focused Test Families

- **Recipes:** fill snap/boundary/union overlap/transform, stroke chain carry/custom stride, explicit order, rotational initial velocity, zero-source retained group, and every typed failure with before/after storage snapshots. [VERIFIED: pinned creation functions/tests; D-05]
- **Mutation:** join rotations and cross-only constraints; split first-longest ties, zombies, intermingled groups, ID preservation; can-be-empty; group destruction occurrence; depth invalidation; group-stat cache invalidation. [VERIFIED: pinned mutation helpers/tests]
- **Topology:** generator/queue/node order, tie retention, connection gates, distance bounds, pair/triad orientation/coefficients, stable first duplicate, rotation remap, split retarget, reactive clearing only after commit. [VERIFIED: pinned Voronoi/topology code]
- **Solver:** one control and activation per S01-S26 and every flag/group flag, plus powder+tensile pressure suppression, static pressure+extra damping, color both-particle gate, repulsion membership combinations, barrier+wall, solid/rigid interactions, moving fixtures, pause, zero dt, and multi-iteration traces. [VERIFIED: pass graph; CONTEXT D-20/D-21/D-28]
- **Inherited Phase 9:** lifecycle, compaction, buffers, contacts, filters/listeners, queries/rays, callbacks, statistics, replay, minimization, D0, debug/release agreement. [VERIFIED: Phase 9 required branches]
- **Upstream test intent:** map Function group tests, Color arithmetic, Confinement barrier/wall, Conservation parameter matrix, Callback group destruction, BodyContacts flags/listeners, and MultipleParticleSystems determinism to native or differential leaves. Testbed examples remain Phase 11. [VERIFIED: pinned `Box2D/Unittests`; CONTEXT Deferred Ideas]

### Numeric Policy Map

| Path family | Initial policy class | Horizon |
| --- | --- | --- |
| IDs, flags, membership, counts, order, pass trace, pair/triad endpoint identity, lifecycle occurrences | Exact structural | Every checkpoint. [VERIFIED: CONTEXT D-29] |
| Recipe input bits, configured coefficients, stored user-supplied values | Exact-bit transport | Creation/mutation checkpoint. [VERIFIED: existing exact float transport] |
| Generated position/depth/weight/rest-distance/triad coefficients | Closed named exact-bit or ULP path selected by canonical probe | Immediately after operation and one step. [VERIFIED: CONTEXT permits path-specific policy; threshold open] |
| Velocity/position/force/pressure/group aggregate state | Closed named ULP, absolute-relative, or dimensioned-absolute path | Fixed short horizons per witness. [VERIFIED: Phase 4 policy and D-29] |
| D0 same-build trace | Exact bytes, excluding declared nondeterministic timing | At least two identical runs. [VERIFIED: CONTEXT D-32] |

### Sampling Rate

- **Per task commit:** focused target(s) for the touched kernel/API plus `cargo fmt --all --check`. [VERIFIED: repo workflow pattern]
- **Per wave merge:** `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. [VERIFIED: `AGENTS.md`]
- **Topology/solver integration wave:** add debug and release oracle comparisons plus replay; do not wait until sign-off to discover schema/order divergence. [VERIFIED: Phase 9 evidence workflow]
- **Phase gate:** full native suite, closed corpus, deterministic D0, debug/release/replay, sanitizer, exact-ref/provenance/content validation, and compatibility generation all green on one current authority set. [VERIFIED: CONTEXT D-33]

### Wave 0 Gaps

- [ ] `particle/group.rs` unit tests or an equivalent private test module for recipe and view invariants. [VERIFIED: file absent]
- [ ] `particle/topology/` focused unit/property tests for connectivity, Voronoi, pairs, triads, and retargeting. [VERIFIED: modules absent]
- [ ] `particle/solver/` focused unit tests and a private pass-trace validator. [VERIFIED: modules absent]
- [ ] `tests/particle_groups.rs`, `particle_group_mutation.rs`, `particle_topology.rs`, `particle_solver_baseline.rs`, `particle_solver_flags.rs`, and `particle_group_properties.rs`, or a smaller cohesive equivalent set. [VERIFIED: targets absent]
- [ ] Phase 10 protocol schema fixtures, C++ decode/execute tests, comparator registry tests, corpus manifest, evidence validator CLI tests, and workflow dispatch. [VERIFIED: only Phase 9 equivalents exist]
- [ ] Focused pinned C++ probes for split metadata and finite degenerate cases identified under Open Questions. [VERIFIED: current upstream tests do not close those fields]

## Security Domain

Security enforcement is enabled because `.planning/config.json` does not explicitly set `security_enforcement: false`. [VERIFIED: config audit and research-role rules]

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
| --- | --- | --- |
| V2 Authentication | No | No authentication surface exists in the library or local oracle protocol. [VERIFIED: repository architecture] |
| V3 Session Management | No | The long-lived oracle process has a versioned request session, not an authenticated user session. [VERIFIED: protocol architecture] |
| V4 Access Control | No | Public safety relies on world/system-scoped typed IDs and lock state, not user authorization. [VERIFIED: Phase 3/9 object model] |
| V5 Validation, Sanitization, and Encoding | Yes | Checked public recipe/mutation types, finite/range/capacity validation, `serde(deny_unknown_fields)`, bounded arrays/actions, exact float-bit schema, and fail-closed C++ decode. [VERIFIED: current Phase 9 schema/decode; CONTEXT D-24] |
| V6 Stored Cryptography | No | No secrets or cryptographic storage are part of Phase 10; SHA-256 evidence digests provide integrity binding, not secret protection. [VERIFIED: evidence manifests and scope] |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
| --- | --- | --- |
| Huge finite geometry/stride creates excessive sampling or Voronoi work | Denial of Service | Checked source-derived axis/cell/product bounds and capacity preflight before allocation. [VERIFIED: identified source formulas; current harness limit pattern] |
| Non-finite values or degenerate denominators poison all solver lanes | Tampering / DoS | Validate public inputs, isolate operation candidates, reject invariant failure without commit, and add targeted source probes. [VERIFIED: CONTEXT D-24; transaction architecture] |
| Wrong-world/system group ID mutates unrelated state | Tampering | Arena generation check plus same-world/same-system ownership validation before candidate construction. [VERIFIED: existing typed identity pattern; D-02] |
| Unknown protocol fields/pass IDs/policies bypass comparison | Tampering | Closed enums/registries, `deny_unknown_fields`, schema bounds, and exact missing/duplicate/unknown validation. [VERIFIED: Phase 9 harness; D-16/D-31] |
| Oracle executable/path substitution | Spoofing / Tampering | Existing allowlisted preset resolution, canonical path containment, executable checks, pinned revision/build identity, and exact-ref provenance. [VERIFIED: `supervisor/executable.rs`; xtask provenance] |
| Partial artifacts are promoted as parity | Repudiation | Same-run identity, SHA-256 bindings, denied prior run IDs, complete authority-set validation, and generated compatibility rows. [VERIFIED: Phase 9 evidence validator; D-33] |
| Hash/parallel order makes evidence irreproducible | Repudiation | Ordered vectors, explicit traversal order, single-threaded baseline, D0 byte identity. [VERIFIED: D-23/D-32] |

## Sources

### Primary (HIGH confidence)

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.cpp` at pinned revision `7f204021...` — group creation/join/split, rotations, topology, depth, full solve graph, kernels, and lifecycle. [VERIFIED: local pinned submodule]
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Particle/b2ParticleSystem.h` — definitions, defaults, buffers, pair/triad records, gates, and private state. [VERIFIED: local pinned submodule]
- `b2ParticleGroup.h/.cpp`, `b2Particle.h/.cpp`, and `b2VoronoiDiagram.h/.cpp` — public/internal flags, group statistics, color/pair/triad semantics, and exact Voronoi algorithm. [VERIFIED: local pinned submodule]
- `third_party/liquidfun/liquidfun/Box2D/Unittests/` — group, flag, callback, confinement, conservation, color, body-contact, and multi-system test intent. [VERIFIED: local pinned submodule inventory]
- `.planning/phases/10-.../10-CONTEXT.md`, `.planning/REQUIREMENTS.md`, and Phase 3/4/9 contexts — locked product, identity, numeric, lifecycle, and evidence decisions. [VERIFIED: repository planning artifacts]
- Current `crates/liquidfun`, protocol, differential, reference, xtask, script, workflow, compatibility, and testing sources cited inline — present implementation and extension seams. [VERIFIED: repository source audit]

### Secondary (MEDIUM confidence)

- Pinned Programmer's Guide Chapter 11 — consumer descriptions of group creation, flags, deferred destruction, maximum velocity, and behavior intent; source code remains authoritative for edge order/equations. [VERIFIED: local pinned documentation]

### Tertiary (LOW confidence)

- None. No unverified web-search claim is used. [VERIFIED: research provenance log]

## Metadata

**Researched:** 2026-07-19

**Domain:** Native Rust particle-group state, deterministic topology, multipass particle solvers, and semantic differential evidence.

**Confidence:** HIGH for pass order, source algorithms, current architecture, and evidence seams; MEDIUM for final numeric thresholds and untested degenerate behavior pending the explicitly listed probes.

**Confidence breakdown:**

- Standard stack: HIGH — current manifests/toolchain and project constraints require no new dependency. [VERIFIED: local metadata]
- Architecture: HIGH — locked single-authority/transaction decisions align with current clone/commit seams and pinned operation boundaries. [VERIFIED: context and source audit]
- Group/topology semantics: HIGH — transcribed from pinned implementation and unit tests. [VERIFIED: pinned source/test inventory]
- Solver graph/kernels: HIGH — exact call graph and gates transcribed from pinned `Solve`; equations were audited by named kernel. [VERIFIED: pinned source]
- Numeric thresholds: MEDIUM — policy classes are locked, but actual per-path thresholds require Phase 10 canonical probes. [VERIFIED: current absence of corpus]
- Evidence/CI design: HIGH — direct extension of the already sealed Phase 9 pipeline. [VERIFIED: existing evidence implementation]
- Pitfalls/security: HIGH for identified code/order/state risks; MEDIUM for degenerate numeric outcomes pending focused oracle probes. [VERIFIED: source analysis and Open Questions]

**Valid until:** 2026-08-18 for repository architecture and tool availability; pinned upstream semantic findings remain valid while the oracle revision stays unchanged. [VERIFIED: pinned-revision policy; 30-day environment freshness convention]
