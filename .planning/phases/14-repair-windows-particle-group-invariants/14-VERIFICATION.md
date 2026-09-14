---
phase: 14-repair-windows-particle-group-invariants
verified: 2026-09-14T07:23:00Z
status: passed
score: 13/13 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T07:23:00Z
lifecycle_validated: true
overrides_applied: 0
tested_source: 417d38dac6951226fb32ea99a97135defe88da7b
run_id: 34815192937
run_attempt: 1
requirements_satisfied: [PART-03, PART-04, PART-09, PART-10, TEST-02, TEST-04]
deferred:
  - truth: "Current-head canonical candidate acceptance, release aggregation, and frozen-source attestation"
    addressed_in: "Phase 15"
    evidence: "Phase 15 requires one frozen full-SHA candidate, complete retained producer evidence, accepted release aggregation, and worktree plus committed-range attestation."
---

# Phase 14: Repair Windows Particle-Group Invariants Verification Report

**Phase Goal:** Restore transactional particle-group creation and mutation on supported Windows without weakening stable identity, authoritative storage, or cross-platform semantics.

**Status:** passed. **Score:** 13/13 merged must-haves verified. **Re-verification:** No — initial formal report; provisional inspections were held until terminal quality success and all four summaries existed.

The source verified here is `417d38dac6951226fb32ea99a97135defe88da7b`. [Cargo CI run 34815192937, attempt 1](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937) completed successfully with all four jobs passing. Subsequent evidence, summary, review and state commits are metadata after this tested source; they do not rebind these results to a later HEAD. Evidence authority is D2 only.

## Goal Achievement

### Observable Truths

