# liquidfun-rs

## What This Is

`liquidfun-rs` is a fun, experimental, open-source Rust implementation of Google's LiquidFun physics engine for learning, games, simulations, and visualization. It develops useful native Rust behavior incrementally against a pinned upstream reference; complete parity and production certification are optional ambitions.

Visitors can also explore six native scenes in a SolidJS GitHub Pages playground compiled from this engine to WebAssembly. The repository retains upstream C++ LiquidFun as a read-only development oracle for research, differential testing, reference data, and benchmark comparison. Ordinary users of the published Rust library must not need the upstream source, a C++ compiler, Bazel, or any cross-language runtime component.

## Core Value

Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.

## Current scope decision — 2026-09-16

The owner prioritizes a fun hobby project over production-quality certification. Apply `PROJECT-SCOPE.md`: mandatory Linux x64 qualification and the dedicated performance-host completion gate are removed. Historical evidence remains truthful and source-bound; strict qualification is opt-in. The owner selected local checks plus one macOS CI job and optional manual expensive suites. The accepted experimental preparation checklist is in RELEASE.md. APIs may evolve with incompatible changes documented before release; durable API/MSRV guarantees are deferred. Preserve Rust 1.97.0 and the declared minimum 1.92, verifying that minimum before publication or deliberately revising manifest and docs together.

## Current State

v1.3 Reference Testbed Scenes is in progress. v1.2 Native Performance Closing is archived (2026-09-21): 4 phases, 23 plans, and 47 tasks. Unprofiled Dam Break Medium on one macOS host recorded `rust_over_cpp_ratio` `2.956857456935513` at stamp `2026-09-21T20-38-50Z` (git `89d3456406c3c79ed500192bca9491c707033647`). `reviewed_reports` stays empty. README and crates.io have no universal “Rust is N×” claim. `just web-player-smoke` recorded 37 Chromium tests, compared with native Rust only. This is not crate publication and not a git release tag. Commit `77fbd84` (zero the force buffer after SolveForce) is after that stamp, so the ratio is not a measurement of later HEAD. See `.planning/MILESTONES.md` and `.planning/milestones/v1.2-REQUIREMENTS.md`.

The v1.1 Web Playground planning milestone is archived (2026-09-20): 6 phases and 39 plans completed. Visitors can run Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel at `https://bright-builds-llc.github.io/liquidfun-rs/` with in-app SVG previews, play/pause/reset, honest Reset labels, pointer interaction, and automatic Pages delivery. This is not a crate or npm release. See `.planning/MILESTONES.md` and `.planning/milestones/v1.1-REQUIREMENTS.md`.

The v1.0 Experimental Foundation planning milestone remains archived (2026-09-17): 16 phases and 252 active plans under hobby scope. Native checks, isolated packaging and macOS CI were verified at `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`.

<details>
<summary>v1.2 phase-by-phase completion notes</summary>

Phase 22: exclusive unprofiled Dam Break stamps, fail-closed samply 0.13.1 profiles, and separate parent-phase timers. Independent review digest `52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`.

Phase 23: `docs/native-performance-audit.md` names `particle_rows` from live pair and samply at `6d98531ac799987c209d3fd1e572e482fcab5da6` (unprofiled ratio `327.53395024734476`) and records a private dhat dump. Independent review digest `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`.

Phase 24: shared scalar kernels until gate stamp `2026-09-21T20-38-50Z` / ratio `2.956857456935513`; five-scene spot-check; same-cluster note; empty `reviewed_reports`. Independent review digest `c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d`.

Phase 25: remaining-delta notes and `just web-player-smoke` (37 tests). Optional step-time note cites native `ms_per_step` `1.053103` and is never versus `oracle-release`.

</details>

<details>
<summary>v1.1 phase-by-phase completion notes</summary>

Phase 16: private WASM wrapper, five copied frame lanes, Chromium proof, independent review, 20/20 closed threats.

Phase 17: shared SolidJS player, Dam Break on Pages, OIDC delivery. Recorded deploy SHA `50a15562b356ed941266eedddc636df3f76e7e7e`.

Phase 18: six native scenes with labeled controls and host-locked credits.

Phase 19: Pointer Events gestures, keyboard/narrow-width polish, Chromium plus live Pages at source `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`.

