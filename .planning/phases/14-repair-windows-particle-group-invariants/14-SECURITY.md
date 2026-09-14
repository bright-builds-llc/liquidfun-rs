---
phase: "14"
slug: "repair-windows-particle-group-invariants"
status: verified
threats_open: 0
asvs_level: 1
created: "2026-09-14"
tested_source: 417d38dac6951226fb32ea99a97135defe88da7b
previous_tested_source: fc3af47291750af418befe98ad437dc40aa87bb3
reviewed_source: 417d38dac6951226fb32ea99a97135defe88da7b
final_candidate_evidence: verified
---

# Phase 14 — Security

All 16 registered threats have verified mitigations at the recorded tested source. The final 32-file source scope and same-SHA platform/quality evidence are verified at `417d38dac6951226fb32ea99a97135defe88da7b`. This is a scoped verification of the four PLAN threat models, not a vulnerability scan, web-application certification, whole-phase acceptance, or release authorization. ASVS level 1 is the workflow default label only. Security enforcement is enabled by the workflow fallback because `.planning/config.json` has no security override; no configured `block_on` override was found. Open registered threats would block advancement; unregistered summary flags are informational.

## Trust Boundaries

| Boundary | Description | Data Crossing |
| --- | --- | --- |
| CI logs to diagnosis | Identify actual source, workflow and runner before interpreting failures | Run/job/attempt metadata, seeds and diagnostic output |
| Public request to isolated candidate | Bound sampling and topology work while preserving legitimate success and typed rejection | Recipes, particle rows, handles and allocation requests |
| Candidate to live world | Publish only fully prepared valid state | Identity arenas, lanes, lifetime/topology state and diagnostics |
| Public API to test model | Preserve exact outcomes and same-world rollback observations | Errors, actual identities, semantic state and lifecycle records |
| Local commit to remote runner | Bind executed commands and results to the intended source | Repository/ref/full SHA, job identity and test inventory |
| D2 results to acceptance authority | Keep portability proof separate from canonical and release authority | Platform results and maturity claims |

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
| --- | --- | --- | --- | --- | --- |
| T-14-01-01 | Spoofing | CI identity | mitigate | Verify repository, workflow path, SHA, attempt, runner and job before citing output. | closed |
| T-14-01-02 | Tampering | Persisted replay | mitigate | Preserve original seed/controls/expanded operations; retain newer cases separately. | closed |
| T-14-01-03 | Denial of Service | Diagnostic replay | mitigate | Preserve 24-operation, 8-group, 32-particle bounds and use bounded probes. | closed |
| T-14-01-04 | Repudiation | Failure report | mitigate | Retain immutable failure logs and digests; distinguish observations from hypotheses and D2 from D1/D3. | closed |
| T-14-02-01 | Denial of Service | create_particle_group | mitigate | Repair demonstrated reachable panic and preserve sampling/topology limits. | closed |
| T-14-02-02 | Tampering | commit_particle_group | mitigate | Validate full owned candidate before publication; exact private rollback and next-allocation tests. | closed |
| T-14-02-03 | Repudiation | Error classification | mitigate | Assert exact legitimate rejection type and valid success; preserve RED trace and cause. | closed |
| T-14-02-04 | Information Disclosure | Public API | mitigate | Keep private storage categories and test snapshots private; retain existing stable errors. | closed |
| T-14-03-01 | Tampering | Model outcomes | mitigate | Preserve typed error categories and explicit valid-operation success assertions. | closed |
| T-14-03-02 | Tampering | RollbackSnapshot | mitigate | Compare actual IDs and complete exposed before/after state, including lifecycle output. | closed |
| T-14-03-03 | Denial of Service | Property generation | mitigate | Preserve bounded work and existing 128-case profile without silent skip/retry. | closed |
| T-14-03-04 | Repudiation | Fixed regression inputs | mitigate | Preserve separate seeds, controls and explicit operation expansions with source provenance. | closed |
| T-14-04-01 | Spoofing | Run identity | mitigate | Verify repository/workflow/ref/full SHA and actual OS job IDs before acceptance. | closed |
| T-14-04-02 | Tampering | Test selection | mitigate | Require exact count one for each named test, complete target inventory, successful full package and no skips. | closed |
| T-14-04-03 | Repudiation | Attempts | mitigate | Retain separate candidate/attempt logs, digests and dispatch reconciliation; never overwrite failed records. | closed |
| T-14-04-04 | Elevation of Privilege | Evidence tier | mitigate | Classify D2 only; leave canonical fixtures, compatibility rows and Phase 15 aggregation untouched. | closed |

