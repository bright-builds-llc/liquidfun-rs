# liquidfun-rs

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/liquidfun-rs)](https://github.com/bright-builds-llc/liquidfun-rs)
[![CI](https://img.shields.io/github/actions/workflow/status/bright-builds-llc/liquidfun-rs/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/bright-builds-llc/liquidfun-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/bright-builds-llc/liquidfun-rs?style=flat-square)](./LICENSE)
[![Rust 1.97.0](https://img.shields.io/badge/Rust-1.97.0-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

An in-progress effort to build an independent native Rust implementation of
Google's LiquidFun physics engine against a pinned C++ oracle.

## Status

This repository is at an early vertical-slice stage. The `liquidfun` crate is
version `0.0.0` and now provides the Phase 6 minimal rigid-world vertical slice:
checked static, kinematic, and dynamic bodies; immutable-shape fixtures;
automatic proxy/contact lifecycle; and one bounded static/dynamic contact
solve. The read-only C++ oracle and private harness verify bounded Phase 4 math,
the 78-case Phase 5 collision corpus, and both Phase 6 `phase6-v1` timelines:
`non_colliding_body_fixture_lifecycle` and `single_contact_lifecycle`. The
generated [compatibility inventory](COMPATIBILITY.md) records each row only at
its demonstrated dimensions.

This is not broad rigid-body support. Phase 7 still owns forces, velocity
controls, damping, sleeping, the general island solver, multi-contact stacks,
CCD/TOI world orchestration, queries, ray casts, and broad world configuration;
joint solving follows in Phase 8. Canonical-platform evidence, performance,
particles, and production maturity also remain pending.

Phase 5's world contact lifecycle gap is covered only by the bounded Phase 6
slice described above. The Phase 5 immutable shape/collision substrate and its
fixed 78-case Phase 5 collision corpora remain the geometric evidence
foundation.
The canonical-platform evidence, performance, and production maturity remain pending.

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

The fixed rigid-world evidence commands are `just rigid-world-debug`,
`just rigid-world-release`, `just rigid-world-replay`, and
`just rigid-world-determinism`. They require the initialized pinned C++ oracle,
exercise both declared Phase 6 timelines, and report local passes as D2 plus
same-build byte identity as D0. They do not promote canonical D1 fixtures or a
platform claim.

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