Phase 20: Kobalte catalog SVG previews and Reset remount to `DEFAULT_PRESET_VALUES`. Chromium `just web-player-smoke` 37 tests.

Phase 21: removed unused `loadProofSession`, unknown-only `FallbackPanel`, named scenes under 628 lines, `upstream_cli` split. Independent review digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`.

</details>

## Current Milestone: v1.3 Reference Testbed Scenes

**Goal:** A visitor can open every JavaScript LiquidFun testbed scene we do not already have, running on this engine in the existing playground.

**Target features:**
- Drawing Particles
- Elastic Particles
- Impulse
- Liquid Timer
- Particles
- Rigid Particles
- Soup
- Soup Stirrer
- Sparky
- Surface Tension
- Theo Jansen
- Wave Machine

The six current playground scenes stay, including Dam Break. Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel remain original scenes. Each new scene is a recognizable port in the existing SolidJS player. Missing engine behavior is in scope only where a listed scene cannot run without it. This is not a sealed C++ parity claim, a public benchmark, or a crate release. Phase numbering continues after 25. Package publication and a git release tag remain separately authorized.

Ideas still left out until a later milestone adopts them: an explicit SIMD or parallel opt-in, relaxing `unsafe_code = "forbid"` for a measured intrinsic, Phase 12 sealed-matrix calibration, and WASM stepping work beyond what these scenes need.

## Requirements

### Active

- [ ] Visitors can run the twelve JavaScript testbed scenes listed above in the existing playground. Requirement IDs are defined in the upcoming `.planning/REQUIREMENTS.md`.

### Validated

- [x] Phase 25 completed PERF-NOTES and PERF-WASM: honesty docs lead with gate stamp `2026-09-21T20-38-50Z` / `rust_over_cpp_ratio` `2.956857456935513` and label kernel-HEAD `2026-09-21T20-36-30Z` as a sibling; `just web-player-smoke` passed 37 Chromium tests; optional step-time note cites native `ms_per_step` `1.053103` with no prior WASM sample and never versus `oracle-release`. `reviewed_reports` stays empty. Not a public “Rust is N×” claim or crate publication. Validated in Phase 25: WASM sanity and honest close.

- [x] Phase 24 completed PERF-ADMIT, PERF-SHARED, PERF-GATE, PERF-BASELINE, PERF-SPOT, and PERF-CANARY2: shared scalar `particle_rows` / leftover kernels until unprofiled Dam Break Medium `pair.json` `rust_over_cpp_ratio` `2.956857456935513` (stamp `2026-09-21T20-38-50Z`); five-scene `just playground-scene-spot`; empty `reviewed_reports`; `unsafe_code = "forbid"`. Independent AI review digest `c9185f396610c250b2861480b0c1aa9cbfd16a2eef73c881fccc9d10ac99e31d` (`issues_found`, WR-01). Not crate publication or a Phase 12 public claim.

- [x] Phase 23 completed PERF-AUDIT and PERF-HEAP: committed `docs/native-performance-audit.md` names `particle_rows` (~93.6% self) from live pair+samply at MEASURED_HEAD `6d98531ac799987c209d3fd1e572e482fcab5da6`, unprofiled ratio `327.53395024734476`; copy-only audit-bundle; private dhat when allocator/`Vec` needles matched; timing doc refreshed as an unreviewed sample. Independent AI review digest `c23661d75e80e8702f7b175ba3e5578479f4999586ae9be5e194f8de2346f01d`. This is not the 3× gate or crate publication.

- [x] Phase 22 completed PERF-PAIR, PERF-PROFILE, and PERF-TIMERS: `just playground-dam-break-bench` persists exclusive `target/dam-break-perf/<utc-stamp>/` `pair.json`/`pair.md`; `just playground-dam-break-profile` wraps samply 0.13.1 onto `[profile.profiling]` and writes `rust.json.gz` plus identity with `not_timing_authority`; `just playground-dam-break-timers` emits parent-phase timers on a sibling process. Missing samply fails closed. `liquidfun` stays bitflags-only. Independent AI review digest `52f2619f480dd0daf1dfb7ec4db6b7624396e987b73979189b70678407a38f06`. This is not the 3× gate, a named-function audit, or crate publication.

- ✓ v1.1 Web Playground — 22/22 requirements, 6 phases, 39 plans, audit passed 2026-09-20. Hosted playground at `https://bright-builds-llc.github.io/liquidfun-rs/`. Not a crate release.

