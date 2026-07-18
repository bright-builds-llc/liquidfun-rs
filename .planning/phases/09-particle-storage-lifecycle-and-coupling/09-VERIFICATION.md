---
phase: 09-particle-storage-lifecycle-and-coupling
verified: 2026-07-18T20:09:14Z
status: gaps_found
score: "5/7 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-18T20:09:14Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: "3/5 roadmap success criteria verified; 10/14 requirements satisfied"
  gaps_closed:
    - G09-DIFFERENTIAL-COMPARISON
    - G09-EXECUTABLE-COVERAGE
  gaps_remaining:
    - G09-PROOF-TOPOLOGY
    - G09-EXACT-REF-AUTHORITY
  regressions: []
gaps:
  - id: G09-PROOF-TOPOLOGY
    truth: "Cross-run replay, D0, debug/release, minimization, and first-divergence evidence proves independently persisted runs through a fail-closed canonical proof-path topology."
    status: partial
    reason: "The current generator emits six appropriate proof files and the validator recomputes their digests and semantic predicates, but validation does not reject baseline-result substitution or aliases between replay-native/replay-oracle, debug/release, or minimized/copied paths. Deduplication of referenced paths and the exact-file set allows a digest-recomputed manifest to satisfy content predicates without proving independent persistence."
    artifacts:
      - path: crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs
        issue: "Validates referenced bytes and predicates but does not validate proof-path relationships or reject baseline paths."
      - path: tools/xtask/src/phase9_evidence.rs
        issue: "cross_run_payload_refs collects references and later file-set construction deduplicates aliases without enforcing a canonical topology."
      - path: tools/xtask/tests/phase9_evidence_cli.rs
        issue: "Has semantic-content mutation tests but no baseline-substitution or independent-pair alias regressions."
    missing:
      - "Require normalized proof payloads below cases/<case-id>/proofs/ and reject baseline request/result/comparison paths."
      - "Require replay-native/replay-oracle, debug/release, and minimized/copied to be distinct pairs while explicitly allowing only reviewed cross-proof-family reuse."
      - "Add digest- and identity-recomputed regressions for baseline substitution, each independent-pair alias, and a first-divergence-only semantic-path mutation."
  - id: G09-EXACT-REF-AUTHORITY
    truth: "The corrected schema and reviewed implementation have fresh exact-ref canonical and sanitizer authority before any Phase 9 platform promotion."
    status: failed
    reason: "No workflow_dispatch occurred after commits 155ffc9, fac6573, 13aa7fd, 1e621f4, and cb7397c. The latest dispatch remains run 29652578231 at pre-fix SHA 22b31c0e1be8896df622b1decd58ba2853a60b04; artifacts 8431920189 and 8431922578 are explicitly superseded and denylisted. The ledger correctly leaves exactly four scoped Phase 9 platform rows not_evidenced."
    artifacts:
      - path: TESTING.md
        issue: "Documents run 29652578231 and both artifacts as superseded forensic history and requires a fresh exact-ref run."
      - path: reference/compatibility.json
        issue: "The four scoped Phase 9 platform_validated records correctly contain not_evidenced with empty references."
      - path: tools/xtask/src/inventory/validation.rs
        issue: "Rejects the superseded WR-01 authority set if used for promotion."
    missing:
      - "After G09-PROOF-TOPOLOGY is fixed and reviewed, push one immutable full SHA and dispatch Oracle CI with evidence_phase=phase9."
      - "Require one successful canonical job and one successful sanitizer job for that exact SHA; independently redownload and exact-ref validate both artifacts."
      - "Promote only the four scoped Phase 9 rows after validation; keep all five Phase 10 rows not_evidenced."
---

# Phase 9: Particle Storage, Lifecycle, and Coupling Verification Report

**Phase Goal:** Implement safe, identity-preserving particle systems and their lifecycle, contact, buffer, query, callback, and rigid-coupling foundations.

**Verified:** 2026-07-18T20:09:14Z

**Status:** gaps_found

**Re-verification:** Yes — after Plans 25–29 and the three-iteration code-review/fix loop.

## Goal Achievement

### Merged Observable Truths

