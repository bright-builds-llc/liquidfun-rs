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
version `0.0.0` and now provides a Phase 8 checked joint and rope slice on top
of the Phase 7 checked rigid-world slice: all eleven world-owned joint kinds, an independent
standalone rope, source-timed collision/pre-solve decisions, owned lifecycle
and destruction evidence, and semantic reconstruction with explicit unsupported
cases. The generated
[compatibility inventory](COMPATIBILITY.md) records each row only at its
demonstrated dimensions.

The earlier Phase 6 minimal rigid-world vertical slice remains in the locked
corpus. Its `phase6-v1` timelines,
`non_colliding_body_fixture_lifecycle` and `single_contact_lifecycle`, cover
checked body/fixture ownership, automatic proxy/contact lifecycle, and the
initial bounded contact solve. That boundary retains its fixed 128-action step,
`BodyTypeChangeError` and `FixtureDestructionError`, and the rule that
positive-origin custom mass requires finite, strictly positive centered inertia.
The real rigid fixture lifecycle requires canonical D1 authority before every write
and independently recomputes the current checkout's adapter-source and effective compile-command digests
before stage, review, or promotion mutation.

The Phase 7 `phase7-v1` nine-family request retains those two families and adds
force/configuration, multi-contact/warm-start, sleep/wake, CCD/sub-step,
continuous-budget, query/ray, and origin-shift witnesses. Local debug, release,
replay, sanitizer, and two-run determinism remain D2/D0 evidence only: the
executed signoff is local D2 and same-build D0, without canonical D1 promotion,
D3 review, or platform coverage. The scheduled Clang ASan/UBSan lane executes
both the C++ protocol tests and the rigid-world path; that wiring does not widen
the local claim.

The Phase 8 `phase8-v1` request accumulates 19 required witness families: the
Phase 6 and Phase 7 families plus all joint types, gear dependencies, standalone
rope, filter/pre-solve/listener timing, destruction cascades, and semantic
reconstruction. GitHub Actions
[run 29379350740](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29379350740)
at commit `e0b5106559b3c0c37beb44e4ade45c3b7919b59d` established
canonical scalar rigid-body and joint differential sign-off for the closed Phase 8 corpus. Its
exact artifacts are
`phase8-canonical-29379350740-e0b5106559b3c0c37beb44e4ade45c3b7919b59d`
and
`phase8-sanitizer-29379350740-e0b5106559b3c0c37beb44e4ade45c3b7919b59d`.
Both bind upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`,
Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, and `phase8-v1`.

This remains a scoped scalar corpus result, not broad project parity. RIGD-10,
particles, D3 review, cross-platform parity, performance, the testbed, and
release readiness remain pending.

Phase 5's world contact lifecycle gap is covered by the bounded later slices.
The Phase 5 immutable shape/collision substrate and its fixed 78-case Phase 5
collision corpora remain the geometric evidence foundation. The private oracle
continues to verify bounded Phase 4 math and those 78-case Phase 5 collision corpora.
The canonical-platform evidence, performance, and production maturity remain pending.

Do not use this crate for simulation yet. Maturity will be reported only as
evidence is added to the compatibility ledger.

## Cargo-only quick start

Ordinary Rust development is Cargo-only and does not require the upstream
submodule, CMake, or a C++ compiler:

```bash
cargo build
cargo test
```

The workspace selects only `crates/liquidfun` by default. Repository tooling
and the private, optional C++ oracle remain maintainer workflows.

## Repository workflows

List the transparent contributor commands with `just` or `just --list`. Run
the applicable foundation checks with `cargo xtask check` or `just check`.
See [TESTING.md](TESTING.md) for the exact verification tiers and
[CONTRIBUTING.md](CONTRIBUTING.md) before sending changes.

The fixed rigid-world evidence commands are `just rigid-world-debug`,
`just rigid-world-release`, `just rigid-world-replay`, and
`just rigid-world-determinism`. They require the initialized pinned C++ oracle,
exercise the 19-family Phase 8 request, and report local passes as D2 plus
same-build byte identity as D0. They do not substitute for the exact canonical
run above, establish D3 evidence, or make another platform claim.

Maintainers can stage the same typed rigid transaction with
`just rigid-fixture-stage <artifact-id>`, then use the explicit review and
promotion recipes. Each mutation repeats the D1 guard; a local D2 run is
rejected before the staging or accepted-evidence tree changes.

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
