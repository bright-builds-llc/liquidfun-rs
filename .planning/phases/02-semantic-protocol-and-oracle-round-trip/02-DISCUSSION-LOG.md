# Phase 2: Semantic Protocol and Oracle Round Trip - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-10
**Phase:** 2-semantic-protocol-and-oracle-round-trip
**Mode:** Yolo
**Areas discussed:** Scenario and protocol contract, Oracle process lifecycle and failure semantics, Semantic trace comparison and regression workflow

***

## Scenario and Protocol Contract

| Option | Description | Selected |
| --- | --- | --- |
| Strict self-contained JSONL request/trace envelopes | Simplest deterministic replay and hashing with one complete trace record, but large traces later require another framing version. | |
| Atomic scenario request plus typed streamed trace records | Keeps scenario hashing/minimization simple while bounding checkpoint records and supporting first-divergence diagnostics. | ✓ |
| JSON-RPC 2.0 over JSONL | Adds standardized method/error correlation but not framing, domain versions, float encoding, bounds, or the required failure taxonomy. | |

**User's choice:** Atomic scenario request plus typed streamed trace records (yolo recommended default).
**Notes:** Use independent integer versions, exact `f32` bit patterns, deterministic typed semantic IDs, strict typed validation, and explicit byte/count/output bounds.

## Oracle Process Lifecycle and Failure Semantics

| Option | Description | Selected |
| --- | --- | --- |
| One subprocess per scenario | Strongest isolation and simplest crash attribution, but poor throughput for corpora. | |
| Sequential long-lived fail-stop session | Amortizes startup with deterministic one-request-at-a-time behavior, but requires reset, concurrent pipe draining, and poisoned-session handling. | |
| Bounded batch-cycled session | Combines reusable sessions with explicit request/resource bounds and periodic exit-time sanitizer checks. | ✓ |

**User's choice:** One configurable supervisor with bounded batch-cycled sessions (yolo recommended synthesis).
**Notes:** A request budget of one provides one-shot isolation; finite budgets provide reuse. Handshake provenance, concurrent capped stderr draining, timeout kill-and-wait, and typed harness failures are mandatory.

## Semantic Trace Comparison and Regression Workflow

| Option | Description | Selected |
| --- | --- | --- |
| Typed policy comparator plus manifested fixtures | Makes exact/numeric/unordered/ordered semantics exhaustive and preserves audited minimized evidence. | ✓ |
| Declarative path-rule comparator plus reviewed goldens | Easier policy-data editing, but stale paths and broad normalization can leave fields unchecked or hide order defects. | |
| Property-based differential runner plus shrink-first persistence | Strong scheduled discovery approach after comparator semantics exist, but generator/shrinker drift can change reproduction. | |

**User's choice:** Typed policy comparator plus manifested fixtures (yolo recommended default).
**Notes:** Exact comparison is the default; float policy is field-specific and versioned. Only explicitly unordered typed collections are canonicalized. Minimized fixtures preserve the same failure signature and full provenance.

## Agent's Discretion

- Private crate/module names and exact protocol spelling.
- Named default bounds and process request budgets within the locked fail-closed model.
- Diagnostic presentation and the invariant-preserving minimization algorithm.

## Deferred Ideas

- Broad subsystem tolerance values and observables.
- Public Rust object-model decisions.
- In-process FFI acceleration.
- Large randomized/property-based corpora and concurrent/distributed oracle services.
