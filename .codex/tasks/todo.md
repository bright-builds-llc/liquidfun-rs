# Codex Tasks

## task-ci-output-budget-and-provenance-history | 2026-07-10 10:59 CDT | Fix CI output budget and provenance history

- [x] Reproduce and trace the supervisor output-budget misclassification.
- [x] Replace per-byte stderr retention with bounded chunk-based first/last retention.
- [x] Ensure an observed output overflow wins over timeout without masking true timeouts.
- [x] Add focused regression tests.
- [x] Fetch full history in Oracle jobs that run provenance validation.
- [x] Run targeted Rust and provenance/workflow verification.
- [x] Review the diff and record residual risk.

Completion review: The supervisor now drains over-limit stderr with deadline headroom, preserves bounded first/last evidence and sanitizer detection, and classifies already-observed overflow ahead of timeout. Every Oracle checkout fetches the history required by provenance validation. Full workspace formatting, lint, build, tests, provenance validation, workflow lint, and diff checks pass. A remote Actions rerun remains pending because no commit or push was requested or performed.

## task-oracle-clang22-debug-build | 2026-07-10 11:41 CDT | Fix canonical Clang 22 oracle build

- [x] Confirm the Clang 22 failure and stdout-only diagnostic loss in current code.
- [x] Apply legacy warning compatibility options only to upstream Box2D for all Clang-family presets.
- [x] Retain stdout and stderr from failed CMake/Ninja processes.
- [x] Add focused regression coverage for stdout-only process failures.
- [x] Run targeted Rust and local oracle-debug verification.
- [x] Reproduce the oracle build in isolated Ubuntu 24.04 with Clang 22.1.8.
- [x] Run full required Rust checks, review the diff, and record residual risk.

Completion review: Canonical Ubuntu 24.04 with CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8 now builds `liquidfun-reference`; the legacy diagnostics remain visible as warnings only on upstream `Box2D`, while repository-authored C++ retains strict `-Werror`. Xtask now retains labeled stdout-only compiler diagnostics, covered by a command-level regression test. Targeted tests, clean local oracle configure/build, exact container verification, full workspace format/lint/build/tests, provenance, inventory, differential compare/replay, Markdown formatting, and diff checks pass. Remote Actions confirmation remains pending because this delegated debug task did not commit or push.

## task-04-05-pinned-probes-identity | 2026-07-11 01:03 CDT | Execute Phase 4 Plan 05

- [x] Implement bit-faithful external C++ math-probe dispatch and exact exceptional-bit transport.
- [x] Add complete strict Rust/C++ build identity and D1/D2/D3 evidence classification.
- [x] Enforce required and forbidden floating-point/compiler flags with D1 fail-closed behavior.
- [x] Run every focused C++/Rust/oracle check and the exact full Rust gate before each atomic task commit.
- [x] Run final debug/release probe comparisons, inspect effective flags/handshake, and verify Cargo-only isolation.
- [x] Create the lifecycle-bound 04-05 summary, update GSD state/roadmap/requirements, and commit metadata.

Completion review: The external C++ adapter now executes the complete 39-case Phase 4 math-probe corpus with exact `uint32_t`/`float` bit transport, bounded closed dispatch, unknown-operation rejection, and reset/reuse proof. Rust and C++ carry the same strict 17-field floating build identity, with D1 fail-closed required/forbidden flag enforcement and non-promotable D2/D3 evidence. Debug and release probes, C++ protocol tests, focused identity tests, effective compile-command and handshake inspection, Cargo package isolation, upstream verification, and the exact ordered full Rust gate all pass. The local Apple Clang 21 compiler lacks `-fdenormal-fp-math-fp32=ieee`, so it is correctly recorded as D2; canonical Clang 22 D1 requirements remain unchanged and fail closed.

## task-04-06-verification-entrypoints | 2026-07-11 01:48 CDT | Execute Phase 4 Plan 06

- [x] Add closed math-probe compare, replay, and fixed two-run D0 commands.
- [x] Compare typed native/C++ results under the reviewed Phase 4 field-policy registry.
- [x] Regenerate deterministic closed scenario and trace schema presentations.
- [x] Add transparent just recipes and supported/canonical CI coverage.
- [x] Run focused schema, fixture, CLI, debug/release/replay/D0, and workflow checks.
- [x] Run the exact ordered full Rust gate and review the task diff before committing.
- [x] Create the lifecycle-bound summary and update GSD progress artifacts.

Completion review: All 39 ordered probe cases compare successfully in debug and release, replay passes, and two independent debug processes produce byte-identical D0 output. The CLI rejects arbitrary paths, executables, compiler flags, profiles, presets, and run counts before effects; schemas remain byte-stable; canonical CI retains exact tools, SHA-pinned actions, read-only permissions, and a final evidence diff assertion. The local Apple Clang evidence remains non-promotable D2, while pinned Linux Clang 22 owns D1 evidence.

## task-04-07-numerical-policy-signoff | 2026-07-11 01:58 CDT | Execute Phase 4 Plan 07

- [x] Document the public math/settings contract and safe-Rust differences.
- [x] Document exact transport, all float policies, special-value rules, collection semantics, horizons, and D0-D3 authority.
- [x] Publish exact debug, release, replay, and two-run D0 commands with prerequisites and evidence limits.
- [x] Update only the three Phase 4 math/settings compatibility rows from executable evidence.
- [x] Extend documentation contracts for required claims, cross-file counts, and accidental local paths.
- [x] Run docs, inventory, provenance, rustdoc, package, oracle, overclaim, diff, and exact ordered Rust gates.
- [x] Create the lifecycle-bound summary and update GSD progress artifacts.

Completion review: The Phase 4 public math, numerical-policy, command, and evidence contracts are now precise and machine-audited. All 39 ordered math probes pass in debug and release, replay is successful, and two independent debug processes are byte-identical. Compatibility status remains conservative: three rows are implemented and unit-tested, only b2Math and the common subsystem carry scoped D2 differential evidence, and zero rows are platform validated. Canonical D1, settings differential parity, shapes, collision, solvers, particles, performance, and production maturity remain pending.

