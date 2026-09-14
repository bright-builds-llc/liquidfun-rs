---
phase: 14-repair-windows-particle-group-invariants
evidence_tier: D2
tested_source: 417d38dac6951226fb32ea99a97135defe88da7b
run_id: 34815192937
run_attempt: 1
status: verified
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 14-2026-07-27T16-15-14
generated_at: 2026-09-14T07:17:30Z
---

# Phase 14 supported-platform evidence

**Source `417d38dac6951226fb32ea99a97135defe88da7b` passes both fixed Windows regressions, all focused particle-group targets, the complete default-feature liquidfun package on Windows/Linux/macOS, and the complete Linux quality/isolation lane.**

These are D2 portability and regression results. No canonical fixture, compatibility row, release or Phase 15 acceptance is promoted. The production validator still enforces real current-head closure identity against frozen promotion records; synthetic contract-test success does not certify that deferred acceptance.

## Verified source and invocation

- Repository: `bright-builds-llc/liquidfun-rs`; verified origin `git@github.com:bright-builds-llc/liquidfun-rs.git` and default branch `main`.
- Workflow `.github/workflows/ci.yml`, Cargo CI; ordinary non-force push to main.
- [Run 34815192937, attempt 1](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937) is completed/success. REST metadata binds repository, workflow path, push event, main branch, full source SHA and attempt; all four actual jobs are completed/success.
- Each platform prints actual `git rev-parse HEAD` and `rustc -vV`. Every result names the tested full SHA, Rust 1.97.0, compiler commit `2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`, commit date 2026-07-07 and LLVM 22.1.6.
- Standing authorization in AGENTS.md dated 2026-09-13 supplied publication and inspection authority. Each candidate used a fresh push-triggered run at attempt 1; no dispatch, rerun, force push, timeout increase or acceptance waiver occurred.
- Tracked source was clean before publication. Known coordinator-owned planning/report drafts were excluded from source publication and never presented as executed source.

## Same-source platform results

| Platform / job | Actual runner | Rust host / image | Exact tests | Focused targets | Full package |
| --- | --- | --- | --- | --- | --- |
| [Linux 103884302645](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937/job/103884302645) | GitHub Actions 1000004196 | x86_64-unknown-linux-gnu; ubuntu-24.04 20260907.300.1 | 1 + 1 pass | 10 + 6 + 12 pass | 957 pass |
| [macOS 103884302627](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937/job/103884302627) | GitHub Actions 1000004197 | aarch64-apple-darwin; macos-15-arm64 20260907.0337.1 | 1 + 1 pass | 10 + 6 + 12 pass | 957 pass |
| [Windows 103884302536](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937/job/103884302536) | GitHub Actions 1000004194 | x86_64-pc-windows-msvc; windows-2025-vs2026 20260907.229.1 | 1 + 1 pass | 10 + 6 + 12 pass | 957 pass |

Every platform step completed successfully, without skipped steps or tolerated errors. Inventory lists six property-test names. Each exact command runs one test and filters five. Focused target order is mutation (10), property (6), creation (12). All default-package harness results have zero failed and zero ignored tests. Naturally feature-disabled target binaries may contain zero tests; neither exact regression is a zero-test selection.

Exact commands after the default build:

```text
cargo test -p liquidfun --test particle_group_properties -- --list
cargo test -p liquidfun --test particle_group_properties persisted_audited_windows_seed -- --exact --nocapture
cargo test -p liquidfun --test particle_group_properties persisted_current_windows_seed -- --exact --nocapture
cargo test -p liquidfun --test particle_group_properties --test particle_groups --test particle_group_mutation
cargo test -p liquidfun
```

Existing subsequent public math/protocol checks pass too. Rust/action pins, supported labels, cache conventions, fail-fast:false, read-only permissions, native-submodule isolation and mandatory complete package testing remain.

### Count boundaries

Remote default-package execution totals 422 library + 512 integration + 23 doctests = 957. Local `cargo test --all-features` runs the default workspace member with 423 library + 567 integration + 22 doctests = 1,012. Differential-internals adds one library and 55 integration cases and removes the default-only `collision::differential_feature_is_disabled` compile-fail doctest. Both totals include doctests and exclude separate exact/focused invocations. The difference is feature selection, not inconsistent counting.