The four roadmap criteria retain their contract wording. Plan truths add diagnostic, error-category, state-comparison and evidence detail. Repeated successful-creation/appending truths are combined in rows 2 and 7; Plan 04's identical-SHA requirement is included in row 4.

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | The persisted minimized seed `4149329052036581951` reproduces before the fix and passes afterward on the supported Windows toolchain. | VERIFIED | `14-DIAGNOSIS.md` binds the original audited failure and the full operation-14 Windows diagnostic to distinct source/run identities. The final Windows job `103884302536` executes `persisted_audited_windows_seed` with exactly one pass on Rust 1.97.0 MSVC. The test requires a successful append and one additional particle, not an expected panic or accepted rejection. |
| 2 | Checked particle/group creation cannot invalidate authoritative storage or reach an internal impossible-state panic. | VERIFIED | `solver_state.rs:319` preflights logical capacity and reserves actual rows. `world/particle_object/group.rs:173` prepares an owned system candidate; publication occurs only after planning succeeds. Candidate-only InvalidLaneBundle handling returns existing typed topology rejection; the authoritative-corruption mapper remains separate. Both exact successful public replays and private negative controls execute. This is the checked candidate contract, not a promise to recover from arbitrary process-wide memory exhaustion. |
| 3 | Every failure path remains transactional: stable particle identities, group topology, optional lanes, derived structures, and rollback state remain coherent. | VERIFIED | Candidate failures return before world publication. Complete private storage/lifetime/arena equality covers inaccessible state; public populated snapshots compare actual IDs and semantics. Late new/append topology, capacity, stale and foreign-owner rejections preserve state; subsequent allocations and lifecycle are checked. This criterion concerns group creation/mutation boundaries and does not introduce universal whole-world step rollback. |
| 4 | Focused public integration/property regressions and the complete relevant particle suite pass on Windows and the ordinary Linux/macOS verification boundaries. | VERIFIED | One full SHA passes both exact tests (1+1), focused mutation/property/creation (10/6/12), and complete default package (957) on Windows, Linux and macOS. All platform steps succeed. The same run's Linux quality job passes all 26 steps, including full all-feature workspace tests and later isolation/docs gates. |
| 5 | Original audited and current Windows failures remain distinguishable by seed, controls, source SHA and run identity. | VERIFIED | Literal seeds `4149329052036581951` and `190752942043209832`, fourteen/fifteen controls and explicit operations remain in `particle_group_properties.rs`. Original run `30070539790`, later run `34777296141` and their logs are separately retained. Seventeen historical metadata/log hashes were independently recomputed. |
| 6 | The first failed candidate transition is demonstrated before production error mapping or storage repair. | VERIFIED | Diagnostic commit `f4ef739` precedes repairs and observes valid 9-row source and 10-row candidate, then an 8,589,934,588-byte scratch reservation failure inside append join. Raw AllocError becomes InvalidLaneBundle. No invalid authoritative predicate was observed. Probes were removed in `49e382f`; the later sizing repair is `e58e028`. |
| 7 | Diagnosis and tests distinguish valid public operations that must succeed from legitimate typed rejections. | VERIFIED | Both named Windows replays require Applied outcomes except intentional foreign join; successful append retains its target/order and adds one member. Populated rejection tests require exact errors and successful neighboring creation, append, join, split and flag operations. The scratch budget test rejects mapper-only suppression. |
| 8 | Legitimate isolated-candidate topology rejection returns InvalidParticleGroupTopology without publishing state. | VERIFIED | Private candidate storage/lifetime mappers are used only during group preparation. Late topology new/append cases assert the exact existing CreateObjectError and full unchanged state. Capacity/handle categories retain their prior meanings. |
| 9 | Private rollback covers complete storage, lifetime, shell/identity allocation and diagnostic state. | VERIFIED | `particle_group_transactions.rs::Before::assert_unchanged` compares every storage field, lifetime, system metadata/order, both arenas and next diagnostic allocation. Arena comparison includes occupied/vacant/retired slots, generations and free ordering. Follow-up tests check the exact next particle/group/diagnostic IDs and deferred listener output. |
| 10 | Both Windows minimized inputs replay deterministically without swallowing unexpected typed errors. | VERIFIED | Literal generator equality and repeated traces are asserted. Version 1, 128 cases and bounds 24/8/32 remain. The model compares rollback on every accepted rejection and fails unexpected errors with operation context. PendingDelete needs a nonzero pending-row witness; ParticleLifecycleInvariant remains unexpected. Three newly exposed minimized inputs are persisted. |
| 11 | Public create, append, join, split and flag rejection preserve actual identities and complete observable semantics. | VERIFIED | Nine registered populated focused rejection tests use actual-ID snapshots, nonempty contacts/pairs/triads/colors/lifetime data, stale/pending identities and exact deferred lifecycle. Cross-system cases compare both sources. `RollbackSnapshot` retains actual identities separately from normalized cross-world replay. |
| 12 | Failed or stale attempts remain separately retained and never satisfy platform closure. | VERIFIED | Final evidence preserves failed `ce37c56`/`fc3af47` and timed-out `430e0ad` with distinct source/attempt logs, metadata and hashes. The cancelled run's passing workspace/OS steps are not whole-run success. Only final `417d38d` terminal records satisfy closure. |
| 13 | Platform results are D2 only; Phase 15 release-candidate aggregation and canonical evidence remain unchanged. | VERIFIED | No net changes under canonical reference artifacts, native source or Phase 15 artifacts. Test history/material fixtures are synthetic and confined; production acceptance/hashing bodies remain strict. Final evidence explicitly reserves actual current-head acceptance, aggregation and attestation for Phase 15. |

### Required Artifacts

All ten PLAN artifact entries pass the GSD existence/substance checks (nine unique paths). Manual implementation and usage inspection establishes wiring; file existence alone was not accepted.