## task-fix-main-ci-4ee1b282 | 2026-07-13 12:50 CDT | Fix Cargo and Oracle CI on main

- [x] Confirm the failing Cargo CI test and canonical Oracle configure root causes.
- [x] Apply focused fixes with regression coverage.
- [x] Run the required Rust pre-commit checks in order and affected oracle verification.
- [x] Review the complete diff and record residual risk.

Completion review: The rigid-promotion regression now uses an explicit D2 identity instead of host-dependent build metadata, and the canonical oracle requires only Clang's supported general IEEE denormal option. The exact Rust format, lint, build, and test sequence passed; workspace-wide CI-equivalent lint, build, and tests passed; local oracle configure/build, provenance, inventory, and diff checks passed. Exact Ubuntu Clang 22.1.8 reproduction confirmed the removed `fp32` spelling was the sole failed capability probe. Residual risk is limited to end-to-end confirmation on the GitHub-hosted canonical runner after push.

Follow-up review: The first pushes proved the original configure and differential-library fixes, then exposed assumptions in later steps. The xtask CLI suite's second host-dependent promotion test was removed because the explicit D2 regression already covers that library behavior. The Oracle option set now expresses precise behavior through non-overlapping explicit controls, avoiding Clang 22's `-Woverriding-option` failure while keeping contraction disabled and IEEE special-value behavior. A release-only warning caused by assertion bookkeeping in the read-only 2014 upstream tree remains visible but no longer fails that upstream target; repository-authored C++ keeps strict `-Werror`. The docs parity test now uses a ledger-backed report check that works in Cargo-only checkouts, while full inventory validation still requires and rescans the pinned source tree. Exact Ubuntu Clang 22.1.8 probes, local debug/release Oracle builds, and all 39 math probes pass. Remote end-to-end confirmation remains pending for the final corrective push.

## task-phase-08-joints-rope-rigid-signoff | 2026-07-13 16:26 CDT | Execute Phase 8

- [x] Capture lifecycle-bound yolo context for joints, rope, callbacks, diagnostics, and rigid sign-off.
- [x] Research and create executable Phase 8 plans with verification coverage.
- [x] Execute every Phase 8 plan and record atomic summaries.
- [x] Complete code review, gap repair, and phase verification.
- [x] Run the required ordered Rust gate and affected repository verification.
- [x] Review the complete diff, record residual risks, and push only after lifecycle validation passes.

Completion review: All 24 plans are committed with lifecycle-bound summaries. Review fixes made ordinary hook-limit failures transactional and every accepted joint mutation observable, including live C++ gear-ratio evidence. Exact workflow-dispatch run `29383445374` passed canonical Linux, fail-fast sanitizer/reset Linux, macOS, and Windows at final reviewed code commit `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`; both exact artifacts and all 33 platform-validation ledger rows bind that evidence. Automatic Cargo CI run `29382964877` and Oracle CI run `29382964854` also passed at the same head. Independent code review is clean, Phase 8 verification passes 75/75 must-haves, lifecycle validation passes, and the ordered Rust, inventory, documentation, Markdown, and diff gates pass. Residual scope remains explicit: hook-owned external side effects are outside world rollback, transactional snapshots add unbenchmarked cost, and RIGD-10, particles, D3/cross-platform numerical parity, performance, testbed, and release readiness remain pending future phases.

## task-09-27-portable-evidence-validator | 2026-07-17 19:20 CDT | Execute Phase 9 Plan 27

- [x] Bind every generated case to retained Phase 8 comparison, exact policy digests, semantic witnesses, and persisted payload hashes.
- [x] Make Phase 9 evidence generation identity-last and fail closed before identity creation.
- [x] Add one typed bounded local/exact-ref validator with archive, metadata, substitution, and corruption rejection coverage.
- [x] Run focused tests plus canonical debug/release and fail-fast sanitizer evidence production.
- [x] Run the exact ordered Rust pre-commit gate, provenance, upstream read-only, and diff verification.
- [x] Create the lifecycle-bound summary and update GSD state, roadmap, and requirements.

Completion review: All seven evidence cases now bind the retained Phase 8 rigid comparison, exact Phase 6/7/8 policy digests, 58 typed semantic witnesses, persisted payload hashes, and the complete semantic manifest. One bounded xtask validator closes local, pre-identity, and exact-ref evidence, including approved run/job/artifact/live metadata and archive safety, while explicitly denying both historical runs. Fresh debug/release canonical and fail-fast ASan/UBSan corpora passed; local canonical/sanitizer evidence validated; the exact ordered Rust gate, provenance, upstream read-only, and diff checks passed. No publication, remote dispatch, compatibility promotion, or upstream mutation occurred. Residual work is limited to later exact-ref publication and promotion plans.

## task-09-28-sanitizer-fixture-recovery | 2026-07-17 20:36 CDT | Repair Phase 9 sanitizer evidence

- [x] Replace the ambient debug compile-database dependency with a hermetic fake-root database.
- [x] Prove the retained process runner works without a workspace oracle-debug database.
- [x] Record run 29625083184 as rejected authority in incomplete Plans 09-28 and 09-29.
- [x] Run the complete Rust, canonical, sanitizer, evidence, provenance, policy, workflow, Markdown, schema, and diff gates.
- [x] Commit and push one verified recovery SHA; record the user's approval and one-time Phase 09 exception.
- [x] Publish the plan-only exception SHA, dispatch exactly once under that scoped authority, and validate the paired exact-ref artifacts.

