---
phase: 07-rigid-solver-world-operations-and-ccd
reviewed: 2026-07-13T14:36:35Z
depth: standard
iteration: 5
review_kind: final_fresh_review
diff_range: eb48e26..5812367
files_reviewed: 74
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
  - tools/reference/src/rigid_world_phase7_execute.hpp
  - tools/reference/tests/protocol_tests.cpp
  - tools/xtask/src/differential.rs
  - tools/xtask/src/docs.rs
  - tools/xtask/tests/differential_cli.rs
  - tools/xtask/tests/docs_contract.rs
findings:
  critical: 0
  warning: 5
  info: 0
  total: 5
status: issues_found
---

# Phase 7: Final Fresh Code Review — Iteration 5

**Reviewed:** 2026-07-13T14:36:35Z
**Depth:** standard
**Files:** 74
**Status:** issues found

## Summary

This fresh review inspected the complete 71-file iteration-4 scope plus every file changed by `eb48e26..5812367`, deduplicated to 74 files, and inspected that full diff. The iteration-5 change correctly replaces the ambiguous reached-clip boolean with exact final interval bits, prevents `Continue` from re-expanding the C++ interval, keeps `Clip(1.0)` exhaustive, and rejects explicit expansion. WR-12 is nevertheless not fully closed: legal arbitrary strict clips can still make closest-hit comparison callback-order-dependent, and zero clips have different traversal behavior in native Rust and pinned C++. WR-13, WR-14, WR-15, and the iteration-1 findings remain closed under the current implementation and regression suite.

Three additional boundary gaps remain. Query observations are accepted without replaying query directives or completion, query/ray hit identities are not bound to checkpoint-local live fixture children, terminated ray results can carry non-finite geometry that is never compared, and numerically degenerate finite ray endpoints can bypass request validation and reach divergent native/C++ failure behavior.

The review applied `AGENTS.md`, `AGENTS.bright-builds.md`, the non-substantive placeholder in `standards-overrides.md`, and the managed architecture, code-shape, testing, verification, and Rust standards. The decisive requirements were closed boundary parsing, semantic identity, fail-closed evidence validation, and documentation that matches executable behavior.

## Warning

### WR-16: Query and ray result identities are not independently bound to live fixture children

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:395-401,441-444,470-476`

**Issue:** `validate_rigid_world_result_against_request` derives the exact checkpoint-local live body and fixture identities, but passes only the action window to observation validation. `ExpectedObservation::Query` accepts every query payload without checking its fixture-child occurrences, directive sequence, or completion. Ray replay validates interval and termination behavior, but an occurrence whose fixture-child selector matches no rule silently receives `Continue`; it is never required to name a fixture child live at that checkpoint.

Consequently, two results can report `terminated` for an all-continue query, or can contain matching fabricated or stale fixture-child query/ray occurrences, and both independently validate before the comparator returns `Match`. A terminated ray can also include fabricated pre-termination hits; the intentional count/status comparator then cannot distinguish them. This contradicts `TESTING.md:422-426`, which requires each result to independently satisfy declared identities and completion states before cross-engine fields are read.

**Fix:** Pass checkpoint-local live fixture declarations into observation validation. Require every query occurrence and ray hit to resolve to a live declared fixture and valid child index. Replay query directives in observed callback order, reject occurrences after termination, and require the recorded query completion to match the replayed state. Keep the existing ray interval replay, but perform identity validation before defaulting an unmatched live hit to `Continue`. Add per-engine declaration regressions for an invalid query completion, a stale/unknown query occurrence, and a stale/unknown ray hit, including a fabricated hit before valid ray termination.

### WR-17: Terminated rays can hide non-finite hit geometry

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs:462-499`, `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:46-73`

**Issue:** Ray result replay validates hit fractions, directive order, termination, and final interval, but never requires the emitted point or normal coordinates to be finite. The comparator intentionally reduces a terminated ray to completion, exact final interval, and callback count, so those numeric fields never reach the Phase 7 float policies. Replacing the point or normal of a valid terminated hit with NaN or infinity leaves the result declaration-valid and can still compare `Match`, despite every Phase 7 numeric policy using `reject_arithmetic_nan`.