| Artifact | Expected behavior | Status / wiring evidence |
| --- | --- | --- |
| `14-DIAGNOSIS.md` | Failure provenance and demonstrated repair seam | VERIFIED — raw original/current/probe records, full input expansion and source call path agree. |
| `crates/liquidfun/src/particle/storage/solver_state.rs` | Actual-row scratch reservation with capacity guards | VERIFIED — zeroed lane is used by optional scratch creation, append and permutation; budget/value/permutation tests protect behavior. |
| `crates/liquidfun/src/world/particle_object/group.rs` | Sole clone-plan-commit authority | VERIFIED — public create calls plan then commit; source clone, topology validation and shell/diagnostic preflight precede publication. |
| `crates/liquidfun/tests/particle_group_properties.rs` | Named exact replays and bounded random properties | VERIFIED — both named cases run explicitly in all three OS jobs; full target runs six tests. |
| `crates/liquidfun/src/world/object/tests/particle_group_transactions.rs` | Complete private transaction comparison | VERIFIED — registered in object tests; full equality and populated state assertions execute in library suites. |
| `crates/liquidfun/tests/particle_group_properties/snapshot.rs` | Actual-ID rollback distinct from normalized replay | VERIFIED — model captures before each operation and compares after every rejection. |
| `crates/liquidfun/tests/particle_groups/transactional_rejection.rs` | Public typed no-effect rejection cases | VERIFIED — registered by `particle_groups.rs`; five new creation/append cases plus companion mutation cases run in focused targets. |
| `.github/workflows/ci.yml` | Existing supported OS matrix with visible selections | VERIFIED — printed SHA/compiler, inventory, exact/focused/full tests and successful actual job logs. |
| `14-PLATFORM-EVIDENCE.md` | Same-SHA executed evidence and retained history | VERIFIED — final metadata, raw logs and hashes independently agree; full quality success separately confirmed. |

### Key Link Verification

| From | To | Via | Status |
| --- | --- | --- | --- |
| Diagnosis | Original public replay | Literal seed, controls, operation-14 Append and expected successful growth | WIRED |
| Diagnosis | `world/particle_object/group.rs` | Relative path plus `World::plan_particle_group` call/stage trace | WIRED |
| Group planner | Storage mutation planner | `plan_group` prepares/validates before `commit_particle_group` | WIRED |
| Private transaction tests | Authoritative storage | Full structural storage equality and cfg(test) state support | WIRED |
| Property model | Snapshot module | Imported `rollback_snapshot`; before/after assertion for Rejected outcomes | WIRED |
| Public rejection child | Integration target | `#[path] mod transactional_rejection` registration | WIRED |
| Cargo matrix | Exact replay target | Two exact command names, inventory six and actual one-test counts | WIRED |
| Platform evidence | Cargo workflow/run | REST repository/path/ref/SHA/attempt plus actual job logs | WIRED |

The generic key-link checker recognizes only three of eight links because it searches literal file paths/patterns. Manual checks resolve the remaining relative Rust modules, reverse parent registration and abbreviated documentation path. These are confirmed connections, not accepted deviations; no override is used.

### Data-Flow Trace (Level 4)

There is no dynamic UI in this headless phase. The applicable semantic-state flow was inspected instead.

| Artifact | Data | Source and consumption | Status |
| --- | --- | --- | --- |
| Group candidate | Sampled rows, IDs, lifetime/contact/topology state | Checked public recipe → cloned ParticleSystem → validated GroupPlan → complete system publication | FLOWING |
| Private rollback comparison | Storage/lifetime/arena/diagnostic state | Actual live world captured before rejection, compared field-for-field afterward; populated inaccessible categories are explicitly test-seeded | FLOWING |
| Public rollback snapshot | Actual particle/group IDs, lanes, rest data, contacts and lifecycle | Public views/snapshots of populated fixture → exact equality → successful follow-up and listener checks | FLOWING |
| Platform proof | SHA/compiler/test results | Actual run/job metadata and raw logs → recomputed counts/digests → source-bound report | FLOWING |

No empty fixture or normalized fresh-world identity replaces same-world rollback proof. Rare retired-generation and inaccessible association/scratch fields are intentionally populated through cfg(test) helpers; this is disclosed test construction, not claimed public reachability.

## Behavioral Verification

