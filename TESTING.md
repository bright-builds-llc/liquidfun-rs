# Testing and Verification

## Current scope and maturity

Phase 2 proves a trustworthy empty-world harness seam: versioned and bounded
scenario input, native Rust execution, a process-isolated pinned C++ oracle,
typed semantic comparison, first-divergence diagnosis, deterministic reduction,
reviewed trace replay, and harness-failure classification. Phase 4 additionally
proves the native math/settings contract and a bounded 39-case Rust/C++ probe
corpus under the reviewed `phase4-v1` policy. Local oracle results are D2
supported-portability evidence, not canonical D1 or all-platform validation.
Shapes, collision, rigid-body and particle solver parity, performance parity,
and the deferred fuzz, Miri, Rust-sanitizer, benchmark, and coverage lanes remain
pending. [COMPATIBILITY.md](COMPATIBILITY.md) remains authoritative for feature
and evidence maturity.

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
| differential | current | Run the same validated empty-world request through Rust and C++ and compare semantic traces. | `cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile one-shot` | Verified and built `oracle-debug` executable plus matching protocol, tolerance, and upstream identities. | Machine report on stdout; bounded request, identity, report, and stderr evidence belongs under `target/differential/failures` on failure. | No deterministic retry; a stable mismatch or harness failure must be diagnosed. | local, pull request, scheduled, manual release | Only validated traces can produce a physics mismatch; process, schema, sanitizer, provenance, and reset errors are harness failures. |
| property | deferred | Generate bounded valid values that probe geometry, ordering, handles, mutation, and later physics invariants. | Planned `cargo test --workspace property -- --nocapture` with persisted seeds and typed shrinkers. | Rust 1.97.0 now; reviewed generator version and deterministic seed model before activation. | Property test output plus the seed and minimized input promoted to an ordinary regression when confirmed. | No deterministic retry; reproduce the same invariant failure from the persisted value. | scheduled, manual release | A property pass samples an invariant domain and cannot prove exhaustive compatibility. |
| checked-in regression | current | Replay reviewed traces and later minimized scenarios that preserve an accepted first-divergence signature. | `cargo xtask differential replay --scenario empty-world --preset oracle-debug --session-profile one-shot` | Reviewed trace in the artifact manifest; later cases also require a minimized scenario under `scenarios/regressions`. | Replay report, manifest provenance, and any same failure signature evidence stored with the regression under `scenarios/regressions`. | No deterministic retry; byte-stable replay must be reproducible. | local, pull request, scheduled, manual release | A checked-in case protects one reviewed behavior or same failure signature from recurrence. |
| fuzz | deferred | Exercise bounded protocol decoders, scenario validation, world mutation, and future unsafe boundaries. | Planned `cargo fuzz run protocol_decode -- -max_total_time=300`; reproduce with `cargo fuzz run protocol_decode fuzz/artifacts/protocol_decode/<case>`. | pinned nightly, `cargo-fuzz`, reviewed target bounds, and no secrets or external service. | libFuzzer logs, exact crashing input, seed when present, and minimized corpus under `fuzz/artifacts`. | No deterministic retry; retain and minimize the exact input before fixing. | scheduled, manual release | A crash, timeout, sanitizer finding, or malformed-boundary defect is a harness failure, not a physics mismatch. |
| Miri/UB-aliasing | deferred | Detect undefined behavior and aliasing defects in the pure Rust subset and any future unsafe modules. | Planned `cargo miri test --workspace --all-features` on a date-pinned nightly after `cargo miri setup`. | pinned nightly with the Miri component; exclude the external C++ process from interpretation. | Miri diagnostics with the exact test, stack, flags, and pinned toolchain identity. | No deterministic retry; preserve the deterministic failing test and environment. | scheduled, manual release | Miri undefined behavior is a harness failure and safety defect, never a physics mismatch. |
| native sanitizer | current | Run the C++ oracle fail-fast and later run supported Rust sanitizer subsets without crossing findings into comparison. | `UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario empty-world --preset oracle-asan-ubsan --session-profile one-shot` | Exact Clang 22.1.8 lane, configured and built `oracle-asan-ubsan`, ASan/UBSan runtimes; Rust sanitizer work additionally needs pinned nightly. | Bounded request, identity, report, and stderr evidence under `target/differential/failures`; CI uploads only this directory on failure. | No deterministic retry; sanitizer markers fail even if a child exits zero. | scheduled, manual release | Any ASan, UBSan, Rust sanitizer, signal, timeout, or reset defect is a harness failure, not a physics mismatch. |
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
scalar Linux x86_64, these passes do not populate the ledger's
`platform_validated` dimension and do not authorize fixture promotion. The
generated report retains `subsystem.contacts-and-filtering` and
`b2ContactManager.h` as not implemented, not unit-tested, and not
differentially validated.

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

Stage only to the confined candidate area:

```bash
cargo xtask differential fixture stage --scenario empty-world --preset oracle-debug --session-profile one-shot --artifact-kind reviewed-trace --artifact-id "$ARTIFACT_ID"
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
Checks never regenerate evidence. Portability or CI jobs never review or promote.

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

From the repository root, run the exact one-shot and bounded reset/reuse corpus:

```bash
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario empty-world --preset oracle-asan-ubsan --session-profile one-shot
UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1 ASAN_OPTIONS=abort_on_error=1:halt_on_error=1 cargo xtask differential compare --scenario empty-world --preset oracle-asan-ubsan --session-profile sanitizer
```

The sanitizer profile sends a finite reset/reuse corpus through the same
supervisor. ASan/UBSan markers fail even if a misconfigured child exits zero.

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
`oracle-debug`, runs one-shot plus bounded reuse, replays the reviewed trace, and
asserts evidence remains byte-identical. The scheduled/manual sanitizer lane
runs both exact fail-fast commands above. Portability builds are non-canonical,
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
