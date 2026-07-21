# Phase 11: Examples, Headless Tooling, and Testbed - Research

**Researched:** 2026-07-21
**Domain:** Upstream corpus closure, deterministic headless scenarios, semantic diagnostics, and optional visualization
**Confidence:** HIGH for architecture and existing seams; MEDIUM for the renderer capability gate

<user-constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Upstream corpus accounting

- **D-01:** Add one dedicated machine-authoritative upstream-corpus manifest for semantic test cases, example scenarios, and registered testbed entries. Join it fail-closed to `reference/discovery.json`, `reference/compatibility.json`, scenario-catalog identities, and evidence references; generated Markdown remains a projection rather than a second authority.
- **D-02:** Discover semantic items rather than counting source files alone. The oracle-enabled refresh path must enumerate actual GoogleTest declarations or authoritative `--gtest_list_tests` output and the pinned testbed registration table, while Cargo-only checks validate the checked-in snapshot without requiring the submodule or C++ toolchain.
- **D-03:** Give every corpus item a stable source-derived identity, source path and symbol or registration identity, upstream revision, applicability, disposition, compatibility impact, rationale, and evidence mappings. Model disposition and impact as separate closed enums so a ported or equivalently covered item may still record behavioral, API, tooling, visual-only, or no compatibility impact.
- **D-04:** Terminal dispositions are explicit: native scenario/test port, equivalent existing evidence, reviewed irrelevance, documented difference, or intentional non-support as justified by the final schema. Every non-port disposition requires a specific reviewed rationale and compatibility impact; vague, empty, self-referential, stale, or missing evidence is rejected.
- **D-05:** Corpus closure rejects unknown, duplicate, unregistered, unmapped, or stale items and validates every referenced scenario, test, fixture, ledger leaf, review record, and source identity. Classification changes must remain reviewable in ordinary repository history; generated reports summarize exact totals and unresolved rows without silently updating authority.

#### Shared scenario catalog and headless controls

- **D-06:** Author scenarios as typed private catalog definitions that resolve into an immutable, bounded, engine-neutral `ResolvedScenario`-style plan before execution. The exact resolved plan, not mutable plugin state or renderer callbacks, is the common currency for Rust, C++ oracle, regressions, benchmarks, and visualization.
- **D-07:** Separate stable catalog slugs from display titles. Identify a generated run with catalog schema version, scenario version, generator identity and version, seed when present, exact run settings, and a content hash of canonical resolved bytes. Persist resolved bytes for regressions and failures; a seed alone is never sufficient replay evidence.
- **D-08:** Represent setup and interactive behavior as closed typed actions with stable semantic entity and action identities, exact `f32` transport, explicit order, and reviewed bounds. Backend-specific simulation logic, hidden mutable scenario state, frame callbacks, and duplicate Rust/C++ scenario implementations are prohibited.
- **D-09:** Put control state in a renderer-neutral run-session controller outside physics state. Pause performs no logical tick and emits no fabricated checkpoint; single-step executes exactly one logical tick and remains paused; restart destroys the current session and reconstructs step zero from the same resolved bytes and settings. A particle system's upstream pause flag remains a distinct typed scenario action.
- **D-10:** Bind deterministic checkpoints to explicit action or logical-step ordinals and stable checkpoint IDs, never render frames, refresh rate, wall time, or UI event timing. The controller owns selection, settings validation, action application, checkpoint capture, and restart semantics; frontends only submit commands and observe results.
- **D-11:** Benchmarks construct or restart from the same resolved plan outside the measured interval and declare the exact measured horizon. Regression fixtures, differential requests, and testbed captures reuse the canonical plan and checkpoint model rather than translating through separate formats.

#### Renderer-neutral observability and comparison capture

