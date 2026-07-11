---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 4-2026-07-11T04-16-20
generated_at: 2026-07-11T04:37:00.000Z
phase: 04-math-settings-and-numerical-policy
status: complete
requirements:
  - COLL-01
  - COLL-08
---

# Phase 4 Research: Math, Settings, and Numerical Policy

## Research conclusion

Phase 4 should establish four connected but independently testable seams:

1. a public, initialized, purpose-built `liquidfun::math` module whose source-order operations map directly to the pinned `b2Math` behavior;
2. a fixed `math::settings` constant surface with exact `f32` encodings, MKS/radian documentation, and no mutable global configuration;
3. a private pure-math probe protocol implemented by both native Rust and the pinned C++ adapter with bit-preserving input/output and complete compiler identity; and
4. a closed versioned numerical-policy registry that separates transport fidelity from semantic equality and names edge-case, ordering, horizon, and platform rules for every authoritative observable.

This split satisfies `COLL-01` without importing C++ accidents into the public API and satisfies `COLL-08` without putting approximate equality into consumer math types. Collision, rigid-body, and particle phases then consume one documented numerical contract rather than inventing local epsilons.

The Phase 4 context is authoritative. Its decisions were informed by three parallel advisor audits of the public math contract, compiler/IEEE behavior, and comparison/determinism policy. The repository standards materially require one cohesive Rust module, invariant-bearing boundary types, pure comparison logic, focused Arrange/Act/Assert tests, transparent C++ orchestration, and the full Rust verification sequence.

## Requirement coverage strategy

| Requirement | Planning implication |
| --- | --- |
| `COLL-01` | Implement and document every scoped upstream math concept—vectors, rotations, transforms, sweeps, matrices, constants, predicates, operation order, units, and deliberate safe-Rust differences—with unit/property tests and public examples. |
| `COLL-08` | Add explicit build identity, IEEE special-value rules, typed field policies, collection semantics, fixed divergence horizons, D0–D3 evidence tiers, pure Rust/C++ probes, and diagnostic first-divergence evidence. |

## Recommended architecture

### Public native-Rust math core

Use `crates/liquidfun/src/math.rs` as the deep module entrypoint with cohesive children under `crates/liquidfun/src/math/`:

- `vector.rs`: `Vec2`, `Vec3`, `Vec4`, source-ordered arithmetic, dot/cross/skew, length, normalization, validity, and focused edge-case tests;
- `matrix.rs`: `Mat22`, `Mat33`, column-major access, zero/identity constructors, solve/inverse operations with pinned determinant branch and expression order;
- `transform.rs`: `Rotation`, `Transform`, composition, inverse application, and explicit radians;
- `sweep.rs`: checked public construction/advance plus the exact private interpolation/normalization kernel;
- `settings.rs`: exact behavior constants, units, upstream spelling/source mapping, and bit-pattern tests;
- `scalar.rs`: compatibility helpers such as validity, ordered abs/min/max/clamp, power-of-two helpers, distance, and safe bit-based inverse square root.

`math.rs` should curate the stable public exports and rustdoc. Keep representations private where invariants or future layout freedom matter. Do not implement `Eq`, `Hash`, `Ord`, `repr(C)`, raw-slice access, unchecked indexing, or a public approximate-equality trait. Ordinary physics constructors validate non-finite inputs at their domain boundary; raw math remains capable of carrying all IEEE encodings for probes.

Do not use a general-purpose math dependency. Its operation grouping, transcendental implementation, feature flags, layout, and API would become an unreviewed parity input. Do not mechanically substitute `f32::abs`, `min`, `max`, `clamp`, `sin_cos`, or `mul_add`: the pinned source's branch direction and evaluation order are observable for signed zero, NaN, and rounding.

### Settings surface

Translate behavior-affecting collision, dynamics, particle, and sleep constants as uppercase Rust `f32`/integer constants with the same expression grouping and exact bits. Preserve `PI` for source mapping and expose `TAU`; use tau-based full-turn expressions per repository guidance. Document MKS units, radians, column-major matrices, transform direction, sweep fractions, and scale guidance.

Exclude allocator hooks, logging/version strings, debug/assert macros, C typedefs, invalid dense-index sentinels, and the 16-bit particle-index switch. Those are C++ infrastructure or contradict the Phase 3 private-index and Rust ownership decisions.

### Probe boundary

Add a private probe request/result model to `liquidfun-test-protocol`; keep it closed, bounded, versioned, and exact-bit based. A probe case should carry:

- stable probe ID and operation kind;
- exact `u32` operand bits and structured scalar/vector/matrix/sweep inputs;
- operation or repeated-composition horizon;
- platform/evidence tier and build identity;
- exact output bits plus float class/sign and discrete branch results.

