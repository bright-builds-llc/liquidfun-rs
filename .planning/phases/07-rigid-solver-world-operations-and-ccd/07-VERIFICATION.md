---
phase: 07-rigid-solver-world-operations-and-ccd
verified: 2026-07-13T17:15:45Z
status: passed
score: "44/44 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 7-2026-07-12T23-36-17
generated_at: 2026-07-13T17:15:45Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - id: RIGD-03
    status: verified
  - id: RIGD-05
    status: verified
  - id: RIGD-06
    status: verified
  - id: RIGD-07
    status: verified
  - id: RIGD-08
    status: verified
  - id: RIGD-09
    status: verified
must_haves:
  roadmap_success_criteria: 5/5
  plan_truths: 39/39
  plan_artifacts: 32/32
  plan_key_links: 26/26
  repository_completion_gates: 1/1
evidence:
  verified_commit: d939bebe5dfc9cbc8a05854a45695927dd64a082
  mandatory_rust_sequence: passed
  full_workspace_gate: passed
  focused_native_protocol_and_differential_suites: passed
  cpp_protocol_debug_fresh_build: passed_d2
  rigid_debug_compare: passed_9_of_9_d2
  rigid_debug_replay: passed_9_of_9_d2
  rigid_determinism: passed_two_runs_d0
  rigid_sanitizer_protocol: passed_d2
  rigid_sanitizer_compare: passed_9_of_9_d2
  repository_checks: passed
  code_review: clean
---

# Phase 7: Rigid Solver, World Operations, and CCD Verification Report

**Phase goal:** Complete scalar rigid-body stepping and the world operations needed for broad rigid compatibility.

**Status:** `passed`

**Score:** 44/44 observable must-haves verified: five roadmap success criteria plus 39 plan truths.

## Verdict

Phase 7 achieves its bounded goal. The native Rust world exposes checked body controls and step configuration, source-ordered scalar islands, transactional contact solving and sleeping, AABB queries, typed ray casts, atomic origin shifting, and bounded resumable CCD/TOI. The closed rigid-world protocol, independent Rust and pinned-C++ adapters, comparator, replay, minimization, evidence authority, documentation, and compatibility ledger exercise the same behavior without putting C++ in the published crate.

All six mapped requirements are verified. All 32 artifact declarations (30 unique files) exist, are substantive, and are wired. All 26 declared links carry real data or control flow. Review iteration 7 is clean across its 79-file scope with zero critical, warning, or informational findings after WR-21 through WR-23 were fixed. No implementation gap or human-only verification item remains.

