# Architecture

## Current status

Phase 2 establishes the permanent semantic-comparison seam and proves one
bounded empty-world scenario through native Rust and the pinned,
process-isolated C++ oracle. The publishable `liquidfun` crate remains a native
Rust `0.0.0` scaffold; this phase does not establish rigid-body or particle
parity, a public world/object model, or final subsystem tolerances. The
[compatibility inventory](COMPATIBILITY.md) remains the authority for maturity.

## Dependency direction

The implemented dependency graph is deliberately one-way:

```text
ordinary Cargo consumer -> crates/liquidfun

crates/liquidfun-differential -> crates/liquidfun-test-protocol
                              -> crates/liquidfun
                              -> subprocess: liquidfun-reference

liquidfun-reference -> repository CMake wrapper -> read-only LiquidFun C++

maintainer -> just -> cargo xtask -> Cargo, differential runner, evidence checks
```

`crates/liquidfun` is the only default member. Neither unpublished harness crate,
`tools/xtask`, CMake, the submodule, nor reference evidence is a dependency or
feature of the published crate. Plain `cargo build`, `cargo test`, and
`cargo doc` therefore remain Cargo-only consumer operations.

Production behavior must be native Rust. C++ is a development oracle, not a
runtime implementation. There is no FFI boundary in Phase 2: the supported
boundary is an external JSON Lines process. An in-process C ABI remains deferred
unless profiling later demonstrates a material bottleneck, and it may never
become a consumer dependency.

## Private protocol and domain core

`crates/liquidfun-test-protocol` is an unpublished functional core. It owns:

- independent protocol, scenario-schema, trace-schema, and tolerance-profile
  version types;
- strict bounded JSON Lines framing and raw-to-domain parsing;
- semantic request, entity, command, checkpoint, provenance, and trace types;
- authoritative `f32` transport as exact `u32` bit patterns;
- named immutable limits and session profiles;
- typed harness failures and deterministic schema/tolerance presentations.

Raw JSON never becomes a generic comparison surface. The Rust and C++ codecs
reject partial records, duplicate or unknown members, unknown record kinds,
unsupported versions, invalid references, and values outside named limits
before execution. Once parsed, the rest of the system receives invariant-bearing
domain values rather than unchecked strings, paths, or maps.

Exact transport bits and comparison policy are separate responsibilities.
Every authoritative float crosses the process boundary exactly. The comparator
then applies the reviewed field policy: exact bits, absolute, relative, or ULP
only where that field's typed policy permits it. This separation prevents a JSON
formatter or global epsilon from silently changing compatibility evidence.

## Differential functional core and imperative shell

`crates/liquidfun-differential` is also unpublished. Its comparator,
canonicalization, first-divergence reporting, failure signatures, and scenario
reducer form the functional core. Typed exhaustive matches compare discrete
values exactly, apply field-specific float policy, and canonicalize only
collections explicitly modeled as sets or multisets. Checkpoints and future
solver, callback, and destruction sequences remain ordered.

The process supervisor, fixture storage, and CLI are the imperative shell. One
synchronous supervisor serves one-shot, bounded reuse, and sanitizer profiles;
there are not separate implementations. Standard-library threads drain stdout
and stderr concurrently. Requests are sequential with exactly one request in
flight. A request budget of one maximizes reproduction isolation, while finite
reviewed budgets exercise reset and bounded process reuse.

Every request constructs isolated adapter state. A complete `trace_end` proves
destruction, cleared semantic mappings, reset verification, and an incremented
reset epoch. Timeout, malformed or oversized output, sanitizer evidence,
unexpected exit, identity mismatch, or reset failure poisons the session. The
supervisor kills when needed, waits and reaps, joins drains, retains bounded
first/last stderr evidence, and never reuses or silently retries a poisoned
child.

The Phase-2 native adapter is private and intentionally supports only the
empty-world lifecycle needed to prove the seam. It is not a provisional public
`World` API and must not force Phase-3 handle, invalidation, callback, or storage
decisions.

## Process-isolated C++ oracle

