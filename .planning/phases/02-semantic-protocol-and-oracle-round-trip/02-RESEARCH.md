---
phase: 02-semantic-protocol-and-oracle-round-trip
status: complete
researched: 2026-07-10
requirements:
  - COMP-03
  - COMP-04
  - COMP-05
  - COMP-06
  - COMP-07
  - COMP-08
  - COMP-09
  - DOCS-05
---

# Phase 2 Research: Semantic Protocol and Oracle Round Trip

## Research conclusion

Build two unpublished Rust crates: one pure protocol/domain crate and one differential runner. Keep process management, comparison, minimization, and C++ orchestration out of `liquidfun`; keep C++ under the existing `tools/reference` wrapper; and leave `default-members = ["crates/liquidfun"]` unchanged. The first vertical slice should be deliberately narrow: a version-1 empty-world scenario, one or more step commands, exact world-count and time checkpoints, and both one-shot and bounded-reuse C++ execution.

This phase proves the permanent harness seam, not physics parity. The native Rust adapter may own only the empty-world lifecycle required for the round trip; it must not force a public `World`/handle design before Phase 3. Later engine phases replace that narrow adapter behind the same validated scenario/trace types.

The repository's local guidance materially shapes this design: `AGENTS.bright-builds.md`, `standards/core/architecture.md`, `standards/core/code-shape.md`, `standards/core/testing.md`, `standards/core/verification.md`, and `standards/languages/rust.md` require strict boundary parsing, invariant-bearing types, a pure comparison core, thin process/CMake shells, focused tests, and repo-native verification. There is no active substantive override in `standards-overrides.md`.

## Recommended private seams

```text
crates/
  liquidfun-test-protocol/          # publish = false; pure domain and wire contract
    src/lib.rs
    src/codec.rs                    # bounded JSONL decoding/encoding
    src/failure.rs                  # shared validation/harness taxonomy
    src/float_bits.rs               # authoritative f32 <-> u32 representation
    src/ids.rs                      # request/scenario/checkpoint/semantic IDs
    src/limits.rs                   # named immutable limit profiles
    src/provenance.rs               # trace/build identity and hashes
    src/scenario.rs                 # raw -> ValidatedScenario parsing
    src/tolerance.rs                # typed field policies
    src/trace.rs                    # handshake and trace state machine records
    src/schema.rs                   # deterministic checked-in schema emission
  liquidfun-differential/           # publish = false; library plus CLI binary
    src/lib.rs
    src/rust_adapter.rs             # Phase-2 native empty-world executor
    src/supervisor.rs               # long-lived C++ process state machine
    src/supervisor/stdio.rs         # concurrent bounded drains
    src/comparator.rs               # exhaustive typed comparison
    src/canonical.rs                # only explicitly unordered values
    src/minimizer.rs                # deterministic validity-preserving reduction
    src/report.rs                   # machine and concise human reports
    src/fixtures.rs                 # reviewed generate/verify/promote workflow
    src/main.rs                     # thin command dispatch
tools/
  xtask/src/differential.rs         # allowlisted build/run entrypoints only
  reference/
    src/main.cpp                    # protocol-only stdout loop
    src/protocol.{hpp,cpp}          # strict JSON decode/encode
    src/oracle_adapter.{hpp,cpp}    # b2World lifecycle and semantic mapping
    src/build_identity.hpp.in       # configured immutable handshake identity
    vendor/nlohmann/                # exact 3.12.0 header and MIT notice
protocol/
  schemas/                          # deterministic generated JSON Schemas
  fixtures/                         # accepted/rejected wire fixtures
  tolerances/phase2-v1.toml         # reviewed field policies
scenarios/
  phase-02/empty-world.json
  regressions/                      # minimized serialized scenarios, later populated
reference/artifacts/traces/         # explicitly reviewed canonical traces
```

Use `foo.rs` plus `foo/`, not `foo/mod.rs`. `xtask` remains an imperative shell: it verifies the upstream, configures/builds the allowlisted preset, locates the adapter executable, and invokes the differential CLI. Protocol validation, comparison decisions, and reduction belong in the private crates, not in `main.rs` or `xtask`.

### Workspace and dependency direction

```text
liquidfun-test-protocol
          ^
          |
liquidfun-differential ---> liquidfun
          |
          +-- subprocess ---> tools/reference/liquidfun-reference ---> Box2D

xtask ---> cargo/CMake/differential commands
```

