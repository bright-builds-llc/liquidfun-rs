---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T15:52:35Z
depth: standard
iteration: 6
review_kind: fresh_certification_review
diff_range: 43ce9d8..ca5bde6
files_reviewed: 77
files_reviewed_list:
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - README.md
  - TESTING.md
  - crates/liquidfun-differential/src/comparator.rs
  - crates/liquidfun-differential/src/minimizer.rs
  - crates/liquidfun-differential/src/rigid_evidence.rs
  - crates/liquidfun-differential/src/rigid_evidence/base.rs
  - crates/liquidfun-differential/src/rigid_evidence/declaration.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/context.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/observation.rs
  - crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs
  - crates/liquidfun-differential/src/rigid_fixtures.rs
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/rigid_world/phase7.rs
  - crates/liquidfun-differential/tests/rigid_fixture_workflow.rs
  - crates/liquidfun-differential/tests/rigid_fixture_workflow/provenance.rs
  - crates/liquidfun-differential/tests/rigid_world.rs
  - crates/liquidfun-differential/tests/support/phase7_comparator.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/tests.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/witness_registry.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/schema/tests.rs
  - crates/liquidfun-test-protocol/src/tolerance/rigid_policy.rs
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/collision/broad_phase.rs
  - crates/liquidfun/src/collision/tree.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/rigid_differential.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/body.rs
  - crates/liquidfun/src/world/body/control.rs
  - crates/liquidfun/src/world/config.rs
  - crates/liquidfun/src/world/contact.rs
  - crates/liquidfun/src/world/contact_manager.rs
  - crates/liquidfun/src/world/contact_solver.rs
  - crates/liquidfun/src/world/contact_solver/toi.rs
  - crates/liquidfun/src/world/continuous.rs
  - crates/liquidfun/src/world/continuous/event.rs
  - crates/liquidfun/src/world/continuous/tests.rs
  - crates/liquidfun/src/world/island.rs
  - crates/liquidfun/src/world/island/toi.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/origin.rs
  - crates/liquidfun/src/world/proxy.rs
  - crates/liquidfun/src/world/query.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/src/world/step/continuous.rs
  - crates/liquidfun/tests/rigid_body_controls.rs
  - crates/liquidfun/tests/rigid_ccd.rs
  - crates/liquidfun/tests/rigid_ccd_selection.rs
  - crates/liquidfun/tests/rigid_contact_solver.rs
  - crates/liquidfun/tests/rigid_island_order.rs
  - crates/liquidfun/tests/rigid_island_solver.rs
  - crates/liquidfun/tests/rigid_origin_shift.rs
  - crates/liquidfun/tests/rigid_sleeping.rs
  - crates/liquidfun/tests/rigid_world_config.rs
  - crates/liquidfun/tests/rigid_world_queries.rs
  - protocol/fixtures/accepted/rigid-world-request.jsonl
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - protocol/tolerances/phase7-v1.toml
  - reference/compatibility.json
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_action_decode.hpp
  - tools/reference/src/rigid_world_decode.hpp
  - tools/reference/src/rigid_world_phase7_execute.hpp
  - tools/reference/tests/protocol_tests.cpp
  - tools/xtask/src/differential.rs
  - tools/xtask/src/docs.rs
  - tools/xtask/tests/differential_cli.rs
  - tools/xtask/tests/docs_contract.rs
findings:
  critical: 0
  warning: 3
  info: 0
  total: 3
status: issues_found
---

# Phase 7: Fresh Certification Code Review — Iteration 6

**Reviewed:** 2026-07-13T15:52:35Z
**Depth:** standard
**Files:** 77
**Status:** issues found

## Summary

This fresh certification review inspected the complete 74-file iteration-5 scope plus every non-planning file changed by `43ce9d8..ca5bde6`, deduplicated to 77 files, and inspected the full fix diff and current code. WR-16, WR-17, WR-19, and WR-20 are closed: result validation now replays observations against action-time live topology, rejects invalid callback identities and post-termination records, requires finite ray payload geometry, rejects both signed-zero clip encodings in both request decoders, and performs equivalent source-ordered derived-ray arithmetic in Rust and C++ before execution.

WR-18 is only partially certified. Exact final-interval evidence and removal of the incorrect closest-hit interpretation fix the original arbitrary-clip ordering defect for ordinary distinct hits, but two adversarial numeric cases remain order- or boundary-sensitive. The C++ decoder also still omits the fixture-child range check previously added to the Rust boundary; a direct freshly built oracle probe accepted and executed `child_index: 1` for a single-child fixture.

The review applied `AGENTS.md`, `AGENTS.bright-builds.md`, the placeholder-only `standards-overrides.md`, and the managed architecture, code-shape, testing, verification, and Rust standards. The decisive requirements were fail-closed cross-language parsing, callback-order-independent evidence canonicalization, policy-consistent numeric comparison, and executable documentation claims.

## Warnings

### WR-21: The C++ request decoder still accepts nonexistent fixture children

**Files:** `tools/reference/src/rigid_world_decode.hpp:185-251`, `tools/reference/src/rigid_world_action_decode.hpp:249-279`

**Issue:** The C++ decoder parses every selector child as an arbitrary `u32` and checks only selector uniqueness while decoding. Its later timeline validation checks that a selector fixture is live, but never resolves the selector against the declared fixture shape or validates the child range. All currently admitted C++ circle and polygon fixtures expose only child zero. A fresh direct oracle probe changed the accepted terminating query selector to `child_index: 1`; `liquidfun-reference` exited successfully and emitted an exhausted query plus a valid result/end pair. Rust rejects the same request as `InvalidQueryDirective`. This leaves a direct C++ path that bypasses the closed request contract and silently changes directive semantics.

