---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 12-2026-07-23T15-16-59
generated_at: 2026-07-23T15:16:59.341Z
---

# Phase 12: Performance, Portability, and Release Hardening - Context

**Gathered:** 2026-07-23
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Turn the complete scalar native-Rust engine into an auditable v1 release candidate. This phase completes comparable performance evidence, platform and MSRV support, fuzz/Miri/sanitizer/coverage hardening, public and maintainer documentation, package and notice verification, and a fail-closed zero-gap release audit. It does not add new physics capabilities or adopt SIMD, parallel stepping, alternate precision, WASM, mobile, or `no_std`.

</domain>

<decisions>
## Implementation Decisions

### Performance evidence contract

- **D-01:** Use a tiered paired performance contract. One repository-owned runner must execute Rust and the pinned C++ oracle from the same sealed resolved scenario bytes, settings, measured horizon, optimization mode, hardware session, and versioned measurement policy. Retain Criterion for Rust-only microbenchmarks and trend diagnosis, but do not compare Criterion's adaptive loop directly with an independently configured C++ harness.
- **D-02:** Expand the existing benchmark catalog into a machine-checked `PERF-01` coverage matrix for world stepping, broad phase, narrow phase, contact solving, CCD, joints, particle lifecycle, particle contact generation, sorting, pressure, large particle systems, mixed worlds, queries, and ray casts. Use size sweeps where cardinality changes scaling, and keep setup, resolution, restart, and teardown outside measured intervals.
- **D-03:** Keep unprofiled wall-clock totals authoritative for benchmark comparisons. Extend storage-neutral phase profiles with common comparable parent phases and optional Rust-only diagnostic child phases. Profile names and boundaries are versioned evidence; profile durations remain performance diagnostics rather than D0/D1 physics-parity observables.
- **D-04:** Calibrate each controlled benchmark host with at least five independent baseline runs. A structural optimization may proceed only when a profile attributes at least 10% of the relevant workload to the target or records a concrete allocation, cache, or scaling bottleneck; the paired 95% confidence interval must clear a predeclared practical threshold of at least `max(3%, calibrated noise floor)`; no mandatory workload may regress beyond its calibrated threshold; and differential, determinism, safety, and public-API gates must remain green.
- **D-05:** Bind every public performance claim to an immutable machine-readable report containing scenario and resolved-content hashes, Rust and oracle revisions, compiler and linker identities, target, effective flags, hardware, warm-up and sample policy, raw measurements, intervals, profile schema, and compatibility status. Claims must name the workload and may not generalize beyond the recorded evidence.

### Platform and MSRV support contract

- **D-06:** Use a layered artifact-first support contract. Treat Linux x86_64, Linux ARM64, macOS ARM64, and Windows x86_64 as durable supported v1 targets. Treat macOS x86_64 as conditional-supported only while a sustainable native runner remains available. Other targets and cross-compiles are evidence-only unless a later reviewed support decision promotes them.
- **D-07:** Keep Rust 1.92.0 fixed as the v1.0.x MSRV and Rust 1.97.0 as the reproducible development pin. Verify the same reviewed `.crate` artifact at the MSRV on canonical Linux and under Rust 1.97.0 across the native supported-platform matrix; do not build a low-value two-toolchain-by-five-target Cartesian gate.
- **D-08:** Extend package verification so every platform lane proves the identical unpacked artifact, feature surface, source isolation, and `rust-version` contract. Ordinary package verification must remain Cargo-only and submodule-free.
- **D-09:** Keep numerical authority orthogonal to platform support. Only canonical scalar Linux x86_64 may produce D1 fixture-promotion evidence. Successful supported-platform and compiler-variation runs are D2 portability evidence and may never overwrite canonical fixtures.
- **D-10:** Document a fail-closed downgrade policy for conditional macOS x86_64 support: loss of sustainable native CI changes the target's documented tier and release evidence rather than silently preserving an unverified support claim.

### Safety, validation, and release gate

