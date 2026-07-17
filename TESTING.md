# Testing and Verification

## Current scope and maturity

Phase 2 proves a trustworthy empty-world harness seam: versioned and bounded
scenario input, native Rust execution, a process-isolated pinned C++ oracle,
typed semantic comparison, first-divergence diagnosis, deterministic reduction,
reviewed trace replay, and harness-failure classification. Phase 4 additionally
proves the native math/settings contract and a bounded 39-case Rust/C++ probe;
Phase 5 proves a fixed 78-case shape/collision corpus. Phase 6 adds checked body
and fixture ownership plus the first bounded contact lifecycle. Phase 7 adds
granular controls, configured deterministic multi-contact islands, warm
starting and sleeping, private resumable CCD, world queries and rays, and
origin shifting through one closed nine-family rigid-world request. Local
oracle results are D2 supported-toolchain evidence and exact two-run
determinism is D0. Phase 8 extends that request to 19 families covering joints,
standalone rope, callback/destruction timing, and reconstruction. Exact
canonical D1 evidence now supports only that accumulated scalar rigid corpus;
D3 review, other platforms, particles, performance, and the deferred fuzz,
Miri, Rust-sanitizer, benchmark, and coverage lanes remain pending.
[COMPATIBILITY.md](COMPATIBILITY.md) remains authoritative for feature and
evidence maturity.

## Required Rust sequence

Before every commit, run these commands in order:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

The root workspace defaults to the published `liquidfun` crate. Changes to
private tooling or harness crates also require the explicit workspace surface:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --all-features
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

CI runs `cargo fmt --all --check`, so format verification is read-only.

## Testing layer contract

This table is machine-audited by `cargo xtask docs check`. It contains exactly
one row for every required layer; every cell is an enforceable contract.

| Layer | Status | Purpose | Command | Prerequisites | Reports and failure artifacts | Retry policy | Placement | Semantic interpretation |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| unit | current | Exercise pure protocol/domain, comparator, reducer, and invariant behavior one concern at a time. | `cargo test --workspace --lib` | Rust 1.97.0 and Cargo-only workspace checkout; no submodule or C++. | Standard test output; a failure identifies the focused behavior and exact assertion. | No deterministic retry; fix or classify the failing behavior. | local, pull request | Focused behavior evidence supports its tested branch only and is not broad physics parity. |
| integration/API | current | Exercise supported private CLI, fixture lifecycle, supervisor, and crate-boundary workflows. | `cargo test --workspace --tests` | Rust 1.97.0; fake-child suites need only repository files and standard process support. | Standard test output plus bounded fake-child request, identity, report, and stderr diagnostics. | No deterministic retry; preserve the failing case and investigate it. | local, pull request | A pass proves the supported API workflow under test, not unimplemented engine behavior. |
| doctest | current | Compile and run public documentation examples independently from nextest-style suites. | `cargo test --workspace --doc` | Rust 1.97.0 with rustdoc; no oracle or external service. | rustdoc output names the failing crate and documentation example. | No deterministic retry; documentation and code must agree. | local, pull request | A documentation example proves only the documented API statement it exercises. |
| upstream compatibility | current | Verify immutable oracle identity and build the repository-owned external adapter. | `cargo xtask upstream verify` then `cargo xtask upstream configure --preset oracle-debug` and `cargo xtask upstream build --preset oracle-debug`. | Exact initialized submodule, reviewed CMake and Ninja, and the lane's recorded C++ compiler. | Tool/upstream identity diagnostics and out-of-tree build evidence under `target/reference`. | No deterministic retry; identity or build failures remain oracle infrastructure results. | local, pull request, scheduled, manual release | A successful oracle build proves oracle infrastructure, not Rust physics compatibility. |
| differential | current | Run the same validated rigid-world request through Rust and C++ and compare semantic traces. | `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` | Verified and built `oracle-debug` executable plus matching protocol, tolerance, and upstream identities. | Machine report on stdout; bounded request, identity, report, and stderr evidence belongs under `target/differential/failures` on failure. | No deterministic retry; a stable mismatch or harness failure must be diagnosed. | local, pull request, scheduled, manual release | Only validated traces can produce a physics mismatch; process, schema, sanitizer, provenance, and reset errors are harness failures. |
| property | deferred | Generate bounded valid values that probe geometry, ordering, handles, mutation, and later physics invariants. | Planned `cargo test --workspace property -- --nocapture` with persisted seeds and typed shrinkers. | Rust 1.97.0 now; reviewed generator version and deterministic seed model before activation. | Property test output plus the seed and minimized input promoted to an ordinary regression when confirmed. | No deterministic retry; reproduce the same invariant failure from the persisted value. | scheduled, manual release | A property pass samples an invariant domain and cannot prove exhaustive compatibility. |
| checked-in regression | current | Replay reviewed traces and later minimized scenarios that preserve an accepted first-divergence signature. | `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot` | Reviewed trace in the artifact manifest; later cases also require a minimized scenario under `scenarios/regressions`. | Replay report, manifest provenance, and any same failure signature evidence stored with the regression under `scenarios/regressions`. | No deterministic retry; byte-stable replay must be reproducible. | local, pull request, scheduled, manual release | A checked-in case protects one reviewed behavior or same failure signature from recurrence. |
| fuzz | deferred | Exercise bounded protocol decoders, scenario validation, world mutation, and future unsafe boundaries. | Planned `cargo fuzz run protocol_decode -- -max_total_time=300`; reproduce with `cargo fuzz run protocol_decode fuzz/artifacts/protocol_decode/<case>`. | pinned nightly, `cargo-fuzz`, reviewed target bounds, and no secrets or external service. | libFuzzer logs, exact crashing input, seed when present, and minimized corpus under `fuzz/artifacts`. | No deterministic retry; retain and minimize the exact input before fixing. | scheduled, manual release | A crash, timeout, sanitizer finding, or malformed-boundary defect is a harness failure, not a physics mismatch. |
| Miri/UB-aliasing | deferred | Detect undefined behavior and aliasing defects in the pure Rust subset and any future unsafe modules. | Planned `cargo miri test --workspace --all-features` on a date-pinned nightly after `cargo miri setup`. | pinned nightly with the Miri component; exclude the external C++ process from interpretation. | Miri diagnostics with the exact test, stack, flags, and pinned toolchain identity. | No deterministic retry; preserve the deterministic failing test and environment. | scheduled, manual release | Miri undefined behavior is a harness failure and safety defect, never a physics mismatch. |
| native sanitizer | current | Run the C++ protocol and oracle fail-fast, including the Phase 8 rigid adapter, and later run supported Rust sanitizer subsets without crossing findings into comparison. | `UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot` after fail-fast CTest. | Exact Clang 22.1.8 lane, configured and built `oracle-asan-ubsan`, ASan/UBSan runtimes; Rust sanitizer work additionally needs pinned nightly. | Bounded request, identity, report, and stderr evidence under `target/differential/failures`; CI uploads only this directory on failure. | No deterministic retry; sanitizer markers fail even if a child exits zero. | scheduled, manual release | Any ASan, UBSan, Rust sanitizer, signal, timeout, or reset defect is a harness failure, not a physics mismatch. |
| benchmark | deferred | Measure equivalent Rust and C++ workloads only after representative native behavior exists. | Planned `cargo bench --workspace` plus a paired oracle runner with recorded release identities. | controlled hardware, release builds, equivalent scenarios, warm-up policy, and recorded compiler flags. | Criterion reports under `target/criterion` plus paired environment and oracle identity records. | No deterministic retry; reruns form an explicitly analyzed sample rather than hiding regressions. | scheduled, manual release | Benchmark data is performance evidence, not parity or correctness evidence. |
| coverage | deferred | Report exercised Rust lines and branches separately from C++ oracle coverage. | Planned `cargo llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/rust.lcov`. | `llvm-tools-preview`, reviewed `cargo-llvm-cov` version, and separately compatible Clang tooling for C++. | Rust LCOV under `target/llvm-cov`; C++ coverage remains a separate report until LLVM compatibility is proven. | No deterministic retry; investigate deterministic coverage changes from the same suite. | scheduled, manual release | coverage is not parity; percentages do not replace semantic differential evidence. |