The five roadmap success criteria remain the non-negotiable contract. Two additional evidence truths are retained from Plans 26–29 because those plans made executable proof integrity and fresh exact-ref authority part of Phase 9 closure.

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Consumers can create, configure, pause, inspect, and destroy multiple systems and particles with stable identities, flags, colors, lifetimes, and safe user data. | ✓ VERIFIED | Public particle/world APIs are live and covered by object, definition, lifecycle, lifetime, creation/eviction, and stable-handle integration tests. |
| 2 | Sorting, rotation, and compaction atomically update all required/optional SoA lanes, ID maps, derived contacts/proxies/constraints/lifetimes/groups, while borrow-scoped views remain safe. | ✓ VERIFIED | The single validated permutation authority remaps all inventoried state and recomputes weights; focused/property/view/editor tests cover rollback, coherence, and borrow-scoped access. |
| 3 | Safe external-buffer equivalents enforce ownership, capacity, growth, and teardown explicitly. | ✓ VERIFIED | Owned lane bundles and fixed/growable modes validate lengths and capacities, return owned lanes, and fail fixed-capacity growth explicitly. |
| 4 | Proxies, neighborhoods, particle/body contacts, strict behavior, lifetimes, zombies, callbacks, and deferred compaction match the pinned oracle. | ✓ VERIFIED | The corrected seven-case native/C++ corpus uses exact action/checkpoint/observation bindings; finite/infinite/equal lifetime, strict-contact, listener/filter, zombie, eviction, and coupling semantic assertions pass in fresh local canonical and sanitizer generation. |
| 5 | Forces, impulses, collision energy, stuck candidates, statistics, AABB/ray queries, and listener/filter flags are exposed and differentially verified through safe APIs. | ✓ VERIFIED | Public tests cover safe APIs; the corrected corpus rejects zero collision energy and empty stuck candidates and semantically compares force/statistics/query/listener/filter outputs. Retained Phase 6–8 comparison runs before particle comparison. |
| 6 | Cross-run proof records enforce an independent, canonical proof-path topology rather than merely valid contents. | ✗ FAILED | The generator currently emits distinct paths, but the shared evaluator/validator accepts baseline substitution and aliases because path relationships are not checked. |
| 7 | The corrected schema and code have fresh exact-ref canonical/sanitizer authority for platform promotion. | ✗ FAILED | The latest dispatch is still stale run 29652578231 at pre-fix SHA 22b31c0…; exactly four scoped Phase 9 platform rows correctly remain not_evidenced. |

**Score:** 5/7 merged must-haves verified

**Roadmap score:** 5/5 success criteria verified at the implementation and local differential level. Phase closure still fails because the plan-added evidence-integrity and exact-ref authority contracts are unresolved.

### Plan Must-Have Regression Matrix

| Plan range | Contract group | Status | Current evidence |
| --- | --- | --- | --- |
| 01–11, 18–20 | Definitions, identity, storage, buffers, views, lifecycle, contacts, coupling, statistics, queries, and protocol validation | ✓ VERIFIED | All declared artifacts exist; focused public/native suites and current corpus pass. |
| 12–15, 20–25 | Closed protocol, pinned oracle, retained rigid comparison, stable first divergence, and Phase 9/10 boundary | ✓ VERIFIED | `compare_complete_phase9_rigid_world_results` composes retained Phase 8 first; retained body, fixture, numeric, ordering, and subprocess mutations reject. |
| 26–27 | Exact semantic witness bindings, 7 cases, 58 bindings, 22 policies, digests, local canonical/sanitizer agreement | ✓ VERIFIED | Fresh schema-v3 local generation and paired validation reproduced all counts and reviewed digests. |
| 26–27 plus review iteration 3 | Independent cross-run proof topology | ✗ FAILED | Content semantics are typed and recomputed, but path independence is not part of the validation contract. |
| 28–29 | Fresh exact-ref canonical/sanitizer authority for the corrected implementation | ✗ FAILED | The only latest dispatch predates all review fixes and is denylisted. |
| 26, 29 | Preserve Phase 10 scope | ✓ VERIFIED | Five deferred Phase 10 rows remain not_evidenced across implementation, unit, differential, and platform dimensions. |

## Required Artifacts