- **D-11:** Authorize a parity-bearing v1 only through a commit-bound evidence manifest and a fail-closed `cargo xtask release audit`. The manifest must bind every accepted result to the exact candidate commit, toolchain and target identity, producer workflow, artifact hash, evidence kind, and reviewed status; mixed-commit or incomplete evidence is rejected.
- **D-12:** Keep fast deterministic checks on pull requests and expensive randomized differential, fuzz, Miri, sanitizer, coverage, benchmark, and broad platform suites in scheduled or explicit release-candidate lanes. The release audit aggregates their commit-bound artifacts instead of duplicating all work in one fragile all-in-one job.
- **D-13:** Add bounded fuzz targets for parsers, shapes/collision, world mutation, particles/groups, and unsafe or ownership boundaries that exist. Every accepted crash or mismatch becomes a minimized provenance-bearing regression with target, exact input, generator/toolchain identity, candidate commit, and failure classification. Physics mismatches remain distinct from harness, sanitizer, timeout, and schema failures.
- **D-14:** Run Miri on supported pure-Rust subsets with no C++ process dependency. Keep Rust safety/sanitizer evidence separate from C++ ASan/UBSan evidence. Generate Rust and C++ coverage separately and report exercised subsystems and differential coverage; coverage percentages alone never prove parity.
- **D-15:** Preserve the workspace `unsafe_code = "forbid"` policy as the v1 zero-unsafe claim. Any proposal to introduce unsafe, SIMD, parallelism, or a structural fast path requires a separate ADR, a safe behavioral baseline, a narrow `SAFETY:` invariant, focused tests, measured need, and all compatibility gates; absent that evidence, Phase 12 must not weaken the prohibition.
- **D-16:** The final auditor must reject release readiness unless rustdoc and the README accurately document units, invariants, errors, callbacks, mutation, ownership, MSRV, platforms, and maturity; `COMPATIBILITY.md`, `BENCHMARKING.md`, `SAFETY.md`, `CONTRIBUTING.md`, and release documentation are complete; package contents and required notices pass; every upstream test/example has a terminal reviewed outcome; and no unexplained compatibility gap remains.

### the agent's Discretion

- Exact private module, command, manifest, artifact, benchmark case, profile, CI job, and report names within the locked contracts.
- Exact benchmark sample counts, warm-up duration, size points, fuzz budgets, Miri partitioning, coverage presentation, and artifact-retention mechanics, provided the versioned evidence records make them explicit and the locked minimum thresholds remain enforced.
- Exact plan decomposition and whether the release audit consumes one aggregate manifest or a typed manifest set, provided commit identity and fail-closed completeness remain authoritative.

</decisions>

<canonical-refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and inherited decisions

