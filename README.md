# liquidfun-rs

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/liquidfun-rs)](https://github.com/bright-builds-llc/liquidfun-rs)
[![License](https://img.shields.io/github/license/bright-builds-llc/liquidfun-rs?style=flat-square)](./LICENSE)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

An in-progress effort to build an independent native Rust implementation of
Google's LiquidFun physics engine against a pinned C++ oracle.

## Status

This repository is at the foundation stage. The `liquidfun` crate is a version
`0.0.0` scaffold with no physics simulation behavior yet. The current work
freezes the upstream oracle, proves Cargo-only package isolation, builds the
read-only C++ reference, and tracks 177 compatibility rows. The generated
[compatibility inventory](COMPATIBILITY.md) currently records zero implemented,
unit-tested, differentially validated, or platform-validated rows.

Do not use this crate for simulation yet. Maturity will be reported only as
evidence is added to the compatibility ledger.

## Cargo-only quick start

Ordinary Rust development does not require the upstream submodule, CMake, or a
C++ compiler:

```bash
cargo build
cargo test
```

The workspace selects only `crates/liquidfun` by default. Repository tooling
and the C++ oracle remain private maintainer workflows.

## Repository workflows

List the transparent contributor commands with `just` or `just --list`. Run
the applicable foundation checks with `cargo xtask check` or `just check`.
See [TESTING.md](TESTING.md) for the exact verification tiers and
[CONTRIBUTING.md](CONTRIBUTING.md) before sending changes.

## Architecture and evidence

- [UPSTREAM.md](UPSTREAM.md) — immutable oracle identity, ancestry, notices,
  and intentional update policy
- [COMPATIBILITY.md](COMPATIBILITY.md) — generated inventory and explicit
  evidence gaps
- [ARCHITECTURE.md](ARCHITECTURE.md) — native Rust dependency direction and
  oracle-isolation boundary
- [TESTING.md](TESTING.md) — local commands, CI lanes, package proof, and
  deterministic verification policy

Original project work is MIT-licensed. Upstream and derived materials retain
their applicable provenance, alteration, and notice duties; see
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