- **D-12:** Build layered public semantic views for current counts, tree metrics, contacts, particle contacts, broad-phase observations, particle statistics, and renderer-neutral geometry, then use one bounded owned canonical checkpoint builder as the authoritative deterministic capture. Reuse and extend existing `WorldDiagnostics`, owned `StepReport` evidence, and borrow-scoped particle/contact views instead of exposing arenas, dense rows, tree nodes, or raw proxy storage.
- **D-13:** Define debug drawing as a closed renderer-neutral primitive vocabulary carrying stable semantic owner and primitive keys, explicit geometry, color/style metadata, and named layer/category. Consumers receive owned records or a narrow sink adapter derived from the same collected semantic model; private traversal order and internal indices are never public identity or comparison keys.
- **D-14:** Canonical checkpoints preserve source-significant order and explicitly canonicalize only declared unordered primitive or observation sets using stable keys and deterministic tie-breakers. Structural fields, identities, kinds, flags, counts, membership, presence, and ordering compare exactly; numeric geometry uses only closed named Phase 4 policies.
- **D-15:** Treat wall-clock phase profiles as a separate diagnostic channel. Profile names and presence may compare structurally, but measured durations are excluded from D0/D1 physics parity and deterministic checkpoints; the testbed may display Rust and oracle timings side by side without claiming numeric timing equality.
- **D-16:** Produce one renderer-neutral comparison model keyed by stable semantic paths. It records exact matches, policy-qualified numeric differences, Rust-only and oracle-only observations, and bounded diagnostic context. Visual diff overlays and mismatch lists consume this model and may not re-read or reinterpret private engine state.

#### Optional visual testbed

- **D-17:** Finish and verify the headless catalog, controller, capture, and comparison capability before choosing a rendering dependency. Then run a private Macroquad 0.4.15 capability spike; retain it only if it proves readable contacts, particles, broad-phase overlays, profiles, side-by-side or overlay diffs, deterministic state capture, screenshots, controls, and supported desktop behavior.
- **D-18:** Use private `winit`/`wgpu`/`egui` integration only when the Macroquad spike records a concrete failure in required UI density, capture fidelity, accessibility, GPU inspection, render-target control, or platform support. Bevy and any renderer-owned simulation schedule remain out of scope.
- **D-19:** Keep the visual testbed in an unpublished, non-default workspace package. Its adapter translates input into controller commands and semantic snapshots into pixels; it owns windowing, frame pacing, camera, UI, GPU resources, and screenshot output, but no physics rules, oracle truth, scenario definitions, world storage, or checkpoint semantics.
- **D-20:** Provide scenario selection, run identity and seed display, pause, one-step, restart, validated timestep and iteration controls, overlay toggles, contacts, particle contacts, broad-phase data, phase profiles, Rust/oracle side-by-side or overlay comparison, mismatch focus, and deterministic semantic capture. Pixel screenshots are diagnostic artifacts, not compatibility authority.
- **D-21:** Default the new testbed UI to a practical dark theme with accessible contrast and distinct state colors. Expose repository source, license truth, Peter Ryszkiewicz/OpenLinks attribution where the UI can carry it without crowding, and visible version, short commit, and build provenance with `Unavailable` for missing values.
- **D-22:** Preserve Cargo-only and packaging isolation: `liquidfun` remains the sole default publishable crate and receives no renderer, windowing, game-engine, C++, protocol, or testbed dependency or default feature. Package checks must prove the published crate builds and tests without the testbed, native oracle source, or graphical environment.

#### Testing, evidence, and phase sign-off

- **D-23:** Extend the existing protocol, native adapter, C++ oracle, comparator, replay, failure-bundle, inventory, and evidence pipelines rather than creating a parallel example/testbed harness. Unknown scenario kinds, actions, observations, debug primitives, corpus dispositions, or policy paths are harness failures.
- **D-24:** Unit-test pure catalog resolution, controller transitions, primitive generation, checkpoint canonicalization, corpus joins, and diff construction one concern at a time. Add public integration tests for headless selection, seed resolution, pause/step/restart, settings, capture, package isolation, and representative rigid, joint, rope, particle, group, query, callback, and mutation scenarios.
- **D-25:** Give every upstream corpus item a terminal reviewed outcome and every native scenario a closed mapping to its tests, oracle or equivalent evidence, regression use, benchmark eligibility, and visualization eligibility. Phase completion requires zero unexplained corpus rows and no maturity claim beyond the exact reviewed evidence.
- **D-26:** Retain D0-D3 authority, strict provenance, replay, sanitizer, exact-reference, and same-run promotion rules from earlier phases. Deterministic semantic checkpoints may support parity; UI pixels, frame rate, and wall-clock profiles remain diagnostic only. Phase 12 still owns broad performance, portability, packaging, and release-readiness claims.

### the agent's Discretion