## Protocol contract and bounds

The four compatibility axes are independent and explicit:

- protocol version 1 defines JSON Lines framing, handshakes, requests, and
  streamed record kinds;
- scenario schema version 1 defines the bounded empty-world request;
- trace schema version 1 defines `trace_begin`, ordered checkpoints, and
  `trace_end` reset proof;
- tolerance profile version 1 is `phase2-v1`, whose reviewed hash is recorded in
  requests, traces, and artifact provenance.

Authoritative `f32` values use exact `u32` bits on the wire. The Phase-2 profile
compares simulation time exactly and world counts discretely; absolute,
absolute-relative, and ULP policies exist only as synthetic comparator tests.
There is no repository-wide numeric tolerance and no claimed rigid-body, joint, or particle
tolerance. NaN is a mismatch unless exact payload policy says otherwise,
infinities require the same sign, and signed zero stays distinct unless a future
reviewed field policy changes it.

`phase2-default-v1` bounds input and output records to 1 MiB, nesting to 32,
general strings to 4 KiB, semantic IDs to 128 bytes, entity/command/checkpoint
collections to 4,096, observables per checkpoint to 128, a trace to 32 MiB,
retained stderr to 256 KiB, and total child output to 64 MiB per request. Startup
and request deadlines are 5 and 10 seconds. `one-shot`, `reuse`, and `sanitizer`
are reviewed named profiles; the latter two use finite request budgets and do
not expose caller-selected limits.

The Rust and C++ boundaries reject blank or partial records, duplicate and
unknown members, unknown record kinds, unsupported versions, invalid semantic
IDs/references, malformed sequences, oversized values, request/identity/payload
mismatches, and missing reset proof before semantic comparison.

## Phase 4 numerical policy

Transport and comparison answer different questions. Every probe `f32` crosses
JSON Lines as an exact `u32` bit pattern, with class and sign metadata; C++ uses
representation-preserving copying and Rust uses `to_bits`/`from_bits`. Semantic
comparison then resolves the case's explicit path in the sorted, hashed
`protocol/tolerances/phase4-v1.toml` registry. There is no wildcard, global
epsilon, runtime widening, or tolerance growth with elapsed steps.

The four float policies are `ExactBits`, `Ulps`, `Absolute`, and
`AbsoluteRelative`. Exact comparison also governs IDs, flags, counts,
predicates, branch results, checkpoint identity, and solver-visible ordering.
Arithmetic NaN is a mismatch even when both sides produce NaN; exact NaN
payload comparison is reserved for named transport/pass-through evidence.
Infinities match only with identical sign when a field explicitly permits
them. Positive and negative zero are distinct by default and may be merged only
by a named field policy that documents why their sign is unobservable.

Collection policy is also explicit: `Ordered` preserves checkpoint, phase,
callback, destruction, and future solver-pass order; `Set` canonicalizes unique
order-insensitive results by stable semantic keys; `Multiset` does the same
while preserving multiplicity. Hash iteration is never observable order.

Divergence horizons bound the claim rather than changing its tolerance:
`Operation` covers one pure kernel, `PhaseLocal` covers one named algorithm
phase, and `ScenarioSteps(n)` covers exactly the declared repeated-evolution
checkpoints. Comparison stops at the first in-horizon mismatch; beyond-horizon
output is diagnostic only.

Evidence authority is fixed:

- D0 is same-build replay determinism and requires byte-identical output.
- D1 is canonical parity from pinned Rust 1.97.0 and Clang 22.1.8 on scalar
  Linux x86_64 with reviewed IEEE flags and runtime witnesses. Only D1 may
  promote canonical fixtures.
- D2 is supported Linux, macOS, or Windows portability with exact structure and
  order plus reviewed numeric policy. D2 cannot promote canonical fixtures.
