---
phase: 05-shapes-and-collision-foundation
verified: 2026-07-11T20:22:19Z
status: passed
score: "35/35 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T20:22:19Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - id: COLL-02
    status: verified
  - id: COLL-03
    status: verified
  - id: COLL-04
    status: verified
  - id: COLL-05
    status: partial_deferred
    deferred_to: "Phase 6 contact lifecycle"
  - id: COLL-06
    status: verified
  - id: COLL-07
    status: verified
must_haves:
  roadmap_success_criteria: 5/5
  plan_truths: 29/29
  plan_artifacts: 33/33
  plan_key_links: 16/16
  repository_completion_gate: 1/1
gaps: []
human_verification: []
deferred:
  - item: "World-owned contact creation, persistence, destruction, waking, joint suppression, listener timing, and warm-start state"
    target: "Phase 6: Minimal Rigid World Vertical Slice"
---

# Phase 5: Shapes and Collision Foundation Verification Report

**Phase Goal:** Implement and verify the complete shape and collision substrate required by rigid worlds and particle-body coupling.
**Verified:** 2026-07-11T20:22:19Z
**Status:** passed

## Goal Achievement

### Roadmap Success Criteria

| # | Observable truth | Status | Fresh evidence |
| ---: | --- | --- | --- |
| 1 | Consumers can define, validate, clone, measure, point-test, bound, and ray-cast circle, edge, polygon, and chain shapes with upstream-equivalent accepted-input results. | VERIFIED | The current public shape suite passed 21/21 tests, production validation boundaries passed, compile-fail/private-field contracts passed, and the 78-case Rust/C++ corpus includes accepted and typed-rejected witnesses for all four shapes. |
| 2 | Overlap, distance, clipping, manifolds, and every supported shape-pair collision produce upstream-equivalent semantic observables. | VERIFIED | Distance passed 12 public tests plus focused simplex/cache tests; manifolds passed 15/15 public tests plus private clipping/tie tests; debug and release oracle comparison matched all 78 ordered cases. |
| 3 | The dynamic AABB tree and broad phase support proxy lifecycle, movement, queries, ray casts, metrics, pair generation, filtering/refiltering, and deterministic solver-relevant ties. | VERIFIED | The broad-phase suite passed 24/24 tests, including scoped identity, exact equality branches, lifecycle reuse, visitors, ordered pair deduplication, filter groups/masks, refilter touch, and seeded invariants; the fixed corpus exercises all planned tree/broad-phase families. |
| 4 | Time-of-impact kernels handle supported sweeps and edge cases within the approved numerical policy. | VERIFIED | The public TOI suite passed 17/17 tests and private tests passed exact target/tolerance, 20/50/polygon caps, separation kinds, support ties, root alternation, bounded diagnostics, and closed failure state; oracle comparison includes ten TOI families. |
| 5 | Focused unit/property tests and pure differential probes protect all collision foundations before world-level solvers consume them. | VERIFIED | Mandatory full-workspace gates passed; protocol validation contains 78 required families and rejects every deletion; debug/release compare, replay, and two-run byte-identical D0 all passed. |

**Roadmap score:** 5/5

### Detailed Plan Truths