- Exact public and private type, module, method, command, manifest, review-record, primitive, observation, profile, and error names within the locked boundaries.
- Exact plan decomposition, bounded capacities, catalog composition helpers, property-case counts, and representative scenario grouping, provided closure remains item-level and fail-closed.
- Exact primitive transport shape and whether the rendering adapter consumes an owned collection or narrow sink, provided both derive from the same semantic model and deterministic checkpoints remain authoritative.
- Exact Macroquad spike acceptance measurements, visual layout, camera gestures, keyboard shortcuts, and overlay styling beyond the required controls, dark default, accessibility, provenance, and source disclosure.

### Deferred Ideas (OUT OF SCOPE)

- Broad benchmark budgets, profiling-led optimization, platform matrix sign-off, coverage expansion, packaging, and release readiness — Phase 12.
- Renderer-specific performance work, advanced GPU inspection, plugin scripting, and alternate testbed frontends beyond the Phase 11 capability gate — future evidence-driven work.
- Pixel-perfect screenshot regression as compatibility authority — intentionally excluded; semantic checkpoints remain authoritative.
</user-constraints>

<phase-requirements>
## Phase Requirements

| ID | Description | Research Support |
| --- | --- | --- |
| RIGD-10 | Public counts, metrics, profiles, and renderer-independent debug drawing | Extend `WorldDiagnostics` and bounded owned semantic records; keep profiles outside parity checkpoints. [VERIFIED: `crates/liquidfun/src/world/diagnostics.rs`; `11-CONTEXT.md` D-12-D-15] |
| TEST-03 | Terminal reviewed outcome for every applicable upstream test | Add semantic corpus authority beside the existing discovery/compatibility validation pipeline. [VERIFIED: `tools/xtask/src/inventory.rs`; `11-CONTEXT.md` D-01-D-05] |
| EXMP-01 | Terminal reviewed outcome for every example/testbed entry | Discover registered semantic items and enforce fail-closed joins. [VERIFIED: `11-CONTEXT.md` D-01-D-05] |
| EXMP-02 | Named/seeded headless controls and deterministic checkpoints | Implement immutable resolution plus a renderer-neutral session controller. [VERIFIED: `11-CONTEXT.md` D-06-D-10] |
| EXMP-03 | One scenario definition for all execution consumers | Make canonical resolved bytes the only backend input. [VERIFIED: `11-CONTEXT.md` D-06-D-11, D-23] |
| EXMP-04 | Optional interactive controls and diagnostics | Add the visual adapter only after headless verification and the Macroquad gate. [VERIFIED: `11-CONTEXT.md` D-17-D-20] |
| EXMP-05 | Visual Rust/oracle differences without private-state access | Feed UI overlays from one semantic comparison model. [VERIFIED: `11-CONTEXT.md` D-12-D-16] |
| EXMP-06 | Published crates remain headless and renderer-free | Preserve `liquidfun` as the sole default member; place the testbed in an unpublished non-default member. [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-22] |
</phase-requirements>

## Summary

Phase 11 is primarily an integration-and-closure phase, not a new simulation subsystem. The repository already has a strict inventory command with closed enums, schema validation, pinned-revision checks, deterministic generated-file checks, and Cargo-only validation; it should be extended with a semantic corpus manifest rather than replaced. [VERIFIED: `tools/xtask/src/inventory.rs`; `11-CONTEXT.md` D-01-D-05, D-23]

The engine already exposes bounded, owned semantic reconstruction plus exact counts and dynamic-tree metrics without exposing storage coordinates. The next public layer should add contacts, particle contacts, broad-phase observations, profiles, and stable debug primitives, then compose those views into one canonical checkpoint and one comparison model. [VERIFIED: `crates/liquidfun/src/world/diagnostics.rs`; `11-CONTEXT.md` D-12-D-16]

**Primary recommendation:** Plan corpus authority, scenario resolution/controller, semantic capture/comparison, and headless consumers as dependency-ordered waves; treat the renderer as a final private capability gate, never as a foundation. [VERIFIED: `11-CONTEXT.md` D-17-D-23; `.planning/research/STACK.md` Rendering and Testbed Recommendation]

## Project Constraints (from AGENTS.md)