The verifier did not rerun Cargo or start services, avoiding contention with executor-owned gates. It independently inspected executable source and retained actual output. Read-only spot checks completed in under ten seconds: all 32 source blobs match the final candidate; all three platform validators recompute identically without writing output; quality metadata/log hash and terminal run success agree; all 17 historical digest entries match; `git diff --check` passes.

| Behavior / command | Observed result | Status |
| --- | --- | --- |
| Both `cargo test -p liquidfun --test particle_group_properties <name> -- --exact --nocapture` selections | Exactly one pass each on Windows/Linux/macOS; five filtered | PASS |
| Focused property/group/mutation targets | 10 mutation + 6 property + 12 creation tests per OS | PASS |
| `cargo test -p liquidfun` | 957 per OS: 422 library + 512 integration + 23 doctests; zero failures/ignored | PASS |
| Local ordered all-feature core gate | 1,012: 423 library + 567 integration + 22 doctests; retained Plan 03/04 gate logs | PASS |
| `cargo test --workspace --all-features` in final quality job | 160 harness result blocks; 2,175 passing executions, zero failures | PASS |
| Complete quality/isolation job | All 26 steps successful in 25m45s, including later headless, package, corpus, focused, docs and inventory gates | PASS |

The workspace suite has one pre-existing ignored `regenerate_case_fixture` authoring tool in `crates/liquidfun-differential/tests/phase9_corpus/execution.rs:207`, already ignored at the phase base. It writes regenerated fixtures and is not a suppressed acceptance test. No phase test or mandatory job step is skipped. All default-platform exact/focused/full-package results have zero ignored tests. The local 1,012 and remote 957 totals are different feature selections, not interchangeable counts.

### Final Source and Evidence Identity

Repository/workflow/event/ref checks confirm `bright-builds-llc/liquidfun-rs`, `.github/workflows/ci.yml`, push/main, source `417d38dac6951226fb32ea99a97135defe88da7b`, run `34815192937`, attempt 1. All platforms print Rust 1.97.0 compiler commit `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`.

| Job | Host | Raw log SHA-256 |
| --- | --- | --- |
| Linux `103884302645` | x86_64-unknown-linux-gnu | `86e32182917204e01dff10cd251562b5a1ac115e8a1d0e6a94b0323186817e2a` |
| macOS `103884302627` | aarch64-apple-darwin | `1b4a4ffe9888cd40ef80fc871a1de16381cca25c22203ade144d71335ea0129e` |
| Windows `103884302536` | x86_64-pc-windows-msvc | `e94f6f3d4250c8471819da09ea78d6414a7658a222290357060b5749b85eef8f` |
| Linux quality `103884302389` | ubuntu-24.04 | `d926df4b7e170700ed02087b32e5b2cd4cdf12d0286dca20f824f315960d6c96` |

Records reside under `target/phase14-platform/417d38dac6951226fb32ea99a97135defe88da7b/attempt-20260914-01/`. This verification recomputed retained evidence rather than fetching a second copy from GitHub; it is not an immutable canonical attestation.

## Requirements Coverage

All six roadmap IDs occur in PLAN requirements; none is orphaned. Satisfaction here closes the audited group repair gaps while retaining previously implemented subsystem coverage. It does not newly assert complete engine parity or release readiness.

| Requirement | Source plans | Description | Status / evidence |
| --- | --- | --- | --- |
| PART-03 | 01–04 | Dense indices may change while stable IDs resolve until destruction | SATISFIED — actual-ID rollback, ordered append/join/split, stale/pending behavior, complete arena comparison and next-allocation proof; broader particle identity tests pass. |
| PART-04 | 02–04 | Permutations update required/optional lanes, maps, proxies, contacts, topology, lifetimes and ranges atomically | SATISFIED — existing storage-owned preparation retained, scratch values/permutations tested, complete populated private equality and public rest/contact/lane snapshots. |
| PART-09 | 01–04 | Public group creation from shapes, strokes, positions/existing groups and complete inspection | SATISFIED — both exact sequences require explicit/filled/stroke/reactive/lifetime creation and append success; group views/snapshots and recipe suites pass. |
| PART-10 | 02–04 | Group lifecycle, joining/splitting/connectivity/empty/solid/rigid semantics | SATISFIED — existing lifecycle/topology authority retained, successful neighboring operations and full group/particle suites pass; zero-rest rejection follows existing pinned witness policy. |
| TEST-02 | 03–04 | Supported public world/rigid/joint/particle/callback/query/destruction integration tests | SATISFIED — new nine-case group regression coverage plus complete package/workspace integration suites; prior public workflows remain passing. |
| TEST-04 | 01, 03–04 | Geometry/broad-phase/handle/permutation/query/world-operation properties | SATISFIED — unchanged bounded 128-case generator, exact persisted seeds and explicit outcomes; complete package/workspace properties pass. |