| Plan | Truths | Status | Evidence summary |
| --- | ---: | --- | --- |
| 05-01 collision domain | 3/3 | VERIFIED | One curated collision namespace, typed finite-domain errors, initialized AABB/ray/mass/child/manifold values, and semantic feature identity compile and pass 18 public contracts. |
| 05-02 shapes | 3/3 | VERIFIED | All four immutable owned shapes, source-ordered accepted polygon behavior, explicit safe rejection, canonical closed-chain storage, and checked adjacency pass focused and property tests. |
| 05-03 distance | 3/3 | VERIFIED | One bounded GJK path, topology-safe initialized cache, strict support/flush behavior, and strict overlap threshold pass public, internal, regression, and property tests. |
| 05-04 narrow phase | 3/3 | VERIFIED | Private ordered clipping, seven closed pair families and reversals, semantic point identity, and distinct unsupported/separated/touching outcomes pass exact branch tests and oracle evidence. |
| 05-05 spatial/broad phase | 4/4 | VERIFIED | Opaque scoped proxy identity, source-faithful private ties, safe visitors, ordered pairs, set-like query/ray collections, and pure refilter reconsideration pass all focused tests. |
| 05-06 TOI | 3/3 | VERIFIED | Checked immutable inputs, fixed source formulas/caps/root order, closed outputs, and bounded private diagnostics pass all focused evidence. |
| 05-07 differential evidence | 6/6 | VERIFIED | One existing supervised subprocess path covers exact-bit policy, typed failure classes, 78 fail-closed families, rejected shapes, and Used/Reset/Rejected semantic cache replay. |
| 05-08 sign-off | 4/4 | VERIFIED | Docs, ledger, package, inventory, provenance, full Rust gates, local CTest, compare/replay/D0, and maturity-boundary contracts all pass. |

**Plan truths:** 29/29 verified

## Required Artifacts

All 33 unique artifacts declared in PLAN frontmatter exist and are substantive. The generic `gsd-tools verify artifacts` helper cannot interpret these plans' plain-string artifact entries, so existence/substance and Level-3 wiring were checked directly and through consumer paths.

| Artifact cluster | Status | Wiring evidence |
| --- | --- | --- |
| Collision root, shared types, errors, public contracts | VERIFIED | `liquidfun::collision` explicitly re-exports concrete public types/functions; child modules consume `crate::math` and do not depend on world ownership or private harness crates. |
| Shape modules and shape tests | VERIFIED | `Shape` dispatch reaches concrete circle/edge/polygon/chain unary kernels; shape-child topology is consumed by distance, manifold, and TOI proxy construction. |
| Distance proxy/simplex/cache and tests | VERIFIED | Checked shape children feed one private proxy/simplex path; semantic cache diagnostics are available only through the feature-gated owned adapter. |
| Narrow-phase modules and tests | VERIFIED | Initialized `Manifold`/`ContactFeatureId` values carry all seven primary pair families and reversals; no packed key or solver impulse surface exists. |
| Dynamic tree, broad phase, and tests | VERIFIED | `BroadPhase<T>` owns `DynamicTree<BroadProxy<T>>`, sorts/deduplicates by private keys, emits opaque identities, and keeps filtering a pure decision. |
| TOI modules and tests | VERIFIED | Checked shapes/children and Phase 4 `Sweep` values feed the source-ordered TOI kernel; only doc-hidden owned diagnostics reach the evidence adapter. |
| Protocol, Rust executor/comparator, C++ adapter, policy, and fixed fixture | VERIFIED | All 78 closed cases execute through the existing supervisor and single `liquidfun-reference` subprocess; the C++ adapter calls pinned shape, distance, manifold, tree/broad-phase, and TOI APIs. |
| Architecture/testing/compatibility sign-off | VERIFIED | Human documentation is machine-checked against the authoritative 177-row ledger and exact registered command shapes. |

**Artifacts:** 33/33 verified

## Key Link Verification

| Link cluster | Status | Details |
| --- | --- | --- |
| Collision domain to Phase 4 math/settings | WIRED | Production collision modules directly import source-ordered `crate::math` types/settings. |
| Shared semantic values to kernels | WIRED | Shape, narrow, distance, tree, broad-phase, and TOI results use initialized typed values; packed-key/inactive-field scans are clean. |
| Shape topology to distance/manifold/TOI | WIRED | All three construct checked shape-child views rather than accepting raw storage. |
| Pair orientation to future contacts | WIRED | `collide_shapes` has a closed seven-family match, explicit `PairOrientation`, and typed unsupported/separated/touching results. |
| Tree/broad phase to Phase 6 seam | WIRED | Exact ordered candidate pairs and pure `FilterData` decisions are exposed without fixture/contact-manager ownership. |
| Private diagnostics to differential tooling | WIRED | Only unpublished `liquidfun-differential` enables `differential-internals`; no-default compile-fail docs and package checks prove default exclusion. |
| Protocol to Rust/C++ executors | WIRED | The same validated 78-case request drives native execution and one pinned C++ process, with declaration-first comparison and supervised failure taxonomy. |
| Machine ledger to public claims | WIRED | `cargo xtask docs check` and `inventory check` pass; contact-lifecycle rows remain unpromoted. |

