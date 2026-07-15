---
phase: 09-particle-storage-lifecycle-and-coupling
verified: 2026-07-15T19:24:02Z
status: gaps_found
score: "4/14 requirements verified; phase goal blocked"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-15T19:24:02Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - id: API-09
    status: blocked
  - id: API-10
    status: verified
  - id: PART-01
    status: blocked
  - id: PART-02
    status: blocked
  - id: PART-03
    status: verified
  - id: PART-04
    status: blocked
  - id: PART-05
    status: blocked
  - id: PART-06
    status: verified
  - id: PART-07
    status: blocked
  - id: PART-08
    status: blocked
  - id: PART-14
    status: blocked
  - id: PART-15
    status: blocked
  - id: PART-16
    status: verified
  - id: PART-17
    status: blocked
must_haves:
  roadmap_success_criteria: 1/5
  plan_truths: blocked_by_implementation_and_evidence_gaps
  plan_artifacts: 32/32_present
  plan_key_links: 29/29_present_but_behavior_not_proven
  repository_completion_gates: 0/1
evidence:
  verified_commit: b2391a8d967fa008a1558abc220d9a44fb2c4766
  approved_evidence_commit: a87f84bbdbfe55fb732d74c481c4a4bda9eec958
  evidence_run: 29439515367
  workflow_conclusion: success_but_test_pipeline_failed
  canonical_trace: hash_bound_failed_4_passed_1_failed
  sanitizer_trace: hash_bound_failed_4_passed_1_failed
  rust_vs_cpp_comparison: absent
  executable_branch_coverage: absent
  focused_phase9_corpus_local: passed_5_of_5_without_oracle_mode
  code_review: findings_2_critical_8_warning
gaps:
  - G09-EVIDENCE-PIPELINE
  - G09-DIFFERENTIAL-COMPARISON
  - G09-EXECUTABLE-COVERAGE
  - G09-STEP-GUARD
  - G09-ZOMBIE-AUTHORITY
  - G09-EVICTION-OCCURRENCE
  - G09-PERMUTATION-WEIGHTS
  - G09-PROTOCOL-VALIDATION
human_verification: []
---

# Phase 9: Particle Storage, Lifecycle, and Coupling Verification Report

**Phase goal:** Implement safe, identity-preserving particle systems and their lifecycle, contact, buffer, query, callback, and rigid-coupling foundations.

**Status:** `gaps_found`

**Score:** 4/14 mapped requirements are independently verified. The phase completion gate fails.

## Verdict

Phase 9 is not complete. The native implementation contains substantial, well-tested particle storage, ownership, buffer, view, force, statistics, query, contact, and lifecycle foundations, but the exact-ref evidence accepted for promotion is invalid and several runtime/protocol defects contradict the phase's observable contracts.

The strongest disconfirming evidence is inside the two downloaded artifacts for approved run `29439515367`. Both hash-bound `phase9-trace.log` files report:

- `required_oracle_mode_proves_replay_and_profile_agreement ... FAILED`
- `Phase 9 checkpoint has no legacy predecessor`
- `test result: FAILED. 4 passed; 1 failed`

The canonical trace SHA-256 is `3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64`; the sanitizer trace SHA-256 is `ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070`. Both recompute exactly to the values in their respective `identity.json` files. The workflow nevertheless concluded success because the test command is piped to `tee` without `pipefail`, so the successful `tee` process masked the failing `cargo test` exit status.

The user's approval correctly binds the run to commit `a87f84bbdbfe55fb732d74c481c4a4bda9eec958`; it does not waive this failed executable result or establish parity. The four promoted compatibility rows must therefore be treated as unsupported until corrected evidence is generated.

This verification applied `AGENTS.md`, `AGENTS.bright-builds.md`, the placeholder-only `standards-overrides.md`, and the local verification, testing, and Rust standards. No override applies.

## Blocking Gaps

### G09-EVIDENCE-PIPELINE: exact-ref artifacts prove a failed test run

`.github/workflows/oracle.yml` runs the Phase 9 test through `cargo test ... | tee ...` without enabling pipeline failure propagation. Both approved artifacts contain the same classified oracle failure while the jobs were marked successful. This directly falsifies Plans 09-16 and 09-17's successful-evidence authority and blocks all D1 promotion.

Required closure:

1. Make the canonical and sanitizer test steps fail on the `cargo test` exit status (`set -o pipefail` or an equivalent status-preserving command).
1. Fix the `Phase 9 checkpoint has no legacy predecessor` oracle failure.
1. Regenerate both exact-ref artifacts from a newly reviewed commit and require the hash-bound logs themselves to contain a passing test result.