Completion review: The hermetic retained-process fixture and scoped Phase 09 exception were published at clean remote SHA `22b31c0e1be8896df622b1decd58ba2853a60b04`. `oracle.yml` was dispatched exactly once for that SHA as run `29652578231`; its canonical and fail-fast sanitizer jobs both succeeded. The two exact Phase 9 artifacts passed safe-archive preflight and the reusable exact-ref validator with runs `29439515367`, `29583793056`, and `29625083184` denied. The fresh evidence proves seven cases, 58 semantic bindings, 22 policies per case, retained-rigid equality, and canonical/sanitizer manifest agreement. No compatibility promotion or ledger mutation occurred; Plan 09-29 remains pending.

## task-09-29-replacement-evidence-promotion | 2026-07-18 12:23 CDT | Promote replacement Phase 9 evidence

- [x] Independently re-query, redownload, and exact-ref validate replacement run 29652578231.
- [x] Promote only the four supported Phase 9 compatibility rows.
- [x] Reject all historical, failed, mixed-run, incomplete, and Phase 10 authority.
- [x] Generate the compatibility report twice and prove byte identity.
- [x] Run the full Rust, oracle, sanitizer, evidence, policy, schema, documentation, diff, and ASVS L1 matrix.
- [x] Commit the implementation and create the lifecycle-bound plan summary.

Completion review: Fresh canonical and sanitizer archives from run `29652578231` independently proved seven cases, exactly 58 semantic bindings, retained Phase 8 comparison, complete policies, and matching semantic manifests. Only the particle public API, particle-system public API, storage/lifecycle, and contacts/coupling rows cite the replacement authority. Inventory regressions reject runs `29439515367`, `29583793056`, and `29625083184`, failed artifact `8423580554`, mixed artifacts, incomplete semantics, and every deferred Phase 10 behavior. The complete closure matrix and ASVS L1 review passed with no high-severity finding. Residual scope is explicit: Phase 10 groups, topology, pairs, triads, source areas, solver behaviors, and cross-engine stable-ID rotation remain not evidenced.

## task-phase-10-groups-solvers-signoff | 2026-07-19 00:19 CDT | Execute Phase 10

- [x] Capture lifecycle-bound yolo context for particle groups, topology, solver passes, and compatibility sign-off.
- [x] Research and create executable Phase 10 plans with verification coverage.
- [x] Execute every Phase 10 plan and record atomic summaries.
- [x] Complete code review, gap repair, and phase verification.
- [x] Run the required ordered Rust gate and affected repository verification.
- [x] Review the complete diff, record residual risks, and push only after lifecycle validation passes.

Completion review: All 32 Phase 10 plans are complete. Exact-reference authority run `29832646127` proves five cases and 80 semantic leaves across the canonical and sanitizer jobs, and the five scoped compatibility rows are promoted without expanding later-phase claims. Two repair cycles fixed all 10 code-review warnings; the third exact-scope review is clean. Protocol, ownership, property, corpus, oracle, comparator, evidence, provenance, inventory, dependency, schema-drift, workflow, Markdown, Rust, and lifecycle gates pass. Remaining work is intentionally deferred: Phase 11 owns examples, shared headless tooling, and the optional testbed; Phase 12 owns performance, broader platforms, packaging, safety/release documentation, and zero-gap v1 readiness.

## task-phase-12-replace-macroquad-renderer | 2026-07-22 12:00 CDT | Replace the advisory-bound diagnostic renderer

- [x] Replace Macroquad before any release-readiness claim.
- [x] Remove the `RUSTSEC-2025-0035` and `RUSTSEC-2026-0192` advisory ignores.
- [x] Preserve the passive renderer boundary and headless catalog/controller tests.
- [x] Prove the published `liquidfun` package remains free of renderer dependencies.
- [x] Run dependency, package-isolation, headless testbed, and applicable local release gates.

Original context: Phase 11 permitted the affected Macroquad graph only in the private, non-default, unpublished diagnostic testbed because neither advisory had a safe upgrade. Phase 12 has removed that bounded waiver by replacing the renderer. This renderer closure does not confer release readiness.

Completion review: The private testbed now uses the reviewed `eframe`/`egui`/`tiny-skia` replacement, `deny.toml` carries no advisory waiver, and the passive renderer has no simulation authority. `package_cli::advisory_policy_has_no_waiver_after_renderer_replacement`, the testbed capability and renderer-contract suites, package isolation, dependency policy, and the ordered full Rust gate pass. Release readiness remains separately blocked on the missing full-SHA `release-candidate` workflow bundle and accepted frozen-source attestation records.

## task-phase13-1-plan21-recovery | 2026-09-13 | Complete candidate-bound structural verification

- [x] Reproduce and diagnose supervisor startup failures at the producer build boundary.
- [x] Apply focused repairs with regression coverage while preserving production limits.
- [x] Pass ordered Rust and repository gates; publish a checked clean candidate.
- [x] Run one complete coherent 72-command producer attempt and independently validate its evidence.
- [x] Record durable evidence and summary, then hand off formal verification to the orchestrator.

Authority: AGENTS.md standing autonomous iteration authorization dated 2026-09-13. Preserve every failed attempt separately; Phase 15 remains deferred.

Completion review: Candidate `d3e70780a984ba25096f288d7c603ffa8153a748` passed all 72 producer commands and independent clean-checkout validation, with canonical run `34775554238` and artifact `10323945329`. Durable evidence and the Plan 21 summary are ready for the orchestrator's formal review and lifecycle checks; phase completion and Phase 15 acceptance are not claimed here.

## task-phase-14-replan | 2026-09-13 | Replan Windows particle-group invariant repair

- [x] Load active lessons, repository guidance, phase context, and current upstream state.
- [x] Research post-refactor code paths and exact Windows failure evidence.
- [x] Replace stale Phase 14 plans while preserving locked decisions and Phase 15 boundaries.
- [x] Run independent plan checking, requirement coverage, and plan structure validation.
- [x] Run required pre-commit checks, review the scoped diff, and record planning completion; preserve the blocked test result and leave changes uncommitted.