`gsd-tools verify artifacts` passed all 71/71 artifacts declared across Plans 01–29.

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun/src/particle/` and `crates/liquidfun/src/world/particle_*.rs` | Native storage, lifecycle, contact, force/statistics, query, buffer, and coupling implementation | ✓ VERIFIED | Substantive, exported through live world APIs, and exercised by focused integration/property tests. |
| `crates/liquidfun-differential/src/rigid_world.rs` | Complete retained-rigid then particle comparator | ✓ VERIFIED | Calls retained Phase 8 comparison first and only evaluates Phase 9 particle policies after retained Match. |
| `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs` | Typed per-observation and cross-run semantic proof evaluator | ⚠ PARTIAL | Per-observation and cross-run contents are substantive and wired; cross-run path topology is not enforced. |
| `crates/liquidfun-differential/tests/phase9_corpus.rs` | Seven executable cases, 58 exact bindings, retained mismatch regressions, local evidence generation | ✓ VERIFIED | Fresh run: 26 passed, 0 failed, 1 explicit regeneration test ignored. |
| `tools/xtask/src/phase9_evidence.rs` | Shared fail-closed local/exact-ref validator | ⚠ PARTIAL | Validates schema, files, digests, results, comparator, bindings, proofs, identities, and canonical/sanitizer agreement; does not validate independent proof-path relationships. |
| `tools/xtask/tests/phase9_evidence_cli.rs` | Evidence corruption and substitution regression suite | ⚠ PARTIAL | 14/14 pass, but topology alias cases are absent. |
| `reference/compatibility.json` and generated `COMPATIBILITY.md` | Truthful evidence state | ✓ VERIFIED | Exactly four Phase 9 platform rows are not_evidenced with empty references; five Phase 10 rows remain fully not_evidenced. |
| `TESTING.md` | Current evidence authority and maturity boundary | ✓ VERIFIED | Explicitly demotes run 29652578231/artifacts 8431920189 and 8431922578 and requires a fresh exact-ref run. |

## Key Link Verification

Automated plan-link checks passed 53/56. The three regex misses were manually verified and are not gaps.

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `world/object.rs` | `particle/storage.rs` | `ParticleSystem.storage` ownership | ✓ WIRED | `ParticleSystem` directly owns one `ParticleStorage`. |
| `world/particle_object.rs` | `particle/storage.rs` | checked edit commit | ✓ WIRED | Public editing reaches `commit_kinematic_edit`, which validates and repairs derived state. |
| `world/step.rs` | `world/particle_coupling.rs` | source-timed contact/coupling prefix | ✓ WIRED | `World::step` calls `run_particle_contact_prefix` at the guarded particle stage. |
| `rigid_world.rs` | retained Phase 8 comparator | complete Phase 9 comparison | ✓ WIRED | `compare_phase8_rigid_world_results` runs before the particle comparator and short-circuits on retained mismatch. |
| corpus manifest | typed witness evaluator | exact action/checkpoint/ordinal/variant/predicate | ✓ WIRED | Seven stale bindings were corrected; digest-recomputed semantic mutation tests pass. |
| cross-run proof records | persisted proof payloads | path, digest, decode, recomputed predicate | ⚠ PARTIAL | Referenced content is proven, but path independence/topology is not. |
| current implementation SHA | GitHub exact-ref canonical/sanitizer authority | workflow dispatch and paired artifacts | ✗ NOT WIRED | No post-fix dispatch or current artifacts exist. |
| compatibility ledger | Phase 9/10 authority policy | inventory validation and deterministic report | ✓ WIRED | Stale Phase 9 authority is denied; Phase 10 promotion mutations are rejected. |

## Data-Flow Trace

| Artifact | Data source | Produces real data | Status |
| --- | --- | --- | --- |
| Public particle views, receipts, lifecycle reports, statistics, and queries | Live world-owned `ParticleStorage`, step stages, and query kernels | Yes | ✓ FLOWING |
| Phase 9 differential observations | Same validated request executed by native Rust and pinned C++ roles | Yes | ✓ FLOWING |
| Complete comparison | Retained Phase 8 walker followed by closed 22-path particle walker | Yes | ✓ FLOWING |
| Per-observation witness evidence | Exact action/checkpoint/observation ordinal and typed semantic predicate in both results | Yes | ✓ FLOWING |
| Cross-run proof evidence | Persisted replay/D0/debug/release/minimized/copied result payloads | Yes for current generator output | ⚠ VALID CONTENT, UNENFORCED TOPOLOGY |
| Platform authority | A post-fix exact-ref GitHub dispatch | No | ✗ DISCONNECTED |

## Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Targeted Phase 9 corpus | `cargo test -p liquidfun-differential --test phase9_corpus` | 26 passed, 0 failed, 1 ignored | ✓ PASS |
| Evidence CLI contract | `cargo test -p xtask --test phase9_evidence_cli` | 14 passed | ✓ PASS |
| Inventory authority contract | `cargo test -p xtask --test inventory_cli` | 21 passed | ✓ PASS |
| Fresh local canonical evidence | `bash scripts/phase9-evidence.sh canonical target/phase9-verification/canonical` | 7 cases/58 bindings generated and content-validated | ✓ PASS |
| Fresh local sanitizer evidence | fail-fast ASan/UBSan environment plus `phase9-evidence.sh sanitizer` | 7 cases/58 bindings generated and content-validated | ✓ PASS |
| Paired local evidence | `cargo xtask phase9-evidence validate --mode local ... --deny-run-id 29652578231` | `Phase 9 evidence verified: 7 cases, 58 semantic bindings` | ✓ PASS |
| Fresh manifest identity | recomputed SHA-256 and manifest assertions | Both manifests byte-identical: `fa5e2ad1d25f074416eb86df1fd3c71404064a2f608f5f0cea0340e7612896b6`; semantic digest `224fcfcb52ca842303510433387ca16541995c3515d7b0ca398dc334b4354be4` | ✓ PASS |
| Inventory/provenance | `cargo xtask inventory check && cargo xtask provenance check` | 177 rows and pinned oracle `7f204021…` verified | ✓ PASS |
| Schema drift | `gsd-tools verify schema-drift 09` | `drift_detected=false`, `blocking=false` | ✓ PASS |
| Current remote dispatch | `gh run list ... --workflow oracle.yml` | Latest dispatch is stale run 29652578231 at `22b31c0…`; no later dispatch | ✗ FAIL |
| Existing checked-out old evidence | current local validator against old `target/phase9-evidence-local` | Correctly rejected schema-v2 manifest: missing `cross_run_proofs` | ℹ EXPECTED STALE OUTPUT |

The temporary schema-v3 verification output was removed after validation. No repository-owned source or evidence ledger file was changed by these checks.

## Requirements Coverage

All 14 Phase 9 requirement behaviors are implemented and have current native plus local differential evidence. The two report gaps block evidence closure and platform authority, not the underlying requirement implementations.

| Requirement | Status | Evidence |
| --- | --- | --- |
| API-09 | ✓ SATISFIED | Borrow-scoped aggregate views and validated closure-scoped edits preserve aliasing and derived-state invariants. |
| API-10 | ✓ SATISFIED | Owned fixed/growable buffer equivalents enforce ownership, capacity, growth, failure, and teardown. |
| PART-01 | ✓ SATISFIED | Multiple configurable, pausable, newest-first systems have stable identity and complete teardown. |
| PART-02 | ✓ SATISFIED | Creation/destruction supports position, velocity, color, flags, lifetime, associations, receipts, and stable IDs. |
| PART-03 | ✓ SATISFIED | Public particle identity survives sort, rotation, and compaction. |
| PART-04 | ✓ SATISFIED | One validated permutation remaps every inventoried lane/map/derived reference atomically. |
| PART-05 | ✓ SATISFIED | Safe bulk views expose all Phase 9 semantic lanes; mutation cannot escape or leave stale state. |
| PART-06 | ✓ SATISFIED | Fixed capacity fails explicitly and growable owned storage expands under checked rules. |
| PART-07 | ✓ SATISFIED | Proxies, neighborhoods, particle/body contacts, strict pruning, and retained rigid effects have exact semantic corpus bindings. |
| PART-08 | ✓ SATISFIED | Finite/infinite/equal lifetime, quantization, eviction, zombie marking, and compaction are natively tested and semantically compared. |
| PART-14 | ✓ SATISFIED | Storage-authoritative zombie state and exactly-once ordered destruction occurrences are tested and compared. |
| PART-15 | ✓ SATISFIED | Listener/filter flag witnesses observe enabled-versus-disabled semantic outcomes rather than declaration bits. |
| PART-16 | ✓ SATISFIED | Force/impulse/statistics APIs are live; evidence requires positive collision energy and nonempty stuck candidates. |
| PART-17 | ✓ SATISFIED | AABB/ray behavior covers clipping, ignore/continue/terminate, start-inside, early termination, and culling. |

**Requirements score:** 14/14 behavior requirements satisfied.

No Phase 9 requirement is orphaned. Every ID mapped to Phase 9 in `REQUIREMENTS.md` appears in at least one plan.

## Compatibility and Deferred Scope

| Scope | Rows | Current state | Verification |
| --- | --- | --- | --- |
| Phase 9 platform authority | `b2Particle.h`, `b2ParticleSystem.h`, particle contacts/coupling, particle storage/lifecycle | Exactly 4 rows: `not_evidenced`, empty platform references | ✓ Honest |
| Phase 10 | particle assembly, particle group API, full particle source area, groups/pairs/triads, solver behaviors | Exactly 5 rows: implementation/unit/differential/platform all `not_evidenced` | ✓ Preserved |

Cross-engine stable-ID rotation, groups, topology, pairs, triads, source areas, and solver behavior remain Phase 10 work. They are not Phase 9 gaps and must not be promoted by the Phase 9 closure run.

## Anti-Patterns and Disconfirmation Pass

| File | Pattern | Severity | Impact |
| --- | --- | --- | --- |
| `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs` | “Independent” proof variants validate equality/content but not path distinctness | ⚠ WARNING | A structurally aliased manifest can impersonate independently persisted proof. |
| `tools/xtask/src/phase9_evidence.rs` | Cross-run reference/file-set handling permits deduplicated aliases | ⚠ WARNING | Exact-file validation does not close the topology gap. |
| `tools/xtask/tests/phase9_evidence_cli.rs` | No alias/baseline-substitution regression | ⚠ WARNING | The unresolved validation branch is not protected. |

Disconfirmation results:

- A partially met requirement was found in the plan-added evidence contract: contents are proven, independence is not.
- A passing test can mislead here: cross-run mutation tests alter shared proof content, so they do not prove path topology.
- The uncovered error path is a digest-recomputed manifest that aliases an independent pair or points a proof to a baseline result.

No new TODO/FIXME/placeholder, empty user-visible implementation, or obsolete commented constructor block was found in the reviewed Phase 9 scope. The symlink cleanup critical finding remains closed by focused regression.

## Human Verification Required

None. Both remaining gaps are deterministic code/evidence-authority gaps and can be closed programmatically. Dispatch and promotion require the existing reviewed workflow, but no subjective product judgment is needed.

## Gaps Summary

Plans 25–27 and review commits `fac6573` and `1e621f4` close the prior verifier’s two remaining implementation/evidence-content gaps:

- retained Phase 6–8 body, fixture, and numeric observations are compared before particle observations;
- all 58 Phase 9 branches bind to exact semantic observations or typed cross-run proof records;
- seven stale bindings were corrected;
- fresh local schema-v3 canonical and sanitizer evidence agrees exactly and validates all 7 cases/58 bindings;
- symlink-safe cleanup, stale-constructor removal, and honest Phase 9/10 ledger demotion are present in commits `155ffc9`, `13aa7fd`, and `cb7397c`.

Phase 9 is not ready to close because the review loop exhausted its third iteration with one unresolved evidence-integrity warning, and the corrected schema has no fresh exact-ref authority. Closure order is mandatory:

1. enforce and test canonical cross-run proof-path topology;
1. rerun the full Rust, corpus, local canonical/sanitizer, inventory, provenance, schema, dependency, workflow, Markdown, read-only, and security gates;
1. push one reviewed immutable SHA and dispatch exactly one fresh Phase 9 exact-ref run;
1. independently validate both artifacts and promote only the four Phase 9 platform rows, leaving Phase 10 untouched.

_Verified: 2026-07-18T20:09:14Z_

_Verifier: the agent (gsd-verifier)_
