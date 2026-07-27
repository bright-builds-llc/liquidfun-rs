---
status: resolved
phase: 13-restore-evidence-and-replay-integrity
source:
  - 13-01-SUMMARY.md
  - 13-02-SUMMARY.md
  - 13-03-SUMMARY.md
  - 13-04-SUMMARY.md
  - 13-05-SUMMARY.md
started: 2026-07-25T21:09:02Z
updated: 2026-07-27T15:03:58Z
---

# Phase 13 User Acceptance Testing

## Current Test

[testing complete]

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

expected: Publish the corrected exact producer SHA, dispatch `.github/workflows/phase13-evidence-producer.yml` at that exact ref, and capture the successful run ID, unique artifact ID/name, provider digest, producer SHA P, and canonical bundle digest B.
result: pass
verified_by: agent
evidence: "Final canonical producer run `30232297731` passed at exact producer SHA `6e8261a66a67a05bf3fadb4ad9d818121c395324`; independently acquired artifact `8640500578` (`phase13-staged-30232297731-6e8261a66a67a05bf3fadb4ad9d818121c395324`) has provider digest `sha256:040d7f02c32c40ef6b208f3daf63fb1d458c0cb8cc78cc3d8ccd13e21488e0a7` and canonical bundle digest `fd7fa1a857c0b8cab3ee02fc1d61a45290b632173a4a1f80a790d4334c7453b2`. The earlier producer result remains preserved in the recovery audit history."

### 5. Projection-aware reviewed live replay

expected: The reviewed live replay shares production acquisition, requires the `legacy_physics_v1` projection to match D1, accepts only the reviewed expanded `capture_schema_drift`, and emits bounded typed failure evidence with exact request authority and RFC 6901 divergence pointers.
result: pass
verified_by: agent
evidence: "Focused Phase 13 and catalog failure suites passed 70/70. Canonical acceptance run `30277799121` completed the final ordered command `xtask phase13 evidence live-check --tracked --require-reviewed` successfully."

### 6. Schema-v2 review and incremental promotion

expected: One fresh acknowledgment authorizes a complete schema-v2 review subject covering all seven replacements; Q changes only the mechanically different subset and preserves every unchanged member byte-for-byte.
result: pass
verified_by: agent
evidence: "The fresh acknowledgment names review SHA-256 `58e41c6d754341f9dba8a9fbfb1a0c2d4dbc485fdf46129a680a62e2af5a5735`. Promotion Q `9f3c7c3480a7e371b4d7c39f7050da3ed4a660e5` changes exactly three recorded paths, preserves the other four from R `88aba114356cd84c9464d4e6ff62f1d6d3872af7`, and binds promoted content digest `ca1dd6abeab2977949507aa9ad88e7abf3e9b29f8f4b21570ee725685806a4bb`."

### 7. Exact-head canonical acceptance

expected: Canonical Linux acceptance proves the exact P/B/R/Q/A chain, ordered reviewed replay, and immutable seven-path evidence before publishing one success-only terminal identity.
result: pass
verified_by: agent
evidence: "Run `30277799121` passed at exact accepted head `dbaa64819debc5da268d32fcd342da7632ac6370`. Independently downloaded terminal artifact `8657594142` (`phase13-terminal-identity-30277799121-dbaa64819debc5da268d32fcd342da7632ac6370`) is schema v2, has provider digest `sha256:6e51b5f49937e283761ec9c805552af1de4da2a6cc28fe8c5f1b2e63fc02a304`, records the exact P/B/R/Q/A identities, and reports all seven ordered steps succeeded."

### 8. Fail-closed acceptance retry

expected: A failed acceptance attempt publishes no terminal identity, remains in audit history, and can be repaired without repromotion only when the fix is outside both producer-affecting closures.
result: pass
verified_by: agent
evidence: "Run `30277369306` failed at strict provenance parsing and uploaded no terminal identity. Acceptance-only commit `dbaa64819debc5da268d32fcd342da7632ac6370` added explicit digest-mode validation outside both producer closures; the canonical rerun then passed without changing P/B/R/Q or the acknowledged reviewed bytes."

## Summary

total: 8
passed: 8
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps

[none]
