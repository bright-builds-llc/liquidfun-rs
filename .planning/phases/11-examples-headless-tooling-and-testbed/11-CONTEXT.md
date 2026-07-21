---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-21T21:10:55.704Z
---

# Phase 11: Examples, Headless Tooling, and Testbed - Context

**Gathered:** 2026-07-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Account for every pinned-upstream test, example, and registered testbed scenario, then expose one renderer-neutral scenario catalog and deterministic controller across native Rust execution, C++ oracle comparison, regression fixtures, benchmarks, headless tooling, and an optional visual testbed. Complete public semantic diagnostics and debug drawing without exposing private storage or adding renderer, windowing, or game-engine dependencies to the published physics crate. Performance, broad portability, release packaging, and v1 hardening remain Phase 12.

</domain>

<decisions>
## Implementation Decisions

### Upstream corpus accounting

- **D-01:** Add one dedicated machine-authoritative upstream-corpus manifest for semantic test cases, example scenarios, and registered testbed entries. Join it fail-closed to `reference/discovery.json`, `reference/compatibility.json`, scenario-catalog identities, and evidence references; generated Markdown remains a projection rather than a second authority.
- **D-02:** Discover semantic items rather than counting source files alone. The oracle-enabled refresh path must enumerate actual GoogleTest declarations or authoritative `--gtest_list_tests` output and the pinned testbed registration table, while Cargo-only checks validate the checked-in snapshot without requiring the submodule or C++ toolchain.
- **D-03:** Give every corpus item a stable source-derived identity, source path and symbol or registration identity, upstream revision, applicability, disposition, compatibility impact, rationale, and evidence mappings. Model disposition and impact as separate closed enums so a ported or equivalently covered item may still record behavioral, API, tooling, visual-only, or no compatibility impact.
- **D-04:** Terminal dispositions are explicit: native scenario/test port, equivalent existing evidence, reviewed irrelevance, documented difference, or intentional non-support as justified by the final schema. Every non-port disposition requires a specific reviewed rationale and compatibility impact; vague, empty, self-referential, stale, or missing evidence is rejected.
- **D-05:** Corpus closure rejects unknown, duplicate, unregistered, unmapped, or stale items and validates every referenced scenario, test, fixture, ledger leaf, review record, and source identity. Classification changes must remain reviewable in ordinary repository history; generated reports summarize exact totals and unresolved rows without silently updating authority.

### Shared scenario catalog and headless controls

- **D-06:** Author scenarios as typed private catalog definitions that resolve into an immutable, bounded, engine-neutral `ResolvedScenario`-style plan before execution. The exact resolved plan, not mutable plugin state or renderer callbacks, is the common currency for Rust, C++ oracle, regressions, benchmarks, and visualization.
- **D-07:** Separate stable catalog slugs from display titles. Identify a generated run with catalog schema version, scenario version, generator identity and version, seed when present, exact run settings, and a content hash of canonical resolved bytes. Persist resolved bytes for regressions and failures; a seed alone is never sufficient replay evidence.
- **D-08:** Represent setup and interactive behavior as closed typed actions with stable semantic entity and action identities, exact `f32` transport, explicit order, and reviewed bounds. Backend-specific simulation logic, hidden mutable scenario state, frame callbacks, and duplicate Rust/C++ scenario implementations are prohibited.
- **D-09:** Put control state in a renderer-neutral run-session controller outside physics state. Pause performs no logical tick and emits no fabricated checkpoint; single-step executes exactly one logical tick and remains paused; restart destroys the current session and reconstructs step zero from the same resolved bytes and settings. A particle system's upstream pause flag remains a distinct typed scenario action.
- **D-10:** Bind deterministic checkpoints to explicit action or logical-step ordinals and stable checkpoint IDs, never render frames, refresh rate, wall time, or UI event timing. The controller owns selection, settings validation, action application, checkpoint capture, and restart semantics; frontends only submit commands and observe results.
- **D-11:** Benchmarks construct or restart from the same resolved plan outside the measured interval and declare the exact measured horizon. Regression fixtures, differential requests, and testbed captures reuse the canonical plan and checkpoint model rather than translating through separate formats.

### Renderer-neutral observability and comparison capture

