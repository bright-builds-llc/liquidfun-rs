---
phase: "09"
slug: "particle-storage-lifecycle-and-coupling"
status: verified
threats_total: 91
threats_closed: 91
threats_open: 0
accepted_risks: 0
transferred_risks: 0
asvs_level: 1
block_on: high
created: "2026-07-18"
verified: "2026-07-18"
---

# Phase 09 — Security

> Per-phase security contract for particle storage, lifecycle, coupling, protocol, evidence, and compatibility-promotion boundaries.

## Audit Result

**SECURED:** All 91 planned threat and boundary entries are closed. There are no open high-severity threats, accepted risks, transferred risks, or unregistered summary threat flags.

The audit verified the mitigations against implementation, tests, workflow configuration, exact-ref evidence, and generated compatibility claims rather than accepting plan or summary assertions alone.

## Trust Boundaries

| Boundary | Description | Data crossing |
| --- | --- | --- |
| Particle domain integrity | Public definitions, handles, buffers, callbacks, and mutations enter identity- and invariant-bearing native Rust state. | Floats, flags, capacities, typed handles, borrowed views, lifecycle effects |
| Protocol and process isolation | Strict JSONL records cross between Rust tooling and the private C++ oracle process. | Bounded requests/results, stable semantic IDs, diagnostics, provenance |
| Comparator and semantic proof | Native and oracle observations become compatibility decisions and persisted proof roles. | Canonical request bytes, retained rigid state, policies, bindings, proof paths |
| Evidence acquisition | A reviewed local commit crosses into GitHub workflow execution and downloaded archives. | Full SHA, run/job/artifact identities, archive entries, hashes, logs |
| Promotion and reporting | Validated exact-ref evidence becomes public compatibility and phase-tracking claims. | Evidence references, allowlisted ledger rows, generated Markdown, GSD status |

## Mitigation Evidence

| Evidence | Verified control |
| --- | --- |
| E1 — Identity and storage | World/system/generation scope validation, retired identities, lane invariants, and bounded capacity in `crates/liquidfun/src/particle/storage.rs` and `crates/liquidfun/src/particle/storage/permutation.rs`. |
| E2 — Transactional mutation | Candidate-before-commit permutation and particle-creation preflight/receipt behavior in `crates/liquidfun/src/particle/storage/permutation.rs` and `crates/liquidfun/src/world/particle_object.rs`. |
| E3 — Lifecycle, borrow, and panic safety | Full step rollback, coherent panic poisoning, lifecycle compaction, borrowed semantic views, and ordered owned effects in `crates/liquidfun/src/world/step.rs` and `crates/liquidfun/src/particle/lifetime.rs`. |
| E4 — Protocol boundary | Closed enums, reviewed aggregate limits, deny-unknown decoding, 58-binding registry, Rust result validation, and C++ pre-allocation validation in `crates/liquidfun-test-protocol` and `tools/reference/src/rigid_world_phase9_decode.hpp`. |
| E5 — Comparator | One canonical request digest is executed by both roles and complete Phase 8 retained-rigid comparison precedes Phase 9 particle filtering in `crates/liquidfun-differential/src/rigid_world.rs`. |
| E6 — Semantic proof | Typed proof roles, exact case-rooted schema-v4 paths, baseline denial, equality allowlist, required pair inequality, and pre-deduplication topology checks in `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs` and `tools/xtask/src/phase9_evidence.rs`. |
| E7 — Workflow and shell | Read-only contents permission, SHA-bound artifact names, fixed Phase 9 commands, fail-fast sanitizer execution, `set -euo pipefail`, fixed output roots, and no secret output in `.github/workflows/oracle.yml` and `scripts/phase9-evidence.sh`. |
| E8 — Exact authority and archives | Full SHA/run/job/artifact cardinality, live metadata equality, historical denysets, same-run pairing, bounded archives, exact regular-file sets, traversal/symlink denial, and recomputed hashes in `tools/xtask/src/phase9_evidence.rs`. |
| E9 — Promotion and reporting | Exact four-row Phase 9 allowlist, five-row Phase 10 denial, rejected-authority filtering, JSON-owned deterministic report generation, and idempotence checks in `tools/xtask/src/inventory/validation.rs` and `tools/xtask/src/inventory.rs`. |
| E10 — Verified isolation | Plans 09-18 and 09-19 changed runtime/native call sites only and introduced no JSONL decoder, workflow, artifact-authority, compatibility-ledger, or promotion boundary. |