## Terminal Linux quality proof

[Quality job 103884302389](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937/job/103884302389), GitHub Actions 1000004193, ran from `06:51:13Z` to `07:16:58Z` on 2026-09-14: 25m45s. All 26 job steps completed successfully within the unchanged 30-minute limit.

Passing gates include Markdown and Rust formatting, workspace Clippy/build, pinned nightly/cargo-fuzz installation, the full all-feature workspace suite under explicit empty display variables, consumer package isolation, corpus closure, explicit headless testbed build/test, protocol/comparison/supervisor/provenance contracts, documentation checks/build with denied warnings, and inventory checks. Repaired suites record phase13_1_gap_verification 66 pass, acceptance 31 pass, materials/evidence 24 pass, and Phase 9 provenance two pass. No failed assertion or cancelled step remains in this run. The pre-existing ignored `regenerate_case_fixture` entry is an explicit regeneration tool, unchanged by this work; it is not a suppressed acceptance test.

The workspace command segment contains 160 successful harness result blocks and 2,175 passing test executions, zero failures, and that one pre-existing ignored authoring tool. These are workspace totals, distinct from the 957 default-package and 1,012 all-feature core totals; later focused commands are not included in this sum.

## Retained current records and digests

Directory: `target/phase14-platform/417d38dac6951226fb32ea99a97135defe88da7b/attempt-20260914-01/`.

Raw per-job logs and job metadata, separate initial/final run metadata, terminal watcher output, and `digests.json` are retained. `validate_platforms.py` independently checks actual source/run/attempt/status, all platform step conclusions, printed checkout/compiler identity, inventory six, exact counts 1/1, focused counts 10/6/12 and full-package total 957; results are in `platform-validation.json`. Separate terminal checks require run/quality completed-success and all 26 quality steps successful. These are inspection aids, not a new canonical attestation schema.

| Raw log | SHA-256 |
| --- | --- |
| `linux.log` | 86e32182917204e01dff10cd251562b5a1ac115e8a1d0e6a94b0323186817e2a |
| `macos.log` | 1b4a4ffe9888cd40ef80fc871a1de16381cca25c22203ade144d71335ea0129e |
| `windows.log` | e94f6f3d4250c8471819da09ea78d6414a7658a222290357060b5749b85eef8f |
| `quality.log` | d926df4b7e170700ed02087b32e5b2cd4cdf12d0286dca20f824f315960d6c96 |

## Preserved Windows failure lineage

| Source / run | Actual result and retained meaning |
| --- | --- |
| `4b6b15ba562cedcad3f37e379d01be9e9cb2e949`, [30070539790 / Windows 89410277623](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/30070539790/job/89410277623) | Original audited failure: two property tests fail. Seed 4149329052036581951 remains literal in the current named regression. |
| `5aadb6c105b98ae09443f74e44c57f8ce7eae19d`, [34777296141 / Windows 103777632638](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34777296141/job/103777632638) | Later Windows failure: 0 pass/3 fail. Seed 190752942043209832 remains literal in the second named regression. |
| `f4ef73930a62aa884b820ce91ffe35a4608b2b8b`, [34799215304 / Windows 103838288703](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34799215304/job/103838288703) | Valid 9/10-row state reaches an 8,589,934,588-byte scratch reservation failure at operation 14. Temporary expected-panic success was not successful-append proof. |
| `e58e028a31ffe845c5465e8d72dd5d933c9d80f5`, [34800684869](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34800684869) | Sizing-only repair passes three default jobs; quality fails independent performance CLI fixtures. See 14-CI-PERFORMANCE-REPAIR.md. This does not certify later source. |

Historical raw paths and digests remain in 14-DIAGNOSIS.md and 14-02-SUMMARY.md.

## Preserved Plan 04 corrective attempts

Each directory below retains its own raw logs, actual job/run metadata and digest inventory. No failed or cancelled record was overwritten, relabeled or reused as final success.

