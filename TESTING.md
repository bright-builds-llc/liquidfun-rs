# Testing and Verification

## Current scope

Phase 1 verifies the repository foundation: Cargo isolation, package contents,
the pinned upstream identity, deterministic inventory generation, provenance
records, and a real CMake oracle build. The native physics engine has not been
implemented, so there are no physics-parity results to report yet. See
[COMPATIBILITY.md](COMPATIBILITY.md) for the exact evidence gaps.

## Required Rust sequence

Before a commit, run these commands in order:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

The root workspace defaults to `crates/liquidfun`. When repository tooling is
changed, also exercise its targets explicitly:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --workspace --all-targets --all-features
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

CI uses `cargo fmt --all --check` so formatting drift fails without rewriting
files.

## Aggregate foundation check

Run the read-only repository aggregate with either command:

```bash
cargo xtask check
just check
```

With the submodule initialized, the aggregate fails fast through inventory,
package isolation, upstream identity, and provenance checks. Without an
initialized `third_party/liquidfun`, it reports a labeled Cargo-only mode,
skips the oracle-dependent checks, and still verifies the independently
unpacked consumer package.

## Focused foundation commands

### Consumer package isolation

```bash
cargo package -p liquidfun --list
cargo xtask package verify
```

`package verify` inspects the archive before extraction, rejects repository or
native-source leakage, then builds and tests the unpacked crate outside the
repository with Rust 1.92.0 and without `third_party/` or `reference/`.

### Inventory and provenance

```bash
cargo xtask inventory check
cargo xtask upstream verify
cargo xtask provenance check
```

These are read-only checks. Refresh discovery and human presentation only as
explicit reviewed actions:

```bash
cargo xtask inventory discover
cargo xtask inventory generate
```

Tests must not regenerate checked-in evidence as a side effect.

### Oracle debug build

Initialize the exact submodule first, then configure and build the external
wrapper:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

`just oracle-debug` is the thin configure-and-build alias. Outputs remain under
`target/reference/`; the upstream worktree must remain clean.

## CI lanes

| Lane | Trigger | Evidence |
| --- | --- | --- |
| Cargo Linux quality | pull request, `main`, manual | format, workspace Clippy/build/test/docs, inventory-checker regressions, package list, unpacked package isolation |
| Cargo default features | pull request, `main`, manual | `liquidfun` build/test on Ubuntu, macOS, and Windows without submodules |
| Rust 1.92 MSRV | pull request, `main`, manual | all-target/all-feature `liquidfun` check at the provisional MSRV |
| Canonical Linux oracle | pull request, `main`, manual | exact CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8 identities; upstream, provenance, inventory, and `oracle-debug` build |
| Oracle portability | manual only | native Apple Clang and Windows clang-cl `oracle-debug` builds; these do not publish canonical artifacts |

Workflows use read-only repository permissions, initialize submodules only in
oracle jobs, and pin external actions by full commit SHA.

## Determinism and retries

Deterministic physics, inventory, provenance, and package checks are not
automatically retried. A failure is investigated as a product, evidence, or
infrastructure result; rerunning a deterministic mismatch does not turn it
into a pass. Generated files must compare byte-for-byte in check mode.

Persist the exact seed, request, stderr, tool identity, and comparator report
when later differential suites fail. Semantic state is the comparison surface;
raw object memory, pointer values, and padding are never compatibility evidence.

## Scheduled and manual expensive suites

As their implementation phases land, full upstream tests, sanitizers, Miri,
fuzzing, coverage, randomized differential corpora, and controlled performance
runs belong in named scheduled or manual lanes. They should not silently expand
the fast pull-request path or overwrite canonical evidence from portability
jobs. Phase 1 does not claim that these future suites exist or pass.