- [x] Phase 21 leftover cleanup: deleted unused `loadProofSession`; kept `rust-wasm-proof.spec.ts` as opt-in historical `just web-smoke`; collapsed `FallbackPanel` to unknown-only Scene not found plus Open Dam Break; confirmed `color_mixer.rs` / `dam_break.rs` / `water_wheel.rs` under 628 lines; split `tools/xtask/tests/upstream_cli.rs` so Bright Builds `all` exits 0. Chromium `just web-player-smoke` passed 37 tests. Independent AI review approved digest `3f4e16206f8be62bbc4c4bd93482edf17658a8f76657053a6c862f78564a4f07`. This is not crate publication.
- [x] Phase 20 completed WEB-01 and WEB-03 gap closure: six in-app DemoNavigation entries show names, short descriptions, and compact static SVG `Static preview` illustrations without restoring the card grid; Reset remounts preset selects to `DEFAULT_PRESET_VALUES`; Chromium `just web-player-smoke` passed 37 tests including sidebar/drawer previews and Reset honesty. Independent AI review approved digest `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`. This is not crate publication.
- [x] Phase 19 completed WEB-05, WEB-07, and WEBTEST-01: each of the six scenes has one documented canvas Pointer Events gesture through a shared CSS-bound camera unproject and WASM `pointer_action`, labeled keyboard controls remain, per-scene figcaption instructions and `select:focus-visible` are in place, Chromium `just web-player-smoke` covers pointer/cancel/resize/375px/hidden-tab, and live Pages at `https://bright-builds-llc.github.io/liquidfun-rs/` source `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e` serves WASM as `application/wasm`. Independent AI review approved digest `a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`. This is not crate publication.
- [x] Phase 18 completed WEB-01, WEB-04, WEB-08, and DEMO-01 through DEMO-06: six ready native scenes run in the shared player with static catalog cards, labeled bounded controls, explicit reset-on-change, and per-scene implementation/inspiration/notice links. Local Chromium `just web-player-smoke` proves open/reset/switch. Independent AI review approved digest `1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`. This is not WEB-05/WEB-07/WEBTEST-01 coverage or package publication.
- [x] Phase 17 completed WASM-04, WEB-02, WEB-03, WEB-06, and HOST-01 through HOST-03: Dam Break runs in the shared player on the live Pages project path, unknown scene hashes fall back usefully, playback/reset and loading/retry work, session teardown and hidden-tab catch-up stay bounded, and every main push builds WASM plus the site from the same checkout then deploys with OIDC. Recorded URL `https://bright-builds-llc.github.io/liquidfun-rs/`; source `50a15562b356ed941266eedddc636df3f76e7e7e`. Independent AI review approved digest `a962555da21e1da79b860c37b003d29fa7f3d6683ef16db1b2beac2a151bf893`. This is not six-scene WEB-01 coverage or package publication.
- [x] Phase 16 completed WASM-01 through WASM-03: generated Rust WebAssembly visibly advances a real particle/rigid scene in Chromium, the reproducible pinned SolidJS build preserves native Cargo/package isolation, and five bounded copied typed arrays cross the browser boundary without raw pointers or per-particle calls. Explicit disposal, exact-digest independent AI review, and 20/20 closed threats were verified against source-bound closure attempt 10.
- [x] Phase 15 hobby wrap-up completed HOBBY-01 through HOBBY-03: experimental guidance, a short preparation checklist, native/package verification and exact-source macOS CI. Checked implementation: `75ead0edcbde68d01b804e2c466f2f4b2a4d30da`; later planning-record changes do not claim new runtime verification. Strict certification remains deferred and no package was published.

<!-- Shipped and confirmed valuable. -->