| Candidate / run | Terminal result and correction |
| --- | --- |
| `ce37c5632507df2be9baf61305967a57a68567d4`, [34808501136](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34808501136) | Three platform jobs pass; quality 103864994314 fails phase13_1_gap_verification (64 pass/2 fail): stale report prose and missing cargo-fuzz. Correction fc3af47 supplies existing pinned prerequisites and checks structured sole deferral. |
| `fc3af47291750af418befe98ad437dc40aa87bb3`, [34809888461](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34809888461) | Three platforms pass; all 66 gap tests and real fuzz build pass; quality 103868959038 then fails a live-history test on an unfetched commit (27 pass/1 fail). A local full-history probe exposes the real deferred current-HEAD Closure mismatch. Correction 430e0ad uses controlled Git fixtures without weakening production validation; later native-input couplings were found statically and fixed with meaningful material fixtures before remote execution. |
| `430e0ad775d64322b04ae04504d5a6275a197dc3`, [34812688006](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34812688006) | Three platforms and full workspace/isolation/corpus steps pass. Quality 103876986960 is cancelled at its 30-minute limit during duplicate private-suite execution, not an assertion failure. Explicit timeout annotations and raw logs are retained. Correction 417d38d makes the original workspace test headless and removes only redundant four-package commands; all test targets, explicit testbed/focused gates and timeout remain. |
| `417d38dac6951226fb32ea99a97135defe88da7b`, [34815192937](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/34815192937) | Final complete success: all four jobs and all required steps pass. |

Retention pattern: `target/phase14-platform/<full-source-sha>/attempt-20260914-01/`. Quality raw digests for the three superseded attempts are respectively `ab7947a0550f025ba77c98afa1aca9793b6763e9ab2f432973317b512f9478e0`, `47218e901505759c863d0f75f0e7cabd5c1e75bea15830160d330bab8e2aabdf`, and `c2a8a363c5238f7e3f71f3c45ae5e35e3ca28a651c4f021a753987bd93dcf8ff`.

Detailed cause, scope and verification are in 14-CI-PREREQUISITE-REPAIR.md, 14-CI-ACCEPTANCE-FIXTURE-REPAIR.md, 14-CI-MATERIALS-REPAIR.md and 14-CI-SCHEDULING-REPAIR.md. No canonical receipt, production validator body or native evidence workflow was changed by these test/CI corrections.

## Local pre-publication verification

- `local-attempt-20260914-ci01/`: actionlint, inventory six, exact filters one each, focused 28 and ordered all-feature core 1,012 pass.
- The coordinator separately gated fc3af47; its note records the targeted deferral test, workspace Clippy and ordered core gate.
- `local-attempt-20260914-isolation01/gate-02.log`: native checkout and existing repository target artifacts hidden; fmt, workspace Clippy, core build/core 1,012 pass; acceptance 31, evidence 24 and Phase 9 provenance two pass.
- Later targets also pass in `later-suites*.log`: promotion 18, Phase 9 evidence 20, platform six, provenance 19, regression eight, attestation six, release 24, safety 14, upstream 17. Earlier logs retain local missing zip/jq and noexec-tmpfs setup failures. Real tools in disposable containers and an executable empty target overlay resolved those environment issues without source workarounds.
- `local-attempt-20260914-ci-scheduling01/`: actionlint, package_cli 22 and ordered core 1,012 pass; independent review checks equivalent coverage.

These local paths are beneath `target/phase14-platform/`. Base execution used Linux ARM64 image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, Rust 1.97.0, read-only rustup and separate Cargo target/registry volumes. Every source commit passed scoped diff checks and the managed checker; final scan covers 939 files, zero findings. Touched Rust files remain below 629 lines. This executor's plan commits changed only parser-owned GSD Markdown; coordinator-owned task-ledger edits are checked separately.

## Documentation and authority boundary

This evidence and subsequent SUMMARY/state/review commits are documentation after tested source 417d38d. They do not silently rebind results to a newer HEAD, and later documentation-triggered CI is not required to substantiate this recorded source. Phase 15 still owns current-head canonical acceptance, release-candidate aggregation and attestation. No package release or canonical promotion occurred.
