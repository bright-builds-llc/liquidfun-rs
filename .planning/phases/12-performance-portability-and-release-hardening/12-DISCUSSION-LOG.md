# Phase 12: Performance, Portability, and Release Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-23T15:16:59.341Z
**Phase:** 12-performance-portability-and-release-hardening
**Mode:** Yolo
**Areas discussed:** Performance evidence contract, Platform and MSRV support contract, Safety, validation, and release gate

***

## Performance evidence contract

| Option | Description | Selected |
| --- | --- | --- |
| Tiered paired contract | Repository-owned paired Rust/C++ runner over sealed scenarios, plus Criterion microbenchmarks for Rust diagnosis and trends. | ✓ |
| Separate native harnesses | Criterion for Rust and a separate C++ benchmark framework, normalized through one manifest. | |
| Rust trend and budget | Strong Rust regression budgets with C++ runs treated as contextual rather than precise comparative evidence. | |

**User's choice:** Tiered paired contract
**Notes:** Yolo selected the advisor's recommended default. It best supports reproducible Rust/C++ ratios and profile-justified structural optimization without pretending two independently adaptive harnesses are equivalent.

***

## Platform and MSRV support contract

| Option | Description | Selected |
| --- | --- | --- |
| Uniform matrix | Run both Rust toolchains across every claimed native target. | |
| Layered artifact-first | Keep MSRV and native platform axes independent while testing one reviewed package artifact. | ✓ |
| Release-only conformance | Keep broad support but defer most platform evidence to scheduled or release workflows. | |

**User's choice:** Layered artifact-first
**Notes:** Yolo selected the advisor's recommended default. It preserves four durable supported targets plus conditional macOS Intel evidence, avoids a low-value Cartesian matrix, and keeps D1 numerical authority separate from D2 portability.

***

## Safety, validation, and release gate

| Option | Description | Selected |
| --- | --- | --- |
| Commit-bound release audit | Aggregate exact-candidate typed evidence into a fail-closed `cargo xtask release audit`. | ✓ |
| All-in-one workflow | Recompute every release suite in one large same-run workflow. | |
| Required checks plus checklist | Rely on branch checks and a maintainer-authored release checklist. | |

**User's choice:** Commit-bound release audit
**Notes:** Yolo selected the advisor's recommended default. It extends the repository's existing typed evidence architecture, keeps expensive suites isolated, and makes a parity-bearing v1 independently auditable.

***

## the agent's Discretion

- Exact private schemas, command names, job names, sample counts, fuzz budgets, Miri partitioning, and report layout within the locked evidence contracts.

## Deferred Ideas

- SIMD, parallel stepping, alternate precision, WASM, mobile, and `no_std` remain post-v1 work.
