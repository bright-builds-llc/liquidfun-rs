# Phase 11: Examples, Headless Tooling, and Testbed - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-21
**Phase:** 11-examples-headless-tooling-and-testbed
**Mode:** Yolo
**Areas discussed:** Upstream corpus accounting, shared scenario catalog and controls, renderer-neutral observability, optional visual testbed

## Upstream corpus accounting

| Option | Description | Selected |
| --- | --- | --- |
| Flat four-way outcome in `reference/compatibility.json` | Smallest schema change, but conflates disposition with compatibility impact and retains file-level granularity. | |
| Orthogonal disposition and impact fields in the compatibility ledger | Keeps one ledger and models separate concerns, but makes the broad ledger more unwieldy. | |
| Dedicated upstream-corpus manifest joined to existing ledgers | Provides granular semantic identities, many-to-many evidence links, orthogonal disposition and impact, and explicit review while preserving one-way projections. | ✓ |

**Agent's choice:** Dedicated upstream-corpus manifest joined fail-closed to discovery, compatibility, scenarios, and evidence.
**Notes:** The current snapshot counts source files rather than all GoogleTest declarations and registered testbed cases; Phase 11 requires semantic-item closure.

## Shared scenario catalog and controls

| Option | Description | Selected |
| --- | --- | --- |
| Typed catalog compiled to immutable resolved plans | One engine-neutral resolved plan feeds every backend and consumer; a separate controller makes restart and replay exact. | ✓ |
| Canonical declarative JSON catalog | Language-neutral but verbose, duplicates validation, and handles generated or interactive behavior poorly. | |
| Stateful scenario trait/plugin | Natural for testbed code but risks hidden state, dual Rust/C++ logic, and irreproducible restart or minimization. | |

**Agent's choice:** Typed catalog compiled to immutable resolved plans with a separate run-session controller.
**Notes:** Stable slug/version/generator/seed identity and exact resolved bytes align with existing protocol and failure-fixture policy.

## Renderer-neutral observability

| Option | Description | Selected |
| --- | --- | --- |
| Owned semantic observation frame | Simplest capture and replay, but broad owned records can allocate heavily and enlarge public schema. | |
| Borrow-scoped streaming debug sink | Low allocation and renderer-friendly, but requires a second collector/canonicalizer and risks visitation-order coupling. | |
| Layered semantic views plus canonical checkpoint builder | Reuses current views, keeps one deterministic owned authority, and lets rendering stream from the same model while separating timing diagnostics. | ✓ |

**Agent's choice:** Layered semantic views plus one bounded canonical owned checkpoint builder.
**Notes:** Stable semantic keys, closed numeric policies, and a separate non-parity timing channel prevent storage and wall-clock leakage.

## Optional visual testbed

| Option | Description | Selected |
| --- | --- | --- |
| Headless controller plus private Macroquad adapter | Small cross-platform 2D/UI integration with screenshots and render targets; requires a capability spike for dense overlays and diffs. | ✓ |
| Headless controller plus private winit/wgpu/egui adapter | Richer diagnostic UI and GPU control, but much heavier event-loop, backend, and dependency complexity. | |

**Agent's choice:** Build headless first, then prototype Macroquad 0.4.15; use winit/wgpu/egui only after a named required-capability failure.
**Notes:** The visual crate is a passive input/render adapter and remains unpublished and outside default workspace members.

## the agent's Discretion

- Exact names, module boundaries, manifest schema details, bounded capacities, representative scenario grouping, and visual layout within the locked contracts.
- Exact primitive collection/sink adapter and Macroquad acceptance measurements, provided deterministic owned checkpoints remain authoritative.

## Deferred Ideas

- Renderer-specific optimization and advanced GPU inspection beyond the Phase 11 gate.
- Broad performance, portability, packaging, and release-readiness work reserved for Phase 12.