- Keep production physics native Rust, renderer-independent, deterministic, safe by default, and Cargo-sufficient for ordinary users. [VERIFIED: `AGENTS.md` supplied with task]
- Preserve the single publishable `liquidfun` default member; developer tooling stays private. [VERIFIED: `AGENTS.md` supplied with task; `Cargo.toml`]
- Use typed errors, no `unwrap()`, public API documentation, tau-based full rotations, and Arrange/Act/Assert tests with one concern per test. [VERIFIED: `AGENTS.md` supplied with task]
- Before any Rust commit, run format, Clippy with denied warnings, all-target/all-feature build, then all-feature tests. [VERIFIED: `AGENTS.md` supplied with task]
- Do not mdformat `.planning/**`; GSD owns this Markdown. [VERIFIED: `AGENTS.md` supplied with task]

## Standard Stack

### Existing core

| Component | Version/scope | Phase use | Confidence |
| --- | --- | --- | --- |
| Rust workspace | Edition 2024, resolver 3, MSRV 1.92 | Native library and private tools | HIGH [VERIFIED: `Cargo.toml`] |
| `serde` / `serde_json` | 1.0.228 / 1.0.150, workspace-private data paths | Strict manifest, scenario, checkpoint, and comparison schemas | HIGH [VERIFIED: `Cargo.toml`; `.planning/research/STACK.md`] |
| `sha2` | 0.10.9 | Canonical resolved-byte content identity | HIGH [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-07] |
| `thiserror` | 2.0.18 | Typed library/boundary errors | HIGH [VERIFIED: `Cargo.toml`; `.planning/research/STACK.md`] |
| Built-in Rust test harness + `proptest` | `proptest` 1.11.0 | Focused transition/canonicalization tests and bounded generated scenarios | HIGH [VERIFIED: `Cargo.toml`; `.planning/research/STACK.md`] |

### Renderer gate

| Choice | Rule | Confidence |
| --- | --- | --- |
| Macroquad 0.4.15 | Spike only after headless completion; retain only on explicit acceptance evidence | MEDIUM [CITED: `.planning/research/STACK.md` Rendering and Testbed Recommendation] |
| `wgpu` 30 + `winit` 0.30.13 + `egui` 0.35.0 | Use only after recording a concrete Macroquad capability failure | MEDIUM [CITED: `.planning/research/STACK.md` Rendering and Testbed Recommendation] |
| Bevy | Do not use for the canonical testbed | HIGH [CITED: `.planning/research/STACK.md` What NOT to Use] |

No new dependency belongs in `crates/liquidfun`; any renderer dependency belongs only in an unpublished, non-default testbed package. [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-19, D-22]

## Architecture Patterns

### Recommended dependency direction

```text
semantic corpus manifest ──> strict inventory joins ──> generated report
typed catalog ──> canonical ResolvedScenario bytes ──> run-session controller
                                                    ├─> native adapter
                                                    ├─> C++ oracle adapter
                                                    ├─> regressions/benchmarks
                                                    └─> visual adapter (last)
public semantic views ──> canonical checkpoint ──> comparison model ──> headless/UI presentation
```

This preserves one authority at each layer: machine manifest for corpus status, resolved bytes for execution, canonical checkpoints for parity, and the comparison model for presentation. [VERIFIED: `11-CONTEXT.md` D-01, D-06-D-16]

### Existing seams to deepen

1. **Inventory seam:** `inventory.rs` already separates `discover`, `generate`, `check`, and `check-report`; extend its typed ledgers and validation modules so refresh is explicit while Cargo-only checks remain read-only and fail on stale generated bytes. [VERIFIED: `tools/xtask/src/inventory.rs`]
1. **Diagnostic seam:** `WorldDiagnostics` already carries exact counts and tree metrics, and `WorldReconstruction` already demonstrates bounded owned capture, output-local coordinates, stable ordering, and typed capacity/invalid-state errors. Follow those patterns for new observations and primitives. [VERIFIED: `crates/liquidfun/src/world/diagnostics.rs`]
1. **Isolation seam:** the root workspace defaults only to `crates/liquidfun`; add any scenario/testbed tooling as non-default and keep renderer crates out of production dependencies. [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-22]

### Prescriptive module boundaries

- Put public renderer-neutral diagnostic types beside the engine world APIs; they may expose semantic identities and owned/borrowed records, never arena slots or tree nodes. [VERIFIED: `11-CONTEXT.md` D-12-D-14]
- Put private catalog definitions, canonical encoding, session state transitions, and checkpoint scheduling in test protocol/differential tooling, not in renderer callbacks. [VERIFIED: `11-CONTEXT.md` D-06-D-11, D-23]
- Make frontends command/query adapters. UI frame rate and input timing must not advance logical state implicitly. [VERIFIED: `11-CONTEXT.md` D-09-D-10, D-19]