Every entry was classified as `mitigate` before verification. No accepted or transferred disposition was substituted for a missing control.

## Threat Verification Evidence

Paths beginning with `14-` are relative to this phase directory. Current platform records are under `target/phase14-platform/fc3af47291750af418befe98ad437dc40aa87bb3/attempt-20260914-01/`. Source references identify the tested candidate, not a later documentation HEAD.

| Threat ID | Evidence |
| --- | --- |
| T-14-01-01 | `14-DIAGNOSIS.md:29` and historical run/job JSON: repository/workflow, full SHA, attempt 1, actual Windows job and runner match; original run 30070539790/job 89410277623 and later run 34777296141/job 103777632638 remain failures. |
| T-14-01-02 | `crates/liquidfun/tests/particle_group_properties.rs:58` and `:133` retain separate literal seeds, controls and expanded operations; `:333` and `:404` assert generated-operation equality. Original seed 4149329052036581951 and later seed 190752942043209832 remain distinct. |
| T-14-01-03 | `crates/liquidfun/tests/particle_group_properties.rs:14` fixes 24/8/32 bounds; `:239` caps operations. `particle_group_properties/model.rs:126` and `:355` enforce creation/split budgets. `14-DIAGNOSIS.md:139` records the bounded 2,048-record temporary observer; its retained probe patch exists and probes were removed. |
| T-14-01-04 | `14-DIAGNOSIS.md:46`, `:133`, `:147`, `:184`: historical hypothesis and later observed allocation error are explicitly separate, with full retained file digests. All 17 historical log/metadata table digests were recomputed successfully; D2 classification remains explicit at `:11`. |
| T-14-02-01 | `crates/liquidfun/src/particle/storage/solver_state.rs:301` preserves declared-capacity/signed-index preflight; `:322` reserves actual particle_count. `solver_state/tests.rs:8` checks ten-row allocation budget. `world/particle_object.rs:34` retains sampling/topology work, cell, queue and node limits. Both fixed appends actually pass on all three OS jobs. |
| T-14-02-02 | `crates/liquidfun/src/world/particle_object/group.rs:173` clones the owned system and preflights identities/diagnostics; `particle/storage/mutation.rs:118` and `:175` validate source/full candidate; `group.rs:254` publishes afterward. `world/object/tests/particle_group_transactions.rs:141` compares storage, lifetime, arenas, system metadata/order and diagnostics; `:214` verifies next allocations and deferred lifecycle. `arena.rs:36` includes free/retired state. |
| T-14-02-03 | `crates/liquidfun/src/world/object/tests/particle_group_transactions.rs:179` and `:193` require exact InvalidParticleGroupTopology for new/append topology rejection. `particle_group_properties.rs:333` and `:404` require successful original/later appends. `14-DIAGNOSIS.md:147` retains the RED raw allocation cause; `14-02-SUMMARY.md` retains sizing RED/GREEN logs and distinguishes grid-limit localization from an observed raw trace. |
| T-14-02-04 | `crates/liquidfun/src/world/particle_object.rs:263` uses private candidate-only mappers and preserves existing CreateObjectError categories. `:448` retains the authoritative-corruption panic control. `particle/storage.rs:37` and `arena.rs:35` gate test helpers with cfg(test). Private reactive cause vocabulary stays inside `particle/storage/group.rs:15`; the separately amended public StepError topology category reveals no storage internals. |
| T-14-03-01 | `crates/liquidfun/tests/particle_group_properties/model.rs:220` preserves explicit creation/pending-delete categories and nonzero pending witness; `:261` requires target identity, member order and exact successful append growth. Fixed regressions `particle_group_properties.rs:333` and `:404` require Applied outcomes. Step only accepts precise topology rejection; `particle/storage/group.rs:27`, its `tests.rs:468`, and `world/particle_coupling/executor.rs:263` distinguish zero-rest pairs from storage corruption and other generation causes. |
| T-14-03-02 | `crates/liquidfun/tests/particle_group_properties/snapshot.rs:160` and `:351` retain actual particle/group IDs, owned system snapshot and lifecycle records alongside normalized semantics. `particle_groups/transaction_support.rs:13` captures actual endpoints, lanes, rest data, contacts and metadata; `:269` asserts populated witnesses; `:306` requires exact deferred listener identity/cause and no later extra output. Both focused rejection modules assert complete before/after equality and successful follow-ups. |
| T-14-03-03 | `crates/liquidfun/tests/particle_group_properties.rs:13` retains generator version 1 and 24/8/32 bounds; `:296` retains 128 cases. `model.rs:126` and `:355` guard growth and emit explicit SkippedAtBound outcomes. These are visible generator budget outcomes, not ignored tests or silent retries. The six-test property target and both exact fixed tests ran successfully on each OS with zero ignored tests. |
| T-14-03-04 | `crates/liquidfun/tests/particle_group_properties.rs:58` and `:133` preserve literal original/current inputs and source provenance; equality assertions protect expansions. `particle_group_properties.proptest-regressions` separately retains three newly exposed inputs. `14-DIAGNOSIS.md:29` preserves distinct original/current source and run identities. |
| T-14-04-01 | Current attempt `run-initial.json` records bright-builds-llc/liquidfun-rs, .github/workflows/ci.yml, push/main, full fc3af47291750af418befe98ad437dc40aa87bb3, attempt 1. Terminal linux/macos/windows-job.json match run 34809888461 and jobs 103868959198/103868959321/103868959151; each raw log prints matching checkout SHA and compiler host. |
| T-14-04-02 | `.github/workflows/ci.yml:114` retains the existing three-OS matrix and full package after inventory/exact/focused commands. Read-only recomputation of current `validate_platforms.py` matches `platform-validation.json`: inventory 6, exact counts 1+1, focused 10/6/12, complete default package 957 per OS; all job steps successful, all harness failures/ignored counts zero. |
| T-14-04-03 | `14-DIAGNOSIS.md:19`, `14-PLATFORM-EVIDENCE.md` failure-history table, and distinct target/phase14-diagnostics and target/phase14-platform source/attempt directories retain failed and current records. Historical digests and current three raw-log hashes match. Current run metadata identifies a push; no dispatch/rerun was required or claimed. |
| T-14-04-04 | `14-PLATFORM-EVIDENCE.md` frontmatter and opening/closing scope explicitly classify D2 and reserve final aggregation/attestation for Phase 15. Base 5aadb6c105b98ae09443f74e44c57f8ce7eae19d to candidate fc3af47291750af418befe98ad437dc40aa87bb3 changed-file inspection found no canonical fixture, compatibility-status or Phase 15 artifact change. |