Guidance: AGENTS.md standing authorization and Markdown exclusions, AGENTS.bright-builds.md, standards-overrides.md, and architecture, code-shape, testing, verification, local-guidance, and Rust standards inform this replan. Active lessons total 11,548 bytes and 3,850 estimated tokens; both files were read completely. The existing July 25 audit baseline has no new trigger.

Completion review: Four sequential plans (eight tasks) replace the stale three-plan set. Research accounts for existing audited input, current failing Windows CI, post-refactor source locations, and standing authorization. Independent checking passed after one targeted revision; all six requirements and D-01 through D-10 are covered. Plan structure, lifecycle, Markdown, managed standards and diff checks pass. Ordered cargo fmt, clippy and build pass; cargo test --all-features stalled for five minutes before Rust startup and was terminated (exit 101/SIGTERM). A process sample showed only \_dyld_start with a 96 KB footprint. Logs: /tmp/liquidfun-phase14-replan-tests.log and /tmp/liquidfun-phase14-replan-startup.sample.txt. No commit or push occurred because the required test gate did not pass. Phase implementation, actual repaired Windows validation and Phase 15 remain future work.

## task-phase-14-execute | 2026-09-13 | Execute Windows particle-group invariant repair

- [x] Validate four-plan lifecycle, dependencies, standing authority and remote synchronization.
- [x] Recover an executable local verification boundary and finalize pending planning artifacts.
- [x] Execute Plan 14-01 and retain actual Windows first-transition diagnosis.
- [x] Execute Plan 14-02 root-cause repair and private rollback proof.
- [x] Execute Plan 14-03 public fixed-input and property regressions.
- [x] Execute Plan 14-04 and retain same-SHA Windows/Linux/macOS D2 results.
- [x] Complete code review, regression checks, independent phase verification and publication.

Authority: AGENTS.md standing autonomous iteration authorization dated 2026-09-13. Preserve failed attempts; Phase 15 remains deferred.

Baseline verification: Ordered format, Clippy, all-target build and all-feature tests passed using local Linux ARM64 Docker with exact Rust 1.97.0, immutable image `sha256:5f3d56072c0c734ebb2381c59f0a20db1330b8884134c4e3a9d369de2d810ea8`, and the existing Rust/cache volumes. Full log: `target/phase14-local-verification/attempt-20260914-linux-container-network/gate.log`. The offline attempt lacked workspace dependency metadata and was retained separately; normal index refresh resolved it. Earlier macOS partial runs and startup-inventory attempts remain non-passing diagnostics. No security setting changed. Native macOS and Windows repaired-source proof remains required in Plan 14-04.

Wave 1 review: `a2759db` retained inputs; diagnostic `f4ef739` proved the Windows operation-14 join reservation failure at 8,589,934,588 bytes for ten particles; `49e382f` removed every probe after passing 987 tests and restored pre-probe source/CI bytes. The trace digest was independently recomputed. Plan 02 now explicitly owns scratch sizing and bounded allocation regressions. Prerequisite CI corrections: `8b4eb56` installs existing formatter plugins; `030c893` preserves all four concurrent CLI evidence cases while restoring workspace Clippy. Neither correction claims repaired Windows physics.

Wave 2 review: `e58e028` bounds scratch reservation and requires valid original replay; `88643e0` adds complete private rollback/allocation evidence; `800a51c` handles candidate-only resource failures with the existing typed boundary; `265598a` records the summary. Final local gate: 998 tests, zero managed findings. The earlier sizing candidate e58e028 passed default-feature Windows, Linux and macOS jobs in run 34800684869; raw logs, source identities and digests are retained separately. This does not label later commits platform-validated or close the full quality lane.

Verification checkpoint: Plan 03 source and public tests are prepared but uncommitted. The precise zero-rest step error preserves the prior rejection contract; the narrowed classifier and negative controls passed independent static amendment review. Latest runtime verification remains outstanding. Docker image/blob and meta.db I/O errors prevent its use. Native formatting, Clippy and build passed for the prior candidate, but test startups stalled and the partial run was stopped for the review correction. A Docker Desktop restart decision is pending because other containers would be interrupted. No reset, user-data deletion, security change, or unverified commit occurred. Full handoff: .planning/phases/14-repair-windows-particle-group-invariants/.continue-here.md.

Recovery and Wave 3 review: The user approved a Docker Desktop restart; image and metadata I/O recovered and Rust 1.97.0 was reverified. The disposable isolated diagnostic build cache was removed after its evidence was preserved. The performance CLI fixture correction passed 13 tests with empty target/native overlays and was committed separately as 82ba701. Plan 03 commits 3953157, 6761b26, 9d826aa and 13e4ef2 passed ordered gates with 1,012 tests and zero managed findings. The explicit zero-rest-only error classifier passed amendment review; other invariant failures remain unexpected. Plan 04 and final review/verification remain pending.

Plan 04 validation progress: ce37c563 and fc3af472 each passed both exact regressions, focused 10/6/12 suites and the full 957-test default package on Windows, Linux and macOS. Their quality failures remain preserved and do not satisfy final acceptance. Formatter/CLI prerequisites, pinned fuzz tooling and the structured deferral assertion are corrected. The remaining repository-history contract test couples current source closure to Phase 15 acceptance; a controlled history fixture is being prepared while production validation and canonical receipts remain unchanged. Final same-SHA quality/platform proof and independent verification remain outstanding.

Final-candidate iteration: 430e0ad passed all three platforms, complete workspace tests, consumer package isolation and corpus closure. Its quality job reached the 30-minute limit while rerunning private tooling suites; the repeated rigid-fixture suite passed 15/15 after 404.47 seconds before later supervisor tests were cancelled. A reviewed CI scheduling correction runs the full workspace suite once with the required empty display environment and retains explicit testbed, focused, package, documentation and inventory gates. The cancelled run remains non-passing evidence; a fresh candidate and final verification are still required.