## Don't Hand-Roll

| Problem | Do not build | Use instead | Why |
| --- | --- | --- | --- |
| Corpus reporting | A second Markdown authority or loose file counts | Typed semantic manifest joined through xtask validation | Existing generated/report checks are deterministic and fail-closed. [VERIFIED: `tools/xtask/src/inventory.rs`; `11-CONTEXT.md` D-01-D-05] |
| Replay identity | Seed-only replay or renderer state | Versioned canonical resolved bytes plus hash and provenance | Seeds do not capture generator/schema evolution. [VERIFIED: `11-CONTEXT.md` D-07] |
| Backend scenarios | Separate Rust/C++/testbed implementations | One closed action plan consumed by adapters | Duplicate logic destroys comparable semantics. [VERIFIED: `11-CONTEXT.md` D-08, D-23] |
| Debug rendering truth | Private-world traversal in UI | Stable renderer-neutral primitives from semantic views | Storage order and indices are not public identity. [VERIFIED: `11-CONTEXT.md` D-12-D-16] |
| Visual diff logic | Renderer-specific mismatch interpretation | Shared semantic comparison model | Headless and UI results must agree. [VERIFIED: `11-CONTEXT.md` D-16] |
| Physics scheduling | Game-engine or render-loop ownership | Explicit renderer-neutral controller | Pause, step, restart, and checkpoints are logical-state contracts. [VERIFIED: `11-CONTEXT.md` D-09-D-10, D-18-D-19] |

## Common Pitfalls

### File-level inventory mistaken for semantic closure

**Failure:** one source file can contain multiple tests or registered scenarios, while stale/unknown semantic entries can pass a file-count audit. **Prevention:** enumerate test declarations and registrations, require closed dispositions, and make every cross-ledger reference resolve. [VERIFIED: `11-CONTEXT.md` D-01-D-05]

### Three clocks accidentally coupled

**Failure:** render frames, logical ticks, and the particle-system pause action become conflated. **Prevention:** keep controller pause outside physics, bind checkpoints to logical/action ordinals, and model particle pause as an explicit scenario action. [VERIFIED: `11-CONTEXT.md` D-09-D-10]

### Unstable identity leaks

**Failure:** arena slots, dense indices, tree traversal order, or primitive visitation order become comparison keys. **Prevention:** use stable semantic owner/action/primitive IDs; output-local indices may describe only one owned record graph. [VERIFIED: `crates/liquidfun/src/world/diagnostics.rs`; `11-CONTEXT.md` D-08, D-13-D-14]

### Diagnostic timing promoted to parity

**Failure:** wall-clock phase durations create nondeterministic mismatches. **Prevention:** compare profile names/presence structurally but keep durations and frame rate outside D0/D1 checkpoints. [VERIFIED: `11-CONTEXT.md` D-15, D-26]

### Renderer chosen too early

**Failure:** a UI framework dictates scheduling, data ownership, or production dependencies. **Prevention:** complete headless capability first, record the Macroquad spike result, and permit the heavier fallback only for a named failure. [VERIFIED: `11-CONTEXT.md` D-17-D-19; `.planning/research/STACK.md` Rendering and Testbed Recommendation]

### Bounds and unknown variants treated permissively

**Failure:** malformed or future data is truncated, ignored, or rendered approximately. **Prevention:** copy the existing `deny_unknown_fields`, closed-enum, finite-capacity, and typed-error patterns; unknown kinds are harness failures. [VERIFIED: `tools/xtask/src/inventory.rs`; `crates/liquidfun/src/world/diagnostics.rs`; `11-CONTEXT.md` D-23]

## Recommended Plan Ordering

