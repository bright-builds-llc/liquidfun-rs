---
phase: 09-particle-storage-lifecycle-and-coupling
verified: 2026-07-17T14:38:33Z
status: gaps_found
score: "3/5 roadmap success criteria verified; 10/14 requirements satisfied"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 09-2026-07-15T02-54-51
generated_at: 2026-07-17T14:38:33Z
lifecycle_validated: true
overrides_applied: 0
re_verification:
  previous_status: gaps_found
  previous_score: "4/14 requirements verified"
  gaps_closed:
    - G09-EVIDENCE-PIPELINE
    - G09-STEP-GUARD
    - G09-ZOMBIE-AUTHORITY
    - G09-EVICTION-OCCURRENCE
    - G09-PERMUTATION-WEIGHTS
    - G09-PROTOCOL-VALIDATION
  gaps_remaining:
    - G09-DIFFERENTIAL-COMPARISON
    - G09-EXECUTABLE-COVERAGE
  regressions: []
gaps:
  - truth: "Phase 9 differential execution preserves retained Phase 6 through Phase 8 rigid semantic comparison while adding particle comparison."
    status: failed
    reason: "The runner calls only the Phase 9 comparator, which validates structure and then filters every checkpoint to particle observations. Body and fixture state is discarded, and the retained Phase 8 comparator is never composed, so a valid retained rigid divergence can return Match."
    artifacts:
      - path: "crates/liquidfun-differential/src/rigid_world.rs"
        issue: "run_phase9_differential invokes compare_phase9_rigid_world_results without inherited rigid comparison."
      - path: "crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs"
        issue: "particle_observations filters out every non-particle observation before comparison."
      - path: "crates/liquidfun-differential/tests/phase9_corpus.rs"
        issue: "Comparator mutation coverage changes particle observations only and does not prove body or fixture divergence detection."
    missing:
      - "Compose compare_phase8_rigid_world_results, or an equivalent inherited rigid walker, into the Phase 9 comparison boundary."
      - "Add a regression that mutates retained body or fixture output and proves Phase 9 reports the first rigid divergence."
  - truth: "Every claimed Phase 9 lifecycle, contact, statistics, replay, and evidence branch is bound to branch-specific semantic native-versus-oracle output."
    status: partial
    reason: "Several of the 58 claimed witnesses assert declarations or configuration bits, accept zero or empty outputs, or reduce replay and divergence claims to request/scenario ID equality. The evidence script checks counts, uniqueness, and digests but not semantic binding between each branch and its declared action/checkpoint/output."
    artifacts:
      - path: "crates/liquidfun-differential/tests/phase9_corpus.rs"
        issue: "Finite/infinite/equal lifetime, strict-contact, listener/filter, collision-energy, stuck-candidate, and evidence-contract branches do not all exercise their named semantic behavior."
      - path: "crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json"
        issue: "Multiple witnesses point to generic inspect-particle or statistics observations that are unrelated to the semantic assertion."
      - path: "scripts/phase9-evidence.sh"
        issue: "Manifest validation proves branch count and uniqueness, not exact witness-to-output binding."
    missing:
      - "Bind each branch to the exact action, checkpoint, observation kind, and semantic output it claims."
      - "Exercise observable finite/infinite/equal lifetime transitions and strict/listener/filter enabled-versus-disabled behavior."
      - "Use nonzero collision-energy and nonempty stuck-candidate scenarios, or document and test an intentional alternative oracle branch."
      - "Prove replay, minimization, first-divergence, D0, and debug/release claims through result digests or deliberate mismatch behavior rather than identity fields alone."
---

# Phase 9: Particle Storage, Lifecycle, and Coupling Verification Report

**Phase Goal:** Implement safe, identity-preserving particle systems and their lifecycle, contact, buffer, query, callback, and rigid-coupling foundations.

**Verified:** 2026-07-17T14:38:33Z

**Status:** gaps_found

**Re-verification:** Yes — after the original eight-gap report and Plans 18–24.

## Goal Achievement

### Observable Truths