- D3 is exploratory evidence for alternate libm, FTZ/DAZ, SIMD, native CPU,
  fast-math, or other noncanonical configurations and is diagnostic only.

Canonical Rust uses the pinned toolchain, baseline target features, ordinary
source-ordered operators, and no explicit fused multiply-add. Canonical C++
must reject fast math, reassociation, reciprocal approximation, native CPU
tuning, and unsafe-math flags; it disables contraction and records the effective
compile-command hash, compiler, target, CPU/features, optimization, floating
flags, SDK/sysroot, OS/libc/libm, Rust profile/codegen/features, and runtime
rounding/gradual-underflow witnesses. Debug and release are independent probes;
an optimization difference is a policy finding, not permission to widen bounds.

The 39-case corpus covers special values, cancellation, halfway rounding,
overflow, underflow, a non-fused FMA witness, ordered helpers, inverse square
root, epsilon-adjacent normalization, matrices, rotations, transforms, and
sweeps. Its evidence is limited to those named operations and horizons; it does
not prove collision, solver, particle, performance, or complete platform parity.

## Phase 4 math-probe commands

Initialize the pinned submodule, then verify, configure, and build both reviewed
profiles before running the commands:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask upstream configure --preset oracle-release
cargo xtask upstream build --preset oracle-release
cargo xtask differential compare --scenario math-probes --preset oracle-debug --session-profile one-shot
cargo xtask differential compare --scenario math-probes --preset oracle-release --session-profile one-shot
cargo xtask differential replay --scenario math-probes --preset oracle-debug --session-profile one-shot
cargo xtask differential verify-determinism --scenario math-probes --preset oracle-debug --runs 2
```

The corresponding aliases are `just math-probes-debug`,
`just math-probes-release`, `just math-probes-replay`, and
`just math-probes-determinism`. Inputs are closed to the checked-in
`scenarios/phase-04/math-probes.json`; outputs and build products stay below
`target/reference` and `target/differential`. The policy registry and its hash,
validated build identity, generated protocol schemas, command output, and
first-divergence report are the evidence locations. Compare, replay, D0,
portability, and CI commands are read-only and have no fixture-promotion path.

## Phase 5 collision comparison policy

The fixed `phase5-v1` registry declares every collision observable separately.
Exact `u32` bit transport preserves the representation crossing JSON Lines;
field comparison is a later typed decision. Discrete result tags, shape child
indices, manifold features and point order, support pairs, cache outcomes and
reason precedence, branches, caps, iteration counts, and broad-phase pair order
compare exactly. All current finite collision fields begin with named
`ExactBits` policies; there is no wildcard or runtime widening. Arithmetic NaN
is rejected, signed zero remains distinct, and any future ULP or dimensioned
absolute/relative rule requires its own semantic path and canonical evidence.

The registry uses `Operation` for shape construction, unary queries, overlap,
clipping, and feature transitions. Distance, manifolds, pair dispatch, tree,
broad phase, filtering/refiltering, and time of impact use `PhaseLocal`.
Operation and phase-local horizons bound only the claim; they never multiply a
tolerance. Broad-phase pairs and collision features are `Ordered`. Ordinary
tree query and ray results are unique `Set` membership, so their callback order
is not a consumer contract.

Evidence tiers remain independent. D0 requires two byte-identical same-build
runs. D1 requires the canonical pinned Rust 1.97.0 and Clang 22.1.8 scalar Linux
x86_64 environment and is the only tier that may promote canonical fixtures.
D2 records supported local platform evidence under the same structural and
numeric policies but cannot promote. D3 is diagnostic only. The successful
Phase 5 local Apple Clang 21.0.0 debug/release comparisons are D2-scoped; they
do not establish D1 or cross-platform validation.

The 78-case `collision-probes` scenario is declaration-first and fail-closed.
Each case declares one required witness family and an accepted or rejected
outcome. Native Rust must satisfy the declaration, then the C++ oracle must
satisfy it, before the engines are compared. The corpus covers safe invalid
shape rejection, four shape kinds and checked children, unary queries, cold and
semantic cache replay outcomes, distance/overlap/clipping, every supported
manifold pair and reversal, feature transitions, dynamic-tree and broad-phase
ties/lifecycle/filter/refilter behavior, and checked TOI states and cap
witnesses. It proves only those operations and horizons.

The diagnostic feature `differential-internals` is non-default,
`#[doc(hidden)]`, development-only, and enabled solely by the unpublished
workspace differential crate. It transfers bounded owned typed diagnostics and
does not expose raw storage, mutable cache state, packed contact keys, private
tree coordinates, or unchecked constructors. Default/no-feature rustdoc and
package verification prove the module is absent for ordinary consumers.

Phase 5 has no bodies, fixtures, contact manager, contact creation,
persistence/destruction, waking, joint suppression, listeners, impulses, or
rigid stepping. `COLL-05` therefore records only ordered pair generation plus
pure filter/refilter reconsideration; the world contact lifecycle remains
pending Phase 6.

## Phase 5 collision-probe commands

