---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
verified: 2026-07-15T02:14:17Z
status: passed
score: "75/75 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-15T02:14:17Z
lifecycle_validated: true
overrides_applied: 0
requirements:
  - id: RIGD-11
    status: verified
  - id: JOIN-01
    status: verified
  - id: JOIN-02
    status: verified
  - id: JOIN-03
    status: verified
  - id: JOIN-04
    status: verified
  - id: JOIN-05
    status: verified
must_haves:
  roadmap_success_criteria: 5/5
  plan_truths: 70/70
  plan_artifacts: 61/61
  plan_key_links: 25/25
  repository_completion_gates: 1/1
evidence:
  verified_commit: beb98bd74b1d26ab0a96c6be33ce1926d349abf0
  canonical_code_commit: beb98bd74b1d26ab0a96c6be33ce1926d349abf0
  canonical_run: 29383445374
  canonical_workflow: passed
  canonical_artifact: phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0
  sanitizer_artifact: phase8-sanitizer-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0
  automatic_cargo_ci: 29382964877_passed
  automatic_oracle_ci: 29382964854_passed
  focused_native_protocol_and_differential_suites: passed
  repository_checks: passed
  code_review: clean
  compatibility_ledger_identity: current_and_drift_guarded
---

# Phase 8: Joints, Rope, Callbacks, and Rigid Sign-Off Verification Report

**Phase goal:** Finish the rigid-body surface and pass the broad semantic compatibility gate before particles expand the world step.

**Status:** `passed`

**Score:** 75/75 observable must-haves verified: all five roadmap success criteria and all 70 plan truths.

## Verdict

The Phase 8 implementation achieves its bounded behavioral goal. Native Rust implements all eleven joint families, standalone rope, safe contact and destruction hooks, transactional lifecycle and step behavior, bounded semantic reconstruction, and the closed Phase 8 differential surface. Exact-commit GitHub Actions run [29383445374](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/29383445374) passed its canonical Linux, fail-fast sanitizer/reset Linux, macOS portability, and Windows portability jobs at final reviewed code commit `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`. Both exact artifacts exist and their identity records agree with the run, code commit, pinned upstream revision, toolchain, and `phase8-v1` policy. Push-triggered Cargo CI run `29382964877` and Oracle CI run `29382964854` also passed at that exact head.

All six mapped requirements are verified, all five roadmap criteria are achieved, focused verification passes, and the final code review is clean. The prior sign-off integrity gap was resolved at `a109440be6a7ed493efdfa7f90b888380b7acb9f`, and the final evidence refresh advances all 33 `platform_validated` entries in `reference/compatibility.json` to run `29383445374` and its exact canonical and sanitizer identity records. The generated compatibility report agrees, and the documentation checker parses the authoritative ledger and rejects a stale identity, a changed reference set, or a row-count drift.

Plan 08-24's exact-evidence truth, ledger artifact, and evidence key link are therefore verified. Independent cleanup review `beb98bd74b1d26ab0a96c6be33ce1926d349abf0` found no new correctness, safety, or evidence issue, and the final exact-commit dispatch passed. No implementation gap or human-only verification item remains.