- `.planning/PROJECT.md` — Native Rust, oracle isolation, deterministic scalar baseline, safety, platform, CI-cost, and truthful maturity constraints.
- `.planning/REQUIREMENTS.md` — `FND-06`, `COMP-10`, `API-11`, `API-12`, `TEST-05` through `TEST-08`, `PERF-01` through `PERF-06`, `PLAT-01` through `PLAT-06`, and `DOCS-01`, `DOCS-04`, `DOCS-06` through `DOCS-09`.
- `.planning/ROADMAP.md` — Fixed Phase 12 goal, success criteria, Phase 11 dependency, and release/MSRV/performance research flags.
- `.planning/phases/01-oracle-provenance-and-repository-foundation/01-CONTEXT.md` — Toolchain, provisional MSRV, package isolation, canonical compiler, and evidence-governance decisions.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — Provenance, failure taxonomy, replay, minimization, exact scenario identity, and process-isolated oracle contract.
- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md` — Scalar IEEE baseline, deterministic operation order, compiler policy, and D0-D3 evidence tiers.
- `.planning/phases/08-joints-rope-callbacks-and-rigid-sign-off/08-CONTEXT.md` — Rigid sign-off, diagnostics, and profile separation from parity evidence.
- `.planning/phases/10-particle-groups-solvers-and-compatibility-sign-off/10-CONTEXT.md` — Complete particle behavior, evidence authority, and Phase 12 hardening boundary.
- `.planning/phases/11-examples-headless-tooling-and-testbed/11-CONTEXT.md` — Shared resolved scenario catalog, benchmark eligibility, profile semantics, upstream corpus closure, and Cargo/package isolation.

### Existing implementation and evidence seams

- `Cargo.toml` and `rust-toolchain.toml` — Workspace membership, sole default package, Rust 1.92 MSRV, Rust 1.97 development pin, lint policy, and `unsafe_code = "forbid"`.
- `crates/liquidfun-benchmarks/Cargo.toml`, `crates/liquidfun-benchmarks/src/lib.rs`, and `crates/liquidfun-benchmarks/benches/catalog.rs` — Existing private benchmark catalog, Criterion entrypoint, and package isolation.
- `crates/liquidfun-testbed/src/capability/fixture.rs` and `crates/liquidfun-testbed/src/capability/report.rs` — Scenario eligibility and structural profile capability seams.
- `crates/liquidfun/src/world/step.rs` — Public step profile and semantic step-report boundary.
- `crates/liquidfun-differential/src/runner.rs` — Existing native/oracle scenario execution, failure, replay, and evidence orchestration.
- `tools/xtask/src/package.rs` and `tools/xtask/src/package/` — Current package metadata, isolation, and unpacked-crate verification.
- `tools/xtask/src/inventory.rs` and `tools/xtask/src/inventory/` — Machine-authoritative compatibility and generated-report validation.
- `.github/workflows/ci.yml` — Cargo-only quality, default-feature, package, documentation, and provisional MSRV lanes.
- `.github/workflows/oracle.yml` — Canonical Linux, sanitizer, Phase 11 evidence, macOS, and Windows oracle lanes.
- `justfile` — Thin contributor command facade that Phase 12 commands must preserve.

### Documentation and release authority

- `README.md` — Current public entrypoint and maturity claims to audit.
- `TESTING.md` — Existing test layers, D0-D3 evidence, sanitizer, replay, promotion, and CI-tier contracts.
- `COMPATIBILITY.md` and `reference/compatibility.json` — Generated compatibility report and machine-authoritative zero-gap inventory.
- `UPSTREAM.md`, `reference/upstream-lock.json`, and `LICENSE` — Pinned oracle identity, provenance, notices, and licensing boundary.
- `.planning/research/STACK.md` — Recommended benchmark, coverage, fuzzing, Miri, sanitizer, MSRV, and CI tooling.
- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — Repo-local GSD, Markdown, Rust, verification, and exception rules.
- `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` — Deep-module, invariant, focused-test, sync, and pre-commit requirements.

</canonical-refs>

<code-context>
## Existing Code Insights

### Reusable Assets

- The private `liquidfun-benchmarks` crate already uses Criterion and validates that benchmark dependencies do not enter the published package.
- Phase 11's resolved scenario catalog and benchmark-eligibility records already provide one workload definition for Rust, C++ oracle, regression, and testbed consumers.
- `DiagnosticStepProfile` and the testbed capability report already separate structural phase names from nondeterministic durations.
- Existing `xtask` package, inventory, provenance, phase-evidence, promotion, and documentation checks provide typed fail-closed patterns for a release auditor.
- Cargo CI already has Rust 1.92 MSRV and multi-OS default-feature jobs; Oracle CI already has canonical Linux, fail-fast sanitizer, macOS, and Windows seams.

### Established Patterns

- `liquidfun` remains the sole default publishable crate. Benchmark, differential, protocol, testbed, C++, fuzzing, and release orchestration stay private and non-default.
- Machine-readable typed records are authoritative; Markdown is a deterministic human projection.
- Exact candidate, upstream, compiler, target, flags, scenario, and policy identities bind evidence. D1 promotion is stricter than D2 portability evidence.
- Expensive evidence may run outside the fast pull-request lane, but the checked-in gate must fail closed on missing, stale, mixed-identity, or unexplained results.

### Integration Points

- Extend the scenario catalog and private benchmark crate with the complete performance matrix and paired runner.
- Extend the public profile vocabulary without exposing storage or making durations compatibility observables.
- Add platform/package artifact fan-out to existing Cargo and oracle workflows while preserving Cargo-only isolation.
- Add typed release-evidence and audit modules to `xtask`, then project their status into release documentation and the final compatibility report.

</code-context>

<specifics>
## Specific Ideas

- Interleave paired Rust/C++ samples on one controlled host so thermal, frequency, and background-load drift affect both engines within the same measurement session.
- Treat benchmark instrumentation overhead as a separately measured diagnostic; never silently include profiled totals in unprofiled public comparisons.
- Reuse one reviewed `.crate` archive across native platform lanes so platform evidence proves the artifact users actually receive.
- Make the final release decision reproducible from hashed artifacts and typed manifests rather than a human checklist or GitHub status page alone.

</specifics>

<deferred>
## Deferred Ideas

- SIMD, parallel stepping, alternate precision, WASM, iOS, Android, and `no_std` remain post-v1 evidence-driven work.
- Dedicated long-lived benchmark hardware and additional platform tiers may be added later without weakening the v1 artifact and evidence contracts.
- A general unsafe optimization framework is not created speculatively; unsafe remains forbidden unless a separate measured ADR justifies a narrow exception.

</deferred>

***

*Phase: 12-performance-portability-and-release-hardening*
*Context gathered: 2026-07-23*