Implement the native adapter in `liquidfun-differential` or a focused child module and the C++ adapter in checked-in `tools/reference/src/` files. Do not embed substantial C++ in Rust strings or CMake. The C++ conversion boundary must use representation-preserving copying, not arithmetic reconstruction, so NaN payloads and subnormals survive transport probes.

The corpus should include ordinary values plus signed zeros, min/max subnormal, min normal, max finite, infinities, several NaN encodings, cancellation, halfway rounding, overflow/underflow, an FMA witness, ordered scalar helpers, inverse square root, normalization around `EPSILON`, singular and near-singular matrices, tau fractions, large angles, transform composition, and sweep endpoints. Run debug and release variants; differing optimization output is evidence, not permission to mutate a policy.

### Compiler and build identity

Canonical C++ math/probe translation units need reviewed scalar flags or verified equivalents that disable fast math and contraction, preserve signed zeros/NaNs/infinities, and request IEEE denormal handling. Configuration must reject fast-math, reassociation, reciprocal approximation, native CPU tuning, and unsafe-math flags in the canonical lane.

Canonical Rust uses Rust 1.97.0, baseline target features, ordinary operators, and no explicit FMA in translated expressions. Build identity must include the effective math/probe compile command hash, compiler version, target triple, CPU/features, SDK/sysroot where applicable, optimization, contraction/denormal flags, Rust profile/codegen settings, feature set, OS/libc/libm identity, and runtime checks for rounding/denormal behavior.

Extend `tools/reference/src/build_identity.hpp.in`, CMake configuration, the protocol handshake, and the Rust validation side together. Treat missing or contradictory identity fields as a harness/provenance failure before physics values are compared.

### Numerical policy model

Extend `crates/liquidfun-test-protocol/src/tolerance.rs` rather than introducing a separate ad hoc comparator. The registry should be a closed domain model whose deterministic hash covers:

- semantic path/observable type;
- `ExactBits`, `Ulps`, `Absolute`, or `AbsoluteRelative` parameters;
- signed-zero rule;
- non-finite rule;
- `Ordered`, `Set`, or `Multiset` semantics;
- `Operation`, `PhaseLocal`, or `ScenarioSteps(n)` horizon;
- D0–D3 platform/evidence tier;
- source/probe justification.

Keep exact comparison for identifiers, counts, flags, predicates, branch/termination results, constant bits, checkpoint identity, and solver-visible order. Use ULP bounds only for finite bounded local kernels, absolute-relative bounds for composite values, and absolute bounds only for dimensioned residuals with a justified scale. Thresholds come from pinned-source analysis and reviewed probe evidence; there is no default epsilon, observed-maximum auto-headroom, runtime widening, or tolerance growth with elapsed steps.

Arithmetic NaN is a mismatch. Exact NaN payload comparison belongs only to transport/pass-through probes. Infinities match only by identical sign in explicitly permitting fields. Signed zero is distinct unless a named field proves it semantically unobservable.

D0 requires byte-identical same-build replay. D1 is the canonical pinned Linux x86_64 scalar lane and alone may promote golden evidence. D2 supported targets preserve exact structural/order rules and reviewed numeric policies but cannot promote canonical fixtures. D3 experimental modes are diagnostic only.

### Comparator and diagnostics

Keep the comparator a pure functional core. Extend `crates/liquidfun-differential/src/comparator.rs`, `report.rs`, and related semantic-path types so a first numeric divergence records expected/actual bits and decimal diagnostics, float class/sign, absolute/relative/ULP differences, applied policy and thresholds, horizon, tier, request/scenario/seed/generator identity, build identity, and bounded neighboring context.

Preserve one stable primary failure signature for replay and minimization. A bounded sibling summary may help diagnosis, but unbounded trace dumps must not replace first-divergence evidence. Harness, schema, provenance, timeout, sanitizer, and physics mismatches remain distinct outcomes.

## Plan decomposition

The planner should create dependency-ordered plans rather than one cross-cutting mega-plan:

1. public scalar/vector/settings foundation and exact-bit tests;
2. matrices, rotations, transforms, and sweeps with checked public invariants and unit/property tests;
3. numerical-policy domain types, hashes, comparator semantics, and diagnostic tests;
4. C++/Rust pure-probe protocol, bit-preserving transport, compiler identity, CMake flags, and scenario corpus;
5. differential evidence, documentation/compatibility updates, package isolation, and full verification.

Plans 1 and 3 may begin in parallel if they avoid the same files. Plan 2 depends on plan 1. Probe protocol work depends on stable math operation names and numerical policy types. Final evidence/documentation depends on all implementation plans.

Every plan must carry `COLL-01` or `COLL-08` in frontmatter as appropriate, and the complete set must cover both. Every task needs concrete `read_first`, action, verification, and acceptance criteria. Plans should avoid sharing hot files within one wave.

## Validation strategy

Unit tests should focus on one concern with Arrange/Act/Assert and exact bits where appropriate:

- constant encodings and units;
- signed-zero/NaN operand-order helpers;
- normalization at, below, and above epsilon;
- singular/near-singular matrix branches;
- rotation/transform/sweep endpoints and invariants;
- safe inverse-square-root bit algorithm;
- exact/ULP/absolute/absolute-relative policy boundaries;
- non-finite and signed-zero policy classification;
- horizon/tier/profile hashing and rejected provenance;
- bit transport for every exceptional float class.

Property tests should cover vector algebra identities only where floating-point conditioning makes the property meaningful, matrix/transform round trips under bounded well-conditioned inputs, sweep monotonicity, and comparator symmetry/boundaries. Do not assert algebraic identities that assume reassociation.

Integration evidence should run native and C++ probes through both debug and release profiles, validate complete identity, compare under the reviewed profile, repeat D0 cases for byte stability, and prove a deliberate mismatch yields the expected first-divergence signature. Ordinary `cargo build/test -p liquidfun` and `cargo package -p liquidfun` must remain independent of the submodule and C++.

Before each commit, run in order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

Run repository-specific protocol/reference/package checks for touched paths, then review the diff. Do not claim supported-platform or broad solver parity from a local noncanonical machine.

## Threat model and security notes

The phase has no network, authentication, or database surface, but ASVS L1 planning should still address integrity and resource threats:

- malformed or oversized probe inputs must pass existing bounded protocol validation before allocation or execution;
- operation kinds and CMake presets remain allowlisted; never execute command strings from probe payloads;
- compiler/build identity and policy hashes are untrusted boundary data and must be parsed strictly before comparison;
- NaN/denormal payloads must not trigger undefined behavior; C++ bit conversion uses `memcpy`/equivalent and Rust uses safe bit methods;
- failure evidence and logs remain bounded and contain no environment secrets or arbitrary compiler command expansion;
- canonical fixture promotion remains an explicit reviewed action and cannot be triggered by a comparison run;
- `unsafe` stays forbidden in `liquidfun`; any C++ representation copying is narrowly auditable and tested.

No high-severity threat is expected if these controls are implemented. A plan that introduces shell interpolation from probe input, unchecked lengths, implicit fixture promotion, or unsafe Rust is blocking.

## Key pitfalls

- Replacing upstream ordered scalar helpers with Rust methods changes signed-zero and NaN behavior.
- `sin_cos` or `mul_add` can alter rounding relative to separate pinned calls.
- Exact bits everywhere creates false cross-platform failures; a global epsilon hides real branch/order defects.
- Arithmetic float reconstruction corrupts NaN payload and subnormal transport evidence.
- Clang's ordinary precise defaults may still contract expressions; invisible target features are provenance bugs.
- `HashMap` iteration or indiscriminate sorting can erase solver-significant order.
- Increasing tolerances with step count converts a horizon into a hidden forgiveness policy.
- Public runtime-tunable settings would undermine the fixed compatibility baseline.
- Copying C++ uninitialized constructors, raw indexing, allocator hooks, or dense-index sentinels would violate the safe Rust architecture.
- Updating docs to claim D1/D2 parity before canonical and supported CI evidence exists would violate transparency requirements.

## Canonical references

- `.planning/phases/04-math-settings-and-numerical-policy/04-CONTEXT.md`
- `.planning/ROADMAP.md` § Phase 4
- `.planning/REQUIREMENTS.md` `COLL-01` and `COLL-08`
- `.planning/phases/02-semantic-protocol-and-oracle-round-trip/02-CONTEXT.md`
- `.planning/phases/03-rust-object-model-and-storage-architecture/03-CONTEXT.md`
- `.planning/research/STACK.md`
- `.planning/research/ARCHITECTURE.md`
- `.planning/research/PITFALLS.md`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Math.h`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Math.cpp`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Common/b2Settings.h`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter01_Introduction.md`
- `third_party/liquidfun/liquidfun/Box2D/Box2D/Documentation/Programmers-Guide/Chapter03_Common.md`
- `crates/liquidfun-test-protocol/src/float_bits.rs`
- `crates/liquidfun-test-protocol/src/tolerance.rs`
- `crates/liquidfun-differential/src/comparator.rs`
- `crates/liquidfun-differential/src/canonical.rs`
- `crates/liquidfun-differential/src/report.rs`
- `tools/reference/CMakeLists.txt`
- `tools/reference/CMakePresets.json`
- `tools/reference/src/protocol_bits.cpp`
- `tools/reference/src/build_identity.hpp.in`
- `ARCHITECTURE.md`
- `TESTING.md`
- `COMPATIBILITY.md`
- `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md`

## RESEARCH COMPLETE

Phase 4 is ready for executable planning around the five decomposition areas above. The implementation can remain dependency-light, safe, Cargo-only for consumers, and behaviorally anchored to the pinned C++ oracle.
