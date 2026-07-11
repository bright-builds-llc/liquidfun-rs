---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T04:16:20.259Z
---

# Phase 4: Math, Settings, and Numerical Policy - Context

**Gathered:** 2026-07-10
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Implement the consumer-facing, purpose-built `f32` math and settings layer required by later collision and solver phases, prove its selected behavior with pure native-Rust and pinned-C++ oracle probes, and establish the versioned numerical compatibility policy that classifies floating-point edge cases, observable-specific tolerances, ordering, divergence horizons, compiler assumptions, and platform evidence tiers. This phase does not implement shapes, collision algorithms, rigid-body solvers, or particle solvers; later phases must consume the contracts established here.

</domain>

<decisions>
## Implementation Decisions

### Public math and settings contract

- **D-01:** Add one public, purpose-built `liquidfun::math` deep module inside the existing `liquidfun` crate. Do not add a general-purpose math dependency, a second published crate, or conversion-only private types that fail the consumer-facing `COLL-01` contract.
- **D-02:** Expose curated Rust types corresponding to the pinned upstream concepts: `Vec2`, `Vec3`, `Vec4`, `Mat22`, `Mat33`, `Rotation`, `Transform`, and `Sweep`. Rustdoc must map each type and operation to its upstream concept while documenting deliberate safe-Rust differences.
- **D-03:** Vector coordinates may be public and float-bearing values may implement `Copy`, `Clone`, `Debug`, `Default`, and `PartialEq` where semantics are clear. Do not implement `Eq`, `Hash`, or `Ord`; do not promise `repr(C)`, FFI layout, raw slices, unchecked indexing, or uninitialized defaults.
- **D-04:** Keep matrix, rotation, transform, and sweep representation private. Provide initialized constructors, accessors, column operations, explicit `ZERO` or `IDENTITY` constants, and safe invariant-bearing APIs. `Default` means zero for vectors and identity for rotation/transform; omit it where a matrix or sweep default would be ambiguous.
- **D-05:** Preserve pinned upstream operation order and expression grouping for vector arithmetic, dot/cross/skew, normalization, matrix solve/inverse, rotation/transform composition, sweep interpolation/advance/normalization, distance, validity, power-of-two helpers, and the approximate inverse square root. Use safe `to_bits`/`from_bits` for bit algorithms; do not introduce `mul_add`, `sin_cos`, algebraic reassociation, or a general math crate's alternate semantics.
- **D-06:** Preserve compatibility-significant helper behavior rather than substituting superficially similar Rust methods. Implement upstream-ordered `abs`, `min`, `max`, and `clamp`; normalization returns `0.0` below `EPSILON`; singular matrix solve/inverse returns zero; `Rotation::from_angle` uses separate sine and cosine calls and does not silently renormalize compositions.
- **D-07:** Raw math values may represent signed zero, subnormals, infinities, and NaNs so pure probes can classify them. Ordinary physics/domain boundaries reject non-finite inputs. Invariant-bearing `Sweep` construction and advancement use checked APIs with typed errors for non-finite, decreasing, or out-of-range fractions, while exact arithmetic kernels remain private and probeable.
- **D-08:** Place fixed compatibility constants in `math::settings` with uppercase Rust names, exact `f32` values and expression grouping, documented upstream spellings, and MKS units. They are behavior-defining constants, not mutable runtime-global knobs. Retain `PI` for source mapping, expose `TAU`, and write full-rotation expressions in tau-based form.
- **D-09:** Expose behavior-affecting collision, dynamics, particle, and sleep constants. Do not expose C typedefs, allocator callbacks, assertion/debug macros, version-scanning strings, the invalid dense particle-index sentinel, or the 16-bit particle-index toggle; these conflict with Rust ownership or Phase 3's private-index contract.
- **D-10:** Document meters-kilograms-seconds, radians and radians-per-second, the upstream scale guidance, column-major matrix semantics, transform direction, and sweep interpolation conventions. Rendering and pixel conversion remain outside the physics layer.