All 16 plan key links are accounted for by these eight paired clusters.

**Wiring:** 16/16 verified

## Requirements Coverage

| Requirement | Status | Acceptance evidence |
| --- | --- | --- |
| COLL-02 | SATISFIED | Four owned shape types, exhaustive dispatch, validation/cloning/mass/point/AABB/ray behavior, focused/property tests, and cross-language shape witnesses pass. |
| COLL-03 | SATISFIED | GJK/overlap/cache, clipping, semantic manifolds, world conversion, all seven pair families and reversals, branch tests, and differential evidence pass. |
| COLL-04 | SATISFIED | Dynamic-tree creation/movement/removal/query/ray/metrics plus source tie behavior and scoped identity are implemented and tested. |
| COLL-05 | PARTIAL — DEFERRED | Phase 5's roadmap-owned portion—ordered broad-phase pair generation and pure filtering/refilter reconsideration—is implemented and evidenced. The aggregate requirement also names contact creation and persistence; those remain intentionally pending Phase 6 and are not claimed complete. |
| COLL-06 | SATISFIED | Supported shape-child sweeps, checked input, source-ordered separation/root logic, fixed caps, closed states, and documented `phase5-v1` policy are evidenced. |
| COLL-07 | SATISFIED | Focused unit/property tests plus pure native/C++ collision probes run before any world solver depends on the substrate. |

**Coverage:** 5 requirements satisfied; 1 aggregate requirement truthfully partial with its remaining lifecycle scope assigned to Phase 6.

## Test Quality Audit

| Evidence surface | Active evidence | Disabled | Circular | Strongest assertion | Verdict |
| --- | ---: | ---: | --- | --- | --- |
| Public collision contracts/shapes/distance/manifolds/tree/TOI | 107 integration tests | 0 | No | Value and behavioral | STRONG |
| Feature-gated diagnostic/cache replay | 14 tests | 0 | No | Exact precedence/value | STRONG |
| Collision protocol/schema/policy/fixed corpus | 8 focused collision tests within 72 protocol unit tests, plus 9 fixture tests | 0 | No | Exact decoding, deletion, byte stability | STRONG |
| Native comparison/supervisor/source identity | 10 integration tests | 0 | No | First-divergence and behavioral | STRONG |
| Xtask/docs/inventory/provenance/package | 16 CLI and 18 docs-contract tests plus focused package/provenance suites | 0 | No | Behavioral failure/success paths | STRONG |
| Pinned external oracle | Debug/release CTest, 78-case comparisons, replay, two-run D0 | 0 | No | Independent external semantic equality | STRONG |

The requirement-linked Rust tests use focused Arrange/Act/Assert structure. Searches found no `#[ignore]`, skip/todo test attributes, requirement-only disabled tests, fixture writer that derives accepted values from the Rust system under test, or circular expected-value generator. Oracle provenance is valid: the external adapter invokes the pinned C++ sources and all 78 engine results are compared through explicit field policies.

## Fresh Automated Verification