This verification applied `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the local architecture, code-shape, testing, verification, and Rust standards. The material rules were validate-before-commit transitions, source-order determinism, semantic rather than memory evidence, transactional failure behavior, closed protocol boundaries, exact ordered Rust verification, and conservative evidence authority. No override was needed.

## Goal Achievement

### Roadmap success criteria

| # | Observable truth | Status | Independent evidence |
| ---: | --- | --- | --- |
| 1 | Forces, torques, impulses, damping, gravity scale, fixed rotation, bullet mode, and velocity changes produce the supported upstream-equivalent state transitions. | VERIFIED | Public `World` controls route checked inputs through candidate `BodyState` values and one commit. All 6 `rigid_body_controls` tests and the `body_control_and_force_policy` Rust/C++ witness pass, including derived-overflow no-effect behavior and `WakePolicy` branches. |
| 2 | Island construction and velocity/position constraints preserve pinned phase order, warm starting, and scalar behavior. | VERIFIED | `build_islands` uses the explicit newest-first body lane, manager contact order, and LIFO traversal. `solve_islands` stages complete solutions before `commit_world_step_candidate`. The 7 island-order, 11 island-solver, 8 contact-solver, and multi-contact differential witnesses pass. |
| 3 | Sleeping/waking, activation, CCD, bullet handling, sub-stepping, and TOI prevent tunneling and reproduce supported outcomes. | VERIFIED | The 12 sleeping, 3 CCD-selection, and 8 CCD tests pass. Checked TOI selection feeds bounded source-ordered TOI islands; accepted events commit atomically, do not pollute discrete warm-start caches, and resume after `ContinuousPending` or a coherent work limit without repeating the discrete solve. |
| 4 | Consumers can configure stepping and force clearing, shift origin, query fixtures, and ray-cast with documented clipping, termination, filtering, and unspecified order. | VERIFIED | The 13 world-configuration, 15 world-query, and 4 origin-shift tests pass. Public APIs are documented, immutable during visitor traversal, preserve child multiplicity, validate ray clips, and stage all coordinate lanes before origin-shift commit. |
| 5 | Required rigid scenario families accumulate first-divergence differential evidence for later sign-off. | VERIFIED | Fresh compare and replay match all nine families: non-colliding lifecycle, single-contact lifecycle, body controls, multi-contact islands, sleeping, continuous/sub-stepping, continuous resume, query/ray, and origin-shift covariance. Omitted observations and identity/multiplicity mismatches fail closed. |

**Roadmap score:** 5/5

### Requirement accounting

| Requirement | Status | Evidence boundary |
| --- | --- | --- |
| RIGD-03 | VERIFIED | Checked force, torque, linear/angular impulse, damping, gravity-scale, fixed-rotation, bullet, sleep, and velocity APIs; atomic invalid/overflow behavior; differential body-control witness. |
| RIGD-05 | VERIFIED | Newest-first bounded DFS islands, manager/manifold order, scalar velocity and position passes, warm-start policy, and all-island transactional commit. |
| RIGD-06 | VERIFIED | Pinned thresholds and inclusive sleep duration, whole-island convergence, allowed-sleep behavior, deterministic mutation/contact wake sources, and rollback with a late island failure. |
| RIGD-07 | VERIFIED | Eligible-contact scan order, strict-less earliest TOI selection, rollback of rejected candidates, source-ordered TOI islands, anti-tunneling, sub-step resume, bounded work, and semantic partial-progress classification. |
| RIGD-08 | VERIFIED | Checked gravity, warm starting, continuous physics, sub-stepping, automatic/manual force clearing, timestep/iteration configuration, prior inverse-timestep state, and atomic origin shifting. |
| RIGD-09 | VERIFIED | Borrow-scoped AABB and ray visitors, semantic fixture/child identity, multiplicity, application-owned filtering, continue/ignore/terminate/clip controls, exact shape narrowing, and explicitly unspecified order. |

`RIGD-11` remains assigned to Phase 8's broader rigid differential sign-off. Canonical D1 and wider-platform D3 evidence are also outside this local Phase 7 verification; those boundaries do not subtract from the six Phase 7 requirements.

## Plan Must-Have Accounting

All 39 truths declared by Plans 07-01 through 07-13 are verified.

| Plan | Truths | Verification result |
| --- | ---: | --- |
| 07-01 | 3/3 | Complete granular body controls, typed wake policy, no-effect invalid/overflow paths, and pinned static/kinematic/preserve-sleep branches. |
| 07-02 | 3/3 | Checked world flags and step inputs, retained inverse timestep, default automatic force clearing, and explicit application-managed clearing. |
| 07-03 | 3/3 | Newest-first awake seed order, LIFO DFS, reusable static boundaries, and preflighted bounded topology. |
| 07-04 | 3/3 | Pinned scalar solve stages, warm-start disable semantics, and one all-island body/impulse/proxy transaction. |
| 07-05 | 3/3 | Whole-island sleep timing, exact wake sources without activation-only waking, and sleep state in the solver transaction. |
| 07-06 | 3/3 | Borrow-scoped AABB/ray visitors, no public proxy identity, preserved child multiplicity, application filtering, and checked clipping/error behavior. |
| 07-07 | 3/3 | Atomic translation of transforms, sweeps, proxies, and tree AABBs; stable identity/topology/move buffer; query/ray covariance. |
| 07-08 | 3/3 | Manager-ordered strict-less CCD selection, private bounded cache/resume state, and full rollback of rejected tentative contacts. |
| 07-09 | 3/3 | Bounded TOI islands without warm-start pollution, one-event sub-step pending, and coherent semantic work-budget checkpoints. |
| 07-10 | 3/3 | One closed bounded exact-bit protocol extension, public semantic CCD results only, and explicit policy registration without fallback. |
| 07-11 | 3/3 | Symmetric validated Rust/C++ execution, stable first-divergence detail, and bounded adapter/comparator output for replay and minimization. |
| 07-12 | 3/3 | Locked nine-family corpus, reproducible local D2 workflows, and no-write D2 rejection of D1 stage/review/promotion operations. |
| 07-13 | 3/3 | Truthful public/contributor contracts, machine-ledger authority with deterministic Markdown generation, and the complete repository gate. |

GSD phase completeness reports 13 plans, 13 summaries, no incomplete plans, no orphan summaries, and no errors or warnings. All 26 task commits named by the summaries resolve in repository history.

## Artifact and Wiring Verification

The plans declare 32 artifact occurrences covering 30 unique non-empty files. The GSD helper verifies all 21 structured declarations in Plans 07-07 through 07-13. Plans 07-01 through 07-06 use scalar path declarations, so their 11 declarations were checked directly for existence and substance. The files contain real implementations, behavior tests, closed schemas/policies, adapters, fixture lifecycle logic, and generated/document authority rather than placeholders.

| Link group | Result | Data/control-flow evidence |
| --- | --- | --- |
| Body controls and step configuration | WIRED | Public `World` methods resolve typed handles, build complete candidate body/timing state, then commit; `StepConfiguration::timing` supplies the solver ratio and `finish_successful_step` applies force policy. |
| Islands, constraints, and sleeping | WIRED | Explicit `body_order` feeds `build_islands`; island body/contact indices feed scalar constraints; solved motion, sleep state, impulses, proxy synchronizations, and timing enter one `WorldStepCandidate`. |
| Queries and origin shifting | WIRED | World visitors wrap broad-phase traversal and exact `Shape::ray_cast`; origin preparation collects every body/proxy/tree candidate before the in-place commit; translated query/ray tests consume the public APIs. |
| CCD and TOI | WIRED | Contact-manager occurrences feed strict-less selection; equalized sweeps call checked `time_of_impact`; selected contacts feed `solve_toi_island` and TOI-specific constraints; `World::step` maps whole events to complete, pending, or bounded resumable outcomes. |
| Schema, witness registry, and policy | WIRED | Runtime validation and generated closed schemas share reviewed bounds; all required witness fields resolve exactly once through `phase7-v1`; byte-stability and unknown/missing/widened-policy tests pass. |
| Rust/C++ adapters and comparator | WIRED | Both adapters decode the same request and emit the same semantic result shape; declaration-first comparison preserves action, phase, identity, field, exact values, policy, and completion at first divergence. |
| Corpus, xtask, and evidence authority | WIRED | The checked-in request drives compare/replay/determinism/minimization; current checkout/build identity and D1 authority are revalidated before the first candidate, review, promotion, or accepted-artifact write. |
| Ledger and documentation | WIRED | `reference/compatibility.json` is the source for deterministic `COMPATIBILITY.md`; xtask checks enforce Phase 7 boundaries and D0/D1/D2 wording in `ARCHITECTURE.md` and `TESTING.md`. |

The textual path-reference heuristic reports expected false negatives for cross-module Rust links, independent Rust/C++ adapters, and generated outputs because those modules do not refer to each other by filesystem path. Manual symbol/data-flow tracing plus passing behavioral and process tests verifies all 26 semantic links.

## Automated Verification Evidence

Evidence was independently reproduced from commit `d939bebe5dfc9cbc8a05854a45695927dd64a082` before this report was written. No production source, accepted fixture, compatibility record, or configuration was modified.

### Mandatory Rust sequence

The repository-required pre-commit sequence passed in exact order:

1. `cargo fmt --all`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo build --all-targets --all-features`
4. `cargo test --all-features`