## Verification Method and Limits

The auditor inspected the registered mitigation paths and their focused tests, compared all 26 review-scope working files with the replacement candidate blobs, and found exact byte equality. The existing `14-REVIEW.md` reports a clean static review; that report is supporting context, not a substitute for mitigation inspection. No implementation file was modified, no Cargo command was run by this auditor, and no commit was created.

The retained platform validator was evaluated read-only through its output construction, omitting its file-writing statements. Its recomputed output equals the retained JSON. Repository/workflow/ref identity was checked independently against run metadata. Historical source/run/job identities and 17 documented historical metadata/log digests were also checked. This verifies the local retained evidence, not a fresh independent GitHub retrieval or an immutable external attestation.

The three completed default-feature jobs prove the named regressions, focused targets and complete default-feature package at the cited SHA. Legitimately feature-disabled full-package targets can report zero tests; neither exact regression is a zero-test success. Existing explicit `SkippedAtBound` model outcomes preserve work budgets and are not ignored harness tests.

At the latest provisional audit cutoff, run `34809888461` has three successful default-feature jobs but failed quality job `103868959038`. The later six-file fixture repair is committed as `430e0ad775d64322b04ae04504d5a6275a197dc3` and has no fresh CI binding. Both failed quality candidates remain preserved. Plan 14-04 has not supplied its final summary. Whole-workflow and whole-phase acceptance remain with the executor/coordinator after terminal evidence. Closing these mitigation entries does not certify that unfinished lane or silently rebind evidence to subsequent documentation commits.

The public `StepError::InvalidParticleGroupTopology` boundary classifies only the existing zero-rest-pair rejection. Authoritative storage errors and other generation causes retain the invariant boundary. Public/private group-mutation rollback tests establish their exercised transaction boundaries; this report makes no new global world-step rollback claim.

Guidance loaded: repo-local `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, standards index and verification/testing/Rust pages, Phase 14 context, all four threat-model blocks, available Plans 01–03 summaries, diagnosis, review and draft platform evidence. Both active lesson files were read in full (5,230 + 6,318 bytes, below startup limits). The auditor's write ownership is limited to this SECURITY.md; coordinator-owned task/lesson and phase lifecycle records were preserved.

## Unregistered Flags

No available SUMMARY has a `## Threat Flags` section or an unmapped executor threat flag. Narrative observations report no new unplanned surface. The narrow zero-rest error-classification amendment maps to T-14-02-04 and T-14-03-01; it is informational within those registered boundaries. Plan 14-04's final SUMMARY remains to be checked when produced.