Initialize the pinned submodule, verify it, then configure and build both
reviewed profiles:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask upstream configure --preset oracle-release
cargo xtask upstream build --preset oracle-release
```

Run the closed read-only comparison and replay commands exactly:

```bash
cargo xtask differential compare --scenario collision-probes --preset oracle-debug --session-profile one-shot
cargo xtask differential compare --scenario collision-probes --preset oracle-release --session-profile one-shot
cargo xtask differential replay --scenario collision-probes --preset oracle-debug --session-profile one-shot
cargo xtask differential verify-determinism --scenario collision-probes --preset oracle-debug --runs 2
```

The commands accept no external scenario path, executable, policy, destination,
or arbitrary determinism count. Inputs are the checked-in 78-case scenario,
`phase5-v1` policy, exact adapter/build identity, and reviewed presets. Outputs
are machine reports; bounded failure evidence belongs under
`target/differential/failures`. A comparison pass is D2 evidence on a supported
local toolchain, replay protects the reviewed corpus, and the two-run command is
D0 byte identity. None proves contact lifecycle, a solver, another platform, or
performance.

### Phase 5 completion evidence (2026-07-11)

The Phase 5 sign-off reran the repository-owned commands after the authoritative
ledger update:

| Check | Observed result | Evidence limit |
| --- | --- | --- |
| Inventory generation and check | 177 rows; 18 implemented, 18 unit-tested, 17 differentially validated, 0 platform-validated, and 18 documented differences | Dimensions are independent; contact-manager rows remain pending. |
| Package isolation | 51 packaged entries built and tested outside the repository | Proves Cargo consumer isolation, not another platform. |
| Oracle debug comparison | 78 ordered cases matched under `phase5-v1` | Local D2-scoped collision operations only. |
| Oracle release comparison | 78 ordered cases matched under `phase5-v1` | A second optimization profile, not canonical D1 authority. |
| Debug replay | 78 ordered cases matched under `phase5-v1` | Protects the reviewed fixed corpus. |
| Debug determinism | 2 runs were byte-identical | D0 same-build authority only. |

The local reference tools were CMake 3.27.9, Ninja 1.13.2, and Apple Clang
21.0.0. Because the canonical lane requires CMake 4.3.3 and Clang 22.1.8 on
scalar Linux x86_64, these passes did not populate the ledger's
`platform_validated` dimension or authorize fixture promotion. At Phase 5
signoff, `subsystem.contacts-and-filtering` and `b2ContactManager.h` were still
pending; the independently executed Phase 6 lifecycle below is the later
evidence that changes those rows.

## Phase 6 rigid-world comparison policy

The closed `phase6-v1` registry names every rigid observable. Body and fixture
declaration order, manager contact order, manifold-point order, lifecycle and
hook order, destruction order, counts, body type, active/sensor/filter state,
feature identity, and solver branch state compare structurally and in order.
Float values cross JSON Lines as exact `u32` bits and are then evaluated only by
their named field policy for transforms, mass, material, manifold, and
impulses. There is no wildcard, repository-wide epsilon, or iteration-based
widening.

The request contains two mandatory witness families. The
`non_colliding_body_fixture_lifecycle` timeline covers static, kinematic, and
dynamic body state; transform, type, activation, fixture material, sensor,
filter, density, explicit/custom mass behavior; a zero-contact step; and
explicit teardown. The `single_contact_lifecycle` timeline covers contact
creation and persistence, feature-keyed warm-start carry, one bounded solve,
sensor overlap without a manifold or pre-solve, filter removal and
reconsideration, activation-driven destruction and recreation, and ordered
fixture/body teardown.

Validation is declaration-first: the native result must satisfy all declared
witnesses, counts, semantic IDs, action/checkpoint phases, and terminal reset;
the oracle result must independently satisfy the same contract; only then may
the comparator read a cross-engine physics field. Agreement on a shared
omission is therefore a protocol failure, not passing evidence.

Evidence authority stays dimensioned. Local successful comparisons are D2
supported-toolchain evidence. D0 requires exactly two byte-identical native
and oracle runs from the same build.
D1 remains the only fixture-promotion authority and requires the pinned scalar
Linux x86_64 Rust 1.97.0/Clang 22.1.8 lane plus complete adapter/build identity.
Local debug/release results neither populate `platform_validated` nor authorize
stage/review/promote publication.

The supported solver claim is limited to one discrete static/dynamic contact
with at most two canonical manifold points and warm-start impulse write-back.
Forces, torques, public velocity controls, damping, gravity scale, sleeping,
the general island solver, multi-contact stacks, CCD/TOI world orchestration,
queries, ray casts, broad world configuration, and joint solving remain Phase
7 or Phase 8. Contacts remain transient; raw handles, proxy coordinates, and
private occurrence identity are not compatibility observables.

## Phase 6 rigid-world commands

Initialize and verify the pinned submodule, then configure and build both
reviewed profiles before comparison:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask upstream configure --preset oracle-release
cargo xtask upstream build --preset oracle-release
```

Run the complete fixed Phase 6 signoff surface exactly:

```bash
cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot
cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot
cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot
cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2
```

The thin aliases are `just rigid-world-debug`, `just rigid-world-release`,
`just rigid-world-replay`, and `just rigid-world-determinism`. All commands are
closed over `protocol/fixtures/accepted/rigid-world-request.jsonl`, the
`phase6-v1` policy, reviewed presets, and the complete C++ adapter/build
identity. Compare and replay exercise both required witness families in one
request. Determinism executes exactly two complete native/oracle runs and
requires byte identity. Outputs remain under `target/reference` and
`target/differential`; no passing command rewrites accepted evidence.

Failure diagnosis preserves the witness family, preceding action, checkpoint,
semantic field path, mismatch kind, policy hash, and bounded process evidence.
Replay protects the checked-in request and result contract. Minimization may
operate only on valid action reductions that reproduce the same signature.
Fixture staging, explicit review, and no-clobber promotion reuse the established
Phase 2 lifecycle, but local D2 runs intentionally fail the D1 promotion gate.

### Phase 6 completion evidence (2026-07-12)

The Phase 6 signoff reran every fixed workflow after the authoritative ledger
and generated report changed:

| Check | Observed result | Evidence limit |
| --- | --- | --- |
| Inventory generation and check | 177 rows; 30 implemented, 30 unit-tested, 29 differentially validated, 0 platform-validated, and 30 documented differences | Only 12 body, fixture, world, contact-manager, circle-contact, and bounded-solver rows were promoted. |
| Package isolation | 58 packaged entries built and tested outside the repository | Proves Cargo consumer isolation, not another platform. |
| Oracle debug comparison | Both required rigid timelines matched under `phase6-v1` | Local D2 for the fixed Phase 6 scope. |
| Oracle release comparison | Both required rigid timelines matched under `phase6-v1` | A second optimization profile, not canonical D1 authority. |
| Debug replay | Both required rigid timelines matched under `phase6-v1` | Protects the reviewed request and declaration contract. |
| Debug determinism | Two complete native/oracle runs were byte-identical | D0 same-build authority only. |