- **D-12:** Build layered public semantic views for current counts, tree metrics, contacts, particle contacts, broad-phase observations, particle statistics, and renderer-neutral geometry, then use one bounded owned canonical checkpoint builder as the authoritative deterministic capture. Reuse and extend existing `WorldDiagnostics`, owned `StepReport` evidence, and borrow-scoped particle/contact views instead of exposing arenas, dense rows, tree nodes, or raw proxy storage.
- **D-13:** Define debug drawing as a closed renderer-neutral primitive vocabulary carrying stable semantic owner and primitive keys, explicit geometry, color/style metadata, and named layer/category. Consumers receive owned records or a narrow sink adapter derived from the same collected semantic model; private traversal order and internal indices are never public identity or comparison keys.
- **D-14:** Canonical checkpoints preserve source-significant order and explicitly canonicalize only declared unordered primitive or observation sets using stable keys and deterministic tie-breakers. Structural fields, identities, kinds, flags, counts, membership, presence, and ordering compare exactly; numeric geometry uses only closed named Phase 4 policies.
- **D-15:** Treat wall-clock phase profiles as a separate diagnostic channel. Profile names and presence may compare structurally, but measured durations are excluded from D0/D1 physics parity and deterministic checkpoints; the testbed may display Rust and oracle timings side by side without claiming numeric timing equality.
- **D-16:** Produce one renderer-neutral comparison model keyed by stable semantic paths. It records exact matches, policy-qualified numeric differences, Rust-only and oracle-only observations, and bounded diagnostic context. Visual diff overlays and mismatch lists consume this model and may not re-read or reinterpret private engine state.

### Optional visual testbed

- **D-17:** Finish and verify the headless catalog, controller, capture, and comparison capability before choosing a rendering dependency. Then run a private Macroquad 0.4.15 capability spike; retain it only if it proves readable contacts, particles, broad-phase overlays, profiles, side-by-side or overlay diffs, deterministic state capture, screenshots, controls, and supported desktop behavior.
- **D-18:** Use private `winit`/`wgpu`/`egui` integration only when the Macroquad spike records a concrete failure in required UI density, capture fidelity, accessibility, GPU inspection, render-target control, or platform support. Bevy and any renderer-owned simulation schedule remain out of scope.
- **D-19:** Keep the visual testbed in an unpublished, non-default workspace package. Its adapter translates input into controller commands and semantic snapshots into pixels; it owns windowing, frame pacing, camera, UI, GPU resources, and screenshot output, but no physics rules, oracle truth, scenario definitions, world storage, or checkpoint semantics.
- **D-20:** Provide scenario selection, run identity and seed display, pause, one-step, restart, validated timestep and iteration controls, overlay toggles, contacts, particle contacts, broad-phase data, phase profiles, Rust/oracle side-by-side or overlay comparison, mismatch focus, and deterministic semantic capture. Pixel screenshots are diagnostic artifacts, not compatibility authority.
- **D-21:** Default the new testbed UI to a practical dark theme with accessible contrast and distinct state colors. Expose repository source, license truth, Peter Ryszkiewicz/OpenLinks attribution where the UI can carry it without crowding, and visible version, short commit, and build provenance with `Unavailable` for missing values.
- **D-22:** Preserve Cargo-only and packaging isolation: `liquidfun` remains the sole default publishable crate and receives no renderer, windowing, game-engine, C++, protocol, or testbed dependency or default feature. Package checks must prove the published crate builds and tests without the testbed, native oracle source, or graphical environment.

### Testing, evidence, and phase sign-off

- **D-23:** Extend the existing protocol, native adapter, C++ oracle, comparator, replay, failure-bundle, inventory, and evidence pipelines rather than creating a parallel example/testbed harness. Unknown scenario kinds, actions, observations, debug primitives, corpus dispositions, or policy paths are harness failures.
- **D-24:** Unit-test pure catalog resolution, controller transitions, primitive generation, checkpoint canonicalization, corpus joins, and diff construction one concern at a time. Add public integration tests for headless selection, seed resolution, pause/step/restart, settings, capture, package isolation, and representative rigid, joint, rope, particle, group, query, callback, and mutation scenarios.
- **D-25:** Give every upstream corpus item a terminal reviewed outcome and every native scenario a closed mapping to its tests, oracle or equivalent evidence, regression use, benchmark eligibility, and visualization eligibility. Phase completion requires zero unexplained corpus rows and no maturity claim beyond the exact reviewed evidence.
- **D-26:** Retain D0-D3 authority, strict provenance, replay, sanitizer, exact-reference, and same-run promotion rules from earlier phases. Deterministic semantic checkpoints may support parity; UI pixels, frame rate, and wall-clock profiles remain diagnostic only. Phase 12 still owns broad performance, portability, packaging, and release-readiness claims.