Completion review: Phase 14 passes 13/13 must-haves and closes all six assigned requirements. Tested source 417d38dac6951226fb32ea99a97135defe88da7b passes Cargo CI run 34815192937: three supported platforms each run both exact regressions, focused 10/6/12 suites and 957 package tests; all 26 quality steps pass. Code review is clean across 32 files and all 16 registered security mitigations are verified. The Docker restart restored the local Rust 1.97.0 gate, which passes 1,012 tests. Failed attempts remain retained. Later completion commits contain metadata only and do not rebind CI results. Phase 15 canonical evidence and attestation remain pending; no universal whole-step rollback or release-readiness claim is added.

Final completion gate: ordered fmt, Clippy, all-target build and all-feature tests pass (1,012 tests, zero failed/ignored) in target/phase14-finalization/attempt-20260914-0727/gate.log. Managed checks report 939 files and zero findings; Markdown, lifecycle, schema-drift and scoped diff checks pass. All 32 reviewed source files remain byte-identical to tested 417d38d.

## task-phase15-candidate-evidence | 2026-09-14T16:43:10.303691+00:00 | Re-establish candidate evidence

Status: Deferred — explicit strict-qualification request only, under the owner's 2026-09-16 hobby-scope revision. Remaining strict items are not hobby-project blockers; preserve their historical check states.

- [x] Review instructions, active lessons, prior decisions and current evidence contracts.
- [x] Capture yolo context and explicit acceptance/infrastructure boundaries.
- [x] Research producer readiness and create independently checked execution plans.
- [ ] Repair and verify prerequisite/tooling defects before candidate freeze.
- [ ] Collect complete same-candidate producer evidence and release aggregation.
- [ ] Validate worktree and committed frozen-source attestation; repeat milestone audit.
- [ ] Complete review, lifecycle verification and strict commit/push finalization.

Progress: Plans 01–06 are complete with retained canonical, safety, coverage, native regression, same-package handoff, evidence-retention and explicit C/A readiness proof. CI restoration and failure controls pass. Plan 07 promotion prerequisites are repaired and independently reviewed at 84405cc. The first canonical producer passed at its original 00358a5; fresh-source production, native Windows process verification, reviewed promotion and exact-head acceptance remain.

Completion review: Pending. Controlled performance runner/identity availability requires external clarification.

## task-independent-ai-review | 2026-09-16 17:47 UTC | Permit independent AI review

- [x] Record the user-authorized review policy and supersession of human-only requirements.
- [x] Accept honestly identified AI reviewers while preserving identity and digest validation.
- [x] Run focused regressions, ordered Rust gates, Markdown and standards checks; review the diff.
- [x] Commit the policy change, regenerate the canonical packet, and obtain independent AI review before promotion.

Completion review: policy commit `8ff7315` and independently AI-reviewed promotion `7794304` are pushed. All local gates and exact-Q canonical Linux acceptance run `35131323038` passed. Other Phase 15 candidate evidence and approved performance-host access remain outstanding.

## task-phase15-deny-cli | 2026-09-16 18:09 UTC | Repair release dependency-check invocation

- [x] Correct the release script's cargo-deny global flag ordering using installed CLI evidence.
- [x] Update the private testbed webbrowser dependency to patched 1.2.2 and run the actual dependency check, focused release checks, and ordered repository gates.
- [ ] Commit the repair and qualify a fresh candidate; preserve the superseded freeze and its run records.

Failure signal: cargo-deny 0.20.2 rejects `cargo deny check --locked`; retained Plan 08 preflight has exit 2 and the argument error.

The corrected command exposed RUSTSEC-2026-0257 in webbrowser 1.2.1. The 1.2.2 patch clears the audit without policy exceptions. Cargo.lock is a recorded replay input, so fresh canonical production, independent review and promotion are required before the replacement candidate.

## task-phase15-particle-statistics-parity | 2026-09-16 19:00 UTC | Resolve Phase 11 particle mismatch

- [x] Reproduce the canonical and sanitizer checkpoint-0001 mismatch and isolate semantic fields.
- [x] Establish the pinned upstream behavior, correct the responsible implementation or capture boundary, and add a regression.
- [x] Retain existing bounded Phase 11 failure logs through diagnostic-only CI artifacts.
- [x] Correct differential-coverage jq entry iteration and verify valid/malformed payload behavior.
- [x] Run focused parity controls and ordered repository gates; independently review the fix.
- [ ] Produce fresh canonical evidence and requalify a replacement candidate; preserve original failed runs and ongoing source-bound outputs.

Failure: Phase 11 run `35137245396` failed both lanes for `particle-forces-and-statistics@1` at candidate `dc29747`. No comparison tolerance or acceptance requirement is waived.

## task-phase15-semantic-record-validation | 2026-09-16 19:39 UTC | Align exact-reference validation with canonical output

- [x] Reproduce rejection of the producer's canonical semantic-result record with retained exact-reference archives.
- [x] Validate the explicit final canonical record strictly, preserve preliminary and historical forms, and reject it in sanitizer payloads.
- [x] Verify malformed fields, missing fields, source/digest mismatches and local authority boundaries.
- [x] Independently review and run ordered repository checks.
- [ ] Commit the repair and regenerate the promotion packet at new R.

The repaired-source producer run `35140576574` passed both lanes. This repair addresses the independent validator's closed topology; it does not rewrite evidence or claim new producer execution.

## task-hobby-linux-scope | 2026-09-17 00:20 UTC | Remove mandatory Linux qualification

- [x] Record the owner’s hobby-project direction and optional Linux x64 qualification; preserve historical evidence.
- [x] Use one default macOS Cargo job and make expensive validation suites manual-only, as selected by the owner.
- [x] Capture remaining release/API reductions as discussion proposals without treating them as accepted.
- [x] Verify workflow/docs contracts and ordered Rust checks; complete independent review.
- [x] Prepare the verified scope revision for normal commit and push to main.

Completion review: Linux and dedicated hardware no longer block hobby development. Existing strict validators retain their meaning. Final ordered Rust, docs, Markdown, managed standards and workflow checks passed; 43 focused expanded workflow tests passed. Managed Bright Builds automation remains separate. Release/API policy simplifications are still proposals; no package or strict qualification release is claimed.

