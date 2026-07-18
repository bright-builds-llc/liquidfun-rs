---
phase: 09-particle-storage-lifecycle-and-coupling
verified: 2026-07-18T22:02:34Z
status: passed
score: "7/7 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T22:02:34Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: "gaps_found"
  previous_score: "5/7 must-haves verified"
  gaps_closed:
    - G09-PROOF-TOPOLOGY
    - G09-EXACT-REF-AUTHORITY
  gaps_remaining: []
  regressions: []
gaps: []
---

# Phase 9: Particle Storage, Lifecycle, and Coupling Verification Report

**Phase Goal:** Implement safe, identity-preserving particle systems and their lifecycle, contact, buffer, query, callback, and rigid-coupling foundations.

**Verified:** 2026-07-18T22:02:34Z

**Status:** passed

**Re-verification:** Yes — after Plans 30–31 closed proof topology and established fresh exact-ref authority.

## Goal Achievement

| # | Observable truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Consumers can create, configure, pause, inspect, and destroy multiple systems and particles with stable identities, flags, colors, lifetimes, and safe user data. | ✓ VERIFIED | Public particle/world APIs and focused object, definition, lifecycle, lifetime, eviction, and stable-handle tests pass. |
| 2 | Sorting, rotation, and compaction atomically update every required and optional SoA lane, ID map, and derived structure while borrow-scoped views remain safe. | ✓ VERIFIED | The validated permutation authority remaps all inventoried state; focused, property, view, and editor tests pass. |
| 3 | Safe external-buffer equivalents enforce ownership, capacity, growth, and teardown explicitly. | ✓ VERIFIED | Owned lane bundles and fixed/growable modes validate lengths and capacities and fail fixed-capacity growth explicitly. |
| 4 | Proxies, neighborhoods, contacts, strict behavior, lifetimes, zombies, callbacks, and deferred compaction match the pinned oracle. | ✓ VERIFIED | The seven-case native/C++ corpus and fresh canonical/sanitizer evidence cover the exact semantic bindings. |
| 5 | Forces, impulses, collision energy, stuck candidates, statistics, queries, and listener/filter flags are exposed and differentially verified through safe APIs. | ✓ VERIFIED | Public tests and the complete Phase 9 comparator cover the safe APIs, positive energy, nonempty stuck witnesses, and query/control behavior. |
| 6 | Cross-run proof records enforce independent canonical proof-path topology. | ✓ VERIFIED | Schema v4 rejects baseline substitution, noncanonical paths, and forbidden required-pair aliases before payload collection or deduplication. |
| 7 | The corrected schema and code have fresh exact-ref canonical/sanitizer authority for platform promotion. | ✓ VERIFIED | Sole run `29661682074` at sealed SHA `9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce` supplied the independently validated artifact pair. |

**Score:** 7/7 merged must-haves verified

**Roadmap score:** 5/5 success criteria verified

**Requirements score:** 14/14 Phase 9 requirements satisfied

## Final Gap Disposition

### G09-PROOF-TOPOLOGY — CLOSED

- Exact case-local schema-v4 paths are required for replay-native, replay-oracle, debug, release, minimized, and copied roles.
- Baseline substitution, traversal, alternative spellings, wrong case IDs, and forbidden aliases fail before file-set deduplication.
- Only the reviewed replay-to-D0 and minimized/copied-to-first-divergence reuse relationships remain allowed.
- `cargo test -p xtask --test phase9_evidence_cli` passed 20/20 tests, including recomputed-digest topology attacks.

### G09-EXACT-REF-AUTHORITY — CLOSED

- Sealed authority SHA: `9f2169ad1ad3c72adeae5e4fb1ea188b20ba84ce`.
- Exact-SHA dispatch count: one.
- Sole successful workflow dispatch: `29661682074`.
- Canonical job/artifact: `88125511292` / `8434547024`.
- Sanitizer job/artifact: `88125511305` / `8434557009`.
- Both artifacts were queried and downloaded independently, pre-inspected for safe bounded archive paths, hash-recomputed, extracted separately, and exact-ref validated.
- Denied runs `29439515367`, `29583793056`, `29625083184`, and `29652578231`, and artifacts `8423580554`, `8431920189`, and `8431922578`, remain rejected.

## Exact-Ref Evidence

