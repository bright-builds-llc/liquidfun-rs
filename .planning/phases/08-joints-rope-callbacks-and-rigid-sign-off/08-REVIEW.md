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
generated_at: 2026-07-15T01:57:00Z
---

# Phase 8 Code Review

## Scope and verdict

Re-reviewed the exact 95-file Phase 8 implementation surface reconstructed
from the 24 phase summaries. The review covered the public joint and rope APIs,
typed joint solvers and four-body gear scatter, hook and lifecycle ownership,
protocol validation and result bounds, native and pinned C++ adapters,
comparison policy, accepted fixtures, oracle CI, and canonical-evidence
claims. The review included fixes `53a0b02` and `e0b5106` and the evidence and
documentation refresh at `e8daec8`. It was reopened for the post-verification
compatibility-ledger fix at `a109440` and the 14-file workspace-Clippy cleanup
at `c809a21`.

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

### GAP-08-01 — Resolved — Compatibility ledger is bound to current evidence

All 33 existing `platform_validated: evidenced` rows now name run
`29379350740`, its exact canonical and sanitizer identity records, and the
Phase 8 testing-policy anchor. A semantic before/after comparison confirms
that `a109440` changed only these reference arrays: no evidence status,
applicability, implementation claim, or maturity claim was promoted. The
superseded run and commit identifiers are absent from the authoritative
ledger.

The documentation checker now parses the ledger directly and enforces one
central exact-reference contract across every platform-validated row, together
with the expected count of 33. The command-level fixture copies the real ledger,
injects a stale canonical identity, and proves the check fails through the
dedicated `docs/phase8-evidence` category. This closes the verification gap
without weakening inventory generation or broadening the Phase 8 sign-off.

### Workspace-Clippy cleanup — Semantically equivalent

The `c809a21` structural refactor preserves all 17 joint mutation branches.
Each supported joint-kind set, public setter, `FloatBits` or vector conversion,
unsupported-kind action error, and setter-error mapping is unchanged. Gear
sources remain dependency-ordered, and coordinate observation still consumes
the live configured ratio.

`ActionReferences` forwards the same eight reference collections, while the
extracted gear-dependent collector preserves declaration order and both source
edges. Joint support, value validation, declaration-distinct mutation checks,
and dependency rules remain exhaustive and fail closed. Lifecycle validation
still runs before finite-value and family checks; checked ordinal conversion
now rejects impossible `usize` to `u32` overflow instead of truncating it.

Borrowed joint and schema snapshots are read-only and clone the same values
into results. Consolidated match arms preserve their prior cases. The
continuous-event diagnostic payload, diagnostic wrapper, and solver-failure
injections remain compiled for `differential-internals`; test-only continuous
candidate control remains available under `cfg(test)`. Only fields and branches
with no non-diagnostic consumer are absent from the ordinary library build.
The associated test refactors retain every mutation, topology, lifecycle,
schema, and first-divergence assertion.

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
- Repository documentation and the compatibility ledger consistently name the
  replacement run. The previous identity remains only in historical records
  and as a deliberate negative-test mutation.
- This run does not bind final cleanup commit `c809a21`. That is expected at
  this review checkpoint; exact-commit canonical and sanitizer evidence must be
  refreshed after this clean review before the updated code is signed off.

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
- `cargo test -p xtask --test docs_contract phase8_contract -- --nocapture`
  passed: 5 tests, including exact ledger evidence drift rejection.
- The ledger contains exactly 33 platform-validated rows, every row has the
  same four current evidence references, and no stale run or artifact identity.
- `cargo test -p liquidfun-test-protocol --all-features` passed: 122 unit and
  11 integration tests.
- Phase 8 differential executor and comparator targets passed: 16 tests.
- Joint-island, rigid-island, sleeping, and CCD focused targets passed: 38
  tests with `differential-internals` enabled.
- `cargo test -p liquidfun --lib --no-default-features --quiet` passed: 184
  tests, confirming the test-only control surface compiles without the
  differential feature.
- `cargo test -p xtask --test differential_cli --quiet` passed: 27 tests.
- `cargo clippy -p liquidfun --lib --no-default-features -- -D warnings`
  passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed.
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