## task-hobby-wrapup-discuss | 2026-09-17 00:45 UTC | Capture Phase 15 hobby wrap-up decisions

- [x] Synthesize recommended release and compatibility choices from current scope and repository tools.
- [x] Update Phase 15 context and append the discussion audit trail; retain old evidence and require replanning.
- [x] Independently review, verify ordered repository checks, and commit the discussion artifacts.

Discussion review: local ordered Rust and documentation/workflow checks passed; context structure checked. Recommendations are captured for replanning only; no implementation or publication. Verification logs: `target/hobby-discuss/gates/`. Independent review acknowledgment is retained in `target/hobby-discuss-review.md`.

Routing correction: `state update Status` matched YAML before the body and regenerated the old value. An exact one-line body correction followed by CLI session normalization restored `status: planning`; the strict campaign remains deferred.

## task-plan15-hobby | 2026-09-17 00:50 UTC | Plan the hobby wrap-up

- [x] Load revised context and map explicit hobby requirements; keep strict qualification deferred.
- [x] Preserve old plan history and create bounded replacement plans with concrete checks.
- [x] Run independent plan verification and resolve findings.
- [x] Record executable handoff, verify required checks, commit and push planning artifacts.

Planning review: two sequential plans cover HOBBY-01/02/03 and D-10–D-21. Twenty-two historical artifacts were moved byte-for-byte out of active discovery. Ordered Rust and docs checks passed; Markdown/managed checks passed after staging renames. Final plan-check acknowledgment is in the phase directory. Execution and publication remain separate. The installed roadmap CLI has no goal/scope mutation command, so the owner-authorized scope replacement used a targeted section edit; subsequent progress/state changes used CLI commands.

## task-execute15-hobby | 2026-09-17 00:55 UTC | Execute hobby wrap-up

- [x] Validate current lifecycle and select only replacement plans 15-13 and 15-14.
- [x] Complete documentation and checklist reconciliation with review and checks.
- [x] Run bounded native/package verification, bind macOS CI and independent review, record limitations.
- [x] Verify phase requirements and prepare final tracking records for the normal main push.

Completion review: both replacement plans passed, HOBBY-01/02/03 verified, independent review clean and lifecycle validation passed. Checked implementation 75ead0e passed macOS Cargo CI35169132504; native/package and regression results are in 15-HOBBY-COMPLETION.md. Generated compatibility report, compiler declarations and strict validators remain unchanged. No GUI, minimum-compiler or publication claim was added. Final tracking commits contain records only.

## task-archive-hobby-milestone | 2026-09-17 01:19 UTC | Archive completed hobby milestone

- [x] Verify current-scope readiness and preserve historical strict gaps.
- [x] Archive roadmap/requirements/audit, write concise milestone history and retrospective.
- [x] Independently review and commit archives before removing active requirements.
- [x] Prepare between-milestone state and verified finalization for normal push without release tags or publication.

Archive review: 16 completed phases/252 active plans archived as planning milestone v1.0, with all three strict deferred IDs still unchecked. Original requirement/roadmap bodies preserved; old failed audit and completion STATE copied byte-for-byte. Independent staged-diff review passed. Archive safety commit 86dcd37 precedes active REQUIREMENTS removal. Phase history remains in place; no release tag, package publication or new milestone scope.

## task-new-web-milestone | 2026-09-17 01:30 UTC | Define SolidJS WASM playground milestone

- [x] Confirm six demos, own Rust WASM engine and playful gallery direction.
- [x] Research stack, example features, engine integration and delivery pitfalls.
- [x] Define testable requirements and a continued-numbering roadmap.
- [x] Prepare independently reviewed planning artifacts and required checks for commit and push.

Milestone review: owner approved six specific demos and phases 16–19; all 22 requirements mapped once. Four research dimensions and synthesis use current primary sources and inspected Rust capabilities, with browser integration and Pages settings explicitly unverified. Previous phase/research history preserved. No dependencies installed, site deployed or implementation begun.

## task-resolve-main-pull-conflicts | 2026-10-01 | Resolve conflicts and push main

- [x] Reconcile stashed scene changes with upstream geometry, controls, and tests.
- [x] Run ordered Rust format, Clippy, build, and test checks plus WASM scene regressions.
- [x] Run web typecheck, unit tests, production WASM build, Markdown and managed checks.
- [x] Review the complete diff, obtain independent review, commit, and verify the main push.

Completion review: source commit `0fa50e1` is on `origin/main`. Ordered Rust checks passed in an isolated target (1032 core tests and 284 WASM scene tests); web typecheck, 402 unit tests, 28 browser tests, production build, Markdown and managed checks passed. Independent AI reviewer `/root/independent_review` acknowledged source digest `07e45e462804d0f583437cc1bccbb15a70d552d04b0afdd915d3bfe23b6845fe` at 2026-10-01 21:07:04 UTC. Keep upstream's verified 3000-particle bubbler initializer because both 1800-particle candidates broke the newer elevator catch regression. Original stashes and failed attempt evidence remain recoverable; the old ignored build cache was not deleted. No release qualification or package publication is claimed.

## task-tesla-reference-geometry | 2026-10-01 21:34 CDT | Match the Tesla Valve reference

- [x] Replace unilateral open hairpins with alternating rounded lobes and closed teardrop islands; share render and collision geometry.
- [x] Update the thumbnail, scene description, portrait framing assertions, and README SVG/WebP previews.
- [x] Verify construction, frame limits, containment, drain behavior, and live flow controls with scene regressions.
- [x] Run Rust format, Clippy, build, and tests; WASM/web build, typecheck, unit tests, browser preview, managed checks, and Markdown checks.
- [x] Review the final diff and record completion evidence.