- [x] Phase 1 selected and pinned official LiquidFun commit `7f20402173fd143a3988c921bc384459c6a858f2`, with release ancestry, licensing, notices, source-mapping, and intentional update rules recorded.
- [x] Phase 1 established a Cargo-first resolver-3 workspace with one publishable `liquidfun` crate, a private `xtask`, a pinned development toolchain, a provisional Rust 1.92 MSRV, and package isolation from C++/reference inputs.
- [x] Phase 1 created an authoritative 177-row compatibility ledger, deterministic 161-entry discovery snapshot, generated human report, and fail-closed inventory/provenance/package checks.
- [x] Phase 2 established a bounded, versioned semantic protocol and private Rust/C++ differential harness with strict provenance, typed comparison and failure classification, replay/minimization, reviewed fixture promotion, and verified one-shot, reuse, and sanitizer empty-world round trips.
- [x] Phase 3 established world-scoped typed handles, checked invalidation and exhaustion, upstream-ordered owned destruction evidence, restricted hooks and deferred mutation, typed user associations, and a stable-particle dense-storage spike with clean review and 17/17 verified must-haves.
- [x] Phase 4 established the public `f32` math/settings foundation, checked matrix/transform/sweep contracts, a closed per-observable numerical policy, bit-faithful Rust/C++ probes, fail-closed build identity, typed mismatch evidence, and 25/25 verified must-haves.
- [x] Phase 5 established immutable validated shapes, source-ordered distance/manifold/tree/broad-phase/TOI kernels, pure pair/filter/refilter seams, and a fail-closed 78-family Rust/C++ collision probe with 35/35 verified must-haves; world-owned contact lifecycle remains Phase 6 work.
- [x] Phase 6 established the native body, fixture, sensor, and contact vertical slice with stable typed identity; atomic mass, proxy, mutation, and destruction behavior; source-ordered contact solving and deferred hooks; checkout-bound oracle provenance; and debug, release, replay, determinism, and sanitizer differential evidence with 77/77 verified must-haves.
- [x] Phase 11 accounted for all 388 upstream corpus rows, established one 43-scenario renderer-neutral catalog across headless, oracle, regression, benchmark, and private testbed consumers, preserved package isolation, and passed 94/94 must-haves including agent-controlled desktop comparison UAT.

- [x] Phase 13.1 restored structural compliance and closed its verification gaps with a complete 72-command local/canonical matrix, independently validated durable evidence, and formal verification at 36/36 with the accepted integration-history override.

- [x] Phase 14 repaired Windows particle-group scratch allocation, preserved typed rejection and complete rollback contracts, and passed 13/13 must-haves. Both fixed regressions and 957 package tests pass on Windows, Linux and macOS at one source SHA, with complete Cargo CI success; canonical candidate acceptance belongs to the deferred optional strict qualification campaign.

### Historical full-parity ambitions

These retained goals describe the original broad implementation ambition, not active Phase 15 acceptance. Current hobby completion follows PROJECT-SCOPE.md and the bounded experimental preparation checklist; broader parity and platform qualification remain optional.