The pinned C++ adapter executes the same declaration-first timeline in
`tools/reference/src/rigid_world.cpp`. Its content identity includes the decode
and trace headers, while build identity requires the exact four reviewed result
translation units and one shared effective compile signature. The successful
local tools were CMake 3.27.9, Ninja 1.13.2, and Apple Clang 21.0.0. They differ
from canonical CMake 4.3.3 and Clang 22.1.8 on scalar Linux x86_64, so all 177
`platform_validated` states remain false and no local fixture promotion is
authorized.

The ledger deliberately keeps general islands, broad world operations,
non-circle contact classes, forces, sleeping, CCD, queries, configuration,
joints, and broad rigid scenarios pending for Phase 7 or Phase 8.

### Phase 6 verification-gap closure evidence

The original verifier findings remain named so documentation checks can prevent
a passing happy-path corpus from silently reopening a source or workflow gap:

| Gap ID | Direct executable evidence | Authority limit |
| --- | --- | --- |
| `aggregate-mass-atomicity` | `cargo test -p liquidfun --test fixture_dynamics aggregate_mass --all-features` proves create/reset rejection preserves fixture adjacency, proxies, contacts, and body mass. | Native safe-API atomicity for the Phase 6 fixture surface. |
| `non-dynamic-contact-admission` | `cargo test -p liquidfun --test rigid_contacts non_dynamic --all-features` plus the two declaration-first overlap witnesses prove both no-dynamic branches. | Fixed contact-admission scope; not general islands. |
| `ignored-step-parameters` | Rust/schema/C++ boundary tests admit only `0x3c888889`, eight velocity iterations, and three position iterations. | Fixed Phase 6 tuple; public step configuration remains Phase 7. |
| `rigid-action-bound-mismatch` | Rust and C++ tests accept exactly 128 actions and reject 129 before execution. | One bounded request contract, not unbounded scenario input. |
| `invalid-centered-inertia-boundary` | Protocol fixture and C++ tests reject source-ordered negative, zero-after-centering, or non-finite centered inertia when origin inertia is positive, while retaining the zero-origin no-inertia branch. | Input-boundary evidence; arbitrary mass/solver behavior is not claimed. |
| `rigid-staging-not-integrated` | `cargo test -p liquidfun-differential --test rigid_fixture_workflow --all-features` and the xtask real-child test prove canonical D1 acceptance, D2 no-effect rejection, exact replay, and repeated pre-write authority checks. | Test-owned D1 identity proves the transaction; local real-oracle D2 runs cannot promote. |
| `rigid-sanitizer-not-executed` | Oracle workflow contracts require fail-fast CTest and `rigid-world` compare under `oracle-asan-ubsan` before read-only assertion. | Executed in scheduled/manual canonical Linux CI; local noncanonical runs do not become D1. |
| `implicit-aggregate-mass-atomicity` | `cargo test -p liquidfun --test fixture_dynamics implicit_aggregate_mass --all-features` proves `BodyTypeChangeError` and `FixtureDestructionError` reject invalid prospective aggregates with exact no-effect body, contact, fixture, proxy, adjacency, and mass evidence. | Native safe-API atomicity; body cascades intentionally do not reset mass because the parent is being destroyed. |
| `zero-centered-inertia-boundary` | Public, protocol-fixture, native-defense, and C++ protocol tests prove that the zero-origin branch remains no-inertia while positive-origin custom mass requires finite, strictly positive centered inertia. | Rust/C++ boundary parity for custom mass; broader mass solving remains deferred. |
| `rigid-fixture-checkout-provenance` | Shared identity tests and stale real-binary stage/review/promotion tests recompute current-checkout adapter-source and effective compile-command digests before every mutation and prove exact no-effect rejection. | Test-owned canonical identities exercise D1 gating; local real-oracle execution does not claim D1. |

These closures preserve the existing evidence labels: local debug/release and replay passes are D2, exactly two
same-build byte-identical runs are D0, and
only the pinned canonical lane can produce D1. Formal phase sign-off is derived
from code and executed evidence rather than this table.

## Phase 7 rigid-world comparison policy

The closed `phase7-v1` profile governs one bounded nine-family request. It
retains both Phase 6 lifecycle families, then adds force/configuration,
multi-contact/warm-start, sleep/wake, CCD/sub-step, continuous-budget,
query/ray, and origin-shift witnesses in registry order. Both native and oracle
results must independently satisfy the declared actions, checkpoints, counts,
identities, completion states, and reset proof before a cross-engine field is
read. Unknown actions, observations, policy paths, or witness families fail
closed.

Action and checkpoint kinds, body/contact/fixture identities, counts, flags,
wake and sleep states, completion, lifecycle order, manifold-point order, and
solve order compare exactly. Numeric body position, angle, velocity, damping,
gravity scale, force, torque, impulse, query, ray, and shift fields use only the
path-specific exact, absolute, absolute-relative, or ULP rule recorded in
`protocol/tolerances/phase7-v1.toml`. Callback directives are validated request
inputs, and signed separation remains private solver state; neither is emitted
as a result observable or registered Phase 7 comparison path. There is no
fallback policy and no widening based on iteration count or elapsed steps.

World query and ray callback order is intentionally outside the consumer
contract. The evidence layer compares query occurrences as a multiplicity-preserving multiset.
For non-terminated rays, it compares ray hits at or below the exact final interval, plus hits within the registered
fraction-policy boundary band, as a multiplicity-preserving multiset and applies the named numeric hit policies.
Each result records the exact final maximum-fraction bits after
replaying callbacks from the initial `1.0` interval; validation rejects any
later clip that would expand the interval. Hits recorded before a strict clip
are excluded only when proven beyond the final-interval boundary band, so
arbitrary valid clip fractions do not make comparison depend on callback
visitation order. An unreached clip or
reached no-op `Clip(1.0)` retains the full hit multiset. Terminated rays compare
completion, final interval, and callback count.
This canonicalization occurs only during comparison; production traversal is
never sorted to satisfy evidence. Solver-visible body, contact, manifold,
lifecycle, and source sequences remain ordered.