`tools/reference/liquidfun-reference` is a repository-owned executable built by
the external CMake wrapper and linked to the pinned read-only submodule. Adapter
sources stay outside `third_party/liquidfun`; generated builds stay below
`target/reference/<preset>`.

The executable emits a startup handshake before accepting scenarios. Its build
identity includes the pinned oracle revision, adapter source digest, preset,
compiler, target, effective flags, sanitizer mode, and stable identity hash.
The runner checks that identity independently before comparing semantic values.
Stdout is protocol-only; diagnostics use stderr.

The C++ side parses bounded JSON events directly into closed domain structs,
constructs a fresh `b2World` per request, emits ordered semantic checkpoints,
destroys request state, verifies reset, and only then emits `trace_end`. C++
pointers, STL objects, dense indices, padding, and raw memory never cross the
protocol.

## Comparison and diagnosis boundary

Only two complete schema-, request-, tolerance-, provenance-, payload-, and
reset-validated traces may reach comparison. A `PhysicsMismatch` therefore means
two trustworthy traces differ under a reviewed typed policy. Process crashes,
timeouts, sanitizer reports, malformed output, wrong provenance, unsupported
schemas, output limits, and reset failures remain `HarnessFailure` outcomes.

The primary diagnostic stops at the first divergent checkpoint, phase, semantic
path, and mismatch kind while retaining adjacent identities and exact-bit plus
human-readable float evidence. A failure signature is stable across reduction;
a candidate that changes the first divergence is a different failure.

Ordering is never globally normalized. Only payloads explicitly typed as
unordered use stable set or multiset canonicalization. NaN, infinities, and
signed zero follow explicit field policy and are never silently normalized.

## Reviewed artifact boundary

Protocol fixtures, named scenarios, and reviewed comparison evidence are
different artifact classes:

1. `protocol/fixtures` proves strict transport acceptance and rejection.
1. `scenarios/phase-02` owns hand-reviewed semantic input.
1. `reference/artifacts` and `scenarios/regressions` own provenance-bound traces
   and minimized same-signature regressions.

Generation writes only below `target/differential/staging`. The lifecycle
replays and validates the candidate, renders a reviewable diff, binds explicit
review metadata, and promotes through a confined no-clobber atomic rename.
`reference/artifacts/manifest.toml` records content/request/scenario/payload
hashes, all four versions, tolerance identity, oracle/adapter/build identity,
compiler/target/flags, source or seed, notices, and review status. Checks are
read-only and never regenerate or hand-mutate golden data.

## Contributor orchestration

`tools/xtask` is a private imperative shell. Its `differential` commands accept
only registered scenarios, presets, session profiles, and lifecycle shapes,
verify upstream identity for oracle-dependent work, and invoke the runner with
canonical structured arguments. `cargo xtask docs check`, protocol fixture
tests, provenance checks, and package verification enforce the checked-in
contracts without generating them.

The root `justfile` remains a transparent discoverability facade. Each
differential or lifecycle recipe is one visible `cargo xtask` command; it owns
no validation, loops, retry policy, evidence mutation, or C++ logic.

## Enforced invariants

- `liquidfun` stays the sole published default member and has no harness,
  serialization, C++, reference-data, or build-script leakage.
- Protocol and comparison decisions live in typed functional cores; process,
  filesystem, Cargo, CMake, and Git effects stay in thin imperative shells.
- One bounded supervisor owns one-shot, reuse, and sanitizer execution with one
  request in flight, reset proof, poison, kill, wait, and reap semantics.
- Exact bits preserve transport fidelity; only reviewed typed field policy may
  permit numeric tolerance.
- Solver-visible and callback/destruction order is preserved; there is no global
  sort or generic JSON-path comparison policy.
- C++ remains process-isolated, read-only, and external to all consumer paths;
  there is no FFI or runtime delegation.
- Artifact checks are read-only. Stage, replay, diff, explicit review, and atomic
  promote are the only accepted mutation path.
- Phase 2 proves the empty-world harness seam only. Broad physics behavior,
  public object ownership, and final tolerance policy remain later-phase work.