## Accepted Risks Log

No accepted risks. No risk waiver, acceptance fiction, or new human approval timestamp was introduced.

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
| --- | --- | --- | --- | --- |

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
| --- | --- | --- | --- | --- |
| 2026-09-14 | 16 | 16 | 0 | gsd-security-auditor |

This State B audit created the report from the authoritative PLAN registers and available execution artifacts. Standing authorization in AGENTS.md dated 2026-09-13 authorizes verification; it does not waive controls. Simplification review retained the existing candidate boundary, narrow cause classification and evidence tier instead of adding another transaction or authority layer.

## Security Audit 2026-09-14 — replacement candidate

| Metric | Count |
| --- | --- |
| Threats found | 16 |
| Closed | 16 |
| Open | 0 |

The initial audit above verified source `ce37c5632507df2be9baf61305967a57a68567d4`, run `34808501136` attempt 1, and OS jobs `103864994066` / `103864994179` / `103864994343`. At its original cutoff quality was pending. It subsequently failed; retained `quality-job.json` confirms the failure and `quality.log` matches SHA-256 `ab7947a0550f025ba77c98afa1aca9793b6763e9ab2f432973317b512f9478e0`. That prior scoped mitigation verification and its evidence are preserved, without treating the failed workflow as accepted.

The replacement audit verifies source `fc3af47291750af418befe98ad437dc40aa87bb3`. The exact incremental diff contains two source changes plus `14-CI-PREREQUISITE-REPAIR.md`: CI installs the existing `nightly-2026-07-15` / `rust-src` and `cargo-fuzz` 0.13.2 `--locked` prerequisites, and the Phase 13.1 test checks the sole deferral in structured frontmatter. The pins match `fuzz.yml`; the real fuzz-build smoke remains enabled. The test still requires exactly one truth and one exact Phase 15 destination, while the historical verification report remains unchanged. These changes preserve T-14-04-02 test execution and T-14-04-04 evidence authority. No particle mitigation source changed; all 26 scoped source files match replacement candidate blobs.

Run `34809888461`, push on main, attempt 1, matches the expected repository/workflow/full source identity. The new validator was independently recomputed read-only and equals its saved JSON. Each terminal OS job has successful steps, property inventory 6, exact tests 1 + 1, focused tests 10 / 6 / 12, full default package 957, and zero failed/ignored harness tests.

| Platform | Job ID | Raw log SHA-256 |
| --- | --- | --- |
| Linux | 103868959198 | d279265d0c526c8d0832bb339eb94a3ec87975367dddfe14b832e42550e0b025 |
| macOS | 103868959321 | c61e623a5ab94a7bf8a01702062b9a7839ab12ff5a477f6b008693b476c279be |
| Windows | 103868959151 | 6e14633c2f44940767af4ddb59cafab7a44c52e784dccfc0165092ae301f84a5 |

The replacement quality lane and final Plan 04 evidence remain pending. No whole-workflow success, new accepted risk, skipped requirement, or Phase 15 promotion is asserted. This incremental audit changes only SECURITY.md and preserves the earlier audit trail.

## Security Audit 2026-09-14 — provisional fixture repair

Static review found no gap in the registered controls. The 32-file scope is recorded in `target/phase14-review-scope-final.json`; exactly six WIP files differ from `fc3af47291750af418befe98ad437dc40aa87bb3`, and the other 26 match that source. No new candidate SHA or CI result is assigned to these WIP bytes.

| Registered controls | Static evidence |
| --- | --- |
| T-14-02-04: private test surface | `tools/xtask/src/phase13_acceptance.rs:10` adds only a cfg(test) child-module registration; production acceptance function bodies are unchanged. |
| T-14-04-02: meaningful execution | Four history cases call the unchanged validator on disposable Git history: valid schema-v2 and metadata-only descendant success, exact Schema rejection for schema-v1 and Closure rejection for replay drift. No case is ignored or converted to unconditional success. |
| T-14-04-03: preserved failure records | Previous quality job 103868959038/run 34809888461 is retained as failure at fc3af47291750af418befe98ad437dc40aa87bb3. Its raw log digest was independently checked: 47218e901505759c863d0f75f0e7cabd5c1e75bea15830160d330bab8e2aabdf. |
| T-14-04-04: no canonical promotion | HistoryFixture creates synthetic bytes in a newly owned temporary root; its writes and Git operations target that root. Materials tests use synthetic inputs for known digest, ordering/duplicates, content-change and missing-file controls, and separately check the tracked 176-entry inventory. They do not certify real native bytes. Phase 9 retains target mutation and repository-binding rejection checks while removing only the real-native-root prelude. |