Continuous evidence exposes only `Complete`, `ContinuousPending`, bounded
work exhaustion, committed event counts, and transient semantic solve records.
Candidate pairs, TOI caches, counters, storage coordinates, and pointers remain
private. A work-budget error must carry a coherent progress checkpoint that a
matching later call resumes without repeating discrete integration. The first
divergence report preserves action, stage, entity, exact values and bits,
policy, profile identity, and surrounding completion state. Minimization must
retain the divergent operation and its complete setup prefix, including
directives, work budget, and transported float bits.

The fixed completion commands are:

```bash
cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot
cargo xtask differential compare --scenario rigid-world --preset oracle-release --session-profile one-shot
cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot
cargo xtask differential verify-determinism --scenario rigid-world --preset oracle-debug --runs 2
cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot
```

The locked request matched all nine families in local debug, replay, and
fail-fast sanitizer execution. The local debug, replay, and sanitizer passes are D2;
exactly two byte-identical same-build runs are D0. The local CMake and Apple
Clang identities differ from the canonical Linux pins, so they cannot publish
canonical fixtures or populate platform evidence. D3 review remains pending,
as do additional platforms, release maturity, broader rigid scenarios, joints,
and particles.

The C++ adapter remains a private, optional out-of-process maintainer tool.
Ordinary `liquidfun` builds and tests are Cargo-only. Passing evidence stays
read-only under `target/reference` and `target/differential`; every canonical
stage, review, or promotion boundary independently recomputes D1 authority and
current checkout, adapter, compile-command, policy, request, and evidence-tier
identity before writing.

## Phase 8 canonical rigid-world sign-off

The `phase8-v1` request is a strict superset with 19 required witness families:
two retained Phase 6 families, seven retained Phase 7 families, and ten Phase 8
families for all joint kinds and gear dependencies, standalone rope,
filter/pre-solve/listener timing, destruction cascades, and diagnostic
reconstruction. Both engines independently validate every declaration,
checkpoint count, action, witness, semantic identity, lifecycle occurrence,
and reset proof before comparison. Unknown or missing values fail closed.

The checked policy file has profile ID `phase8-v1` and SHA-256
`e31c47660bb5cce5aeb502ad510448176b419e604ef5048d74403bdef2f3a493`.
Transported configuration, IDs, kinds, branch states, counts, dependencies,
ordering, multiplicity, and signed zero remain exact. Named computed coordinate,
speed, force, and torque paths use only their parser-locked exact, ULP,
absolute, or absolute-relative policies. There is no wildcard, repository-wide epsilon,
or automatic widening.

GitHub Actions
[run 29383445374](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29383445374)
was a successful `workflow_dispatch` for exact head
`beb98bd74b1d26ab0a96c6be33ce1926d349abf0`. The unique successful
`canonical-linux` and `sanitizer-linux` identities were downloaded from:

- `phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`
- `phase8-sanitizer-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`

Both identities record Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8,
upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`, and policy profile
`phase8-v1`. The canonical job completed debug and release comparison, replay,
and exactly two byte-identical D0 runs; the sanitizer job completed the
fail-fast protocol and rigid corpus. Together they support
canonical scalar rigid-body and joint differential sign-off for the closed Phase 8 corpus.

That result populates platform evidence only for the 16 accumulated rigid rows
and 17 Phase 8 joint/rope rows exercised by this request. The 19-family corpus
does not complete RIGD-10, particles, D3, cross-platform parity, performance,
the testbed, or release readiness. Local runs remain D2/D0 and cannot substitute
for the exact run, SHA, artifact, job, toolchain, upstream, and policy identities
above.

## Phase 9 evidence dispatch and artifact contract

Phase 9 evidence is opt-in. Dispatch `.github/workflows/oracle.yml` with the
`evidence_phase` input set to `phase9` only after the intended commit is pushed.
The workflow runs the closed
`crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json`
corpus against already-committed code. It does not stage, review, promote, or
modify compatibility claims.

The `canonical-linux` job uses Linux x86_64, Rust 1.97.0, Clang 22.1.8, CMake
4.3.3, Ninja 1.13.2, target `x86_64-unknown-linux-gnu`, and upstream revision
`7f20402173fd143a3988c921bc384459c6a858f2`. It validates debug/release
agreement, exact replay, two-run D0 byte identity, the `phase9-v1` corpus,
provenance, inventory, and the read-only evidence boundary. The
`sanitizer-linux` job runs the same corpus with ASan abort/halt and UBSan
halt/stacktrace behavior.

Both jobs invoke `scripts/phase9-evidence.sh` with a repository-owned output
directory. The runner enables `set -euo pipefail`, so `cargo test` retains its
exit status through `tee`; it then requires an explicit `test result: ok.`
marker and rejects every `FAILED` marker before creating `identity.json`. The
generated manifest contains seven executed case records and exactly 58 unique
branch witnesses. Every record binds the identical native/C++ request digest,
both validated result digests, the comparison digest, and the complete 22-path
`phase9-v1` policy ledger. Provenance, inventory, and read-only checks also run
inside the same fail-closed runner before identity creation.

The 58-branch Phase 9 closure deliberately excludes cross-engine stable-ID
rotation. Native rotation remains covered by unit/property evidence; the first
truthful cross-engine rotation witness is deferred to Phase 10, where group
operations expose the required public behavior without a test-only seam.

Local non-promotable dry runs use the same entrypoint after building the named
oracle presets:

```bash
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask upstream configure --preset oracle-release
cargo xtask upstream build --preset oracle-release
bash scripts/phase9-evidence.sh canonical target/phase9-evidence-local/canonical

