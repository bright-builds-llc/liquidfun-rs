# liquidfun-rs

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/liquidfun-rs)](https://github.com/bright-builds-llc/liquidfun-rs)
[![CI](https://img.shields.io/github/actions/workflow/status/bright-builds-llc/liquidfun-rs/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/bright-builds-llc/liquidfun-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/bright-builds-llc/liquidfun-rs?style=flat-square)](./LICENSE)
[![Rust 1.97.0](https://img.shields.io/badge/Rust-1.97.0-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

An independent, renderer-neutral Rust implementation of Google's LiquidFun
physics engine, developed against an exact pinned C++ oracle.

## Maturity and evidence

The publishable crate is still version `0.0.0`; this repository has not declared
a parity-bearing v1 release candidate. The native scalar engine includes math,
collision, rigid bodies, contacts, CCD, all eleven joint kinds, standalone rope,
particles and groups, queries, semantic observations, debug primitives, and
safe owned particle-buffer transfer.

Capability is not the same as verified parity. The generated
[compatibility inventory](COMPATIBILITY.md) is authoritative for row-by-row
implementation, differential, platform, and documented-difference evidence.
Historical Phase 4 through Phase 8 corpora remain bounded evidence inputs, not
a generalized claim about the complete project. Performance claims likewise
apply only to immutable reports for named workloads.

A parity-bearing release requires a frozen full candidate commit and a complete
reviewed manifest accepted by fail-closed `cargo xtask release audit`. This
checkout is **not release-ready**: it has no completed full-SHA
`release-candidate` workflow run, retained complete evidence bundle, or tracked
source/manifest/report records accepted by
`cargo xtask release attestation validate`. Local green checks do not substitute
for that run-bound attestation. See [RELEASE.md](RELEASE.md) for the
non-publication rule and the exact path to a future readiness claim.

## Cargo-only install and use

The crate declares Rust 1.92.0 as its v1.0.x MSRV contract. Repository
development is reproducibly pinned to Rust 1.97.0 by `rust-toolchain.toml`.
Until a public release is published, build the reviewed repository checkout:

```bash
cargo build -p liquidfun
cargo test -p liquidfun --all-features
```

Ordinary use is Cargo-only. It does not initialize the upstream submodule,
discover CMake, compile C++, start an oracle process, or include the private
testbed:

```rust
use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, World};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::new()?;
    let body = world.create_body(&BodyDef::new(
        BodyType::Dynamic,
        Vec2::ZERO,
        0.0,
        true,
    )?)?;
    assert!(world.contains_body(body));
    Ok(())
}
```

## Platform support

Every supported lane verifies the same reviewed `.crate` bytes. Platform
results are D2 portability evidence and cannot create or promote canonical D1
physics fixtures.

| Target | Policy tier | Current contract |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | durable supported | Rust 1.97 native verification; canonical Linux also verifies Rust 1.92.0 |
| `aarch64-unknown-linux-gnu` | durable supported | Rust 1.97 native verification |
| `aarch64-apple-darwin` | durable supported | Rust 1.97 native verification |
| `x86_64-pc-windows-msvc` | durable supported | Rust 1.97 native verification |
| `x86_64-apple-darwin` | `conditional_supported` | Requires native evidence no older than 90 days; missing or expired evidence downgrades the current disposition to unsupported |

Targets outside this table are evidence-only unless a reviewed support decision
promotes them.

## Headless, catalog, and testbed workflows

The catalog is the shared renderer-independent scenario authority:

```bash
cargo xtask catalog list
cargo xtask catalog run --scenario rigid-stack-stability --timestep 0.016666668 --velocity-iterations 8 --position-iterations 3 --particle-iterations 1 --oracle-preset oracle-debug --session-profile one-shot --output human --commands auto
```

The private testbed consumes the same semantic catalog and cannot confer parity
or performance authority:

```bash
cargo run -p liquidfun-testbed -- --capability-check --fixture crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json --output target/testbed-capability
cargo run -p liquidfun-testbed --bin interactive
```

See [TESTING.md](TESTING.md) for replay, differential, sanitizer, fuzz, Miri,
coverage, benchmark, and evidence-promotion workflows.

## Optional C++ oracle

Maintainer-only differential work requires the exact recursive upstream
checkout, CMake 3.25 or newer, Ninja 1.11 or newer, and a compatible C++
compiler:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

Canonical Linux evidence records the stricter pinned identities documented in
[UPSTREAM.md](UPSTREAM.md). The C++ oracle is out of process and never enters
the published crate.

## Contributing and licensing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before changing source, evidence, or
generated reports. Original project work is MIT-licensed under [LICENSE](LICENSE).
Pinned upstream and derived materials retain separate attribution, alteration,
and notice duties recorded in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Architecture and evidence

- [UPSTREAM.md](UPSTREAM.md) — immutable oracle identity, ancestry, notices,
  and intentional update policy
- [COMPATIBILITY.md](COMPATIBILITY.md) — generated inventory and explicit
  evidence gaps
- [ARCHITECTURE.md](ARCHITECTURE.md) — native Rust dependency direction and
  oracle-isolation boundary
- [TESTING.md](TESTING.md) — local commands, CI lanes, package proof, and
  deterministic verification policy
- [SAFETY.md](SAFETY.md) — handle, callback, owned-buffer, panic, and zero-unsafe
  contracts
- [RELEASE.md](RELEASE.md) — candidate freeze, audit, package reuse, and
  non-publication policy
