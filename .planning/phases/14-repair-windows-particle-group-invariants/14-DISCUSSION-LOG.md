# Phase 14: Repair Windows Particle-Group Invariants - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-07-27
**Phase:** 14-repair-windows-particle-group-invariants
**Mode:** Yolo
**Areas discussed:** Root-cause and transaction boundary, Failure semantics and rollback observability, Regression and platform evidence

## Root-Cause and Transaction Boundary

| Option | Description | Selected |
| --- | --- | --- |
| Repair the existing clone-and-swap `ParticleSystem` candidate | Preserve one authoritative `ParticleStorage`, the established shell preflight, and the short final swap while diagnosing the first invalid candidate transition. | ✓ |
| Replace incremental mutation with one prepared storage payload | Encode the complete creation transition in one storage-owned payload if incremental candidate construction is proven structurally unable to preserve invariants. | |

**User's choice:** Auto-selected the existing clone-and-swap candidate as the recommended default, with escalation only if diagnosis proves it insufficient.
**Notes:** Platform conditionals, validation bypasses, topology regeneration, identity substitution, and parallel storage paths are excluded because they would mask the root cause or weaken locked Phase 9/10 contracts.

## Failure Semantics and Rollback Observability

| Option | Description | Selected |
| --- | --- | --- |
| Existing typed no-effect topology error | Map legitimate isolated-candidate validation rejection to `CreateObjectError::InvalidParticleGroupTopology` while proving comprehensive rollback. | ✓ |
| New public storage-invariant error | Add a precise public variant that exposes the private storage failure category. | |
| Keep `unreachable!` after repairing the immediate cause | Preserve panic semantics for all storage invariant errors even though a safe public Windows sequence reached the path. | |

**User's choice:** Auto-selected the existing typed no-effect error boundary for candidate-side rejection.
**Notes:** Panic and debug assertions remain appropriate only for corruption of authoritative live storage or states proven unreachable from safe public candidate construction.

## Regression and Platform Evidence

| Option | Description | Selected |
| --- | --- | --- |
| Named public regression plus existing OS matrix | Persist the exact audited seed and minimized controls, assert public and private rollback, and run the focused/full relevant suites on Windows, Linux, and macOS. | ✓ |
| Proptest persistence file plus existing OS matrix | Rely on framework-native seed replay tied to the current strategy and generator. | |
| Dedicated Phase 14 evidence workflow | Add a new candidate-bound receipt and validation workflow for platform results. | |

**User's choice:** Auto-selected a named exact-input regression within the existing multi-platform CI matrix.
**Notes:** The current checked regression seed differs from the audit seed. Platform results stay D2 and cannot promote canonical compatibility evidence.

## the agent's Discretion

- Narrow implementation names and decomposition after the first invalid transition is reproduced.
- Regression helper placement and exact affected-suite command composition.
- Escalation from the existing candidate boundary to a storage-owned payload only when diagnosis supplies concrete evidence.

## Deferred Ideas

- Phase 15 owns final release-candidate evidence aggregation and frozen-source attestation.