cargo xtask upstream configure --preset oracle-asan-ubsan
cargo xtask upstream build --preset oracle-asan-ubsan
cmake --build target/reference/oracle-asan-ubsan --target liquidfun-reference-protocol-tests
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 ctest --test-dir target/reference/oracle-asan-ubsan --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 bash scripts/phase9-evidence.sh sanitizer target/phase9-evidence-local/sanitizer
```

The jobs upload exactly named artifacts:

- `phase9-canonical-${run_id}-${sha}`
- `phase9-sanitizer-${run_id}-${sha}`

Each artifact contains its logs, `phase9-trace.log`, `phase9-manifest.json`, and
`identity.json`. The identity record contains numeric `run_id` plus `job`,
`head_sha`, `upstream_revision`, `rust`, `cmake`, `ninja`, `clang`, `target`, and
`policy` fields. The job values are `canonical-linux` and `sanitizer-linux`, the
target is `x86_64-unknown-linux-gnu`, and the policy is `phase9-v1`. Its `trace`
and `manifest` objects each contain an artifact-relative `path` and a lowercase
64-hex `sha256`; both named files are present beside the identity so a reviewer
can recompute the digests.

Promotion is a later, explicit sequence: download both artifacts for the same
successful run and exact head SHA, verify the two identities and both embedded
digests, confirm the unique successful canonical and sanitizer jobs, then use
the repository's reviewed stage, review, and promotion workflow. A local pass,
a mismatched SHA, one artifact, or an unverified digest cannot promote the Phase
9 claim.

### Rejected Phase 9 evidence run (2026-07-15)

GitHub Actions run
[`29439515367`](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29439515367)
completed successfully for the human-approved commit
[`a87f84bbdbfe55fb732d74c481c4a4bda9eec958`](https://github.com/bright-builds-llc/liquidfun-rs/commit/a87f84bbdbfe55fb732d74c481c4a4bda9eec958).
The run was dispatched manually through `Oracle CI`; exactly one canonical job
and one sanitizer job succeeded for that head SHA.

The exact, unexpired artifacts were independently downloaded again before any
compatibility-ledger edit:

- [`phase9-canonical-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`](https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352859391/zip)
  has GitHub archive digest
  `sha256:f237d6f1ebe0e59f65a5ae0609140eecdd8b32247e9d2064c83748be1ab9f5ea`.
  Its identity binds `canonical-linux` and `phase9-v1`; the recomputed trace and
  manifest SHA-256 values are
  `3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64`
  and
  `36cfaad1f56505f8427408733e2231ad613984a4cb3eb3b8d757e7a14b2c38e0`.
- [`phase9-sanitizer-29439515367-a87f84bbdbfe55fb732d74c481c4a4bda9eec958`](https://api.github.com/repos/bright-builds-llc/liquidfun-rs/actions/artifacts/8352881868/zip)
  has GitHub archive digest
  `sha256:95ad57e5d5711ae6aa93847ad1efd4a04025bd2956b4996535fa0e5f45a5893f`.
  Its identity binds `sanitizer-linux` and `phase9-v1`; the recomputed trace and
  manifest SHA-256 values are
  `ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070`
  and
  `0c89f0136eda6689118d3eaa909defb1d182d5723e7a64ea1e958396066dce15`.

Independent revalidation proved the identities and hashes are internally
consistent, but both bound trace logs contain `test result: FAILED. 4 passed; 1 failed` and `Phase 9 checkpoint has no legacy predecessor`. The workflow's
unchecked `cargo test | tee` pipeline masked those failures. Run `29439515367`
and both artifacts listed above are therefore rejected as compatibility
authority and must not be used for D1 promotion. A new exact-SHA run is required
after the fail-closed runner and executable native/C++ corpus are committed and
human-approved. No local run or Plan 09-22 change directly promotes a
compatibility claim.

## Differential commands

Initialize, verify, configure, and build the oracle first:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

Run one-shot isolation, bounded reuse, replay, and deterministic minimization:

```bash
cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile one-shot
cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile reuse
cargo xtask differential replay --scenario empty-world --preset oracle-debug --session-profile one-shot
cargo xtask differential minimize --scenario empty-world --preset oracle-debug --session-profile one-shot
```

The equivalent thin aliases are `just differential-compare`,
`just differential-reuse`, `just differential-replay`, and
`just differential-minimize`. Xtask accepts only the checked-in `empty-world`
scenario, three reviewed presets, three reviewed session profiles, and fixed
lifecycle option shapes. It accepts no caller-provided executable, output,
destination, or exact-request path.

## Outcomes, diagnosis, and retention

Only two complete validated traces can produce `PhysicsMismatch`. The runner
validates protocol/schema/tolerance versions, request and scenario hashes,
engine roles, oracle/adapter/build identity, ordered record sequence, payload
hash, checkpoint count, and reset epoch first.

The typed harness-failure taxonomy covers startup and request timeouts, malformed
handshake, unsupported version, wrong provenance, nonzero exit or signal,
sanitizer report, unexpected EOF or partial record, malformed or unknown record,
record/trace/total-output limits, sequence or request-ID violation, trace
identity mismatch, rejected scenario, Rust/C++ adapter failure, and adapter reset
failure. These are harness failures, not physics mismatches.

A physics mismatch reports the first divergent checkpoint, named phase,
semantic path, and mismatch kind. It includes exact bits and diagnostic decimals
plus adjacent identities. IDs, counts, flags, membership, record kinds, and
ordered sequences compare exactly. Only fields typed as sets or multisets may be
canonicalized; checkpoint and future solver/callback/destruction order is never
globally sorted.

Failure evidence is bounded. `target/differential/failures` is the CI upload
boundary and may contain only the exact request, build/session identity,
machine-readable comparator or harness report, bounded first/last stderr, exit
status, limit profile, and killed/reaped state. Do not retain raw unlimited child
streams, secrets, pointers, memory snapshots, or unreviewed external data.

Deterministic failures are not automatically retried. Preserve the exact
request or serialized scenario, seed/source metadata, first-divergence failure
signature, bounded stderr, and identities. A rerun is reproduction evidence,
not permission to discard the first failure.

## Reference evidence lifecycle

The accepted classes are protocol fixtures, named scenarios, reviewed oracle
traces, and minimized regressions. `reference/artifacts/manifest.toml` records
content, request, scenario, payload, policy, oracle, adapter, build, compiler,
target, flags, source/seed, notices, reviewer, UTC review time, and status.

Stage only to the confined candidate area. The rigid path uses the same lifecycle
with its closed real-binary transaction:

```bash
cargo xtask differential fixture stage --scenario empty-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id "$ARTIFACT_ID"
cargo xtask differential fixture stage --scenario rigid-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id "$ARTIFACT_ID"
```

Replay the candidate, inspect its receipt and reviewable diff, then bind an
explicit decision:

```bash
cargo xtask differential fixture review --artifact-id "$ARTIFACT_ID" --reviewer "$REVIEWER" --reviewed-at "$REVIEWED_AT_UTC" --review-status approved
```

Only an unchanged approved candidate may be promoted:

```bash
cargo xtask differential fixture promote --artifact-id "$ARTIFACT_ID"
```

Promotion derives the accepted path from typed artifact kind and scenario ID,
refuses existing destinations, and publishes through a no-clobber atomic rename.
Rigid stage validates request/result/build identity and comparison before the
first write; review and promotion replay the exact candidate and independently
repeat the D1 authority guard before their own writes. Checks never regenerate
evidence. Portability or CI jobs never review or promote.

Minimization operates on validated typed scenarios in stable transform order.
Every candidate revalidates and must reproduce the same failure signature. A
different first divergence is a different failure. Persist the minimized
serialized scenario under `scenarios/regressions`, not only a generator seed,
and promote it through the same manifest/replay/review path.

## Expensive evidence workflows

### Fuzzing

Fuzz targets are deferred until their owning parser/world surfaces land. The
scheduled/manual command shape is:

```bash
rustup run nightly cargo fuzz run protocol_decode -- -max_total_time=300
rustup run nightly cargo fuzz run protocol_decode fuzz/artifacts/protocol_decode/<case>
```

Pin the nightly and `cargo-fuzz` version before enabling CI. Persist and minimize
the exact input. A crash, timeout, sanitizer report, or boundary defect is a
harness failure.

### Miri and Rust sanitizers

Miri is deferred to a pinned nightly subset that excludes the external process:

```bash
rustup run nightly cargo miri setup
rustup run nightly cargo miri test --workspace --all-features
```

Future Rust sanitizer targets must also pin nightly and fail fast, for example:

```bash
RUSTFLAGS="-Zsanitizer=address" RUSTDOCFLAGS="-Zsanitizer=address" cargo +nightly test -p liquidfun-test-protocol -Zbuild-std --target x86_64-unknown-linux-gnu
```

Miri, ASan, UBSan, signals, and UB/aliasing findings are harness/safety failures,
not physics mismatches.

### C++ fail-fast sanitizer corpus

Configure and build the reviewed preset:

```bash
cargo xtask upstream configure --preset oracle-asan-ubsan
cargo xtask upstream build --preset oracle-asan-ubsan
```

From the repository root, first execute the C++ protocol tests and the complete
Phase 8 rigid/joint/rope/callback/teardown/trace path under the same fail-fast
environment, then preserve the existing empty-world one-shot and bounded
reset/reuse corpus:

```bash
cmake --build target/reference/oracle-asan-ubsan --target liquidfun-reference-protocol-tests
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 ctest --test-dir target/reference/oracle-asan-ubsan --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario empty-world --preset oracle-asan-ubsan --session-profile one-shot
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario empty-world --preset oracle-asan-ubsan --session-profile sanitizer
```

Every command is status-propagating and runs before the read-only assertion;
there is no retry or `continue-on-error` path. The sanitizer profile sends a
finite reset/reuse corpus through the same supervisor. ASan/UBSan markers fail
even if a misconfigured child exits zero. On job failure, CI uploads only the
existing bounded `target/differential/failures` directory, requires it to exist
with `if-no-files-found: error`, and retains failures for seven days. A CTest
failure with no harness bundle therefore remains a failed job and cannot widen
the artifact path to capture arbitrary workspace files.

### Coverage

Coverage is deferred until representative subsystem suites exist. Install the
pinned `cargo-llvm-cov` and the pinned Rust toolchain's `llvm-tools-preview`, then
run:

```bash
cargo llvm-cov --workspace --all-features --lcov --output-path target/llvm-cov/rust.lcov
```

Generate C++ coverage separately with a compatible pinned Clang/LLVM toolchain.
Coverage is not parity; report unit/integration coverage separately from
subsystem differential coverage.

### Benchmarks

Benchmarks are deferred until equivalent native behavior exists. The planned
Rust entrypoint is `cargo bench --workspace`; paired C++ runs must record the
same scenario, optimization mode, compiler, flags, target, hardware, warm-up,
and measurement method. Performance data cannot substitute for compatibility.

## Aggregate and CI placement

`cargo xtask check` and `just check` are read-only. In Cargo-only mode they run
package isolation, protocol schema/fixture checks, documentation contracts, and
artifact provenance without an initialized submodule or C++. With the submodule
initialized they additionally run inventory, upstream identity, and full
provenance.

Cargo pull-request jobs stay submodule-free and exercise every private Rust
crate, comparator/minimizer behavior, supervisor failure injection, protocol and
provenance-schema fixtures, rustdoc, docs contracts, and consumer package
isolation. Oracle jobs alone initialize the submodule and use CMake/C++.

Canonical Linux oracle CI verifies exact tool and source identity, builds
`oracle-debug` and `oracle-release`, compares the closed Phase 8 rigid corpus,
replays it, and asserts exactly two runs remain byte-identical. The
scheduled/manual sanitizer lane runs the exact fail-fast protocol, rigid,
one-shot, and reuse commands above.
Portability builds are non-canonical,
read-only, and cannot publish or promote evidence. All workflows use
`contents: read`, full-SHA external actions, bounded job timeouts, and no secrets.

## Focused repository commands

```bash
cargo xtask docs check
cargo xtask package verify
cargo xtask inventory check
cargo xtask upstream verify
cargo xtask provenance check
cargo xtask check
just check
```

Generated schemas, scenarios, traces, manifests, and reports must remain
byte-for-byte unchanged under all check and replay commands.