### Floating-point and compiler semantics

- **D-11:** Canonical evidence uses scalar IEEE-754 binary32 behavior with round-to-nearest/ties-even, gradual underflow, signed zeros, NaNs, and infinities honored. Canonical C++ configuration explicitly disables fast math and contraction and requests IEEE denormals using reviewed Clang flags or verified equivalents; native CPU tuning, reassociation, reciprocal approximation, unsafe math, and implicit SIMD are prohibited.
- **D-12:** Canonical Rust builds use the pinned toolchain, baseline target CPU/features, ordinary source-ordered operators, and no explicit `mul_add` in translated upstream expressions. Debug and release probes both run; an optimization-dependent result is a policy finding, not permission to widen a tolerance automatically.
- **D-13:** Build identity must record or hash the effective compile command for math/probe translation units plus compiler, target triple, CPU/features, SDK/sysroot when applicable, optimization, contraction/denormal/fast-math flags, Rust profile/codegen settings, OS/libc/libm identity, feature set, and runtime floating-environment probe outcome.
- **D-14:** Pure probes accept and emit exact `u32` float bits with class and sign metadata. They cover `+0.0`, `-0.0`, min/max subnormal, min normal, max finite, infinities, multiple NaN encodings, cancellation, halfway rounding, overflow, underflow, an FMA witness, operand-sensitive helpers, inverse square root, normalization around epsilon, near-singular matrices, tau-fraction and large-angle rotations, transforms, and sweep endpoints.
- **D-15:** Representation transport is bit-preserving. Repair any protocol conversion that reconstructs floats arithmetically; C++ bit transport must use a representation-preserving copy mechanism and tests must prove all exceptional classes survive unchanged.

### Comparison, ordering, and divergence policy

- **D-16:** Keep exact `f32` bit transport separate from semantic comparison. Extend the Phase 2 closed, versioned tolerance model into a field-policy registry; every authoritative observable names its semantic path, float/discrete policy, signed-zero and non-finite policy, collection semantics, divergence horizon, platform tier, and source/probe justification. There is no wildcard or global epsilon and no runtime tolerance widening.
- **D-17:** Use exact comparison for IDs, flags, counts, membership, predicates, branches, termination results, constant bit patterns, checkpoint/phase identity, and solver-visible ordering. Use exact bits for finite elementary kernels only where canonical probes establish it, ULP bounds for bounded finite local math, absolute-relative bounds for composite vectors/transforms and later physical state, and absolute bounds only for dimensioned residuals with a defined physical scale.
- **D-18:** Any arithmetic NaN is an invariant or physics mismatch by default even when both sides produce NaN. Exact NaN payload comparison is allowed only for explicit transport/pass-through probes. Infinities match only with identical sign and only for fields that explicitly permit them. `+0.0` and `-0.0` remain distinct unless a named field policy proves the sign semantically unobservable.
- **D-19:** Collection semantics remain explicit: ordered checkpoints, phases, callbacks, destruction records, solver passes, and any sequence feeding future physics; sets for unique query results whose upstream order is unspecified; multisets where multiplicity matters. Canonicalize only order-insensitive collected results by stable semantic keys; never use hash iteration as observable order.
- **D-20:** Divergence horizons define evidence scope, not growing tolerances: one operation for pure math, one named algorithm phase for phase-local kernels, or an explicit `ScenarioSteps(n)` for repeated evolution. Compare every checkpoint with fixed field policies, fail on the first in-horizon mismatch, and label beyond-horizon results diagnostic rather than passing evidence.
- **D-21:** Establish four evidence levels: D0 same-build replay determinism must be byte-identical; D1 canonical parity uses pinned Rust 1.97.0 and Clang 22.1.8 on scalar Linux x86_64 and may generate canonical fixtures; D2 supported portability covers the declared Linux, macOS, and Windows targets with exact structural/order semantics and reviewed numeric policies but cannot promote canonical fixtures; D3 exploratory configurations such as alternate libm, FTZ/DAZ, SIMD, native CPU, or other targets provide diagnostic evidence only.
- **D-22:** Extend first-divergence evidence with horizon and platform tier, expected/actual bits and decimal diagnostics, absolute/relative/ULP differences, applied policy and thresholds, float class/sign, collection policy and canonical keys, request/scenario/seed/generator version, compiler/target/flags/features/policy hash, and bounded neighboring checkpoints. Preserve one stable primary failure signature for replay/minimization; keep harness, provenance, sanitizer, and physics outcomes distinct.