- Add both private crates to workspace `members`; do not add them to `default-members`.
- Neither private crate may become a dependency or feature of published `liquidfun`.
- The default Cargo build must not inspect the submodule or discover CMake.
- Full-workspace compilation and unit tests must work without starting or building C++.
- Oracle integration tests run only through an explicit `xtask`/`just` command or oracle CI lane.

## Dependencies already supported by repository research

| Dependency | Scope | Phase-2 use |
| --- | --- | --- |
| `serde` 1.0.228 | private Rust | Strict tagged records and domain serialization |
| `serde_json` 1.0.150 | private Rust | UTF-8 JSON Lines codec; never deserialize boundary input to unrestricted `Value` |
| `sha2` 0.10.9 | private Rust | Scenario, trace, identity, tolerance, and content hashes |
| `toml` 0.9.8 | private Rust | Reviewed tolerance and artifact metadata |
| `thiserror` 2.0.18 | private Rust libraries | Typed validation, protocol, supervisor, and comparison errors |
| `nlohmann/json` 3.12.0 | private C++ | Vendored adapter JSON parser with its MIT license and checksum |

Do not add Tokio, a generic approximate-equality crate, a schema framework, FFI generators, or RNG dependencies for this slice. Standard-library threads and channels are sufficient for one sequential in-flight request. Emit the small checked-in JSON Schemas deterministically from `schema.rs`; typed Rust/C++ validation remains authoritative for cross-field invariants. `rand_chacha = 0.10.0` remains the prescribed later generator dependency, but Phase 2 only stores and replays supplied seeds.

## Protocol version 1

Version these axes independently as integer newtypes:

- `ProtocolVersion(1)`: framing, handshake, request/response record kinds.
- `ScenarioSchemaVersion(1)`: bounded empty-world scenario shape.
- `TraceSchemaVersion(1)`: trace begin/checkpoint/end payloads.
- `ToleranceProfileVersion(1)`: comparison decisions.

Every line is one complete UTF-8 JSON object terminated by `\n`. Stdout is protocol-only. Reject blank lines, unknown record kinds, unknown fields, duplicate members, unsupported versions, trailing bytes, and EOF after a partial record. Use tagged structs with `#[serde(deny_unknown_fields)]`; do not route boundary records through `serde_json::Value`, which would weaken duplicate-field and shape checks.

### Startup handshake

The C++ process emits exactly one `handshake` record before accepting requests:

```text
protocol_version
record_kind = handshake
supported scenario/trace/tolerance versions
oracle_revision
adapter_revision + adapter_content_sha256
CMake preset
compiler ID + complete version
target/system/architecture
build type + effective compile/link flags
sanitizer mode
identity_sha256
```

`identity_sha256` is the canonical hash of the preceding identity fields. The runner independently knows the expected oracle revision from `reference/upstream-lock.toml` and rejects disagreement before sending a scenario.

### Scenario request

One input line contains:

```text
protocol_version
record_kind = scenario_request
request_id
scenario_schema_version
requested_trace_schema_version
tolerance_profile_version + tolerance_profile_sha256
scenario
```

`ScenarioV1` contains:

- a validated stable `scenario_id`;
- a sum type for `source`: `named { name }` or `seeded { generator_id, generator_version, seed }`;
- exact `gravity_x_bits` and `gravity_y_bits`;
- an explicitly ordered entity-definition list (empty is the only supported Phase-2 case);
- ordered `StepCommand`s with command IDs, timestep bits, and solver iteration counts;
- uniquely identified checkpoint requests referencing a command boundary;
- typed requested observables (`world_counts` and `simulation_time` in version 1).

The empty entity list is a deterministic creation order. Reject nonempty entity definitions in scenario schema 1 rather than inventing Phase-3 object types. The first body/fixture scenario can add a new tagged scenario variant under a deliberate scenario-schema revision without changing transport framing.

Use semantic ID newtypes with kind plus ordinal for future references; never use Rust handles, C++ addresses, dense indices, or object memory. Validate ID syntax once: ASCII lowercase leading alphanumeric followed by lowercase alphanumeric, `.`, `_`, or `-`, with a 128-byte maximum.

### Streamed trace

A successful request produces this exact state machine:

```text
trace_begin(request_id, trace_schema_version, scenario_id, scenario_sha256,
            source/seed, tolerance identity, engine identity, stable identity hash)
checkpoint(request_id, checkpoint_id, ordinal, named phase,
           simulation_time_bits, typed observables)
... zero or more additional checkpoints ...
trace_end(request_id, checkpoint_count, trace_payload_sha256,
          reset_epoch, reset_verified = true)
```

