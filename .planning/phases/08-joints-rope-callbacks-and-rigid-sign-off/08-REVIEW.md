---
phase: 08-joints-rope-callbacks-and-rigid-sign-off
depth: standard
status: clean
files_reviewed: 95
counts:
  critical: 0
  warning: 0
  info: 0
  total: 0
generated_by: gsd-code-review
lifecycle_mode: yolo
phase_lifecycle_id: 8-2026-07-13T21-26-30
generated_at: 2026-07-15T00:52:36Z
---

# Phase 8 Code Review

## Scope and verdict

Re-reviewed the exact 95-file Phase 8 implementation surface reconstructed
from the 24 phase summaries. The review covered the public joint and rope APIs,
typed joint solvers and four-body gear scatter, hook and lifecycle ownership,
protocol validation and result bounds, native and pinned C++ adapters,
comparison policy, accepted fixtures, oracle CI, and canonical-evidence
claims. The review included fixes `53a0b02` and `e0b5106` and the evidence and
documentation refresh at `e8daec8`.

The Phase 8 surface is clean at standard review depth. Both prior warnings are
resolved, the exact replacement canonical run and artifacts are internally
consistent, and no new actionable correctness, safety, or evidence issue was
found.

## Prior finding resolution

### CR-08-25-01 — Resolved — Hook-limit failures are transactional

`World::step` now snapshots the complete rigid-world state that can change
during locked stepping before pair discovery, lifecycle dispatch, solving, or
continuous work. An ordinary hook `LimitExceeded` restores bodies, fixtures,
joints, broad phase, contact manager, continuous state, and configuration
before returning the error. The distinct `ContinuousWorkLimitExceeded` path
retains its documented resumable-progress behavior.

The focused regressions cover event and command limits at both zero and one,
assert exact immediate state restoration, then retry and compare the result
with a clean one-shot world. The existing continuous-work regression confirms
that budget exhaustion still resumes rather than rolling back.

### CR-08-25-02 — Resolved — Closed-corpus mutations are observable

Phase 8 validation now resolves each mutation target to its full declaration
and rejects declaration-equivalent values across all 17 closed mutation
branches. The comparison is fail-closed for an unknown branch, while accepted
tests exercise declaration-distinct values for every supported mutation.

The mouse target, motor correction factor, and gear ratio actions now use
values distinct from their declarations and are followed by positive stepping
and inspection. Both adapters invoke the actual setters, and the C++ gear
observer reads `b2GearJoint::GetRatio()` rather than reconstructing the live
coordinate from the declaration ratio. Focused fixture regressions reject each
of the three former no-ops.

## Findings

None.

## Canonical evidence

- GitHub Actions run `29379350740` completed successfully for exact commit
  `e0b5106559b3c0c37beb44e4ade45c3b7919b59d`.
- Canonical Linux, fail-fast sanitizer/reset-corpus Linux, macOS portability,
  and Windows portability jobs all succeeded.
- The run published exactly two unexpired artifacts:
  `phase8-canonical-29379350740-e0b5106559b3c0c37beb44e4ade45c3b7919b59d`
  and
  `phase8-sanitizer-29379350740-e0b5106559b3c0c37beb44e4ade45c3b7919b59d`.
- Both downloaded identity records bind the same run and commit, upstream
  revision `7f20402173fd143a3988c921bc384459c6a858f2`, Rust 1.97.0,
  CMake 4.3.3, Ninja 1.13.2, Clang 22.1.8, and `phase8-v1` policy.
- Repository documentation consistently names the replacement run and marks
  the previous evidence as superseded only in historical completion records.

## Verification evidence

- `cargo test -p liquidfun --all-features --test hook_limit_transaction`
  passed: 2 tests.
- `cargo test -p liquidfun --all-features --test hook_contract` passed: 8
  tests.
- The focused resumable continuous-work regression passed.
- `cargo test -p liquidfun-test-protocol --all-features phase8` passed: 18
  tests.
- `cargo test -p liquidfun-differential --all-features --test
  rigid_world_phase8 --quiet` passed: 10 tests.
- `cargo test -p liquidfun-differential --all-features --test
  phase8_comparator --quiet` passed: 6 tests.
- `cargo xtask docs check` passed with all five Phase 8 document contracts.
- `cargo xtask inventory check` passed with 177 compatibility rows.
- `cargo fmt --all` passed.
- `cargo clippy --all-targets --all-features -- -D warnings` passed.
- `cargo build --all-targets --all-features` passed.
- `cargo test --all-features` passed.
- No new `unsafe` block, `unwrap()`, placeholder implementation, ignored error
  path, or unintended file was found in the reviewed production surface.

## Residual risks

The transactional backup adds cloning cost to rigid stepping, and hook-owned
external side effects remain outside the `World` rollback boundary. Neither is
an actionable Phase 8 finding: the contract is exact rigid-world restoration,
the external-hook boundary is explicit, and performance claims remain gated
on later benchmark evidence. Particle parity, broader determinism evidence,
and release-readiness work also remain outside this closed phase scope.

***

_Reviewer: gsd-code-review_
_Lifecycle: 8-2026-07-13T21-26-30_
