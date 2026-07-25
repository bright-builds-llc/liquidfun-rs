# Phase 13: Restore Evidence and Replay Integrity - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md; this log preserves the alternatives considered.

**Date:** 2026-07-25
**Phase:** 13-restore-evidence-and-replay-integrity
**Mode:** Yolo
**Areas discussed:** Exact-head provenance authority, independent `rigid-stack-v1` replay recovery, exact-head Linux and Oracle CI acceptance gate

## Exact-head provenance authority

| Option | Description | Selected |
| --- | --- | --- |
| Existing aggregate digest | Regenerate against the current broad adapter digest and freeze the adapter afterward. |  |
| Target-scoped materials | Bind only the complete transitive materials that can affect the Phase 9 witness and regenerate twice on canonical Linux. | ✓ |
| Candidate-bound attestation | Move the witness immediately into a broader reviewed CI attestation model. |  |

**User's choice:** Target-scoped Phase 9 materials digest plus canonical regeneration.
**Notes:** The witness and probe hashes remain stable while unrelated Phase 10–12 inputs caused the aggregate digest to drift. The selected approach repairs the authority model without weakening source binding.

## Independent `rigid-stack-v1` replay recovery

| Option | Description | Selected |
| --- | --- | --- |
| Versioned legacy D0 projection | Preserve the reviewed historical identity while validating newly added checkpoint fields separately. | Conditional |
| Fresh canonical same-run regeneration | Diagnose first, then require two native D0 repeats plus pinned-oracle D1 from the same resolved bytes before reviewed promotion. | ✓ |
| Split physics and diagnostic identities | Migrate to separate long-lived authoritative physics and diagnostic-capture identities. |  |

**User's choice:** Fresh canonical same-run regeneration when the changed fields are intentionally authoritative; otherwise preserve a versioned legacy projection.
**Notes:** Diagnosis must classify the changed semantic paths before any manifest edit. The current native-only helper is not independent evidence and cannot bless a corrected identity.

## Exact-head Linux and Oracle CI acceptance gate

| Option | Description | Selected |
| --- | --- | --- |
| Dedicated atomic Phase 13 gate | One exact-SHA gate composes provenance, replay, diagnosis, regressions, oracle build, and comparisons in fail-closed order. | ✓ |
| Extend canonical Oracle job | Add the missing catalog and diagnosis checks to the existing Oracle workflow job. |  |
| Multi-job DAG | Parallelize producers and join them through an unconditional typed final validator. |  |
| Cross-workflow aggregator | Join existing Cargo, Oracle, and regression workflow evidence into a candidate record. |  |

**User's choice:** Dedicated atomic Phase 13 gate.
**Notes:** The existing primitives are split across Cargo and Oracle CI. The selected gate gives Phase 13 one independently reviewable acceptance boundary without expanding into Phase 15 release aggregation.

## the agent's Discretion

- Exact private schema, module, command, job, and report names.
- Exact plan decomposition within the locked diagnosis, independent-generation, and exact-SHA gates.

## Deferred Ideas

- Windows particle-group invariant repair remains Phase 14.
- Final release-candidate evidence and attestation remain Phase 15.