| Command or surface | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS |
| `cargo build --workspace --all-targets --all-features` | PASS |
| `cargo test --workspace --all-features` | PASS; all workspace unit, integration, property, CLI, and doctest targets green |
| `cargo xtask docs check` | PASS; 12 layers, four Phase 4 and four Phase 5 contracts |
| `cargo xtask inventory check` | PASS; 177 rows |
| `cargo xtask provenance check` | PASS; oracle `7f20402173fd143a3988c921bc384459c6a858f2`, one reviewed artifact |
| `cargo xtask package verify` | PASS; 51 package entries built/tested outside the repository |
| No-default check and doctests | PASS; 11 no-default doctests, including hidden-diagnostic compile-fail |
| Warning-denied workspace rustdoc | PASS |
| Debug/release configure, build, CTest | PASS; one strict reference protocol test per preset |
| Debug collision comparison | PASS; 78 ordered cases |
| Release collision comparison | PASS; 78 ordered cases |
| Debug replay | PASS; 78 ordered cases |
| Debug D0 determinism | PASS; two byte-identical runs |
| Lifecycle validation before report | PASS; context, eight plans, and eight summaries share lifecycle `5-2026-07-11T14-53-25` in yolo mode |
| `git diff --check` | PASS |

## Anti-Patterns and Standards Audit

- No TODO/FIXME/placeholder, empty-return, log-only, disabled-test, unsafe, packed-key, solver-impulse, raw-node/proxy/simplex constructor, mutable cache replay, or world contact-lifecycle implementation blocker was found in the Phase 5 surface.
- The source files above the 628-line review trigger (`distance.rs`, `tree.rs`, and `toi.rs`) were explicitly reviewed in their summaries. Proxy/simplex/replay, pool/traversal, and separation responsibilities are already split into cohesive child modules; the remaining entrypoints retain source-ordered orchestration and private branch tests. This is a documented readability tradeoff, not a goal blocker.
- The verification applied `AGENTS.md`, `AGENTS.bright-builds.md`, the placeholder-only `standards-overrides.md`, and the local architecture, code-shape, testing, verification, and Rust standards. No active substantive override applies.

## Deferred Items

| Deferred item | Later phase | Why it is not a Phase 5 gap |
| --- | --- | --- |
| Body/fixture-owned contact creation, persistence, update/destruction, waking, joint suppression, listener timing, material mixing, sensor semantics, and warm-start state | Phase 6: Minimal Rigid World Vertical Slice | The Phase 5 context, D-17, roadmap success criteria, architecture docs, compatibility ledger, and Phase 6 goal all place this lifecycle above the collision substrate. Phase 5 supplies and proves only ordered candidates, pure filter/refilter decisions, manifolds, and TOI kernels. |

## Human Verification Required

None. The phase consists of deterministic library behavior, type/API boundaries, machine-checked documentation, and process-isolated semantic comparison; no subjective visual, interactive, or external-service behavior remains.

## Residual Risks

- Local oracle evidence used CMake 3.27.9 and Apple Clang 21.0.0 rather than the canonical CMake 4.3.3 / Clang 22.1.8 Linux lane. The ledger correctly keeps `platform_validated` at zero and treats these passing runs as supported local evidence, not D1/platform promotion.
- A natural public TOI `Failed` geometry was not found in the bounded witness search. Exact cap boundaries, termination labels, and the closed `Failed` result are nevertheless tested at private decision seams; this limitation is documented and does not weaken the bounded public contract.
- The full `COLL-05` checkbox must remain pending until Phase 6 proves world-owned contacts. Marking it complete now would be an overclaim.

## Gaps Summary

No Phase 5 gaps found. All five roadmap success criteria, 29 plan truths, 33 declared artifacts, 16 key links, and the repository completion gate are verified. The contact-lifecycle remainder of aggregate requirement `COLL-05` is a specific Phase 6 deferral and remains unclaimed.

## Final Assessment

Phase 5 achieves its goal. Rigid-world work can now depend on immutable validated shapes, source-ordered distance and manifold kernels, opaque dynamic-tree identity, exact broad-phase candidates, pure filter/refilter decisions, checked TOI, and a fail-closed 78-family differential evidence path without inheriting C++ runtime coupling or premature contact-lifecycle claims.

*Verified: 2026-07-11T20:22:19Z*
*Verifier: gsd-verifier*