1. **Corpus authority and joins:** define semantic manifest schemas, discovery refresh, compatibility/evidence joins, deterministic projections, and Cargo-only closure checks. This independently closes TEST-03/EXMP-01 and creates catalog IDs needed downstream. [VERIFIED: `11-CONTEXT.md` D-01-D-05]
1. **Canonical catalog/resolution:** define typed catalog entries, bounds, exact float transport, canonical bytes/hash, run identity, actions, and checkpoints. [VERIFIED: `11-CONTEXT.md` D-06-D-08]
1. **Session controller and headless interface:** implement selection, validation, pause/step/restart, action scheduling, and deterministic capture commands without renderer dependencies. [VERIFIED: `11-CONTEXT.md` D-09-D-10]
1. **Public observability:** extend semantic views and debug primitives, preserving bounded ownership and storage opacity. [VERIFIED: `crates/liquidfun/src/world/diagnostics.rs`; `11-CONTEXT.md` D-12-D-15]
1. **Canonical checkpoint/comparison:** integrate native/oracle outputs, named policies, missing-side observations, replay, fixtures, and failure bundles. [VERIFIED: `11-CONTEXT.md` D-14-D-16, D-23]
1. **Consumer convergence:** route regressions and benchmark setup through resolved plans; add representative rigid/joint/rope/particle/group/query/callback/mutation scenarios. [VERIFIED: `11-CONTEXT.md` D-11, D-24-D-25]
1. **Package-isolation gate:** prove default/package/headless operation before adding any visual dependency. [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-22]
1. **Visual capability spike and adapter:** test Macroquad 0.4.15 against explicit acceptance measurements, retain or document failure, then implement only the selected private adapter. [VERIFIED: `11-CONTEXT.md` D-17-D-21]
1. **Final closure:** require zero unexplained corpus rows, all scenario mappings closed, representative tests green, and no claim beyond D0-D3 evidence. [VERIFIED: `11-CONTEXT.md` D-25-D-26]

## Validation Architecture

### Test framework

| Property | Value |
| --- | --- |
| Framework | Rust built-in test harness; `proptest` 1.11.0 where generated cases add value [VERIFIED: `Cargo.toml`] |
| Quick run | `cargo test -p liquidfun --lib` plus the focused package/test target changed by a task [ASSUMED] |
| Full phase suite | `cargo test --all-features` after `cargo fmt --all`, Clippy, and all-target/all-feature build [VERIFIED: `AGENTS.md` supplied with task] |

### Requirements-to-test map

| Requirement | Required automated evidence | Wave 0 status |
| --- | --- | --- |
| RIGD-10 | Public integration tests for counts, contacts, particle contacts, broad-phase observations, profiles, primitive stability, and no private coordinates | Missing Phase 11 coverage [VERIFIED: `11-CONTEXT.md` D-24] |
| TEST-03 / EXMP-01 | Inventory fixture tests for unknown, duplicate, stale, unmapped, and non-terminal corpus rows; deterministic projection check | Missing semantic-corpus fixtures [VERIFIED: `11-CONTEXT.md` D-05, D-24] |
| EXMP-02 | Pure transition tests for pause/no tick, step/one tick/still paused, restart/same bytes, bounds, settings, checkpoint ordinals | Missing controller tests [VERIFIED: `11-CONTEXT.md` D-09-D-10, D-24] |
| EXMP-03 | Integration test proving one resolved byte sequence feeds native, oracle request, fixture replay, benchmark setup, and visual adapter input | Missing convergence test [VERIFIED: `11-CONTEXT.md` D-06-D-11, D-24] |
| EXMP-04 / EXMP-05 | Headless adapter tests for every UI command and semantic overlay input; visual smoke/capture checks remain diagnostic | Missing adapter contract tests [VERIFIED: `11-CONTEXT.md` D-19-D-20, D-24] |
| EXMP-06 | Default-member, package, and headless-environment checks with no renderer/oracle source required | Missing explicit Phase 11 isolation gate [VERIFIED: `Cargo.toml`; `11-CONTEXT.md` D-22, D-24] |

### Sampling and phase gate

- **Per task:** run the narrow unit/integration target for the changed layer. [ASSUMED]
- **Per wave:** run all affected private-tool and `liquidfun` tests plus read-only inventory checks. [ASSUMED]
- **Phase gate:** run the repository's required Rust format, Clippy, build, and test sequence; then run corpus closure, package isolation, deterministic replay, and representative headless scenarios. [VERIFIED: `AGENTS.md` supplied with task; `11-CONTEXT.md` D-22-D-26]

### Wave 0 gaps

- Add semantic corpus validation fixtures before the manifest implementation.
- Add pure catalog-resolution and controller-transition test modules before adapters.
- Add canonical checkpoint/comparison fixtures covering ordered and explicitly unordered collections.
- Add a package/headless isolation command or integration test before creating a testbed package.