## Threat Register

Categories use STRIDE initials: spoofing (`S`), tampering (`T`), repudiation (`R`), information disclosure (`I`), denial of service (`D`), and elevation of privilege (`E`). An asterisk marks a category inferred from a later plan's trust-boundary description.

| Threat ID | Category | Disposition | Evidence | Status |
| --- | --- | --- | --- | --- |
| T-09-01-01 | T | mitigate | E1 | closed |
| T-09-01-02 | D | mitigate | E1 | closed |
| T-09-02-01 | T | mitigate | E2 | closed |
| T-09-02-02 | E | mitigate | E1, E2 | closed |
| T-09-03-01 | S | mitigate | E1 | closed |
| T-09-03-02 | T | mitigate | E2, E3 | closed |
| T-09-04-01 | T | mitigate | E1, E2 | closed |
| T-09-04-02 | D | mitigate | E1 | closed |
| T-09-05-01 | I | mitigate | E3 | closed |
| T-09-05-02 | T | mitigate | E2, E3 | closed |
| T-09-06-01 | T | mitigate | E3 | closed |
| T-09-06-02 | R | mitigate | E2, E3 | closed |
| T-09-07-01 | T | mitigate | E3 | closed |
| T-09-07-02 | D | mitigate | E3 | closed |
| T-09-08-01 | T | mitigate | E2, E3 | closed |
| T-09-08-02 | R | mitigate | E3 | closed |
| T-09-09-01 | T | mitigate | E2, E3 | closed |
| T-09-09-02 | E | mitigate | E3 | closed |
| T-09-10-01 | T | mitigate | E3 | closed |
| T-09-10-02 | I | mitigate | E3 | closed |
| T-09-11-01 | T | mitigate | E3 | closed |
| T-09-11-02 | D | mitigate | E3 | closed |
| T-09-12-01 | D | mitigate | E4 | closed |
| T-09-12-02 | R | mitigate | E4, E5 | closed |
| T-09-13-01 | D | mitigate | E4 | closed |
| T-09-13-02 | I | mitigate | E4, E5 | closed |
| T-09-14-01 | R | mitigate | E6 | closed |
| T-09-14-02 | S | mitigate | E7, E8 | closed |
| T-09-15-01 | S | mitigate | E7, E8 | closed |
| T-09-15-02 | R | mitigate | E6, E8 | closed |
| T-09-16-01 | S | mitigate | E8 | closed |
| T-09-16-02 | T | mitigate | E8 | closed |
| T-09-17-01 | S | mitigate | E8 | closed |
| T-09-17-02 | T | mitigate | E8, E9 | closed |
| T-09-18-01 | T* | mitigate | E1, E3 | closed |
| T-09-18-02 | T* | mitigate — verified not applicable | E10 | closed |
| T-09-18-03 | R* | mitigate — verified not applicable | E10 | closed |
| T-09-18-04 | D* | mitigate | E1 | closed |
| T-09-19-01 | T* | mitigate | E2 | closed |
| T-09-19-02 | T* | mitigate | E2, E3 | closed |
| T-09-19-03 | T* | mitigate — verified not applicable | E10 | closed |
| T-09-19-04 | R* | mitigate — verified not applicable | E10 | closed |
| T-09-20-01 | T* | mitigate | E4 | closed |
| T-09-20-02 | R* | mitigate | E4 | closed |
| T-09-20-03 | R* | mitigate | E8, E9 | closed |
| T-09-20-04 | D* | mitigate | E4 | closed |
| T-09-21-01 | S* | mitigate | E5 | closed |
| T-09-21-02 | T* | mitigate | E4, E5 | closed |
| T-09-21-03 | R* | mitigate | E4, E5 | closed |
| T-09-21-04 | R* | mitigate | E8, E9 | closed |
| T-09-22-01 | D* | mitigate | E4 | closed |
| T-09-22-02 | R* | mitigate | E4, E6 | closed |
| T-09-22-03 | T* | mitigate | E7 | closed |
| T-09-22-04 | D* | mitigate | E4, E7 | closed |
| T-09-23-01 | S* | mitigate | E8 | closed |
| T-09-23-02 | S* | mitigate | E8 | closed |
| T-09-23-03 | R* | mitigate | E6, E8 | closed |
| T-09-23-04 | I* | mitigate | E7, E8 | closed |
| T-09-24-01 | S* | mitigate | E8, E9 | closed |
| T-09-24-02 | D* | mitigate | E4, E8 | closed |
| T-09-24-03 | E* | mitigate | E7, E8 | closed |
| T-09-24-04 | T* | mitigate | E8, E9 | closed |
| T-09-25-01 | T* | mitigate | E5 | closed |
| T-09-25-02 | S* | mitigate | E5 | closed |
| T-09-25-03 | R* | mitigate | E5 | closed |
| T-09-26-01 | R* | mitigate | E4, E6 | closed |
| T-09-26-02 | D* | mitigate | E4 | closed |
| T-09-26-03 | S* | mitigate | E4, E5 | closed |
| T-09-26-04 | R* | mitigate | E6 | closed |
| T-09-27-01 | R* | mitigate | E5, E6 | closed |
| T-09-27-02 | R* | mitigate | E6 | closed |
| T-09-27-03 | E* | mitigate | E8 | closed |
| T-09-27-04 | S* | mitigate | E8 | closed |
| T-09-27-05 | E* | mitigate | E7, E8 | closed |
| T-09-28-01 | S* | mitigate | E8 | closed |
| T-09-28-02 | S* | mitigate | E8 | closed |
| T-09-28-03 | E* | mitigate | E8 | closed |
| T-09-28-04 | R* | mitigate | E6, E8 | closed |
| T-09-29-01 | S* | mitigate | E8, E9 | closed |
| T-09-29-02 | R* | mitigate | E6 | closed |
| T-09-29-03 | D* | mitigate | E4 | closed |
| T-09-29-04 | T* | mitigate | E7, E9 | closed |
| T-09-30-01 | E* | mitigate | E6, E8 | closed |
| T-09-30-02 | T* | mitigate | E6 | closed |
| T-09-30-03 | D* | mitigate | E4 | closed |
| T-09-30-04 | E* | mitigate | E7, E8 | closed |
| T-09-31-01 | S* | mitigate | E8 | closed |
| T-09-31-02 | S* | mitigate | E8 | closed |
| T-09-31-03 | R* | mitigate | E6, E9 | closed |
| T-09-31-04 | I* | mitigate | E7, E8 | closed |
| T-09-31-05 | T* | mitigate | E8, E9 | closed |