- [ ] Extend the implemented minimal rigid-world slice with complete force and impulse application, configurable solvers, sleeping, continuous collision detection, world queries, and world ray casts from the selected LiquidFun revision.
- [ ] Implement all supported joints, standalone rope, remaining callbacks and listeners, diagnostic dump, debug drawing abstractions, and the broad rigid-body compatibility gate.
- [ ] Implement the full LiquidFun particle system, including storage, creation, destruction, lifetimes, buffers, spatial proxies, contacts, body contacts, groups, flags, pair/triad logic, queries, ray casts, callbacks, and every upstream solver behavior.
- [ ] Extend the Phase 2 C++/Rust reference harness beyond verified empty-world, math, collision, and minimal rigid-world probes to broad rigid-solver, joint, and particle scenarios with their semantic state, tolerance, regression, and diagnostic requirements.
- [ ] Create layered unit, integration, upstream-compatibility, differential, property, fuzz, Miri, sanitizer, and regression testing appropriate to each subsystem.
- [ ] Extend the Phase 4-6 numerical-stability, ordering, determinism, and platform-tier policy with broad rigid-solver, joint, and particle observables and tolerances before treating those later differential results as compatibility evidence.
- [ ] Measure performance against equivalent upstream C++ workloads and optimize only from profiling evidence without silently sacrificing API clarity, safety, determinism, or parity.
- [ ] Support Linux x86_64, Linux ARM64, macOS ARM64, macOS x86_64 where practical, and Windows x86_64, while investigating WASM, mobile, and realistic smaller `no_std` subsets.
- [ ] Provide discoverable repository automation through a root `justfile`, CI, pinned toolchains, dependency/license policy, documentation checks, coverage, benchmarks, and optional manual extended verification.
- [ ] Maintain accurate project, architecture, upstream, compatibility, testing, benchmarking, safety, contribution, release, and roadmap documentation without claiming unverified maturity or parity.

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Using the upstream C++ library as the runtime implementation — the production deliverable must be an independent Rust engine.
- Publishing a thin Rust binding layer as the port — FFI exists only for development-time reference, testing, and benchmarking workflows.
- Requiring C++, Bazel, the upstream submodule, or reference data for ordinary Cargo consumers — cross-language tooling is a repository-development concern.
- Mechanically translating the whole upstream repository in one pass — each subsystem must move through inventory, API design, minimal implementation, unit tests, upstream comparison, differential validation, optimization, documentation, and compatibility sign-off.
- Claiming full parity or production readiness before the traceability matrix has no unexplained gaps and the documented acceptance evidence exists — status reporting must remain precise.
- Treating an unrelated modern Box2D implementation as automatically compatible — reuse requires license review and behavioral validation against the selected LiquidFun ancestry.
- Coupling core simulation crates to a rendering or game-engine framework — visualization is optional and simulation remains headless.
- Introducing default parallel simulation that changes deterministic or upstream-compatible behavior — any such mode must be explicit and documented.
- Promising complete-engine `no_std`, embedded, iOS, Android, or WASM support before feasibility is established — smaller portable subsets may be pursued when evidence supports them.
- Selecting a final project license before upstream and derivative-work compatibility is reviewed — required notices and provenance come first.

## Hobby success standard

Useful simulations and examples work on the development machine, relevant local checks pass, and limitations are described honestly. Work can proceed without Linux x64 qualification or a dedicated performance runner.

## Optional full-parity success standard

The previous strict target remains available as an optional ambition. A future explicit claim of full feature parity requires all of the following:

- Every relevant public upstream feature is implemented or explicitly documented as irrelevant to the Rust library.
- Every upstream particle behavior, supported shape, supported joint, world operation, callback, and query has a Rust equivalent.
- Upstream examples have Rust equivalents, and upstream tests are ported, replaced, or explicitly accounted for.
- Differential tests cover representative and edge-case behavior using semantic state, reproducible inputs, and documented tolerances.
- Known behavioral, numerical, ordering, platform, API, and performance differences are documented.
- Performance is measured against upstream with comparable workloads and optimization levels.
- The production Rust library has no runtime dependency on C++ or the upstream source.
- The compatibility matrix contains no unexplained gaps.

## Historical delivery strategy

Development proceeds incrementally and in dependency order:

1. Establish repository foundations, toolchains, documentation, licensing analysis, CI, upstream pinning, and the build-orchestration decision.
1. Inventory upstream features, APIs, tests, examples, build systems, Box2D ancestry, and LiquidFun-specific changes.
1. Define the compatibility matrix, numerical policy, reference harness, and Rust ownership/object model.
1. Port foundational math, shapes, collision primitives, dynamic tree, and broad phase.
1. Port rigid-body dynamics, contacts, solvers, sleeping, continuous collision detection, queries, and ray casts.
1. Port joints, callbacks, filters, listeners, and debug draw abstractions.
1. Establish broad rigid-body differential validation.
1. Port particle storage, lifecycle, spatial structures, contacts, and body contacts.
1. Port particle solvers, flags, behaviors, groups, and pair/triad logic incrementally.
1. Establish comprehensive particle differential validation.
1. Port examples and testbed scenarios with headless support.
1. Optimize from benchmarks and profiling, then harden safety, platforms, documentation, and release policy.
1. Complete a parity audit before publication claims.

The ordering is a starting hypothesis. Research may refine phase boundaries, but it must preserve dependency-aware, testable progress and early risk retirement.

## Historical expected early deliverables

Before substantial physics porting begins, the project should have:

- A refined project definition, scoped requirements, executable roadmap, risk register, and milestone acceptance criteria.
- `ARCHITECTURE.md`, `UPSTREAM.md`, `COMPATIBILITY.md`, `TESTING.md`, and dependency/licensing analysis.
- Decision records for build orchestration, the Rust object model, and differential testing.
- A pinned upstream Git submodule and documented reference build/test commands.
- Initial Cargo workspace scaffolding, a root `justfile`, pinned Rust toolchain, and minimal CI for both the Rust skeleton and upstream reference implementation.
- A verified subsystem/API inventory and initial compatibility traceability matrix.

## Historical phase context

The snapshots below describe their original phases, not current work or next steps. Current State and Next Milestone Goals above supersede their unfinished-target wording. The completed hobby milestone does not claim full parity.

- Phase 1 is complete: the repository now contains a Cargo-first Rust scaffold, private orchestration, an immutable upstream oracle, reproducible CMake/Ninja build commands, compatibility/provenance records, package isolation, and separated CI workflows. Broad physics behavior is not implemented yet.
- Phase 2 is complete: the repository now has a strict semantic JSONL contract, native Rust and process-isolated C++ adapters, typed comparison and failure taxonomy, replay/minimization and evidence lifecycles, and verified empty-world one-shot, reuse, and sanitizer round trips. This proves the harness seam, not broad physics parity.
- Phase 3 is complete: the native crate now has typed world-scoped identities, explicit stale/cross-world/cross-system failures, checked arena retirement, upstream-ordered destruction cascades with pre-mutation snapshots, restricted step hooks, bounded deferred commands, panic poisoning, typed association side tables, and stable particle identity over transactional dense permutations. Broad solver behavior and the complete Phase 9 particle-buffer API remain deferred.
- Phase 4 is complete: consumers have documented source-ordered scalar, vector, matrix, rotation, transform, sweep, and fixed-setting APIs; a closed 25-path policy defines special values, comparison modes, horizons, ordering, and authority tiers; and supervised Rust/C++ probes verify 39 ordered cases in debug and release. Local AppleClang evidence remains non-promotable D2, while canonical D1 execution remains pinned CI evidence.
- Phase 5 is complete: immutable checked shapes, source-ordered collision kernels, dynamic-tree and broad-phase behavior, and TOI are implemented and covered by the fail-closed collision differential probe.
- Phase 6 is complete: native bodies, fixtures, sensors, contact lifecycle, the minimal contact solver, deferred hook mutation, and semantic rigid-world differential execution now work end to end. The next target is Phase 7's complete rigid solver, sleeping, world operations, and CCD; broad rigid parity remains unclaimed.
- Phase 11 is complete: every upstream corpus row has a terminal reviewed disposition; the shared renderer-neutral catalog drives headless execution, exact oracle comparison, regressions, benchmarks, and the private interactive testbed; the published crate remains renderer-free. Phase 12 owns performance, portability, coverage, dependency hardening, and release readiness.
- Google LiquidFun extends the Box2D 2.3.0 / revision-280 lineage. Official commit `7f20402173fd143a3988c921bc384459c6a858f2` is the immutable behavioral oracle; `UPSTREAM.md` and ADR 0001 record the release-to-candidate delta and maintenance state.
- The upstream C++ implementation is the behavioral oracle during development, not a production dependency or the desired public architecture.
- Particle simulation is a central deliverable, not an optional extension after rigid-body work.
- The project is necessarily long-running and may span multiple milestones; compatibility status must therefore be visible and evidence-based throughout development.
- Safe abstractions may alter layout, identity, ordering, and performance relative to C++. These differences need deliberate designs and differential evidence rather than incidental behavior.
- Callback reentrancy, mutation during stepping, stable object identity, destruction invalidation, intrusive-list replacement, external particle buffers, and user data are early architectural risks.
- Numerical divergence may arise from floating-point precision, compiler optimization, SIMD, collection ordering, solver order, and platform behavior. Exact bit parity is not assumed where unjustified.
- Existing permissively licensed Rust implementations may be considered only after provenance, license, ancestry, API, and behavioral compatibility review.
- Repository tooling should make advanced workflows discoverable without hiding their underlying commands or making ordinary Cargo use opaque.

## Constraints