The real repository's replay closure drift remains the Phase 15 deferral. Synthetic promotion receipts are explicitly test fixtures, not actual authorization, acceptance records, or canonical evidence. No production hashing/validation body, real receipt, canonical fixture, manifest or Phase 15 artifact was changed by these six files. The previous 16 closures remain recorded for the bound source; this provisional inspection does not extend its old platform results to WIP.

Reviewed WIP SHA-256 snapshots:

| File | SHA-256 |
| --- | --- |
| `tools/xtask/src/phase13_acceptance.rs` | 8c66d9e4c812daa9ddc4e0208b9abe0f10748a669f65f870f64cae7918ad4405 |
| `tools/xtask/tests/phase13_acceptance_contract.rs` | 11bdc318d394de0cdc7ce08e21438684f806f45a10711de436495e77f70a8d7c |
| `tools/xtask/tests/phase13_acceptance_contract/history_fixture.rs` | 005040c847cddbfba148e8338495586de5d181362f39774fd1c6b80a7ed30dd4 |
| `tools/xtask/tests/phase13_evidence_contract.rs` | b411de65cf7764d0f30e5011f9fba94edc8df8547fe0a60e7ed5e315c215500c |
| `tools/xtask/tests/phase13_evidence_contract/materials.rs` | 67cff4c1d685a591f543e4316261f687a8b6925e15fe04eef1b7a475d27e890d |
| `tools/xtask/tests/phase9_witness_provenance.rs` | 248c5b53c623eaac9e8cd4981b303bf37abddce286abd6b7b8e836574e52fb5a |

Source binding and fresh same-SHA platform/quality evidence remain pending the coordinator's handoff. No Cargo commands, source edits, commits or risk waivers were performed by this auditor. Current AGENTS.md local authority and overrides were reloaded; previously loaded sidecar and verification/testing/Rust guidance continue to apply.

## Source Binding 2026-09-14 — fixture repair candidate

The coordinator committed the six reviewed Rust fixture files and two repair notes as `430e0ad775d64322b04ae04504d5a6275a197dc3`. All 32 scoped working source files were compared byte-for-byte with that candidate's Git blobs and match. The provisional six-file hashes above remain the reviewed static snapshots. No registered mitigation gap was found; no additional source change occurred during binding.

`reviewed_source` identifies this new static candidate, while `tested_source` retains the previous source with independently checked three-platform logs. Fresh same-SHA platform/quality evidence is pending and old passing platform results do not certify this candidate. The executor reports passing local workspace Clippy, 1,012 core tests and affected targets 31/24/2; these are contextual local results, not Cargo commands rerun by this auditor or complete CI acceptance.

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log (none)
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-14 by the security auditor for registered mitigation existence only. Whole-phase and whole-workflow acceptance remain subject to the executor's terminal evidence and remaining verification gates.

## Scheduling Mitigation Review — 2026-09-14T06:52:55Z

The coordinator checked the workflow-only change at `417d38dac6951226fb32ea99a97135defe88da7b` against the Plan 04 threats, supplemented by the independent verifier’s coverage review. Complete workspace test selection is retained with explicit headless environment; redundant selected-package execution is removed. Exact platform commands, count checks, source identity, read-only permissions, checkout isolation and evidence authority remain unchanged. All 32 scoped source files match this reviewed candidate. No registered mitigation gap was introduced; 16/16 remain closed and zero are open. The cancelled `430e0ad775d64322b04ae04504d5a6275a197dc3` quality run remains failed evidence, and fresh same-SHA quality/platform binding is still pending.

## Final D2 Evidence Binding — 2026-09-14T07:18:24Z

The coordinator independently checked retained run `34815192937`: repository `bright-builds-llc/liquidfun-rs`, push on main, full source `417d38dac6951226fb32ea99a97135defe88da7b`, terminal success. All 26 quality-job steps (`103884302389`) succeeded, and its raw log digest is `d926df4b7e170700ed02087b32e5b2cd4cdf12d0286dca20f824f315960d6c96`. The three platform raw-log hashes match their retained records and each complete package totals 957 tests. The independent phase verifier also checked source and platform identities. This closes the pending fresh-evidence binding for the existing 16 registered mitigations, with zero open threats. Prior failures remain excluded; these are Phase 14 D2 results and do not confer Phase 15 canonical or release acceptance.