Guidance: AGENTS.md standing authorization and hobby scope, AGENTS.bright-builds.md, standards-overrides.md, and local architecture, code-shape, verification, testing, Rust, and frontend standards. GSD quick task: 261001-tvf.

Completion review: source commit `7d7ca9d` replaces the unilateral loops with four alternating lobes and closed solid islands. Ordered core Rust checks (1032 tests), affected WASM checks (287 tests), web typecheck and 402 unit tests, 44 browser tests, production build, managed checks, Markdown and diff checks passed. Desktop/phone previews and the regenerated ten-second SVG/WebP match the new silhouette. Independent AI reviewer `/root/tesla_review` acknowledged source digest `b9c646fe9a40f438e69b2bce8d049bc875fcb1eea25cf42f2c4212a7e8a09f47` at 2026-10-02 02:53:08 UTC. Four-stage throughput uses a six-second window with the existing 80-particle margin, earlier drain arrival and continuous containment. Local WebP text uses an explicit Arial font path while CI retains DejaVu. Ordinary main publication uses standing authorization. This records experimental simulation evidence, not a measured real-world valve performance claim.

## task-tesla-wider-necks | 2026-10-01 22:01 CDT | Increase Tesla Valve throughput

- [x] Measure current drainage at the default source rate.
- [x] Widen all stage necks with shared render/collision geometry and preserve direction/control/containment regressions.
- [x] Demonstrate higher drainage at the same rate, refresh gallery and browser preview, and run required Rust/web/Markdown/managed checks.
- [x] Record independent review, commit, and verify the main push.

Guidance: continue the GSD quick workflow (261001-uk9), AGENTS.md standing authorization and hobby scope, the Bright Builds sidecar/overrides, and loaded Rust, architecture, testing and verification standards.

Completion review: `a849c09` widens all four stage necks through one shared geometry parameter; collision and drawing remain aligned. Native/production WASM ten-second forward discharge at unchanged 180 particles/s rises from 853 to 1526 (about 79% more over that startup window), with continuous native conduit containment and preserved forward preference. Ordered core checks (1032 tests), full affected WASM checks (289 tests), 402 web unit tests, 44 browser tests, production build, managed/Markdown/diff checks and independent AI review pass. Reviewer `/root/tesla_review` acknowledged exact source digest `aca3482aa908243cde39ee43a7ac5a915864b26daaad5bc35751c5a514d79b20` at 2026-10-02 03:13:26 UTC. Gallery and existing in-app preview show the wider gaps. This is fixed-window simulation evidence, not a steady-state or physical-valve efficiency claim. Ordinary main publication uses standing authorization.

## task-tesla-smooth-flow | 2026-10-01 23:11 CDT | Smooth curves and refine particle flow

- [x] Replace angular curve samples with dense shared analytic geometry while retaining wider necks and solid splitter islands.
- [x] Reduce particle radius to about 5 mm and raise default flow to 1440 particles/s with distinct, non-overlapping spawn positions.
- [x] Raise the bounded Rust/TypeScript segment lanes together and verify dense acceptance plus overflow rejection; synchronize flow-control defaults and bounds.
- [x] Validate curvature, collision containment, controls, drainage and default/maximum-rate performance; rebuild browser and gallery previews.
- [x] Run required checks, obtain exact-diff independent review, commit and verify publication.

Plan: GSD quick task 261001-w0a. Standing authorization, current hobby scope and loaded Bright Builds architecture, Rust, testing and verification rules apply. The earlier wider-neck commit is integrated with the preview bot update on main at c5bc0a3.

Completion review: source `4881057` draws 300 smooth segments, uses solid rounded islands and sampled overlapping thick wall boxes, and reduces radius to 5 mm with 1440/s default and 2880/s maximum emission. Strict native/production-WASM four-case containment passes; failed thin-chain/cusp candidates remain retained. Ordered core checks (1032 tests), affected WASM checks (292 tests), 404 unit tests, final 44 browser tests, production build, managed/Markdown/diff checks and independent review pass. `/root/tesla_review` acknowledged exact source digest `19d2e4360323388816757c79f4d9058ae12c8bba30240c2713645f3e2c94858a` at 2026-10-02 05:17:04 UTC. Dense frame limits are matched at 512 on both sides; gallery is refreshed. Maximum reverse flow is more costly; measured local results are not a universal 60 fps guarantee. Standing authorization applies to normal main publication.

## task-tesla-circular-pipes | 2026-10-02 01:55 CDT | Circular bends and uniform channels

- [x] Verify the previous version is committed and pushed to main; integrate the compatible preview-bot refresh.
- [x] Derive a gently winding main pipe and alternating bypasses from one clear width, with concentric semicircular bends and parallel runs.
- [x] Preserve smaller particles and inlet rates; align the source with the new inlet and maintain robust thick walls and solid islands.
- [x] Prove circularity, width, tangent joins, containment and directional flow; update framing, thumbnail, browser and gallery.
- [x] Run required checks and independent review, then finalize the requested work.

Plan: GSD quick 261002-1yc. Uniformity applies to runs and bends; split/merge regions need short transitions. Starting snapshot was on main at b77b116. Repo-local standing authorization and existing verification requirements remain in force.

Completion review: source `c2b5e36` replaces oval curves with true 180-degree bypasses, concentric bends and parallel runs sharing 17 cm physical clear width. Short fork/merge regions remain wider. Five-millimeter particles and 1440/2880 rates are preserved, with a rotated 16×3 inlet grid and 420 segments. Exact native/production-WASM containment, earlier/higher forward drainage, 1032 core tests, 294 affected WASM tests, 404 web unit tests, final 44 browser tests, production build, managed/Markdown/diff checks pass. A broader browser invocation passed 53 tests but accidentally enabled an old opt-in forensic profile without its prerequisite; its failure remains preserved. Independent AI `/root/tesla_review` acknowledged source digest `fee38086a717bf52c1b7380e8a3ac8b7b63c230f19989e8cb0a2c5aa271995fd` at 2026-10-02 07:41:37 UTC. Phone/desktop framing, thumbnail and gallery are refreshed. Reverse buildup is slower; runtime samples do not promise 60 fps or physical engineering performance. Normal main publication uses standing authorization.