### the agent's Discretion

- Exact public and private type, module, method, command, manifest, review-record, primitive, observation, profile, and error names within the locked boundaries.
- Exact plan decomposition, bounded capacities, catalog composition helpers, property-case counts, and representative scenario grouping, provided closure remains item-level and fail-closed.
- Exact primitive transport shape and whether the rendering adapter consumes an owned collection or narrow sink, provided both derive from the same semantic model and deterministic checkpoints remain authoritative.
- Exact Macroquad spike acceptance measurements, visual layout, camera gestures, keyboard shortcuts, and overlay styling beyond the required controls, dark default, accessibility, provenance, and source disclosure.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and inherited contracts

- `.planning/PROJECT.md` — Native-Rust, oracle-isolation, Cargo-first, safety, determinism, rendering, testing, and truthfulness constraints.
- `.planning/REQUIREMENTS.md` — `RIGD-10`, `TEST-03`, and `EXMP-01` through `EXMP-06` acceptance requirements.
- `.planning/ROADMAP.md` — Fixed Phase 11 goal, success criteria, Phase 10 dependency, and renderer/headless research flags.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — Typed JSONL, exact float transport, stable scenario identity, replay, minimization, and process-isolated oracle contracts.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — Deterministic expression/order policy, closed comparison policies, special values, and D0-D3 evidence authority.
- `.planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-CONTEXT.md` — Owned semantic diagnostics, renderer-neutral boundary, counts/tree metrics, and profile exclusion from parity evidence.
- `.planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-CONTEXT.md` — Complete particle behavior, existing evidence authority, public particle views, and explicit Phase 11 boundary.

### Project evidence and implementation seams

- `ARCHITECTURE.md` — Current engine, protocol, oracle, renderer-independence, and future testbed boundaries.
- `COMPATIBILITY.md` — Generated compatibility inventory and unresolved upstream test/example rows.
- `TESTING.md` — Unit, integration, property, differential, replay, D0-D3, sanitizer, and promotion contracts.
- `UPSTREAM.md` — Pinned revision, source provenance, read-only oracle policy, and testbed source role.
- `reference/discovery.json` — Machine-readable pinned source discovery snapshot, currently including test and example files.
- `reference/compatibility.json` — Machine-authoritative compatibility ledger and evidence-state vocabulary to join without duplicating.
- `crates/liquidfun/src/world/diagnostics.rs` — Existing public counts, tree metrics, and owned semantic reconstruction.
- `crates/liquidfun/src/world/step.rs` — Owned step reports, source-timed occurrences, and current contact evidence.
- `crates/liquidfun/src/particle/view.rs` — Borrow-scoped stable particle, contact, pair, and triad views.
- `crates/liquidfun/src/particle/statistics.rs` — Existing particle-system statistics and semantic diagnostic seam.
- `crates/liquidfun-test-protocol/src/scenario.rs` and `crates/liquidfun-test-protocol/src/scenario/` — Current typed named scenario, validation, exact transport, and rigid-world extension points.
- `crates/liquidfun-differential/src/runner.rs` and `crates/liquidfun-differential/src/rigid_world.rs` — Named execution, native/oracle routing, comparison, replay, and evidence seams.
- `tools/xtask/src/inventory.rs` and `tools/xtask/src/inventory/` — Existing discovery, validation, and generated-report ownership.

### Pinned upstream corpus and testbed