**Fix:** Independently validate every ray hit point and normal component as finite before completion-based canonicalization. Add per-engine result rejection regressions that mutate each terminated hit coordinate to NaN and both infinities while preserving count, completion, and final interval.

### WR-18: Arbitrary strict clips do not establish callback-order-independent closest-hit results

**File:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:611-617`, `crates/liquidfun-differential/src/rigid_evidence/phase7/ray.rs:49-55,76-123`

**Issue:** Request validation permits any finite clip in `0.0..=1.0`, unrelated to the triggering hit fraction. The comparator treats every strict final interval reduction as a closest-hit query. With hits at fractions `0.25` and `0.75` and a rule on the `0.75` fixture returning `Clip(0.1)`, one legal callback order records only `0.75` before pruning while another records `0.25, 0.75`; both validate with the same exact final interval `0.1`. Their minimum identity sets differ, so two contract-valid orderings become a physics mismatch. The exact interval proves a reduction, but not closest-hit semantics.

**Fix:** Either constrain closest-mode clips to the triggering hit fraction through independently validated exact/tolerance semantics, or define a comparison model for arbitrary interval clips that does not reinterpret pre-clip callback history as a closest-hit set. Add reversed-order regressions with a clip below the triggering hit and prove equivalent valid traversals compare the same.

### WR-19: `Clip(0.0)` has different traversal semantics in native Rust and pinned C++

**Files:** `crates/liquidfun-differential/src/rigid_world/phase7.rs:426-440`, `crates/liquidfun/src/collision/tree/traversal.rs:146-156`, `tools/reference/src/rigid_world_phase7_execute.hpp:144-150`

**Issue:** Zero is a valid protocol clip. Native traversal applies `Clip(0.0)` by narrowing the segment AABB and continuing its stack, so additional fraction-zero fixtures can still reach the callback. Pinned `b2DynamicTree::RayCast` treats a callback return value equal to zero as immediate termination. The C++ adapter returns the clip value directly, so its traversal stops even though the semantic result is labeled `exhausted`. Multiple fixtures intersecting at the ray start can therefore produce different valid callback counts and closest identity sets across adapters.

**Fix:** Give zero one shared meaning at the adapter boundary: either reject zero clips, normalize them to explicit termination with matching completion, or change native/C++ adapter control flow so both traverse identically. Add native and compiled-C++ regressions with multiple fraction-zero hits and both positive and negative zero clip bits.

### WR-20: Numerically degenerate ray endpoints bypass the request boundary

**Files:** `crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs:538-543`, `crates/liquidfun/src/collision/tree/traversal.rs:98-106`

**Issue:** The request boundary rejects a ray only when the exact endpoint bit records are equal. Numerically equal signed-zero endpoints such as `(+0.0, +0.0)` and `(-0.0, +0.0)` are therefore accepted, as are distinct finite endpoints whose squared direction underflows to zero. Native traversal later returns `DegenerateRay`; pinned C++ asserts that the squared direction is positive before normalization, so a request accepted as valid can become an adapter error or process abort instead of a stable pre-execution rejection.

**Fix:** Validate the decoded numeric direction at the shared request boundary with the source-ordered derived arithmetic required by both adapters: reject numerically zero, non-finite, underflowed-zero, or overflowed squared length. Mirror the check in the C++ decoder as defense in depth. Add signed-zero, subnormal-underflow, and finite-overflow endpoint fixtures that both decoders reject before world execution.

## WR-12 Partial Certification

- `RigidRayObservation.final_max_fraction_bits` is required by the closed typed result and tracked JSON schema. Validation starts from the exact `1.0_f32` bit pattern and accepts the recorded field only when it exactly equals callback replay.
- Request validation rejects non-finite and out-of-range clip fractions; result replay rejects non-finite/out-of-interval hit fractions, post-termination hits, non-finite or out-of-range clips, and clips that would expand the current interval. It does not yet reject non-finite point/normal geometry.
- Native execution tracks the exact strict-reduction bits and returns an error after any attempted expansion. The public world traversal independently rejects clips outside its current sub-input.
- C++ `Continue` and `Ignore` both return `-1.0F`, preserving the current Box2D interval. Equal clips are no-ops, strict decreases update the recorded interval, and expansions throw before result publication.
- A reached `Clip(1.0)` records the exact initial interval and therefore uses exhaustive multiplicity-preserving multiset comparison. Strictly reduced results first exact-compare their validated final interval, then compare equal-minimum identities as a set and numeric hit fields under their named policies.
- Termination replays without post-termination hits and compares exact completion, exact final interval, and callback count, but requires a separate finite-geometry boundary before numeric payloads can safely be discarded.
- The typed profile closes 37 Phase 7 paths and the accepted request carries canonical profile SHA-256 `a28360556d0339627ec26dc988fcb5585d12e14ba88ecb801c9cf9bdb1a193fe`. Architecture and testing documentation match the implemented interval and collection policies.

## Prior-Finding Reconfirmation

| Finding | Iteration-5 disposition | Evidence |
| --- | --- | --- |
| CR-01 | Closed | Custom mass preparation remains transactional; derived center/velocity overflow regressions pass. |
| WR-01 | Closed | Pending CCD resume survives the pre-continuous hook limit; focused regression and workspace suite pass. |
| WR-02 through WR-11 | Closed | Exact observation presence, closed policy paths, contact identity reuse defense, selector request bounds, collection semantics, immutable policy provenance, real minimization, typed replay provenance, atomic staging, and nine-family C++ coverage all remain implemented and covered. |
| WR-12 | Partially closed; WR-18 and WR-19 remain | Exact effective interval replaces the ambiguous reached-clip boolean, but arbitrary strict clips and zero clips do not yet have callback-order-independent cross-adapter semantics. |
| WR-13 | Closed | Checkpoint-local lifecycle replay requires exact live body and fixture snapshot identities, including body-destruction fixture cascades. |
| WR-14 | Closed | Shared checkpoint action windows bind later observations and minimizer prefixes to the correct action/stage. |
| WR-15 | Closed | Architecture and testing docs describe only emitted fields and the implemented multiset/set/termination policies. |
| IN-01 | Closed | Workspace Clippy passes with warning denial. |

WR-16 is distinct from WR-02 and WR-13: observation presence and snapshot identity are closed, but callback payload identity and query completion are not yet independently validated.

## Verification Evidence

- Ordered workspace gate passed: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo build --workspace --all-targets --all-features`, and `cargo test --workspace --all-features`.
- Focused protocol rigid-world tests passed 24 unit and 2 fixture tests.
- Focused schema presentation tests passed 4/4; focused Phase 7 policy tests passed 6/6.
- Differential rigid-world integration passed 33/33, including no-op clip, exact interval disagreement, expansion rejection, closest/equal-minimum, termination, WR-13, and WR-14 regressions.
- `cargo xtask upstream configure --preset oracle-debug` and `cargo xtask upstream build --preset oracle-debug` passed.
- `ctest --test-dir target/reference/oracle-debug --output-on-failure --no-tests=error` passed 1/1 compiled C++ protocol test.
- Rigid compare and replay each matched all 9 required families under `phase7-v1` at local D2-supported authority.
- `cargo xtask docs check` verified all 5 Phase 7 documentation contracts.
- `cargo xtask inventory check` verified 177 compatibility rows.
- `cargo xtask check` passed package isolation for 69 entries, schema/fixture presentation, documentation, inventory, upstream identity, and provenance.
- `gsd-tools verify schema-drift 7` reported no drift and no blocker.
- `git diff --check eb48e26..5812367` and final `git diff --check` passed.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8. Successful local compare/replay remains D2 evidence and does not promote canonical D1 or platform-wide authority.

## Worktree State

No source file was edited and no commit was created. The pre-existing `.planning/config.json` modification remained byte-for-byte unchanged (`git hash-object`: `621946b2b075747d8342124a8abb2226e77546ad`). The only review-created worktree change is this report.

***

_Reviewed: 2026-07-13T14:36:35Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