## Verified Isolation Entries

Four register entries are closed by verified non-applicability rather than by introducing a new control:

- `T-09-18-02`: Plan 09-18 did not decode JSONL or invoke a process oracle.
- `T-09-18-03`: Plan 09-18 produced no compatibility artifact or workflow claim.
- `T-09-19-03`: Plan 09-19 made no decoder changes.
- `T-09-19-04`: Plan 09-19 runtime tests could not promote compatibility evidence.

## Accepted Risks Log

No accepted risks.

## Transferred Risks

No transferred risks.

## Unregistered Threat Flags

No Phase 09 summary contains a separate `Threat Flags` entry, and the audit identified no unregistered planned threat.

## Verification

- 25 focused native particle tests passed.
- 25 protocol and comparator tests passed.
- Phase 09 corpus passed 26 tests; one explicit fixture-regeneration test remained intentionally ignored.
- 41 evidence and inventory CLI adversarial tests passed.
- Exact-ref validation passed with 7 cases and 58 semantic bindings.
- Provenance validation passed.
- Inventory validation passed for 177 compatibility rows.
- Four Phase 09 rows remain evidenced across implementation, unit, differential, and platform dimensions.
- Five Phase 10 rows remain `not_evidenced` across all four dimensions.
- The audit left implementation and tracked state unchanged.

## Security Audit Trail

| Audit date | Threats total | Closed | Open | Accepted | Transferred | Run by |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 2026-07-18 | 91 | 91 | 0 | 0 | 0 | `gsd-security-auditor` and Codex orchestrator |

## Sign-Off

- [x] All threats have a disposition.
- [x] Every mitigation is linked to implementation, tests, workflow, or verified isolation evidence.
- [x] Accepted and transferred risks are empty.
- [x] `threats_open: 0` is confirmed.
- [x] `status: verified` is set in frontmatter.

**Approval:** verified 2026-07-18