### G09-DIFFERENTIAL-COMPARISON: no Rust-versus-C++ semantic comparison runs

`corpus_executes_with_stable_ids_and_d0_bytes` compares two native Rust results. `required_oracle_mode_proves_replay_and_profile_agreement` compares C++ debug/replay and debug/release results. No test compares `NativeRigidWorldExecutor` output with the pinned oracle output for the same request, and the closed `phase9-v1` policy registry is never consumed by such a comparison.

Consequently, even a repaired all-green workflow could accept arbitrarily different Rust and C++ particle results. The promoted `differentially_validated` rows are not supported.

Required closure: execute identical requests through both engines, compare every declared semantic path under `phase9-v1`, prove complete policy consumption, and fail at the first mismatch before producing promotable artifacts.

### G09-EXECUTABLE-COVERAGE: branch names are not bound to executed witnesses

The manifest lists lifecycle, eviction, listener/filter, contact, strict-contact, coupling, culling, and ray-directive branches, but the only generated request creates four flag-zero particles, performs basic edits/force/statistics/query/ray actions, marks and compacts one particle, then destroys both systems. It does not step live particle systems beside live rigid bodies. The coverage test only set-compares label strings and never connects a case to request bytes, actions, checkpoints, observations, or reached branches.

Required closure: provide executable branch-specific scenarios and observations, record mechanically which branches each execution reached, and bind all executed requests/results into the evidence artifacts.

## Independently Confirmed Runtime and Protocol Gaps

| Gap | Concrete current behavior | Affected contracts |
| --- | --- | --- |
| Particle step guard | `run_particle_lifecycle_step` and `run_particle_contact_prefix` execute before the fresh-positive-time guard, so zero-time and continuous-continuation calls can repeat particle work. | PART-01, PART-07, PART-08, PART-14, PART-15 |
| Zombie authority | Public creation can store `ZOMBIE` without moving identity to pending, while `mark_particle_for_destruction` calls `mark_delete` without setting the zombie bit. | PART-02, PART-08, PART-14 |
| Capacity occurrence | `prepare_capacity_for_creation` returns compaction occurrences, but both preflight and commit callers discard the outcome. | PART-08, PART-14 |
| Permutation coherence | Permutation candidates initialize weights to zero while retaining remapped contacts, so aggregate views can expose contacts beside stale zero weights. | API-09, PART-04, PART-05 |
| Result validation | A particle action accepts any outer `Particle` observation without validating its nested kind, IDs, ordering, lengths, or action relationship. | PART-07, PART-08, PART-15, PART-17 |
| Request lifecycle validation | Phase 9 action validation checks shapes and vectors but not create/use/destroy order, liveness, ownership, or cross-system references. | PART-01, PART-02, PART-03 |
| Mixed identity | Rust emits live body IDs in mixed state; C++ emits an empty body-ID array. A real comparison would already fail. | PART-07, PART-15 |
| Negative infinite lifetime | Production accepts finite lifetimes at or below zero as infinite, while protocol validation rejects negative finite lifetime bits. | PART-08 |

These findings were validated against the actual implementation and pinned-source integration paths rather than accepted from `09-REVIEW.md` on trust.

## Roadmap Success Criteria

| # | Criterion | Result |
| ---: | --- | --- |
| 1 | Multiple systems and particles expose stable identity, flags, colors, lifetimes, and safe user data. | BLOCKED: the broad API exists, but zombie flags and lifecycle authority disagree and source equivalence is not differentially demonstrated. |
| 2 | Every permutation updates all state atomically while safe views remain coherent. | BLOCKED: identity/reference transactionality is strong, but retained contacts can be published beside zeroed weights. |
| 3 | Safe external buffers enforce ownership, capacity, growth, and teardown contracts. | VERIFIED: owned transfer, explicit fixed/growable limits, allocation preservation, teardown return, and compile-time alias exclusion have focused evidence. |
| 4 | Contacts, strict behavior, lifetimes, zombies, callbacks, and compaction match the pinned oracle. | BLOCKED: runtime defects exist and no valid Rust-versus-C++ evidence executes these branches. |
| 5 | Forces/statistics/queries/callback flags are exposed and differentially verified. | BLOCKED: the APIs are present, but executable branch coverage and cross-engine comparison are absent. |

**Roadmap score:** 1/5.

## Requirement Accounting