### Agent's Discretion

- Exact private file split and helper names inside `math`, provided the module remains cohesive and its public surface follows the locked concepts.
- Exact typed error names for invalid sweep/domain-boundary inputs.
- Exact probe record/schema names, scenario grouping, and bounded case counts.
- Initial numeric thresholds for non-exact fields, provided every value is derived from pinned-source analysis and recorded probe evidence rather than a global default or automatic headroom.
- Exact documentation layout across rustdoc, `TESTING.md`, `COMPATIBILITY.md`, and architecture/policy records, provided units, platform tiers, compiler assumptions, edge-case rules, horizons, and evidence limitations are discoverable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Scope and inherited contracts

- `.planning/ROADMAP.md` § Phase 4 — fixed phase goal, dependency, success criteria, and research/ADR flags.
- `.planning/PROJECT.md` — native-Rust, Cargo-only consumer, safety, determinism, semantic-testing, parity, platform, and transparency constraints.
- `.planning/REQUIREMENTS.md` — `COLL-01`, `COLL-08`, and related platform/evidence requirements.
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md` — exact float-bit transport, typed comparison, ordering, provenance, and first-divergence decisions inherited from Phase 2.
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md` — one-crate architecture, opaque storage, private dense indices, and deterministic ordering decisions inherited from Phase 3.

### Reconciled project research and documentation

- `.planning/research/STACK.md` — custom math recommendation, canonical toolchains, fast-math restrictions, differential stack, and platform strategy.
- `.planning/research/ARCHITECTURE.md` — semantic trace, comparator, ordering, and native-engine dependency direction.
- `.planning/research/PITFALLS.md` — floating-point, global-tolerance, ordering, provenance, and false-parity failure modes.
- `ARCHITECTURE.md` — current Phase 2/3 production, protocol, comparator, and storage boundaries.
- `TESTING.md` — current protocol contract, exact transport, outcome taxonomy, verification lanes, and deferred final tolerance policy.
- `COMPATIBILITY.md` — authoritative compatibility ledger entries for common math/settings and platform evidence.

### Pinned upstream behavior

- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Math.h` — pinned math types, inline helpers, operation order, rotation/transform/sweep behavior, predicates, and inverse square root.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Math.cpp` — pinned `Mat33` solve and inverse expression ordering.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Settings.h` — pinned constants, MKS units, compile-time options, and non-math surfaces that must remain private.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter01_Introduction.md` — upstream scale and radians guidance.
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter03_Common.md` — application-facing settings and math intent.

### Existing protocol and toolchain integration points

- `crates/liquidfun-test-protocol/src/float_bits.rs` — exact `f32` transport representation.
- `crates/liquidfun-test-protocol/src/tolerance.rs` — closed Phase 2 float, discrete, and collection policy model to extend.
- `crates/liquidfun-differential/src/comparator.rs` — current exhaustive comparator and IEEE edge-case behavior.
- `crates/liquidfun-differential/src/canonical.rs` — stable order-insensitive canonicalization patterns.
- `crates/liquidfun-differential/src/report.rs` — first-divergence and failure-signature model.
- `crates/liquidfun-differential/build.rs` — current Rust-side build identity input.
- `tools/reference/CMakeLists.txt` — repository-owned C++ adapter configuration and future canonical floating flags.
- `tools/reference/CMakePresets.json` — reference profiles and current compiler/build identity surface.
- `tools/reference/src/protocol_bits.cpp` — bit conversion boundary that must become representation-preserving for exceptional floats.

### Repository standards

- `AGENTS.md` and `AGENTS.bright-builds.md` — project constraints, GSD workflow, Rust verification gates, deep-module guidance, and sync-first requirements.
- `standards-overrides.md` — local exception registry; no substantive active override replaces the defaults.
- `standards/core/architecture.md` — functional-core and invariant-bearing domain guidance.
- `standards/core/code-shape.md` — shallow control flow, cohesive module, and diagnosable tooling guidance.
- `standards/core/testing.md` — focused behavior tests with Arrange/Act/Assert structure.
- `standards/core/verification.md` — repository-native pre-commit verification requirements.
- `standards/languages/rust.md` — Rust module shape, guards, optional naming, domain types, and verification guidance.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `crates/liquidfun`: existing safe, dependency-light publishable crate and integration root for the new public math deep module.
- `crates/liquidfun-test-protocol`: strict typed schemas, exact `FloatBits`, versioned tolerance identity, and collection policy types suitable for Phase 4 extension.
- `crates/liquidfun-differential`: exhaustive comparator, stable canonicalization, first-divergence reporting, replay/minimization, and failure bundles that can carry math-probe evidence.
- `tools/reference`: isolated C++ adapter build and protocol-bit boundary where pure pinned-upstream probes and complete compiler identity belong.

### Established Patterns

- Production physics stays in one native Rust crate; private harness crates and C++ adapters depend inward and never shape consumer build requirements.
- Boundary data is strict and invariant-bearing; float transport is exact bits while comparison is a separately versioned typed policy.
- Solver-visible order and evidence order are explicit, and machine-readable evidence is authoritative over generated presentation.
- Unsafe code remains forbidden in the production crate, making safe bit conversion and initialized math values the baseline.

### Integration Points

- Add `crates/liquidfun/src/math.rs` plus cohesive `math/` children and curate public exports from `lib.rs`.
- Extend private protocol/differential schemas and comparator diagnostics without exposing approximate equality in the public math API.
- Extend the reference adapter and CMake profiles with pure math/settings probes, explicit floating flags, complete build identity, and bit-preserving conversion.
- Update testing, architecture, and compatibility evidence so future collision and solver plans can consume one approved numerical policy.

</code_context>

<specifics>
## Specific Ideas

- Treat transport fidelity, operation parity, and semantic tolerance as three separate questions; none may silently substitute for another.
- Preserve odd pinned-upstream helper behavior, including operand-sensitive signed-zero results, when it is part of the selected oracle rather than “cleaning it up” to modern library semantics.
- Use repeated composition probes to expose drift, but never turn step count into an automatic epsilon multiplier.
- Keep canonical golden-data authority narrow and reproducible while still making supported-platform differences visible through explicit D2 evidence.

</specifics>

<deferred>
## Deferred Ideas

- Shape validation, overlap, distance, manifolds, clipping, broad phase, and time-of-impact behavior — Phase 5, consuming this phase's math and numerical policy.
- Rigid-body and particle solver-specific observable lists, tolerance values, and step horizons — their implementation phases, derived from this policy and subsystem evidence.
- SIMD, explicit FMA, native CPU tuning, fast-math, FTZ/DAZ, and parallel acceleration — experimental only until a later measured, separately reviewed mode proves value without weakening the canonical baseline.
- Public approximate-equality helpers or global epsilon configuration — intentionally excluded unless a future consumer requirement establishes a safe, non-parity API distinct from authoritative comparison.

</deferred>

***

*Phase: 04-math-settings-and-numerical-policy*
*Context gathered: 2026-07-10*
