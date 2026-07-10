# Phase 1: Oracle, Provenance, and Repository Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in `01-CONTEXT.md`; this log preserves the alternatives considered.

**Date:** 2026-07-09
**Phase:** 1-oracle-provenance-and-repository-foundation
**Mode:** Yolo
**Areas discussed:** Oracle selection and provenance, Cargo-first repository foundation, Compatibility inventory and evidence governance

***

## Oracle Selection and Provenance

| Option | Description | Selected |
| --- | --- | --- |
| Official `v1.1.0` release commit | Resolve the release tag to an immutable commit, verify ancestry and buildability, and use it as the clearest audit baseline. | ✓ |
| Official post-tag candidate `7f204…` | Reuse the exact official candidate tree already inspected by project research, but require a complete release delta and rationale. | |
| Official base plus patch series | Preserve a release base while applying versioned project patches with preimage hashes and behavioral classifications. | |

**Auto-selected choice:** Official `v1.1.0` release commit.
**Notes:** The release-first choice has the clearest identity and lineage. A bounded audit may still justify the post-tag candidate; wrapper-only build compatibility is preferred over source patches.

***

## Cargo-First Repository Foundation

| Option | Description | Selected |
| --- | --- | --- |
| Balanced shallow workspace | One publishable `liquidfun` crate, private `xtask`, Cargo default-members isolation, and an external CMake/Ninja subprocess oracle. | ✓ |
| Minimal package plus direct presets | One crate and direct `just`/CMake preset calls, with only minimal validation tooling. | |
| Nested tooling workspace | Separate production and tooling Cargo workspaces with independent lockfiles and policies. | |
| Expanded private workspace | Create protocol and differential crates in Phase 1 alongside the engine and `xtask`. | |

**Auto-selected choice:** Balanced shallow workspace.
**Notes:** This establishes a durable contributor seam without pulling Phase 2 protocol decisions forward or fragmenting the workspace.

***

## Compatibility Inventory and Evidence Governance

| Option | Description | Selected |
| --- | --- | --- |
| Hand-maintained Markdown matrix | Maintain a readable compatibility table directly, with limited mechanical exhaustiveness guarantees. | |
| Single manifest plus generated documentation | Use one authoritative curated manifest and deterministically generate human documentation. | |
| Manifest-led hybrid | Combine a curated authoritative inventory with a conservative discovery snapshot, generated report, and provenance-bearing artifact manifests. | ✓ |
| Distributed subsystem ledgers | Keep evidence beside each subsystem and aggregate multiple ledgers into the public report. | |

**Auto-selected choice:** Manifest-led hybrid.
**Notes:** Independent evidence dimensions, reviewed exclusions, provenance agreement, generated-file integrity, and package-isolation checks best match Phase 1's acceptance criteria.

***

## Agent's Discretion

- Exact machine-readable file formats and internal generator layout.
- Exact validated local tool floors, CMake preset names, and human report presentation.
- Exact `xtask` command naming, provided `just` remains thin and commands are directly discoverable.

## Deferred Ideas

- Phase 2 owns the semantic protocol and differential runner.
- In-process FFI waits for profiling evidence.
- Bazel waits for a measured need and ADR.
- Distributed evidence ledgers wait for demonstrated scale or concurrency pressure.