For Phase 2, `WorldCounts` contains exact body, fixture, joint, contact, particle-system, group, and particle counts, all zero. `simulation_time_bits` proves exact float transport. `trace_end` is emitted only after the adapter destroys the per-request world, clears semantic mappings, verifies reset, and increments `reset_epoch`. A missing or false reset proof poisons the session.

Every Rust and C++ trace records the selected comparison oracle revision, adapter revision, compiler/build flags, target, source/seed, schema versions, and tolerance profile. Rust provenance distinguishes `engine_kind = native_rust` from `engine_kind = cpp_oracle`; it must not pretend the Rust binary was built from upstream.

## Boundary limits

Start with a named `phase2-default-v1` `HarnessLimits` profile. Make values constants, hash the profile into diagnostics, and test each limit at `N` and `N + 1`.

| Limit | Initial value |
| --- | ---: |
| Input JSONL record | 1 MiB |
| JSON nesting depth | 32 |
| General decoded string | 4 KiB |
| Typed ID | 128 bytes |
| Entity definitions | 4,096 |
| Commands/steps | 4,096 |
| Checkpoints | 4,096 |
| Observables per checkpoint | 128 |
| Output JSONL record | 1 MiB |
| Complete trace | 32 MiB |
| Retained stderr | 256 KiB |
| Total child output per request | 64 MiB |
| Startup timeout | 5 seconds |
| Phase-2 request timeout | 10 seconds |
| Default request budget | 1 |
| Reusable corpus budget | 100 requests |

Enforce record bytes and JSON depth before typed allocation; enforce string and collection bounds during custom Serde visitors; enforce entity/reference uniqueness and aggregate counts after parsing; enforce record, trace, stderr-retention, total-output, and deadline limits while running. A retained-stderr cap does not permit blocking: always drain, track total bytes, and keep a bounded first/last diagnostic window with an explicit truncation count.

Do not expose arbitrary limit overrides on the Phase-2 CLI. One-shot, corpus, and sanitizer profiles are reviewed named configurations. Later scalability work may add another recorded profile without silently weakening this contract.

## Process supervisor and failure taxonomy

Model session state as an enum so illegal reuse is unrepresentable:

```text
Starting -> Handshaking -> Ready -> InFlight -> Ready
                         \-> Poisoned -> Reaped
                         \-> Exited
```

Use `std::process::Command` with piped stdin/stdout/stderr. Dedicated stdout and stderr threads drain concurrently and send bounded events over channels. The controlling thread uses deadlines, permits exactly one request in flight, writes and flushes one request line, and validates the output sequence incrementally. On timeout, protocol corruption, output overflow, sanitizer evidence, or unexpected exit: mark poisoned, kill, wait/reap, join drain threads, and only then return the preserved evidence. Never reuse a poisoned session and never silently retry a deterministic request.

Use a typed `HarnessFailureKind` at minimum:

- `StartupTimeout`, `HandshakeMalformed`, `UnsupportedVersion`, `WrongProvenance`;
- `RequestTimeout`, `ChildNonZeroExit`, `ChildSignaled`, `SanitizerReport`;
- `UnexpectedEof`, `PartialRecord`, `MalformedRecord`, `UnknownRecordKind`;
- `RecordTooLarge`, `TraceTooLarge`, `TotalOutputExceeded`;
- `SequenceViolation`, `RequestIdMismatch`, `TraceIdentityMismatch`;
- `ScenarioRejected`, `RustAdapterFailure`, `CppAdapterFailure`, `AdapterResetFailure`.

The failure report includes request/scenario hashes, request ID, session identity, exit status, elapsed time, last valid record, bounded stderr, limit profile, and whether the child was killed/reaped. `PhysicsMismatch` is a separate outcome and is constructible only from two complete validated traces.

Sanitizer preset behavior must be fail-fast. Add `-fno-sanitize-recover=undefined`; run with `UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1` and `ASAN_OPTIONS=abort_on_error=1:halt_on_error=1`. Treat a recognized sanitizer report as a harness failure even if a misconfigured runtime returns zero.

## C++ adapter and build integration

Extend the existing wrapper after `add_subdirectory`:

- Add `liquidfun-reference` and link it privately to the existing `Box2D` target.
- Compile only the adapter as C++17; do not modify upstream language settings.
- Keep all adapter sources outside `third_party/liquidfun`.
- Configure a generated build-identity header from the lock revision, adapter source digest/revision, preset, CMake compiler identity/version, target/system, build type, and effective flags.
- Have `xtask` pass and independently verify the expected lock revision and adapter digest; the executable self-reports them in the handshake.
- Change the allowlisted build target from bare `Box2D` to `liquidfun-reference`; linking builds Box2D transitively.
- Preserve the current out-of-tree directories under `target/reference/<preset>`.

The adapter loop reads one line, validates it into strict C++ domain structs, constructs a fresh `b2World`, maps semantic IDs only inside the request scope, emits buffered/flush-complete records, destroys state, verifies empty adapter maps, and emits `trace_end`. Catch `std::exception` at `main`, write diagnostics only to stderr, and exit nonzero. Do not serialize C++ objects or allow pointers/indices into records.

Prove reset with one process handling two distinguishable empty scenarios, different checkpoint counts, and `reset_epoch` 1 then 2. Also prove periodic cycling at the configured request budget.

## Comparator and policy model

Keep comparison pure: `compare(expected, actual, policy) -> DifferentialOutcome`.

Recommended types:

```text
ValidatedTrace
CanonicalTrace
ToleranceProfile { id, version, sha256, field policies }
FloatPolicy = ExactBits | Absolute { max_bits } |
              AbsoluteRelative { abs_bits, rel_bits } | Ulps { max }
CollectionPolicy = Ordered | Set | Multiset
MismatchKind = Missing | Unexpected | Exact | Numeric | Order | Multiplicity
FailureSignature = checkpoint_id + phase + semantic_path + mismatch_kind
```

- Compare protocol/schema/tolerance/scenario/provenance compatibility before observables.
- Compare request IDs, checkpoint IDs, record kinds, counts, flags, membership, and event kinds exactly.
- `phase2-v1` uses exact bits for simulation time; unit fixtures exercise all numeric policy variants without pretending a broad physics tolerance is known.
- Represent tolerance thresholds as `FloatBits(u32)`, not JSON decimal floats.
- NaN is a mismatch unless a field explicitly requires exact payload comparison; infinities require exact sign; signed zero remains distinct unless a reviewed field policy says otherwise.
- Canonicalize only values whose Rust type explicitly carries `Set` or `Multiset` semantics. Checkpoints, phases, callbacks, destruction events, and any solver-significant sequence remain ordered.
- Stop the primary report at the first divergent checkpoint/phase/path, while including adjacent checkpoint identities and both bit/diagnostic decimal renderings.

Do not use generic JSON paths as the policy authority. Exhaustive matches over typed observable variants ensure a new field cannot compile without a comparison decision.

## Fixtures, minimization, and provenance

Separate three artifact classes:

1. Protocol fixtures test accepted and rejected bytes, duplicate fields, bounds, sequencing, and exact bits.
2. Named scenarios are hand-reviewed semantic inputs such as `empty-world`.
3. Reviewed oracle traces/minimized regressions are provenance-bound evidence recorded in `reference/artifacts/manifest.toml`.

For every reviewed trace or regression record:

- exact input content SHA-256 and canonical scenario SHA-256;
- protocol/scenario/trace/tolerance versions and profile SHA-256;
- oracle revision, adapter revision/content hash, preset, compiler, target, and flags;
- trace content SHA-256, notice references, generator revision, and review status.

Generation must write under `target/differential/staging`, replay and validate the candidate, render a reviewable diff, and only then atomically replace an accepted artifact through an explicit maintainer command. Tests and `check` commands never regenerate evidence.

Implement deterministic hierarchical delta debugging over typed `ValidatedScenario` values. Candidate transforms remove checkpoint groups, steps, commands, and later dependency-closed entity groups in stable order. Each candidate must revalidate and reproduce the same `FailureSignature`; a different first divergence is not the same failure. Enforce attempt/time budgets and always persist the minimized serialized scenario plus the original seed/source metadata. Test the reducer with an injected synthetic mismatch because the empty-world Rust/C++ traces should match.

## Requirement coverage

