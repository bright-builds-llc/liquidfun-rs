---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T15:38:14Z
iteration: 6
findings_in_scope: 5
fixed: 5
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes — Iteration 6

All five iteration-5 warnings are fixed in six atomic commits. Query and ray evidence is now independently validated against action-time live topology and callback rules, ray payloads and requests fail closed on invalid floating-point geometry, and arbitrary clip histories compare only within the exact final interval shared by both engines.

## Findings

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| WR-16 | Warning | `2c3d346`, `42ea28d` | Binds query occurrences and ray hits to live fixture-child topology, replays directives and completion, rejects post-destruction identities, and evaluates topology at each observation's action rather than at checkpoint end. |
| WR-17 | Warning | `82a0b90` | Rejects every non-finite ray hit point and normal component before completion-based canonicalization. |
| WR-18 | Warning | `e767dfa` | Exact-compares final intervals first, projects non-terminated hits into that interval, and compares the remaining records as a multiplicity-preserving multiset without inventing closest-hit semantics. |
| WR-19 | Warning | `06abde9` | Rejects positive and negative zero clip directives at both Rust and C++ request boundaries before traversal. |
| WR-20 | Warning | `ca5bde6` | Replays source-ordered direction and squared-length arithmetic in both decoders and rejects zero, non-finite, underflowed, and overflowed derived ray geometry before execution. |

## Implementation Evidence

- Independent result validation reconstructs checkpoint lifecycle state action by action. Query and ray fixture-child selectors must resolve against the topology live when the observation executes, including body-destruction fixture cascades.
- Query observations replay callback directives in occurrence order, reject occurrences after termination, and require the declared completion state to equal the replayed state. Ray observations retain the same fail-closed directive, completion, and exact final-interval replay.
- Every ray hit fraction, point coordinate, and normal coordinate is finite before any comparator projection can discard payload detail. Per-engine mutation tests cover NaN, positive infinity, and negative infinity for all four point/normal components.
- Non-terminated ray observations exact-compare their validated final interval, discard only hits beyond that interval, and compare all retained semantic records as a multiset. Terminated rays remain intentionally reduced to completion, exact final interval, and callback count after independent validation.
- Removing the obsolete equal-minimum identity rule closes the Phase 7 tolerance profile at 36 explicit fields. Its canonical SHA-256 is `59cf32e2564d857bbf56ec7e8423bd73046f4c7698f2e0e3eb83c5ea7ab2b86a`, and the accepted rigid-world request carries that hash.
- Both request decoders reject `Clip(+0.0)` and `Clip(-0.0)`. Native and compiled-C++ regressions use multiple fraction-zero candidates so rejection cannot be confused with accidental single-hit behavior.
- Both request decoders evaluate ray direction components, their squares, and the sum in source order. Regressions cover signed-zero endpoint equality, subnormal squared-length underflow, subtraction overflow, and finite-component squared-length overflow.
- Architecture, testing, xtask documentation contracts, accepted request provenance, and Phase 7 state metadata match the executable comparison and validation behavior.

## Verification

Each atomic commit passed the required Rust pre-commit sequence in order before commit:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

The final committed tree passed the full workspace sequence:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`

Focused Rust evidence passed:

- Phase 7 protocol rigid-world suite: 26/26 tests.
- Rigid-world fixture boundary suite: 2/2 tests.
- Protocol schema presentation suite: 4/4 tests.
- Typed Phase 7 policy suite: 6/6 tests.
- Differential rigid-world integration suite: 40/40 tests.
- Rigid fixture workflow integration suite: 15/15 tests.

Cross-language and repository evidence passed:

- Fresh `oracle-debug` configure and build succeeded after the adapter digest was regenerated from the final C++ sources.
- CTest reference protocol suite passed 1/1 compiled test.
- Rigid compare and replay each matched all 9 required families under `phase7-v1` at local D2-supported authority.
- `cargo xtask docs check` verified all 5 Phase 7 document contracts.
- `cargo xtask inventory check` verified 177 compatibility rows.
- `cargo xtask check` passed package isolation for 69 entries, protocol schema and fixture presentation, documentation, inventory, upstream identity, and provenance.
- GSD schema-drift verification reported no drift and no blocker.
- `git diff --check 43ce9d8..HEAD` and final `git diff --check` passed.

The first post-WR-16 integration run exposed that checkpoint-end topology incorrectly rejected an observation that preceded later teardown in the same window. Commit `42ea28d` corrected validation to replay lifecycle state at action time, and the complete final gate proves the combined behavior.

The first WR-20 C++ build attempt correctly rejected a stale configured adapter digest after the source changed. A fresh configure regenerated the identity; the subsequent build and newly compiled CTest run passed. No stale CTest output is counted as evidence.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8, so successful local compare and replay remain D2-supported evidence rather than canonical D1 authority.

## Worktree and Residual Risk

No iteration-6 finding was skipped. `.planning/config.json` remained user-owned, unstaged, and unedited throughout this fix run. Its final observed SHA-256 is `440f14fa5b03113fe46105f252bace03fa84094e2b862c9ec1757a855fca5eba`; its Git blob hash is `621946b2b075747d8342124a8abb2226e77546ad`. This iteration-6 report intentionally remains uncommitted for the review workflow.

The final worktree is expected to contain only `.planning/config.json` and this report as unstaged modifications. Expensive scheduled fuzzing, sanitizer, and randomized differential campaigns remain later evidence lanes rather than blockers for these focused boundary and comparator corrections.