| Requirement | Status | Evidence boundary |
| --- | --- | --- |
| API-09 | BLOCKED | Borrow-scoped views and editors exist, but a valid permutation can expose stale zero weights beside retained contacts. |
| API-10 | VERIFIED | Owned lane adoption/return, fixed/growable capacity, no-silent-growth behavior, and alias exclusion are focused-tested. |
| PART-01 | BLOCKED | Multiple-system APIs exist, but zero-time/continuation stepping and invalid evidence prevent upstream-equivalent completion. |
| PART-02 | BLOCKED | Stable creation/destruction APIs exist, but the public zombie flag and pending identity state can diverge. |
| PART-03 | VERIFIED | Independent storage properties and public integration tests preserve scoped stable IDs through reorder and compaction. |
| PART-04 | BLOCKED | Central permutation remaps references atomically, but does not preserve/recompute the required weight lane coherently with retained contacts. |
| PART-05 | BLOCKED | All named semantic views exist, but the view can expose the permutation weight/contact inconsistency. |
| PART-06 | VERIFIED | Fixed and growable owned-buffer behavior is explicit, transactional, and allocation-preserving. |
| PART-07 | BLOCKED | Contact kernels exist, but step timing, mixed identity, executable coverage, and cross-engine parity remain defective. |
| PART-08 | BLOCKED | Lifetime kernels and a narrow tie witness exist, but zombie authority, eviction effects, negative lifetime protocol coverage, and canonical execution are incomplete. |
| PART-14 | BLOCKED | Requested destruction behavior is not authoritative across public marking and capacity eviction. |
| PART-15 | BLOCKED | Local flag-gating tests exist, but the promoted corpus neither executes nor compares the complete callback/filter branches. |
| PART-16 | VERIFIED | Checked force/impulse and semantic statistics APIs have focused transactionality and source-arithmetic tests. |
| PART-17 | BLOCKED | Query/ray APIs exist, but the declared directive/culling branches are not tied to executable differential witnesses. |

## Automated Verification

The focused local command `cargo test -p liquidfun-differential --test phase9_corpus -- --nocapture` passed 5/5 because `LIQUIDFUN_PHASE9_ORACLE_MODE` was unset. That pass is useful disconfirmation: the default suite can remain green while the artifact's required oracle-mode test fails.

The exact downloaded identities and payload digests were recomputed successfully. This proves artifact integrity, but in this case integrity faithfully binds the artifacts to failing logs; it does not prove successful physics evidence.

### Exact local artifact reproduction

The failing payloads are available at these repository-relative paths:

- `target/phase9-evidence/phase9-canonical/phase9-trace.log`
- `target/phase9-evidence/phase9-canonical/identity.json`
- `target/phase9-evidence/phase9-sanitizer/phase9-trace.log`
- `target/phase9-evidence/phase9-sanitizer/identity.json`

The failure can be inspected without executing either engine:

```bash
rg -n 'required_oracle|FAILED|test result|legacy predecessor' \
  target/phase9-evidence/phase9-{canonical,sanitizer}/phase9-trace.log
```

Both logs report the required oracle-mode test failure and `test result: FAILED. 4 passed; 1 failed`.

The exact content identities were recomputed with:

```bash
for directory in \
  target/phase9-evidence/phase9-canonical \
  target/phase9-evidence/phase9-sanitizer
do
  (
    cd "$directory"
    shasum -a 256 phase9-trace.log phase9-manifest.json
    jq -r '.trace.sha256,.manifest.sha256' identity.json
  )
done
```

Results:

- Canonical trace: `3a339387b4c4acccc15b5fc4944d6bec9c7e1d315f4753034ae52a5ff97f2e64`; manifest: `36cfaad1f56505f8427408733e2231ad613984a4cb3eb3b8d757e7a14b2c38e0`.
- Sanitizer trace: `ee75462d49275c5b7d02b8677eb6f9bf82c241c6b993c16d6df08a2ae231a070`; manifest: `0c89f0136eda6689118d3eaa909defb1d182d5723e7a64ea1e958396066dce15`.

Each recomputed value exactly equals the corresponding field in `identity.json`.

The implementation and ordinary Rust suites may otherwise be green, but no repository completion gate can override a failed canonical executable result, absent cross-engine comparison, or the concrete runtime defects above.

## Human Verification

None. The blockers are deterministic code, protocol, workflow, and artifact facts. Additional human approval cannot convert these failed or absent checks into parity evidence.

## Conclusion

Phase 9 must remain open. Repair the evidence pipeline first, close the runtime/protocol gaps, add executable branch witnesses and an actual Rust-versus-pinned-C++ comparator, then publish a new human-approved exact commit and regenerate canonical and sanitizer artifacts. Only after those artifacts contain passing hash-bound logs and the compatibility ledger is regenerated from the new authority should Phase 9 be reverified.

_Verifier: gsd-verifier_

_Result: gaps_found_