This verification applied `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the local architecture, code-shape, testing, verification, and Rust standards. No override was needed.

## Goal Achievement

### Roadmap success criteria

| # | Observable truth | Status | Independent evidence |
| ---: | --- | --- | --- |
| 1 | Consumers can create, inspect, mutate, simulate, and destroy every pinned joint family with validated identities and lifecycle behavior. | VERIFIED | `JointKind`, `JointDef`, definitions, snapshots, world dispatch, typed solvers, and destruction paths are exhaustive across revolute, prismatic, distance, pulley, mouse, gear, wheel, weld, friction, rope, and motor joints. Focused native and Phase 8 differential suites pass. |
| 2 | Joint limits, motors, reactions, dependencies, collision suppression, and destruction cascades reproduce supported upstream behavior. | VERIFIED | Typed velocity and position constraint lanes cover all eleven families; gear A/B/C/D dependencies, newest-first adjacency, refilter behavior, reaction queries, and lifecycle cascades are tested and included in the exact canonical corpus. |
| 3 | Standalone rope is independent, bounded, transactional, and upstream-equivalent within the closed policy. | VERIFIED | `Rope` owns its particles and stretch/bend constraints independently of `World`; focused tests pass 11/11, including an exact-bit oracle witness and no-effect failure behavior. |
| 4 | Safe filters, listeners, destruction notifications, and pre-solve controls preserve supported timing without exposing mutable storage identity. | VERIFIED | Borrow-scoped fixture/pre-solve views, validated directives, authoritative lifecycle ordering, hook-limit rollback, and destruction notifications pass focused tests and differential witnesses. |
| 5 | The broad rigid corpus passes exact-commit semantic differential, replay, determinism, sanitizer, reset, and portability execution before particle work. | VERIFIED | Run `29383445374` passed all four jobs at `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`; the canonical and sanitizer artifacts are present and unexpired with matching identities. |

**Roadmap score:** 5/5

### Requirement accounting

| Requirement | Status | Evidence boundary |
| --- | --- | --- |
| RIGD-11 | VERIFIED | The closed corpus covers non-colliding, colliding, stacked, sleeping, fast-moving, filtered, queried, and destroyed rigid worlds through first-divergence semantic comparison, replay, determinism, sanitizer/reset, and exact-commit CI evidence. |
| JOIN-01 | VERIFIED | All eleven joint definitions, snapshots, mutations, reaction queries, typed island solvers, and destruction paths are exhaustive and exercised. |
| JOIN-02 | VERIFIED | Limits, motors, anchors, reactions, warm starting, dependencies, collision suppression, gear lanes, and explicit/implicit destruction behavior have focused and differential witnesses. |
| JOIN-03 | VERIFIED | Standalone rope has an independent public model, stretch and bend constraints, checked bounds, transactional mutation, focused tests, and an upstream oracle witness. |
| JOIN-04 | VERIFIED | Filters, begin/end/pre-solve events, borrow-scoped views, validated directives, hook budgets, and destruction listeners preserve supported source timing and atomic failure behavior. |
| JOIN-05 | VERIFIED | Each joint family, standalone rope, filter/listener path, reconstruction/dump representation, mutation branch, and gear mode is registered in the closed Phase 8 policy and covered by focused and differential tests. |

The requirements and their evidence publication surfaces are both verified.

## Plan Must-Have Accounting

All 70 truths declared by Plans 08-01 through 08-24 are verified against implementation, tests, exact external evidence, and the corrected sign-off surfaces.

| Plan | Truths | Verification result |
| --- | ---: | --- |
| 08-01 | 3/3 | Closed eleven-kind contract, opaque generational identity, candidate-first lifecycle, and collision/refilter effects are complete. |
| 08-02 | 3/3 | Revolute and prismatic limits, motors, reactions, mutation, warm starting, and position correction use typed source-equivalent lanes. |
| 08-03 | 3/3 | Distance, pulley, and mouse definitions, live state, reactions, mutation, and solver behavior are complete. |
| 08-04 | 3/3 | Wheel, weld, friction, rope-joint, and motor contracts and solvers are exhaustive and focused-tested. |
| 08-05 | 3/3 | Gear validation, reverse dependencies, A/B/C/D body lanes, ratio mutation, and cascade behavior are complete. |
| 08-06 | 3/3 | Joint islands, collision suppression/refiltering, wake behavior, origin shifting, and atomic commit integrate with the rigid world. |
| 08-07 | 3/3 | Standalone rope is independent, bounded, transactional, and backed by focused and exact-oracle witnesses. |
| 08-08 | 3/3 | Filter and pre-solve views are borrow-scoped, source-ordered, validated, and atomic under hook limits. |
| 08-09 | 3/3 | Owned lifecycle and destruction events preserve exact explicit/implicit source timing and authoritative identity. |
| 08-10 | 3/3 | Semantic reconstruction is bounded, dependency-safe, deterministic, and explicit about unsupported mouse reconstruction. |
| 08-11 | 3/3 | The Phase 8 schema, bounds, 19-family registry, 53 witnesses, and fail-closed policy are complete. |
| 08-12 | 3/3 | The native adapter executes the closed declaration, mutation, step, hook, lifecycle, rope, and reconstruction surface. |
| 08-13 | 3/3 | The C++ adapter, comparator, and witness execution mirror the same closed semantic contract without weakening identity or policy. |
| 08-14 | 3/3 | Shared live-solver staging and exhaustive typed dispatch replaced the generic constraint seam while retaining transactional commit. |
| 08-15 | 2/2 | Revolute and prismatic live velocity/position solvers preserve their typed warm-start and runtime-state behavior. |
| 08-16 | 2/2 | Distance, pulley, and mouse live solvers preserve their typed constraint and mutation behavior. |
| 08-17 | 2/2 | Wheel, weld, friction, rope-joint, and motor live solvers preserve their family-specific constraint behavior. |
| 08-18 | 3/3 | Four-body gear solving and complete island integration preserve source order, scatter lanes, rollback, and reactions. |
| 08-19 | 3/3 | The accepted corpus carries positive steps and observable non-no-op mutations through closed schemas and policy. |
| 08-20 | 3/3 | Native step-bearing evidence covers all eleven joint kinds, gear modes, rope, hooks, lifecycle, and reconstruction. |
| 08-21 | 3/3 | Pinned C++ step-bearing execution uses live setters and runtime gear ratio with strict protocol behavior. |
| 08-22 | 4/4 | Comparator, local compare/replay/determinism, review closure, and oracle-ready fail-closed checks are complete. |
| 08-23 | 3/3 | Final reviewed code commit `beb98bd` passed canonical, sanitizer/reset, and portability execution with two bound artifacts. |
| 08-24 | 3/3 | Documentation and the 33-row authoritative ledger bind only the exact replacement evidence and retain every residual scope limit. |

GSD phase completeness reports 24 plans, 24 summaries, no incomplete plans, and no orphan summaries. Lifecycle provenance is consistent across context, plans, summaries, and this report.

## Implementation and Wiring Evidence

| Area | Result | Data/control-flow evidence |
| --- | --- | --- |
| Joint contract and lifecycle | WIRED | Public closed definitions enter private `JointRecord` state only through checked `World` operations. Generational world-scoped handles, topology, capacity, lock/poison, wake/refilter, adjacency, and destruction effects are candidate-validated before commit. |
| Typed joint solving | WIRED | `JointVelocityConstraint` exhaustively constructs per-family candidates for all eleven kinds. Island order feeds typed warm-start, velocity, and position stages; gear constraints preserve four-body dependency lanes. No legacy generic constraint fallback remains. |
| Transactional world stepping | WIRED | Step backup includes bodies, fixtures, joints, broad phase, contact manager, continuous state, and configuration. Ordinary limit failures restore the complete snapshot; continuous work-limit progress remains deliberately resumable. |
| Standalone rope | WIRED | Rope creation and stepping use owned checked particle/constraint state, bounded iteration, stretch/bend models, and clone-then-commit failure semantics without entering the rigid world. |
| Hooks and destruction | WIRED | Contact-manager source order feeds borrow-scoped filter/pre-solve views and owned lifecycle records. Validated directives affect supported contact state; explicit and implicit destruction publish exact authoritative order. |
| Semantic reconstruction | WIRED | Bounded reconstruction emits bodies and fixtures, then non-gear joints, then dependency-safe gear joints, with exact counts and tree metrics. Unsupported mouse reconstruction is explicit rather than silently approximated. |
| Protocol and comparator | WIRED | Nineteen accumulated families and 53 Phase 8 witnesses enter a closed schema/policy. The comparator rejects missing or unknown fields, preserves signed zero, lifecycle multiplicity, and gear lanes, and reports the first semantic divergence. |
| Native/C++ adapters | WIRED | Rust and pinned-C++ adapters execute the same declaration/mutation/step corpus. Live mouse target, motor correction, and gear-ratio mutation paths are observable; gear coordinate calculation consumes the configured live ratio. |
| Evidence workflow | WIRED | The workflow checks exact checkout identity, builds debug/release/sanitizer variants, compares and replays the corpus, verifies D0 determinism, exercises reset behavior, and runs macOS/Windows portability jobs. |
| Compatibility ledger | WIRED | Exactly 33 platform-validated rows bind to run `29383445374`, both exact artifact identity records, and the testing-policy anchor. The docs checker parses the ledger directly and rejects stale identity, reference-set, or row-count drift. |

## Automated Verification Evidence

### Focused native suites

All independently rerun focused checks passed using a separate target directory:

| Target | Result |
| --- | --- |
| `joint_island_solver` | 7/7 passed. |
| `standalone_rope` | 11/11 passed. |
| `contact_hook_timing` | 8/8 passed. |
| `lifecycle_timeline` | 4/4 passed. |
| `destruction_listener` | 5/5 passed. |
| `hook_limit_transaction` | 2/2 passed. |
| `semantic_reconstruction` | 5/5 passed. |
| `liquidfun-test-protocol` Phase 8 rigid-world target | 16 selected tests passed. |
| `liquidfun-differential` Phase 8 rigid-world target | 10/10 passed. |
| `liquidfun-differential` Phase 8 comparator target | 6/6 passed. |

`cargo xtask inventory check` passed for 177 compatibility rows, and `cargo xtask docs check` passed all five Phase 8 documentation contracts plus the exact ledger-evidence contract. `cargo test -p xtask --test docs_contract phase8_contract -- --nocapture` passed 5/5 tests, including fail-closed platform-evidence drift rejection. The clean Phase 8 review and review-fix report record the mandatory ordered Rust format, Clippy, all-target build, and all-feature test gates passing after the final implementation fixes.

### Exact external evidence

GitHub Actions run `29383445374` was independently queried rather than inferred from planning summaries:

- Event: `workflow_dispatch`; conclusion: `success`.
- Head commit: `beb98bd74b1d26ab0a96c6be33ce1926d349abf0`.
- Jobs: canonical Linux oracle, fail-fast sanitizer/reset Linux, macOS portability, and Windows portability all succeeded.
- Canonical artifact: `phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`.
- Sanitizer artifact: `phase8-sanitizer-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0`.
- Both artifacts are present and unexpired. Their identity records bind the same run and commit to upstream revision `7f20402173fd143a3988c921bc384459c6a858f2`, Rust 1.97.0, CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, and `phase8-v1`.
- Automatic Cargo CI run `29382964877` and automatic Oracle CI run `29382964854` passed at the same exact head before the full dispatch.

The canonical code head `beb98bd74b1d26ab0a96c6be33ce1926d349abf0` includes the final workspace-Clippy cleanup and its clean independent review. The subsequent evidence-only refresh changes the 33 evidence-reference arrays and durable sign-off records; it does not promote an applicability, implementation, test, or maturity status.

## Disconfirmation and Failure-Path Checks

| Could the apparent pass be misleading? | Counter-check | Result |
| --- | --- | --- |
| The ledger could retain a superseded identity while prose names the replacement run. | Enumerated every `platform_validated: evidenced` row and compared its complete ordered reference array. | PASS: exactly 33 rows share only the current run, canonical identity, sanitizer identity, and testing-policy anchor. |
| The inventory check could prove only generated-file consistency and miss evidence drift. | The docs checker now deserializes `reference/compatibility.json`; `phase8_contract_rejects_platform_evidence_drift` injects the prior canonical identity. | PASS: the mutation fails through `docs/phase8-evidence`; the unmodified repository passes. |
| Fixing the ledger could silently broaden maturity or platform status. | Delta review compared the ledger semantically before and after `a109440`. | PASS: only reference arrays changed; status and applicability fields are unchanged. |
| The replacement evidence could belong to another code commit or toolchain. | Queried run `29383445374`, both artifact records, and both downloaded identity files. | PASS: run, code commit, upstream revision, tools, jobs, and `phase8-v1` agree. |

## Authority and Scope Limits

- The verified sign-off is limited to the closed Phase 8 scalar rigid-body and joint corpus. It does not establish complete LiquidFun parity.
- `RIGD-10` remains assigned to Phase 11. Phase 8 reconstruction covers bounded semantic counts, tree metrics, and deterministic reconstruction; timing profiles and renderer-independent debug draw remain pending.
- Particle simulation and particle/world coupling remain pending later phases.
- D3 and cross-platform numerical parity are not claimed. The macOS and Windows jobs are portability execution, while the exact canonical scalar comparison authority remains the pinned Linux lane.
- Performance qualification, testbed completeness, release readiness, and production-stability claims remain pending.
- The published engine remains native Rust. The C++ oracle is private test tooling and is not part of the consumer runtime.

No broader parity, particle, D3, performance, testbed, or release claim was accepted by this verification.

***

_Verifier: gsd-verifier_

_Result: passed_
