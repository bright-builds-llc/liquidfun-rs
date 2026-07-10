# Architecture

## Current status

Phase 1 establishes repository boundaries and evidence systems. The publishable
`liquidfun` crate is still a version `0.0.0` native Rust scaffold; it does not
yet implement LiquidFun simulation behavior. The generated
[compatibility inventory](COMPATIBILITY.md) is the authority for implementation
and validation maturity.

## Dependency direction

The repository has two deliberately separate paths:

```text
ordinary Cargo consumer -> crates/liquidfun

maintainer -> just -> cargo xtask -> repository evidence
                                -> CMake wrapper -> read-only upstream oracle
```

Dependencies point toward the native Rust library, never from the library into
repository tooling or upstream C++.

### Published native Rust library

`crates/liquidfun` is the only Cargo default member. Plain `cargo build`,
`cargo test`, and `cargo doc` therefore select the consumer library without
initializing a submodule, finding CMake, or compiling C++.

Production behavior must be implemented in native Rust. Runtime delegation to
LiquidFun C++ is prohibited. No `liquidfun` build script, default feature, or
runtime dependency may discover, compile, link, or load the upstream engine.
The published crate must not depend on `tools/xtask`, `reference/`, or
`third_party/`.

The simulation library remains headless and renderer-independent. Debug-draw
data or traits may eventually live in the library, but windows, input, frame
pacing, UI, and renderer implementations belong in private adapters that
depend on `liquidfun`.

### Private repository orchestration

`tools/xtask` is an unpublished imperative shell. It owns repository effects:
validating boundary data, inspecting Git state, creating and inspecting Cargo
packages, checking generated evidence, and invoking allowlisted external
commands with structured arguments. It does not contain production physics.

The root `justfile` is only a discoverability layer. Recipes expose Cargo and
`cargo xtask` commands without duplicating validation or build logic.

### Read-only C++ oracle

`third_party/liquidfun` is an immutable Git submodule used only for research,
comparison, reference generation, upstream tests, and benchmark comparison.
Repository-owned CMake files under `tools/reference` adapt the legacy build
without changing the submodule. All C++ build outputs stay under
`target/reference/` and outside the consumer package.

The exact repository, revision, release lineage, and patch identity are frozen
in `reference/upstream-lock.toml` and explained in [UPSTREAM.md](UPSTREAM.md).
An initialized mismatch or dirty upstream tree is a hard error.

### Compatibility and provenance evidence

Machine-readable files under `reference/` are repository evidence, not runtime
inputs to `liquidfun`:

- `compatibility.json` is the curated compatibility ledger.
- `discovery.json` is the deterministic pinned-tree discovery snapshot.
- `source-map.toml` records origins, derivation, alterations, and notice classes.
- `artifacts/manifest.toml` records hashes and build provenance for reviewed
  reference artifacts.
- [COMPATIBILITY.md](COMPATIBILITY.md) is generated presentation and is never
  the source of truth.

Tests and check commands validate these records read-only. Regeneration is an
explicit maintainer action followed by review.

## Deferred protocol boundary

Phase 2 may add a private, long-lived out-of-process protocol for semantic
comparison with the C++ oracle. That protocol is not part of Phase 1 and will
not become a feature or dependency of the published crate. In-process FFI is
also deferred unless later profiling demonstrates a concrete need.

## Enforced invariants

- Cargo remains sufficient for ordinary users.
- C++ remains a development oracle, never the production implementation.
- Upstream source stays read-only and all adaptations remain outside it.
- Compatibility claims require explicit ledger evidence; compilation is not
  physics validation.
- Solver-visible ordering and seeded scenarios must remain deterministic unless
  a later reviewed policy explicitly states otherwise.
- Rendering and testbed choices cannot dictate the simulation architecture.
