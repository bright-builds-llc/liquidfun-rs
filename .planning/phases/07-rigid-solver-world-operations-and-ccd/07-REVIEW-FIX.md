---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T13:40:20Z
iteration: 4
findings_in_scope: 4
fixed: 4
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes — Iteration 4

All four iteration-4 warnings are fixed in separate commits. The corrections make ray clipping evidence execution-derived, validate exact live checkpoint identities, localize checkpoint evidence to the checkpoint's own action window, and align the architecture/testing documentation with the implemented public differential contract.

## Findings

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| WR-12 | Warning | `d2039e2` | Adds adapter-produced `clipping_applied` evidence and uses equal-minimum ray-set semantics only when a valid clip callback actually shortened both executions; declared but unreached clip directives retain exhaustive multiset comparison. |
| WR-13 | Warning | `64cbc46` | Replays lifecycle actions through one shared implementation and requires each checkpoint's declared live body and fixture identities to equal the exact declaration-order live set, including body-destruction fixture cascades. |
| WR-14 | Warning | `b04720e` | Resolves checkpoint action/stage evidence inside the current checkpoint's local action window and adds a two-checkpoint mismatch/minimization regression that proves setup preservation through the second checkpoint's divergent action. |
| WR-15 | Warning | `85f68c1` | Updates architecture and testing contracts to distinguish exhaustive/filtered ray multisets, callback-applied clipping, declared-but-unreached clipping, and termination semantics, while excluding request-only callback directives and private separation state from result policy claims. |

## Implementation Evidence

- `RigidRayObservation::clipping_applied` is emitted by both adapters only when a matching `Clip` callback rule is invoked, included in the closed protocol schema, validated against hit identity and callback declarations, and consumed by the comparator before selecting clipping semantics.
- A regression declares a clip target outside the shortened interval and proves a differing nonminimum hit is still reported as a mismatch.
- `apply_lifecycle_action` now drives both checkpoint count replay and exact live-identity replay. Result validation rejects same-count stale body or fixture substitutions on either engine side.
- Checkpoint evidence uses the action window after the previous checkpoint. The second-checkpoint regression binds the mismatch to `phase7-action-20` / `phase7-adapter` and proves minimization retains its required setup prefix.
- `ARCHITECTURE.md` and `TESTING.md` now describe only implemented observables and policy semantics; request callback directives and private signed separation are explicitly outside the result contract.

## Verification

Each finding commit passed the required Rust pre-commit sequence in order:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

The final committed tree passed the full workspace gate:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`

Focused and cross-language evidence also passed:

- Phase 7 differential rigid-world integration suite: 29 passed.
- C++ `oracle-debug` configure and build: passed.
- CTest reference protocol suite: 1 of 1 passed.
- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot`: 9 required families matched.
- `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot`: 9 required families matched.
- `cargo xtask docs check`: 5 Phase 7 document contracts verified.
- `cargo xtask inventory check`: 177 compatibility rows verified.
- `git diff --check`: passed.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from the canonical tool pins, so this run remains correctly reported as supported D2 evidence; both oracle and native executions agreed at that tier.

## Worktree and Residual Risk

No iteration-4 finding was skipped. `.planning/config.json` remains the pre-existing user-owned modification with blob hash `621946b2b075747d8342124a8abb2226e77546ad`; it was never staged. This iteration-4 report intentionally remains uncommitted for the review workflow. The final worktree contains only that config modification and this report modification.

Expensive scheduled fuzzing, sanitizer, and randomized differential campaigns remain normal later evidence lanes rather than blockers for these focused warning corrections.