**Fix:** During C++ timeline validation, resolve each query and ray selector to its declared fixture and reject `child_index >= shape_child_count`; for the current closed shape set, require zero. Add compiled C++ protocol tests for invalid query and ray children and assert rejection occurs before world execution.

### WR-22: Greedy duplicate-hit pairing makes the ray multiset comparator order-dependent

**File:** `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:97-123`

**Issue:** After identity multiplicities match, `compare_hit_multisets` greedily assigns each expected hit to the first unmatched actual hit whose five numeric fields satisfy tolerance. Tolerance compatibility is not transitive, so greedy assignment does not establish multiset equality. Under the registered four-ULP fraction policy, duplicate hits with one identity and fraction bit patterns `expected = [b, b-4]`, `actual = [b-2, b+4]` have a valid perfect pairing (`b -> b+4`, `b-4 -> b-2`). With actual order `[b-2, b+4]`, the current loop consumes `b-2` for `b` and then reports a mismatch; reversing the actual callback order makes it match. Result validation permits these finite, in-range duplicate live-identity hits, so two contract-valid callback orderings can compare differently.

**Fix:** Group retained hits by semantic identity and use a deterministic maximum bipartite matching over complete numeric-policy compatibility, rather than first-fit greedy consumption. When no perfect matching exists, derive a stable first numeric divergence after matching. Add duplicate-hit regressions whose only difference is callback order and whose valid assignment requires reassignment.

### WR-23: Exact interval projection bypasses the ray fraction tolerance at the boundary

**File:** `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:48-51,193-199`

**Issue:** Each engine's hits are independently filtered with raw `fraction <= final_max_fraction` before numeric pairing. A callback can legally report a hit while the current interval is `1.0` and then apply an exact `Clip(0.5)`. If one engine reports that hit at exact `0.5` (`0x3f000000`) and the other one ULP above (`0x3f000001`), both results independently validate and the fractions satisfy the registered four-ULP policy. The current projection retains the first hit and discards the second, then reports an identity-multiset mismatch without applying the fraction policy. Thus a tolerated boundary difference becomes a false physics mismatch, and the pre-clip hit is discarded before it is semantically safe to do so.

**Fix:** Treat hits within the registered fraction-policy boundary band as retained, or pair same-identity hits under policy before deciding which pre-clip records are definitively outside the final interval. Discard only hits proven outside beyond the allowed fraction policy. Add exact-boundary and one-through-four-ULP straddle regressions in both engine directions.

## WR-16 Through WR-20 Certification

| Finding | Certification | Evidence |
| --- | --- | --- |
| WR-16 | Closed | `result.rs` reconstructs topology before the checkpoint window, applies each lifecycle action before its observation, validates live fixture-child identities, replays query/ray directives and completion, and rejects post-termination records. Focused per-engine stale, invalid-child, cascade, and completion tests pass. |
| WR-17 | Closed | Every hit fraction is finite and within the active interval; every point and normal component is finite before terminated-ray canonicalization. The NaN and both-infinity mutation matrix passes on each engine side. |
| WR-18 | Partially closed; WR-22 and WR-23 remain | Exact final intervals, arbitrary positive clips, pre-clip projection, multiplicity, and termination count/status are implemented and documented. Ordinary reversed histories pass, but duplicate tolerance assignment and interval-boundary tolerance remain order-sensitive. |
| WR-19 | Closed | Rust and C++ decoders use `fraction <= 0.0`, rejecting both positive and negative zero before execution; focused Rust and compiled C++ tests pass. |
| WR-20 | Closed | Both decoders evaluate component subtraction, component squares, and the sum in the same `f32` source order and reject signed-zero equality, subnormal underflow, subtraction overflow, squared overflow, and non-finite derived values before any C++ physics assertion. |

Earlier CR-01, WR-01 through WR-15, and IN-01 remain closed under current code inspection and the complete workspace and focused suites. WR-21 is the independently reproduced C++ half of the earlier selector-boundary requirement rather than a regression in Rust result validation.

## Verification Evidence

- Ordered workspace gate passed independently: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --all-targets --all-features`, and `cargo test --workspace --all-features`.
- Focused protocol evidence passed: 25 rigid-world unit tests, 2 rigid fixture-boundary tests, 4 schema presentation tests, and 6 typed rigid-policy tests.
- Focused differential evidence passed: rigid-world integration 40/40 and rigid fixture workflow 15/15.
- Fresh `oracle-debug` configure succeeded, the reference executable and protocol-test target rebuilt, and CTest passed 1/1.
- Rigid compare and replay each matched all 9 required families under `phase7-v1` at local D2-supported authority.
- `cargo xtask docs check` verified all 5 Phase 7 document contracts; `cargo xtask inventory check` verified 177 compatibility rows.
- `cargo xtask check` passed package isolation for 69 entries, schema and fixture presentation, documentation, inventory, upstream identity, and provenance.
- `gsd-tools verify schema-drift 7` reported no drift and no blocker.
- `git diff --check 43ce9d8..ca5bde6` and final pre-report `git diff --check` passed.
- Direct C++ adversarial selector probe returned `{"completion":"exhausted",...}` and exit status 0 for a terminating query rule targeting child 1 of a single-child fixture, confirming WR-21 independently of the Rust harness.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8. Successful local compare/replay remains D2 evidence and does not promote canonical D1 or platform-wide authority.

## Worktree State

No source file was edited and no commit was created by this reviewer. The pre-existing `.planning/config.json` modification remained byte-for-byte unchanged (SHA-256 `440f14fa5b03113fe46105f252bace03fa84094e2b862c9ec1757a855fca5eba`; Git blob `621946b2b075747d8342124a8abb2226e77546ad`). The only review-created worktree change is this report.

***

_Reviewed: 2026-07-13T15:52:35Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