- **Implementation**: Production physics behavior must be native Rust — runtime delegation to upstream C++ is prohibited.
- **Reference isolation**: FFI and C++ builds are limited to differential testing, comparison, reference generation, benchmark comparison, and upstream test/example execution — published crates remain independent.
- **Build system**: Cargo is primary and sufficient for normal users — Bazel, CMake, or hybrid orchestration requires a documented evidence-based decision.
- **Upstream provenance**: The canonical source and exact revision must be pinned before implementation assumptions harden — moving branches are not acceptable references.
- **Licensing**: Upstream LiquidFun, Box2D, copied or translated code, tests, data, and all dependencies require explicit license review and attribution — final project licensing follows compatibility analysis.
- **Safety**: Safe Rust is the default — every `unsafe` block must be narrow, justified by a measurable need, document its invariant with a `SAFETY:` comment, and receive focused tests where practical.
- **API design**: APIs are experimental and may evolve; document incompatible changes before release. Preserve safe handles, checked mutation, explicit lifetimes/invalidation, and private storage details. Durable API guarantees are deferred.
- **Behavior**: Improve parity incrementally against the pinned upstream behavior; document known differences and retain regression protection. Exhaustive closure is an optional strict qualification goal.
- **Determinism**: Stable ordering and reproducible seeded scenarios take precedence over unproven parallel or SIMD gains — nondeterministic acceleration must be explicit.
- **Testing**: Meaningful semantic state must be compared — serialized raw memory alone is not an acceptable compatibility oracle.
- **Quality**: Production code avoids `unwrap()`, propagates errors, uses useful invariant messages for genuinely impossible states, documents public APIs, and follows the repository's Rust and Bright Builds guidance.
- **Angles and naming**: Full rotations use tau-based expressions, and optional internal values use `maybe_` naming where it improves clarity — project conventions remain consistent with repository standards.
- **Architecture**: Prefer cohesive deep modules and functional-core/imperative-shell separation — do not over-fragment crates or hide substantial foreign-language logic inside strings.
- **Rendering**: Simulation stays renderer-independent, optional, and headless — testbed framework choices cannot dictate core architecture.
- **Platforms**: One macOS Cargo CI job records tested coverage. Non-macOS platforms are best effort with optional manual testing; retain existing platform fixes and regressions.
- **CI cost**: Keep local checks and one macOS Cargo CI job. Cross-platform, differential, sanitizer, fuzz, coverage and benchmark suites are optional manual checks.
- **Transparency**: Documentation and README maturity claims must match verified implementation and compatibility evidence — incomplete parity is never marketed as complete.

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
| --- | --- | --- |
| Build a genuine Rust implementation rather than production C++ bindings | Independence, safety, idiomatic APIs, Cargo usability, and long-term maintainability are core goals | Accepted in Phase 1 architecture and package boundaries |
| Use a pinned upstream C++ LiquidFun revision as the behavioral oracle | Compatibility needs a stable, inspectable target and reproducible evidence | Accepted in Phase 1: `7f20402173fd143a3988c921bc384459c6a858f2` |
| Restrict FFI and C++ tooling to development-time comparison workflows | Published Rust users must not inherit a C++ runtime or toolchain dependency | Accepted in Phase 1; package isolation verified |
| Keep Cargo primary and sufficient for ordinary use | Rust consumers and contributors need a conventional, transparent workflow | Accepted in Phase 1; `liquidfun` is the sole default member |
| Evaluate Bazel or hybrid orchestration before adoption | Cross-language and CI orchestration may help, but maintenance cost must be justified | Bazel deferred in Phase 1 absent measured need |
| Prefer safe Rust and encapsulate any necessary unsafe code | Safety must not be traded away without measurable benefit and explicit invariants | — Pending |
| Treat compatibility inventory and differential testing as first-class product work | Parity claims require traceable evidence, not implementation intuition | Inventory accepted in Phase 1; differential protocol begins in Phase 2 |
| Design the Rust ownership, handle, callback, and user-data model before broad porting | C++ pointer and mutation semantics are foundational and expensive to revise late | Accepted and verified in Phase 3; future subsystem APIs must preserve the documented identity, callback, event, mutation, and storage boundaries |
| Keep particle systems in core scope | Particle behavior is the defining LiquidFun extension and cannot be deferred as optional polish | — Pending |
| Keep rendering optional and simulation headless | Core portability, testing, server use, and framework independence depend on this boundary | Accepted and verified in Phase 11: the catalog/controller/checkpoint model is renderer-neutral, `liquidfun-testbed` is private and non-default, and package isolation rejects renderer dependencies from `liquidfun` |
| Prioritize correctness and parity before optimization | Premature layout, SIMD, or parallel decisions could hide incompatibilities and destabilize the API | Accepted in Phase 4 through source-ordered scalar kernels and fail-closed rejection of nonbaseline CPU, SIMD/FMA, contraction, and unsafe floating options |
| Require explicit compatibility sign-off per subsystem | Incremental, reviewable evidence prevents premature global parity claims | Accepted for math/settings in Phase 4, collision in Phase 5, and the minimal rigid-world slice in Phase 6; later subsystems must repeat scoped inventory, unit, differential, documentation, review, and security evidence |
| Keep a bounded WASM playground independent of native Cargo consumers | Visitors can explore scenes in a browser without making C++ or wasm-pack a published-crate requirement | Accepted in v1.1: private `liquidfun-wasm`, SolidJS Pages site, `liquidfun` remains the sole default member |
| Ship six native playground scenes instead of a scene editor or JS physics clone | Owner-approved catalog using existing engine capabilities | Accepted in v1.1: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, Water Wheel |
| Use semantic HTML plus Kobalte Dialog for the responsive playground shell | Accessible modal behavior without a broad design-system migration | Accepted in v1.1; recorded in `standards-overrides.md` |
| Deploy the playground from main with OIDC Pages, not a PAT | Automatic website delivery without reviving Linux native qualification | Accepted in v1.1 HOST-01..03 |
| Treat v1.1 as a planning label, not a package or git release tag | Archive completion is not publication | Accepted 2026-09-20; same policy as v1.0 |
| Use Dam Break Medium native pair as the v1.2 numeric gate (Rust ≤ 3× C++) | The existing exploratory pair already shows ~300×; a same-order close is the honest first bar | ✓ Achieved at stamp `2026-09-21T20-38-50Z`, ratio `2.956857456935513`; not a public claim |
| Profile and fix shared particle/rigid hot paths; do not revive the Phase 12 sealed 32-case public matrix | Phase 12 defined method, but the reviewed-report manifest is empty; this milestone is observability plus closing the canary | ✓ Shared scalar kernels plus a same-cluster spot-check; `reviewed_reports` stays empty |
| Hold the scalar deterministic compatibility baseline while chasing the ~300× gap | Correctness and determinism still beat unproven SIMD/parallel defaults; SIMD/parallel stay explicit opt-in | ✓ Held: no default SIMD or Rayon, `unsafe_code = "forbid"` |
| Native is the C++ comparison; WASM/playground is a post-gate sanity check only | WASM vs C++ is not a fair pair; still record whether the playground improved | ✓ `just web-player-smoke` recorded; the note never compares WASM to `oracle-release` |
| Treat v1.2 as a planning label, not a package or git release tag | Archive completion is not publication | Accepted 2026-09-21; same policy as v1.0 and v1.1 |

## Open Questions

- Which handle, arena, lifetime, callback, user-data, and mutation model best balances safety, API ergonomics, identity stability, and upstream behavior?
- Which observables and tolerances define acceptable parity for each subsystem and platform?
- How should deterministic iteration and contact ordering be preserved without exposing internal storage?
- Which existing permissively licensed Rust components, if any, are compatible enough to reuse after audit?
- Which lightweight rendering stack best supports an optional upstream-equivalent testbed without coupling the core library?
- Which subsets can realistically support full-engine WASM, mobile, or `no_std` without distorting the native library? A bounded playground WASM wrapper exists; complete-engine WASM support is still unclaimed.

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):

1. Requirements invalidated? → Move to Out of Scope with reason
1. Requirements validated? → Move to Validated with phase reference
1. New requirements emerged? → Add to Active
1. Decisions to log? → Add to Key Decisions
1. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):

1. Full review of all sections
1. Core Value check — still the right priority?
1. Audit Out of Scope — reasons still valid?
1. Update Context with current state

______________________________________________________________________

*Last updated: 2026-09-21 after starting v1.3 Reference Testbed Scenes. Strict native certification remains optional. Package publication and release tags remain separately authorized.*