## Anti-Patterns and Disconfirmation

No blocking stub, orphaned artifact, new ignored phase test, diagnostic hook, production Windows conditional or canonical-evidence bypass was found in the 32-file scope. The only scanned Windows branch is a pre-existing-style executable filename selection in a private CLI fixture. Code review reports zero findings; security review records all 16 registered mitigations closed and no accepted risks. These reviews supplement the source and evidence checks rather than substitute for them.

Three plausible false passes were explicitly rejected: mapper-only suppression fails scratch-budget/success assertions; rejection-only replays fail Applied checks; broad topology classification fails real corrupt-lane and other-generation-error controls. Only `ConstraintError::ZeroLengthPairDistance` maps to the new step topology category. The existing Phase 10 `zero_length_pair` witness requires typed rejection; no geometry, epsilon or expected oracle bits changed.

Forced host allocator exhaustion is not reproduced as a final stress test: bounded reservation behavior, the historical raw Windows failure and private exact mapper controls provide the relevant evidence without hazardous multi-gigabyte allocation. Private complete equality plus public populated follow-ups support group transactionality; no global whole-step transaction claim follows from the specific coincident-reactive no-effect test.

The adjacent CI corrections preserve production validators and meaningful negative controls. Disposable history tests exercise valid v2, schema-v1 rejection, metadata-only descendants and changed-closure rejection. Materials tests check known independent digests, content changes and missing files without requiring an omitted native checkout. The final scheduling change runs all four private packages in the earlier all-feature workspace suite under the same headless variables, removes only redundant repetitions, and retains explicit testbed/focused gates and the original timeout.

## Human Verification Required

None. This phase changes headless checked Rust behavior and automated platform verification; all required behaviors have executable regression evidence. No visual, interactive or external-service behavior remains unverified within Phase 14.

## Deferred Scope

| Item | Addressed in | Roadmap evidence |
| --- | --- | --- |
| Actual current-head canonical acceptance, complete candidate producer bundle, aggregation and frozen-source attestation | Phase 15 | Its success criteria require one frozen full-SHA candidate, complete retained evidence, accepted aggregation and worktree/committed-range attestation. |

This is the pre-existing explicit phase boundary, not a newly waived Phase 14 criterion. Synthetic fixture acceptance does not erase the observed real replay-closure drift or promote canonical receipts.

## Lifecycle and Completion

All nine upstream formal CONTEXT/PLAN/SUMMARY frontmatter blocks carry `lifecycle_mode: yolo` and `phase_lifecycle_id: 14-2026-07-27T16-15-14`; their generators are the formal discuss/plan/execute roles, with no direct-fallback provenance. This report copies that lifecycle. No prior VERIFICATION.md or override was present.

Repo-local AGENTS guidance, Bright Builds sidecar/overrides and architecture/code-shape/verification/testing/Rust standards informed this assessment. Both active lesson files were loaded within their combined 11,548-byte/3,850-estimated-token budget; no project skill directories exist. The verifier modified only this parser-owned GSD report, ran no Cargo command and made no commit.

No Phase 14 gap remains. The goal is achieved at the explicitly tested source. The orchestrator owns final metadata gates, commits and roadmap/requirement completion; Phase 15 remains deferred.
