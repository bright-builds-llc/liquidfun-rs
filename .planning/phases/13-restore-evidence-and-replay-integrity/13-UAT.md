---
status: partial
phase: 13-restore-evidence-and-replay-integrity
source:
  - 13-01-SUMMARY.md
  - 13-02-SUMMARY.md
  - 13-03-PLAN.md
started: 2026-07-25T21:09:02Z
updated: 2026-07-25T21:09:02Z
---

# Phase 13 User Acceptance Testing

## Current Test

[testing paused — 1 item outstanding]

## Tests

### 1. Target-scoped witness provenance

expected: The Phase 9 witness provenance contract derives a target-scoped materials closure, rejects edited or incomplete materials, and fails closed when Phase 13 source, alteration, or notice metadata is missing.
result: pass
verified_by: agent
evidence: "`cargo test -p xtask --test phase9_witness_provenance` passed 2/2 tests: `target_scoped_materials` and `phase13_evidence_classes_fail_closed`."

### 2. Rigid-stack replay drift diagnosis

expected: Replay diagnosis checks sealed input before parity-bearing physics, preserves the reviewed legacy identity, and classifies the current `rigid-stack-v1` divergence as expanded capture schema at `$.checkpoints[0].debug_primitives.length`.
result: pass
verified_by: agent
evidence: "`cargo test -p liquidfun-differential --test catalog_regressions diagnosis_` passed 4/4 tests; `cargo test -p liquidfun-differential --test catalog_regressions rigid_stack_v1_diagnosis -- --exact` passed 1/1."

### 3. Local Phase 13 producer and workflow contract

expected: The local producer contract rejects non-identical witness/native repeats, failed D1 comparison, malformed provenance, tampering, closure drift, unsafe staging paths, and symlinks; the workflow contract enforces exact checkout, immutable action pins, one aggregate producer, one unique bundle, acquisition identity publication, and no tracked promotion.
result: pass
verified_by: agent
evidence: "`cargo test -p xtask --test phase13_evidence_contract producer_` passed 11/11; `cargo test -p xtask --test phase13_evidence_contract workflow_` passed 5/5; `actionlint .github/workflows/phase13-evidence-producer.yml` passed."

### 4. Canonical producer dispatch and immutable bundle acquisition

expected: Publish exact producer SHA `21ef24dbca3f59cba81ac303d187a7cda73a2d39`, dispatch `.github/workflows/phase13-evidence-producer.yml` at that exact ref, and capture the successful run ID, unique artifact ID/name, provider digest, producer SHA P, and canonical bundle digest B.
result: blocked
blocked_by: third-party
reason: "This requires an intermediate push of the currently local producer commit and a GitHub Actions dispatch. The strict discuss-plan-execute-commit-and-push workflow has not authorized a pre-verification push, so no external state was changed."

## Summary

total: 4
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 1

## Gaps

[none]