| Requirement | Phase-2 evidence |
| --- | --- |
| COMP-03 | `ScenarioV1`, named/seeded source enum, strict limits, stable typed IDs, ordered empty creation list, steps, and checkpoint references |
| COMP-04 | Native Rust adapter and process-isolated `liquidfun-reference` consume the same `ValidatedScenario`; protocol contains semantic values only |
| COMP-05 | Trace begin identity plus manifest records all versions, oracle/adapter identity, compiler/flags/target, source/seed, and tolerance hash |
| COMP-06 | Exhaustive exact discrete comparison and versioned field-specific `FloatPolicy`; exact time bits in the vertical slice |
| COMP-07 | Typed ordered/set/multiset policies; only explicitly unordered payloads reach canonicalization |
| COMP-08 | Name/exact-value replay, first-divergence `FailureSignature`, deterministic reducer, and checked-in minimized scenario format |
| COMP-09 | Supervisor taxonomy and poisoned-session lifecycle keep protocol/process/sanitizer/provenance failures outside `PhysicsMismatch` |
| DOCS-05 | Expand `TESTING.md` with versions, commands, diagnosis, review/promotion, minimization/replay, sanitizer and CI tier contracts |

## Recommended implementation order

1. **Protocol core:** add workspace/private manifests, invariant newtypes, exact float bits, limit-aware codec, scenario/trace state machines, tolerance file, schemas, and protocol fixtures.
2. **Pure diagnosis core:** add exhaustive comparator, typed canonicalization, reports, failure signatures, and synthetic minimizer tests.
3. **Native vertical slice:** implement the private Rust empty-world adapter and deterministic trace/replay tests without introducing a public object model.
4. **C++ vertical slice:** vendor the reviewed JSON dependency/notice, add adapter sources and generated identity, build target, strict reset, and one-shot/reuse integration tests.
5. **Supervisor and orchestration:** add concurrent drains, deadlines, poison/reap behavior, `xtask differential` commands, and focused failure-injection helper processes.
6. **Evidence and docs:** stage/replay/promote the reviewed empty-world trace, extend provenance checks, update `ARCHITECTURE.md` and `TESTING.md`, add thin `just` aliases, and wire a small oracle smoke lane.

Keep each plan independently verifiable. The C++ task depends on the protocol fixtures, and the end-to-end/evidence task depends on both adapters; comparator and minimizer work can proceed from fixtures before C++ exists.

## Verification commands

Run the repository-required Rust sequence in order:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

Then exercise private workspace tooling and docs:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --all-features
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
cargo xtask package verify
cargo xtask provenance check
```

With the exact submodule initialized:

```bash
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask differential compare --scenario empty-world --preset oracle-debug
cargo xtask differential compare --scenario empty-world --preset oracle-debug --session-profile reuse
cargo xtask differential replay --scenario empty-world --preset oracle-debug
```

The explicit sanitizer lane should configure/build `oracle-asan-ubsan` and run the same one-shot/reset corpus. Add failure-injection tests for timeout, nonzero exit, partial output, oversized output, wrong request ID, wrong identity, malformed sequence, sanitizer marker, and reset failure. Finally run `cargo xtask check`/`just check`; it must remain useful in Cargo-only mode when the submodule is absent.

## High-risk pitfalls to prevent in planning

- **Serde shape erosion:** `Value` or untagged maps can collapse duplicate members and bypass exhaustive comparison. Parse directly into strict bounded structs.
- **Pipe deadlock:** waiting for stdout while stderr fills can hang forever. Drain both from process start and kill/reap on every poisoned path.
- **False parity:** a valid Rust trace and a crashing/wrong-revision C++ trace are not a mismatch. Validate schema and provenance before comparison.
- **Over-canonicalization:** sorting all output hides solver/callback order defects. Canonicalization requires an explicitly unordered typed field.
- **Global epsilon:** do not add one to make tests green. Phase 2 proves policy machinery with exact empty-world values and synthetic policy tests.
- **Reset leakage:** a reusable process can carry world or semantic-ID state. Require trace-end reset proof and a two-request isolation test.
- **Recovering UBSan:** sanitizer continuation can emit a successful-looking trace. Configure fail-fast and inspect bounded stderr.
- **Golden mutation:** checks must not rewrite artifacts. Stage, replay, diff, review, then atomically promote.
- **Build leakage:** do not add CMake, C++ discovery, protocol dependencies, or a build script to `liquidfun`.
- **Premature schema breadth:** Phase 2 should not guess bodies, handles, callbacks, or final tolerance values. Revise the scenario schema when those audited domains arrive.

## Planning confidence and open points

Confidence is high for the process-isolated JSONL boundary, private-crate split, failure taxonomy, CMake wrapper integration, and exact empty-world proof because they follow the locked Phase-1 architecture and reconciled repository research. The exact diagnostic formatting and reducer heuristics remain implementation discretion. No additional web research or broad upstream inspection is needed to plan this phase; any C++ semantic question beyond empty-world construction belongs to the later pinned-source subsystem audits.