| # | Roadmap success criterion | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Consumers can create, configure, pause, inspect, and destroy multiple systems and particles with stable identities, flags, colors, lifetimes, and safe user data. | ✓ VERIFIED | Public APIs are exported from `lib.rs` and `particle.rs`; `World` owns multiple newest-first systems; creation receipts, snapshots, lifecycle tests, and stable-handle tests exercise the behavior. |
| 2 | Sorting, rotation, and compaction atomically update SoA lanes, identity maps, derived contact/proxy/constraint/lifetime/group state, while borrow-scoped views remain safe. | ✓ VERIFIED | `storage/permutation.rs` builds a complete candidate, remaps every derived lane, recomputes contact weights, and commits only after validation. `ParticleSystemView` and closure-scoped `ParticleEditor` prevent escaped borrows; property and public regression suites cover coherence. |
| 3 | Safe external-buffer equivalents enforce ownership, capacity, growth, and teardown explicitly. | ✓ VERIFIED | `OwnedLaneBundle` and `ParticleBufferMode` retain Rust ownership, validate lane lengths/capacities, distinguish growable and fixed modes, and return explicit capacity errors. |
| 4 | Proxies, neighborhoods, particle/body contacts, strict behavior, lifetimes, zombies, callbacks, and deferred compaction match the pinned oracle. | ✗ FAILED | Native behavior is substantive and well unit-tested, but the promoted differential corpus does not semantically exercise several named lifetime, strict-contact, and listener/filter branches. |
| 5 | Forces, impulses, collision energy, stuck candidates, statistics, AABB/ray queries, and listener/filter flags are exposed and differentially verified through safe APIs. | ✗ FAILED | Safe APIs and native tests exist, and force/query branches have semantic observations. The collision-energy and stuck witnesses accept zero/empty output, listener/filter witnesses inspect declaration bits, and the comparator can miss retained rigid divergence. |

**Score:** 3/5 roadmap success criteria verified