Publication follow-up for task-tesla-circular-pipes: source/tracking `c2b5e36` / `b8a2804` pushed successfully to main and the preview was refreshed. A repeat build exposed an unchanged encoder event-race test (403/404); the failed log is retained. Its isolated eight tests and a fresh complete 404-test production build passed. This is a recorded retry, not an encoder fix; geometry source and its exact review digest are unchanged.

## task-tesla-six-centimeter-pipe | 2026-10-02 15:28 UTC | Set Tesla Valve clear width to 6 cm

- [x] Reduce the shared physical clear width to 0.06 m, retaining circular bends and the existing centerline layout.
- [x] Fit all source slots inside the narrower inlet while preserving particle radius, selected rates and speed.
- [x] Verify geometry, source placement, strict containment, drainage and directional behavior; refresh the thumbnail, gallery and live preview.
- [x] Run required checks, obtain independent exact-diff review, and finalize normal main publication.

Plan: GSD quick 261002-ejw. The shared width governs runs and bends; fork/merge junctions retain their short transitions. Starting main is 5a3dc13 after integrating the compatible preview-bot asset refresh. Loaded local guidance, Bright Builds sidecar/overrides, geometry/verification/language standards and active lessons apply. Preserve the 5 mm particle radius and 1440/2880 source settings.

Completion review: source `46eecd2` sets 6 cm clear channels through shared offsets, keeps circular radii and existing physics/rates, and fits 48 emission slots in a 4×12 inlet grid. Twelve focused geometry/emitter checks, 1032 core tests including docs, all 296 release-mode WASM-package tests, four-case exact production-WASM containment, 404 web unit tests, final 44 browser tests, builds, managed/Markdown/diff checks pass. The intermediate manual-UI-build provenance failure is retained; the repository-owned rebuild and complete browser rerun pass. Independent AI `/root/tesla_review` acknowledged exact source digest `942c84c5abbc973c5a4ede12f2c31a20f8b7b7802c2997775785ed62a5f3a16f` at 2026-10-02 15:46:50 UTC. Preview/copy/gallery reflect 6 cm; branch junctions remain short merged transitions, and fixed-window experimental evidence does not promise indefinite containment or a universal frame rate. Normal main publication uses standing authority.

## task-tesla-continuing-stems | 2026-10-02 15:58 UTC | Make loop returns become the main pipe

- [x] Derive and check a continuous return-wall layout matching the owner's sketch, retaining circular bends and 6 cm clear channels.
- [x] Replace branch rejoin stubs with actual main-pipe continuation; align source/outlet and preserve robust shared collision/render geometry.
- [x] Verify tangent/line continuity, widths, solid islands, frame bounds, strict flow containment and existing directional behavior; refresh previews.
- [x] Run required checks, obtain independent review and publish the verified iteration normally to main.

Plan: GSD quick 261002-f8p. Current baseline is f010016. Existing local guidance, Bright Builds sidecar/overrides, relevant managed standards and experimental scope apply. The loop returning wall must define the next trunk section, not merely hide its old junction in the drawing. Preserve 6 cm clear width, 5 mm particles and 1440/2880 source rates.

Completion review: source `3de2929` makes all four loop return walls actual continuous next-main boundaries, including the outlet leg. Six-centimeter channels, semicircular loops and particle/rate settings remain. Fifteen focused geometry/source/topology tests, 1032 core tests including docs, all 299 release WASM-package tests, strict four-case production containment, 404 web units, final 53 browser passes (one existing optional skip), builds and managed/Markdown/diff checks pass. Two retained broad resize failures required a bounded convergence wait with unchanged final viewport assertions. Symmetric vertical ports, coordinated drain/frame/phone clearances and actual-boundary thumbnail/gallery match the supplied sketch. Independent AI `/root/tesla_review` acknowledged source digest `029d8f1f54dfc142c4fae6df4578a97562aaa55da94d1346c5a5903352c5b759` at 2026-10-02 17:00:19 UTC. Reverse buildup is more costly; these fixed-window experiments do not certify real fluid-diode efficiency, indefinite pressure containment or guaranteed frame rate. Standing authority covers ordinary main publication.

## task-tesla-performance | 2026-10-02 18:31 UTC | Optimize physics and live playback with staged evidence

- [x] Establish and commit a repeatable original benchmark with immutable raw reports, source/runtime/hardware identity and semantic checkpoints.
- [x] Spatially filter wall contacts; preserve ordering and collision behavior; benchmark and commit Stage 1.
- [x] Share immutable collision geometry and reuse transaction-safe buffers; benchmark and commit Stage 2.
- [x] Remove unnecessary neighbour-pair construction from drain queries; benchmark and commit Stage 3.
- [x] Move Tesla live playback into an ordered, bounded worker lifecycle; retain direct export/testing paths; benchmark and commit Stage 4.
- [ ] Cache/batch rendering with correct invalidation and unchanged particle/geometry presentation; benchmark and commit Stage 5.
- [ ] Complete native/WASM/web/browser and four-case flow checks, independent review, historical comparison, normal main publication and refreshed live preview.

Plan: GSD quick coordinator 261002-irz and bounded serial execution prompts. Starting source is 8c419c4, clean and synced. Preserve 6 cm geometry, 5 mm particles, four particle iterations, fixed 1/60 timestep, 1440/2880 source settings and the existing inlet policy. Primary timed workloads are default-rate forward/reverse at fixed simulated checkpoints; extended maximum-rate timing is optional and must remain a separate comparable configuration. All four rate/direction quality cases remain required. Reports live under docs/benchmarks/tesla-valve; no optimization may start before the original baseline is persisted. Core and frontend implementations are gated sequentially for attribution. Current hobby scope, standing authority, loaded Bright Builds guidance and active lessons apply.