The stricter workspace-wide sequence also passed in order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo build --workspace --all-targets --all-features`
4. `cargo test --workspace --all-features`

The default-member test run includes 150 unit tests, every Phase 7 native integration target, and 12 doctests. The workspace run additionally covers protocol, schema, policy, adapter, comparator, fixture lifecycle, CLI, documentation, inventory, package, provenance, and upstream orchestration tests. The strict Clippy and build gates completed without errors.

### Focused native suites

| Target | Result |
| --- | --- |
| `fixture_dynamics` / `rigid_body_controls` / `rigid_world_config` | 22/22, 6/6, and 13/13 passed, including custom-mass derived-overflow and candidate-overflow atomicity. |
| `rigid_island_order` / `rigid_island_solver` / `rigid_sleeping` | 7/7, 11/11, and 12/12 passed, including shared-static reuse, manager ordering, late-island rollback, and whole-island sleep. |
| `rigid_world_queries` / `rigid_origin_shift` | 15/15 and 4/4 passed, including child multiplicity, equal-fraction ties, invalid clip handling, and translation covariance. |
| `rigid_ccd_selection` / `rigid_ccd` | 3/3 and 8/8 passed, including source-order selection, rollback, no warm-start pollution, resume behavior, and anti-tunneling. |
| `rigid_contacts` / `rigid_contact_solver` | 10/10 and 8/8 passed, preserving Phase 6 contact lifecycle and solver behavior. |

### Focused protocol, comparison, and lifecycle suites

| Command / target | Result |
| --- | --- |
| `cargo test -p liquidfun-test-protocol scenario::rigid_world --all-features` | 26/26 passed: closed Phase 7 requests/results, bounds, identities, exact bits, derived ray rejection, and completion classification. |
| `cargo test -p liquidfun-test-protocol schema::tests --all-features` | 4/4 passed: strict closed, byte-stable schema and tolerance presentations. |
| `cargo test -p liquidfun-test-protocol rigid_policy --all-features` | 6/6 passed: complete exact/set/multiset/absolute-relative/ULP registration and fail-closed unknown/missing/widened rules. |
| `cargo test -p liquidfun-differential --test rigid_world --all-features` | 46/46 passed: native execution, adapter shape, first divergence, multiplicity, tolerant boundary projection, minimization, supervision, and authority. |
| `cargo test -p liquidfun-differential --test rigid_fixture_workflow --all-features` | 15/15 passed: stage/replay/minimize provenance plus D2 no-effect rejection of canonical mutations. |
| `cargo test -p liquidfun-differential --test round_trip real_oracle_rejects_invalid_query_child_without_result_records --all-features` | 1/1 passed against the real subprocess oracle. |

### Fresh C++ and differential execution

1. Fresh `oracle-debug` configure/build, a clean 63-step build of `liquidfun-reference-protocol-tests`, and CTest passed 1/1. Repository-authored C++ compiled as C++17 with `-Wall -Wextra -Wpedantic -Werror`.
2. `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` matched all nine required families under `phase7-v1`; native and oracle were both `d2_supported`.
3. The matching replay command also matched all nine families at D2.
4. `verify-determinism --runs 2` executed exactly two runs and reported byte-identical native and oracle-debug responses at D0.
5. Fresh `oracle-asan-ubsan` configure, clean protocol-test build, and CTest passed 1/1 under `ASAN_OPTIONS=abort_on_error=1:halt_on_error=1` and `UBSAN_OPTIONS=halt_on_error=1:print_stacktrace=1`.
6. The one-shot sanitizer compare matched all nine families at D2 under the same fail-fast environment.

Local tools were CMake 3.27.9, Ninja 1.13.2, and Apple Clang 21.0.0. The pinned upstream checkout was independently verified at `7f20402173fd143a3988c921bc384459c6a858f2`.

### Repository integrity and completion gate

- `cargo xtask docs check` passed 12 testing layers and all five Phase 7 document contracts.
- `cargo xtask inventory check` passed for 177 compatibility rows.
- `cargo xtask provenance check` verified the pinned oracle and one recorded artifact.
- `cargo xtask check` passed inventory, package isolation for 69 entries outside the repository, strict schema/fixture presentation, documentation contracts, upstream identity, and provenance.
- `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps` passed.
- GSD schema drift reports `drift_detected: false`; phase completeness is 13/13 plans/summaries; lifecycle provenance is valid and consistent across context, plans, and summaries.
- All six mapped requirement checkboxes and traceability rows are complete. Prior Phase 1 through Phase 6 verification reports remain passed, and the full workspace suite exercises their regression paths.
- Review iteration 7 is `clean` for all 79 scoped files. Its final three warnings were closed by C++17-safe selector regression coverage, maximal duplicate-ray matching, and tolerant boundary-ray projection.

The repository completion gate is therefore 1/1.

## Disconfirmation and Failure-Path Checks

| Could the apparent pass be misleading? | Counter-check | Result |
| --- | --- | --- |
| Rust and C++ could both omit the new observations and still compare equal. | `comparison_rejects_omitted_phase7_observations_on_each_engine_side` removes Phase 7 data independently from each side. | PASS: comparison fails closed. |
| A resumed CCD hook/command error could erase or corrupt the pending checkpoint. | `failed_resume_hook_limit_preserves_pending_checkpoint` forces the error, retries, and compares with uninterrupted progress. | PASS: checkpoint and semantic result are preserved. |
| Query or ray matching could erase child multiplicity, accept stale identities, or greedily pair duplicate boundary hits. | Native child-multiplicity/equal-tie tests, the real-oracle invalid-child test, and Phase 7 comparator maximal-matching regressions exercise these branches. | PASS: multiplicity and identity remain exact; tolerant numeric matching does not weaken topology. |
| Derived arithmetic failure could panic or partially mutate world state. | Body-control and fixture custom/aggregate-mass overflow regressions compare complete before/after semantic state. | PASS: typed no-effect errors, no panic, no partial commit. |
| Local evidence could be mistaken for canonical authority. | Build identity classifies this toolchain as D2; stage/review/promotion guards run before filesystem mutation. | PASS: D2 is usable for local comparison but cannot create canonical D1 evidence. |

An independent scoped scan found no `TODO`, `FIXME`, `HACK`, stub/not-implemented marker, `unwrap()`, or new `unsafe` block in the Phase 7 production paths. The clean review and strict workspace Clippy gate corroborate the scan.

## Authority and Scope Limits

- D0 proves exactly two byte-identical runs for this local native/oracle-debug pair. D2 proves supported local behavior on the recorded macOS toolchain.
- This report does not claim canonical Linux D1 authority. Canonical evidence requires the configured CMake 4.3.3 and Clang 22.1.8 lane plus its reviewed promotion controls.
- This report does not claim D3 cross-platform equivalence. Wider Linux, macOS, and Windows evidence remains a later CI/release responsibility.
- Phase 7 accumulates the nine required differential witness families. Phase 8 owns broader rigid sign-off (`RIGD-11`), including the next compatibility breadth rather than retroactively widening this phase.
- Production remains native Rust. The C++ oracle is a private subprocess used only for reference testing and is absent from the published `liquidfun` package path.
- Query callback order remains explicitly unspecified. CCD cache entries, candidate indices, sweep state, counters, and resume checkpoints remain private; the public boundary exposes only semantic completion/progress and body/contact outcomes.
- Joint constraint solving and joint-owned origin-shift state remain later-phase work and are not implied by this phase's contact-only island and rigid-world claims.

No human verification is required. No gaps were found.

***

_Verifier: gsd-verifier_

_Result: passed_