The goal is not yet achieved because “match” and “differentially verified” are explicit parts of criteria 4 and 5. Passing native tests and successful CI execution cannot substitute for a comparator that observes the required state or for branch witnesses that exercise the behavior they name.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/liquidfun/src/particle/definition.rs`, `storage.rs`, `storage/`, `buffer.rs` | Definitions, stable identity, complete SoA storage, owned/fixed buffer behavior | ✓ VERIFIED | Exists, substantive, exported or owned by live world objects, and covered by focused/property tests. |
| `crates/liquidfun/src/particle/view.rs`, `editor.rs` | Borrow-scoped bulk view and closure-scoped validated edits | ✓ VERIFIED | Lifetimes prevent view/editor escape; edits validate before mutation and synchronously repair spatial state. |
| `crates/liquidfun/src/particle/lifetime.rs`, `world/particle_lifecycle.rs` | Lifetime ordering, zombie authority, eviction, callbacks, compaction | ✓ VERIFIED | Fresh positive-step guard, ZOMBIE synchronization, synchronous eviction receipts, and ordered callback regressions exist. |
| `crates/liquidfun/src/particle/proxy.rs`, `contact.rs`, `body_contact.rs`, `force.rs`, `statistics.rs`, `query.rs` | Contact, coupling, force/statistics, and query foundations | ✓ VERIFIED | Real state is produced and consumed by world stepping, views, snapshots, and query results. |
| `crates/liquidfun/src/world/particle_object.rs`, `particle_coupling.rs`, `step.rs` | Public world integration and source-timed rigid coupling | ✓ VERIFIED | `World::step` runs lifecycle/contact/coupling only on fresh positive steps and commits candidate systems/bodies transactionally. |
| `crates/liquidfun-differential/src/scenario/rigid_world/result/phase9.rs` | Closed Phase 9 request/result validation | ✓ VERIFIED | Action-specific result schemas, lifecycle occurrence identities, and finite-value validation are substantive and wired into both executors. |
| `tools/reference/src/phase9_particle.cc` and native executor | Independent native/C++ semantic execution | ✓ VERIFIED | Both roles consume the canonical request; the accepted canonical and sanitizer jobs executed seven cases successfully. |
| `crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs` | Complete Phase 9 plus retained Phase 6–8 comparison | ✗ INCOMPLETE | Exhaustive over nine particle observation variants and 22 particle policies, but it drops body/fixture observations and never composes the inherited rigid comparator. |
| `crates/liquidfun-differential/tests/phase9_corpus.rs`, Phase 9 manifest, `scripts/phase9-evidence.sh` | Branch-specific semantic evidence with fail-closed collection | ✗ INCOMPLETE | Pipeline failure propagation is fixed and evidence is digest-bound, but several branch assertions are declarative or trivial rather than semantic. |
| `reference/compatibility.json`, `COMPATIBILITY.md` | Honest promotion from accepted evidence | ⚠ PARTIAL | Four scoped rows reference the correct approved run/artifacts, but the differential-validation claim is stronger than the comparator and witness semantics support. |

`gsd-tools verify artifacts` passed all 56/56 declared plan artifacts. Three of 46 declarative key-link regex checks reported pattern misses; manual semantic tracing verified those three links: `ParticleSystem.storage` in `world/object.rs`, `edit_particle` to `commit_kinematic_edit`, and `World::step` to `run_particle_contact_prefix`. These regex false negatives are not gaps.

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| Public particle API | `World` particle methods | exported IDs, definitions, snapshots, receipts, views, and errors | ✓ WIRED | Consumer-facing calls resolve to live world-owned systems and stable particle identities. |
| `ParticleSystem` | `ParticleStorage` | sole storage owner | ✓ WIRED | `world/object.rs` stores one `ParticleStorage`; public access routes through validated world handles. |
| `ParticleEditor` / `ParticleSystemView` | storage lanes and derived state | scoped borrow plus validated commit/repair | ✓ WIRED | Immutable views borrow the world; edits copy, validate, then call `commit_kinematic_edit`. |
| `World::step` | lifecycle and contact/coupling stages | fresh positive-step guard | ✓ WIRED | `runs_particle_stages` requires `Fresh` and `time_step > 0.0`; lifecycle precedes contact prefix and rigid solve. |
| storage permutation | all lanes, stable IDs, contacts, weights, groups, lifetimes | candidate/remap/recompute/atomic commit | ✓ WIRED | Remapped contacts feed `recompute_contact_weights`; commit replaces every required/optional lane and derived record. |
| canonical request | native and pinned C++ executors | one serialized request and exact request digest | ✓ WIRED | `run_phase9_differential` decodes one canonical value and sends it to both roles. |
| native/C++ results | Phase 9 comparator | typed observation walker | ✗ PARTIAL | Particle observations are compared; retained body and fixture observations are filtered out. |
| branch manifest | semantic output | witness action/checkpoint/predicate | ✗ PARTIAL | Every label reaches an assertion, but some assertions inspect inputs/configuration or accept non-demonstrative empty output. |
| accepted evidence artifacts | compatibility ledger | exact run, commit, artifact names, and digests | ✓ WIRED | Run `29583793056` and approved SHA `b27fc14f6b29fb82ca815fa1effba71bae09d424` are cited consistently. |

### Data-Flow Trace (Level 4)

| Artifact | Data variable | Source | Produces real data | Status |
| --- | --- | --- | --- | --- |
| Particle view/snapshot/creation receipt | stable IDs, lanes, destruction occurrences | world-owned `ParticleStorage` and lifecycle outcomes | Yes | ✓ FLOWING |
| World particle step | positions, velocities, contacts, weights, body reactions | lifecycle, neighborhood/contact generation, and coupling against live fixtures/bodies | Yes | ✓ FLOWING |
| Statistics and queries | counts, energy, stuck IDs, AABB/ray hits | current storage/contact/body state | Yes | ✓ FLOWING |
| Phase 9 differential result | native and C++ checkpoint observations | same canonical request executed by independent engines | Yes | ✓ FLOWING |
| Phase 9 comparison outcome | mismatch or match | filtered particle observation slices | Partially | ✗ HOLLOW FOR RETAINED RIGID STATE |
| Branch-evidence manifest | reached branch names and digests | corpus witness assertions and evidence script | Partially | ⚠ STATIC/DECLARATIVE FOR SEVERAL BRANCHES |

### Behavioral Spot-Checks

Fresh independent gates were supplied by the orchestrator; redundant long workspace tests were not rerun.

| Behavior | Command or evidence | Result | Status |
| --- | --- | --- | --- |
| All-feature native behavior and public docs | `cargo test --all-features` | All tests passed, including 16 doctests | ✓ PASS |
| Phase 9 native/C++ corpus | canonical and sanitizer Phase 9 gates | 7 passed, 0 failed, 1 deliberately ignored in each authority lane | ✓ PASS |
| Fail-closed repository/evidence checks | provenance, inventory, cargo-deny, actionlint, Markdown, read-only, ASVS L1 | All passed; schema drift reported false/nonblocking | ✓ PASS |
| Declared artifacts | `gsd-tools verify artifacts` across Plans 01–24 | 56/56 passed | ✓ PASS |
| Retained rigid mismatch detection | Static call/data-flow trace | Phase 9 runner has no call to `compare_phase8_rigid_world_results`; comparator filters to particle observations | ✗ FAIL |
| Branch-specific semantic evidence | Static assertion/manifest trace | Multiple branches prove declarations, empty output, or identity only | ✗ FAIL |

### Requirements Coverage

| Requirement | Source plans | Status | Evidence |
| --- | --- | --- | --- |
| API-09 | 05, 19 | ✓ SATISFIED | Borrow-scoped views and closure-scoped edits prevent aliasing; edits repair derived state and permutations recompute weights. |
| API-10 | 04 | ✓ SATISFIED | Owned lane bundles and fixed/growable modes define ownership, capacity, growth, failure, and teardown. |
| PART-01 | 01, 03, 12 | ✓ SATISFIED | Multiple newest-first systems expose configuration, pause, inspection, destruction, capacity, and iteration controls. |
| PART-02 | 01, 02, 06, 19 | ✓ SATISFIED | Particle creation/destruction supports required fields, stable IDs, safe user associations, lifetimes, and synchronous eviction receipts. |
| PART-03 | 02 | ✓ SATISFIED | Dense rows can reorder without changing public particle identity. |
| PART-04 | 01, 02, 05, 19 | ✓ SATISFIED | Candidate permutations update required/optional lanes, identity, proxies, contacts, weights, pairs/triads, lifetimes, and group ranges atomically. |
| PART-05 | 05 | ✓ SATISFIED | Safe bulk read and checked mutation APIs cover the named particle properties without exposing mutable lane aliases. |
| PART-06 | 04 | ✓ SATISFIED | Fixed capacity fails explicitly; growable storage remains Rust-owned and expands under checked rules. |
| PART-07 | 08, 09, 14, 15, 22 | ✗ BLOCKED | Native proxies/neighborhood/contact behavior exists, but strict-contact enabled/disabled evidence checks configuration rather than a semantic contact difference. |
| PART-08 | 06, 14, 15, 18, 19, 22 | ✗ BLOCKED | Native lifetime/zombie/eviction regressions pass, but finite/infinite/equal lifetime differential branches inspect request declarations instead of lifecycle outcomes. |
| PART-14 | 06, 07, 18, 19 | ✓ SATISFIED | Storage-authoritative zombies and requested/unrequested destruction outcomes are ordered and emitted exactly once. |
| PART-15 | 08, 09, 14, 22 | ✗ BLOCKED | Listener/filter APIs and native gates exist, but promoted enabled/disabled witnesses inspect flag bits rather than callback/collision outcomes and ordering. |
| PART-16 | 10, 14, 22 | ✗ BLOCKED | Force, impulse, counts, and statistics APIs are substantive; collision-energy and stuck-candidate branches accept zero/empty values and therefore do not prove those behaviors. |
| PART-17 | 11, 14 | ✓ SATISFIED | System/world AABB and ray paths exercise ordering, clipping, ignore/continue/terminate, start-inside exclusion, and culling through semantic outputs. |

**Requirements score:** 10/14 satisfied

No Phase 9 requirement is orphaned: all 14 IDs mapped to the phase in `REQUIREMENTS.md` appear in at least one Phase 9 plan.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `crates/liquidfun/src/particle/storage.rs` | 416 | Commented-out obsolete constructor call | ℹ INFO | No runtime effect, but stale code should be removed during cleanup. |
| `crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs` | 237 | Filters observations before comparison | 🛑 BLOCKER | Allows retained rigid disagreement to escape the Phase 9 authority boundary. |
| `crates/liquidfun-differential/tests/phase9_corpus.rs` | 726–730, 782–799, 868–869, 916–923 | Input/configuration, zero/empty, or identity-only branch assertions | 🛑 BLOCKER | Inflates the claimed executable semantic branch coverage and blocks parity claims. |

The general TODO/FIXME/placeholder/empty-implementation scan found no additional user-visible stubs. Empty match arms found in query/storage code are deliberate exhaustive no-op branches, not hollow implementations.

### Human Verification Required

None. The remaining failures are deterministic code and evidence-boundary defects that can be verified programmatically.

### Deferred-Scope Check

Phase 10 explicitly owns particle groups, pairs/triads, group topology, solver families, and the cross-engine stable-ID rotation witness. Those items were not treated as Phase 9 gaps. Neither remaining gap is deferred: no later roadmap goal explicitly promises to restore retained Phase 6–8 comparison inside the Phase 9 runner or replace weak Phase 9 branch witnesses.

### Gaps Summary

Plans 18–24 materially closed six of the eight original gaps:

- particle stages now require a fresh positive step;
- storage flags and pending-destruction state agree on ZOMBIE authority;
- capacity eviction returns synchronous exactly-once occurrences;
- permutation weights are recomputed from remapped contacts;
- Phase 9 request/result protocol validation is action-specific and fail-closed;
- the evidence pipeline uses `pipefail`, passing-test markers, exact-ref approval, canonical/sanitizer artifacts, digests, and read-only gates.

The original differential-comparison and executable-coverage concerns improved but are not closed. A real native-versus-C++ particle comparator now exists, yet it omits retained rigid state. Seven cases execute under both engines, yet several of the 58 named branches do not semantically demonstrate the behavior they claim. Because the compatibility ledger promotes differential validation from this evidence, these are goal-blocking gaps rather than documentation warnings.

_Verified: 2026-07-17T14:38:33Z_

_Verifier: the agent (gsd-verifier)_