These file names and exact commands are planner discretion because the allowed source set does not expose the current protocol/differential test layout. [ASSUMED]

## Security Domain

- Treat every manifest, scenario request, action list, and checkpoint as untrusted boundary data: deny unknown fields/variants, enforce reviewed bounds, validate paths and identities, and reject incomplete joins. [VERIFIED: `tools/xtask/src/inventory.rs`; `11-CONTEXT.md` D-05, D-08, D-23]
- Never allow UI input or renderer timing to bypass settings validation or controller transitions. [VERIFIED: `11-CONTEXT.md` D-09-D-10, D-19]
- Keep C++ execution process-isolated and private; no renderer or oracle dependency enters the published library. [VERIFIED: `11-CONTEXT.md` D-22-D-23]

## Assumptions Log

| # | Claim | Section | Risk if wrong |
| --- | --- | --- | --- |
| A1 | Exact quick/per-wave test commands will be selected during planning. | Validation Architecture | Planner must inspect package-specific test targets before locking commands. |
| A2 | Wave 0 test file names remain open because the recovery source set excludes current test directories. | Validation Architecture | Planner must avoid inventing conflicting modules. |

## Resolved Questions

1. **Macroquad acceptance outcome — RESOLVED:** Plan 11-24 is the executable capability gate. It tries Macroquad 0.4.15 only after Plan 11-18 proves the headless stack, retains Macroquad only when every named measurement passes, and permits the private `winit`/`wgpu`/`egui` fallback only after a concrete allowed-category failure is recorded. [VERIFIED: `11-CONTEXT.md` D-17-D-18; `11-24-PLAN.md`]
1. **Exact public primitive transport — RESOLVED:** Plan 11-09 owns one bounded semantic primitive collector returning the authoritative owned collection and may additionally expose a narrow sink implemented from that same collector; downstream renderers do not define a second traversal or semantic model. [VERIFIED: `11-CONTEXT.md` D-13; `11-09-PLAN.md`]
1. **Exact package/module names — RESOLVED:** The selected boundaries are the plan-owned modules: public observations and debug primitives in `liquidfun` (Plans 11-08/09), catalog and checkpoint contracts in `liquidfun-test-protocol` (Plans 11-03/06/10), controller/native execution/comparison/supervision in `liquidfun-differential` (Plans 11-07/11/13/14), the C++ process adapter under `tools/reference` (Plan 11-12), the private benchmark package (Plan 11-17), and the unpublished non-default testbed package selected by Plan 11-24. [VERIFIED: `11-03-PLAN.md`, `11-06-PLAN.md`, `11-07-PLAN.md`, `11-08-PLAN.md`, `11-09-PLAN.md`, `11-10-PLAN.md`, `11-11-PLAN.md`, `11-12-PLAN.md`, `11-13-PLAN.md`, `11-14-PLAN.md`, `11-17-PLAN.md`, `11-24-PLAN.md`]

## Sources

### Primary (HIGH confidence)

- `.planning/phases/11-examples-headless-tooling-and-testbed/11-CONTEXT.md` — locked architecture, evidence, UI, and scope decisions.
- `.planning/ROADMAP.md` Phase 11 — goal, success criteria, dependency, and requirements.
- `.planning/REQUIREMENTS.md` — RIGD-10, TEST-03, EXMP-01 through EXMP-06.
- `Cargo.toml` — current workspace members, default member, versions, and lint policy.
- `tools/xtask/src/inventory.rs` — strict inventory schema and deterministic validation/report seams.
- `crates/liquidfun/src/world/diagnostics.rs` — bounded owned semantic diagnostics and exact metric seams.

### Secondary (MEDIUM confidence)

- `.planning/research/STACK.md` — previously researched testbed dependency recommendation and fallback trigger.

## Metadata

**Confidence breakdown:**

- Corpus/inventory architecture: HIGH — directly grounded in current xtask code and locked decisions.
- Scenario/controller architecture: HIGH — locked in Phase 11 context.
- Diagnostic architecture: HIGH — current public/owned patterns are visible in engine code.
- Renderer choice: MEDIUM — intentionally contingent on a Phase 11 capability spike.
- Exact test commands/files: LOW — recovery scope excluded the detailed test layout.

**Research date:** 2026-07-21
**Valid until:** 2026-08-20, or until Phase 11 context/stack decisions change.