- `third_party/liquidfun/liquidfun/Box2D/Unittests/` — Pinned GoogleTest corpus whose semantic cases require granular accounting.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/TestEntries.cpp` — Authoritative registered upstream testbed scenario list.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Tests/` — Pinned example and testbed scenario implementations.
- `third_party/liquidfun/liquidfun/Box2D/Testbed/Framework/Test.h` and `third_party/liquidfun/liquidfun/Box2D/Testbed/Framework/Test.cpp` — Upstream scenario controls, stepping, diagnostics, and debug-draw behavior to inventory rather than copy architecturally.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Dynamics/b2WorldCallbacks.h` — Pinned debug-draw primitive and callback surface.

### Research and repository standards

- `.planning/research/STACK.md` — Headless-first private testbed recommendation, Macroquad spike, wgpu fallback triggers, and package-isolation policy.
- `.planning/research/ARCHITECTURE.md` — Scenario/testbed app boundary and renderer-neutral data flow.
- `.planning/research/FEATURES.md` — Upstream accounting and interactive comparison feature expectations.
- `AGENTS.md` and `AGENTS.bright-builds.md` — GSD, Rust quality, sync-first, verification, renderer, and task-artifact rules.
- `standards-overrides.md` — Local exception registry; no substantive active override replaces the managed defaults.
- `standards/core/architecture.md` — Functional-core/imperative-shell, boundary parsing, and invariant modeling.
- `standards/core/code-shape.md` — Shallow control flow, deep-module sizing, and rerunnable diagnostic tooling.
- `standards/core/frontend-ui.md` — Dark default, source disclosure, maintainer attribution, and public-app UI expectations.
- `standards/core/testing.md` — Focused pure/business tests with Arrange, Act, Assert structure.
- `standards/core/verification.md` — Sync and repository-native pre-commit gates.
- `standards/languages/rust.md` — Rust modules, guards, optional naming, invariant types, and verification guidance.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- `reference/discovery.json` and `reference/compatibility.json` already enumerate 14 upstream test files and 73 example files, while Phase 11 needs a finer semantic-item layer and reviewed closure.
- `tools/xtask/src/inventory.rs` already owns pinned-tree discovery, closed compatibility kinds, strict parsing, evidence dimensions, deterministic reports, and Cargo-only check separation.
- `crates/liquidfun-test-protocol` already owns stable IDs, exact `f32` bits, typed scenario validation, bounded JSONL, rigid-world actions, checkpoint requests, provenance, and closed schemas.
- `crates/liquidfun-differential` already owns named runs, native and C++ execution, comparison, replay, minimization, deterministic evidence, fixtures, failure bundles, and machine/human reports.
- `WorldDiagnostics`, `StepReport`, particle views/statistics, broad-phase metrics, and semantic reconstructions provide the public observation vocabulary to deepen rather than bypass.

### Established Patterns

- `liquidfun` is the only default publishable crate; protocol, differential, xtask, oracle, benchmark, and testbed concerns remain private and non-default.
- Boundary data is strict, typed, bounded, versioned, and fail-closed. Stable semantic IDs replace pointers, slots, dense indices, and raw memory.
- Machine-readable evidence is authoritative and generated human reports are projections. Source-significant order is preserved; only explicitly unordered collections are canonicalized.
- C++ is a read-only development oracle reached through the process boundary. Renderers and frontends are adapters and never simulation authorities.

### Integration Points

- Extend inventory discovery/validation/reporting with semantic corpus records and cross-ledger joins.
- Add a private scenario catalog/controller surface above the current protocol and native/oracle adapters without creating a second physics implementation.
- Extend public world diagnostics and debug observations, then add protocol/comparator records over the same semantic checkpoint builder.
- Add headless xtask/just entrypoints first, regression and benchmark consumers second, and the private testbed adapter only after the capability gate.

</code-context>

<specifics>
## Specific Ideas

- Treat the resolved scenario bytes and action/checkpoint log as the durable replay artifact; seeds are provenance inputs, not sufficient fixtures.
- Distinguish controller pause from the upstream particle-system pause action so UI state cannot accidentally change physics semantics.
- Use stable primitive keys to focus visual diffs without turning renderer visitation order into public API.
- Make the renderer decision an executable capability gate: Macroquad first, heavier GPU/UI stack only after a named failure is recorded.

</specifics>

<deferred>
## Deferred Ideas

- Broad benchmark budgets, profiling-led optimization, platform matrix sign-off, coverage expansion, packaging, and release readiness — Phase 12.
- Renderer-specific performance work, advanced GPU inspection, plugin scripting, and alternate testbed frontends beyond the Phase 11 capability gate — future evidence-driven work.
- Pixel-perfect screenshot regression as compatibility authority — intentionally excluded; semantic checkpoints remain authoritative.

</deferred>

***

*Phase: 11-examples-headless-tooling-and-testbed*
*Context gathered: 2026-07-21*