| Evidence | Verified value |
| --- | --- |
| Manifest schema | 4 |
| Case-record schema | 3 |
| Cases | 7 |
| Unique semantic bindings | 58 |
| Policies per case | 22 |
| Retained comparison | `phase8-v1` `Match` before particle comparison |
| Canonical manifest SHA-256 | `74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c` |
| Sanitizer manifest SHA-256 | `74998e953e79f5ed04a58097d43abbca3cc814bee4fc86d0fd552d2951b1ae7c` |
| Semantic manifest SHA-256 | `a319f771c5d9e952b9389160bb3ad19ce487da43271e62568828ce2ae22a33aa` |
| Canonical archive SHA-256 | `22a37f91965eaf494b3e1fea041e1c54da9be03c06da5e276a641ee6cf536084` |
| Sanitizer archive SHA-256 | `849b8dba5b4c5a0f5e6ea4cddf10bf8243a71bdeec3b75676677358aa34d4316` |

The exact-ref validator recomputed request, native result, oracle result, complete comparison, replay, minimized/copied, trace, manifest, identity, semantic-manifest, and archive hashes. Canonical and sanitizer semantic manifests are byte-identical.

## Behavioral Verification

| Check | Result |
| --- | --- |
| `cargo test -p xtask --test inventory_cli` | 21 passed |
| `cargo test -p xtask --test phase9_evidence_cli` | 20 passed |
| `cargo test -p liquidfun-differential --test phase9_corpus` | 26 passed; 1 explicit regeneration test ignored |
| `cargo test -p liquidfun-differential --test particle_oracle` | 13 passed |
| `cargo test -p liquidfun-differential --test particle_protocol` | 25 passed |
| Paired local canonical/sanitizer evidence | 7 cases and 58 bindings verified |
| Exact-ref evidence | 7 cases and 58 bindings verified |
| Inventory generate/check twice | 177 rows; byte-identical output |
| Provenance | Pinned upstream `7f20402173fd143a3988c921bc384459c6a858f2` verified |
| Dependency policy | Advisories, bans, licenses, and sources passed |
| Schema drift | `blocking=false` |
| Workflow lint and Markdown | Passed |
| Full ordered Rust gate | Format, Clippy, build, and tests passed |

## Compatibility and Deferred Scope

| Scope | Rows | Final state | Status |
| --- | --- | --- | --- |
| Phase 9 platform authority | `b2Particle.h`, `b2ParticleSystem.h`, particle contacts/coupling, particle storage/lifecycle | Exactly four `platform_validated` rows cite the fresh 15-item authority set | ✓ VERIFIED |
| Phase 10 | particle assembly, particle group API, full particle source area, groups/pairs/triads, solver behaviors | Exactly five rows remain `not_evidenced` with empty references for implementation, unit, differential, and platform dimensions | ✓ PRESERVED |

No production physics, public API, CMake input, oracle workflow, global policy, or pinned upstream source changed during the authority recovery and promotion.

## Requirements Coverage

| Requirement | Status |
| --- | --- |
| API-09 | ✓ SATISFIED |
| API-10 | ✓ SATISFIED |
| PART-01 | ✓ SATISFIED |
| PART-02 | ✓ SATISFIED |
| PART-03 | ✓ SATISFIED |
| PART-04 | ✓ SATISFIED |
| PART-05 | ✓ SATISFIED |
| PART-06 | ✓ SATISFIED |
| PART-07 | ✓ SATISFIED |
| PART-08 | ✓ SATISFIED |
| PART-14 | ✓ SATISFIED |
| PART-15 | ✓ SATISFIED |
| PART-16 | ✓ SATISFIED |
| PART-17 | ✓ SATISFIED |

## Security and Disconfirmation

ASVS L1 review found no high-severity issue.

- GitHub workflow permissions remain `contents: read`; no secret enters the evidence protocol.
- Run, job, SHA, artifact, digest, and expiry identities are closed and cross-checked against live metadata.
- JSON and log inputs are bounded; strict schemas reject unknown fields.
- Target and archive paths reject absolute paths, traversal, symlinks, and unexpected file sets.
- Shell arguments remain fixed or quoted, and `actionlint` passes.
- Required proof roles are validated before deduplication, and adversarial alias/substitution tests pass.
- Generated compatibility claims are deterministic and constrained to the reviewed four-row scope.

Residual nonblocking risk: GitHub metadata and archive acquisition are externally sourced snapshots. The audit independently re-queried the live run and artifacts and recomputed local archive hashes; the exact-ref validator deliberately validates the recorded snapshot rather than performing network access itself.

## Human Verification Required

None.

## Verdict

Phase 9 is complete. Both final gaps are closed, all roadmap and requirement truths pass, exactly four Phase 9 platform rows have current authority, and Phase 10 scope remains untouched.

_Verified: 2026-07-18T22:02:34Z_

_Verifier: gsd-verifier_
