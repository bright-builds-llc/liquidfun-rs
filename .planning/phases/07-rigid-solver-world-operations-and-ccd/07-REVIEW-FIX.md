---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T14:27:31Z
iteration: 5
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes — Iteration 5

The remaining WR-12 warning is fixed in one atomic commit. Ray evidence now records the exact effective callback interval rather than a reached-clip boolean, so a reached no-op `Clip(1.0)` remains exhaustive and a later clip cannot silently expand an interval that was already shortened.

## Finding

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| WR-12 | Warning | `5812367` | Replaces `clipping_applied` with exact `final_max_fraction_bits`, replays callback-ordered directives during independent result validation, rejects interval expansion in both adapters, and selects closest-hit set comparison only after a strict interval reduction on both sides. |

## Implementation Evidence

- `RigidRayObservation` records the exact final maximum-fraction bits, initialized from the shared exact `1.0` constant. The closed schema, Phase 7 policy, accepted request policy hash, and documentation all describe that field.
- Native and C++ adapters maintain a monotone callback interval. Equal clips are no-ops, strict decreases replace the exact recorded bits, and expanding clips fail closed. The C++ continue path preserves the existing interval instead of returning a value that can re-expand the upstream dynamic-tree bound.
- Independent result validation replays hit identities and directive rules in callback order, rejects hits outside the current interval, rejects post-termination hits and expanding clips, and requires completion plus the recorded final interval to match replay.
- The comparator exact-compares the validated final interval before ray-hit semantics. Exhaustive and reached `Clip(1.0)` results use multiplicity-preserving multiset comparison; only two strictly shortened intervals use closest-hit comparison with equal-minimum identities as a set.
- Regressions prove a reached no-op clip cannot hide a nonminimum mismatch, final-interval disagreement is the first structural divergence, inconsistent result evidence is rejected, and multiple clips cannot expand a previously shortened interval in native Rust or the C++ oracle.

## Verification

The atomic finding commit passed the required Rust pre-commit sequence in order after the final documentation-contract correction:

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

- Phase 7 protocol rigid-world suites: 24 unit and 2 fixture tests passed.
- Phase 7 differential rigid-world integration suite: 33 of 33 tests passed, including all new WR-12 regressions.
- Protocol schema presentation suite: 4 of 4 byte-stability and closed-schema tests passed.
- The typed Phase 7 policy closes 37 explicit fields with canonical SHA-256 `a28360556d0339627ec26dc988fcb5585d12e14ba88ecb801c9cf9bdb1a193fe`.
- C++ `oracle-debug` configure and build passed; CTest reference protocol suite passed 1 of 1.
- `cargo xtask differential compare --scenario rigid-world --preset oracle-debug --session-profile one-shot` matched all 9 required families.
- `cargo xtask differential replay --scenario rigid-world --preset oracle-debug --session-profile one-shot` matched all 9 required families.
- `cargo xtask docs check` verified all 5 Phase 7 document contracts.
- `cargo xtask inventory check` verified 177 compatibility rows.
- `cargo xtask check` passed package isolation, schema and fixture drift, documentation, inventory, upstream identity, and provenance checks.
- `git diff --check` passed.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from the canonical tool pins, so the successful compare and replay remain correctly reported as supported D2 evidence rather than canonical D1 authority.

## Worktree and Residual Risk

No iteration-5 finding was skipped. `.planning/config.json` remains the pre-existing user-owned modification with blob hash `621946b2b075747d8342124a8abb2226e77546ad`; it was never staged. This iteration-5 report intentionally remains uncommitted for the review workflow. The final worktree contains only that config modification and this report modification.

Expensive scheduled fuzzing, sanitizer, and randomized differential campaigns remain normal later evidence lanes rather than blockers for this focused warning correction.
