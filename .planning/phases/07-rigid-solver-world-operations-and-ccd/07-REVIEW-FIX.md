---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T16:46:20Z
iteration: 7
findings_in_scope: 3
fixed: 3
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes — Iteration 7

All three iteration-6 warnings are fixed in four atomic commits. The C++ request boundary now rejects nonexistent selector children before execution, duplicate ray hits compare through deterministic maximum matching rather than order-sensitive first-fit pairing, and final-interval projection retains every hit inside the registered ray-fraction boundary tolerance.

## Findings

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| WR-21 | Warning | `777abca`, `75e1021` | Resolves query and ray selectors against live declared fixtures during C++ timeline validation, rejects children outside each shape's topology before adapter execution, and keeps the compiled regression C++17-clean. |
| WR-22 | Warning | `c0475df` | Canonically groups duplicate hits by semantic identity and uses deterministic maximum bipartite matching over all five registered numeric-field policies. |
| WR-23 | Warning | `6b7b958` | Exact-compares final interval evidence, then retains hits at or below the interval or within the registered four-ULP fraction-policy boundary band before multiset comparison. |

## Implementation Evidence

- C++ timeline validation tracks declared fixture shape topology alongside live fixture identity. Both terminating query and terminating ray selectors reject `child_index: 1` for the current single-child circle and polygon shapes before world execution.
- Compiled C++ protocol tests exercise both invalid selector families. A real-process Rust regression proves that the oracle emits no result record, exits nonzero, and reports the stable invalid-child diagnostic.
- Ray hits are sorted canonically and partitioned by `(fixture_id, child_index)`. Each identity group uses augmenting-path maximum matching, where an edge exists only when fraction, point, and normal fields all satisfy their named Phase 7 policies.
- A perfect matching is order-independent. When none exists, the comparator reports a stable canonical numeric divergence and fails closed if a supposedly unmatched pair is still policy-compatible. Duplicate identity multiplicity remains exact.
- Duplicate-hit regressions cover the adversarial reassignment case in both callback orders, a stable no-perfect-matching fraction diagnostic, and duplicate multiplicity loss.
- Final maximum-fraction bits remain exact evidence. For non-terminated rays, projection retains raw `fraction <= final` hits and hits matching the exact final bits under `rigid_world.phase7.ray.fraction`; only hits proven beyond both tests are discarded.
- Boundary regressions cover the exact boundary and one through four ULPs in both engine directions, five ULPs in both directions, payload differences discarded beyond the boundary band, and payload differences compared when a hit remains inside the band.
- Architecture, testing, and executable xtask documentation contracts describe the tolerance-aware final-interval projection.

## Verification

The required Rust pre-commit sequence was run in order for the scoped commits:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

WR-21, WR-23, and the C++17 portability follow-up passed the literal sequence before commit. During WR-22, macOS repeatedly stalled repository-path test executables in `_dyld_start`; the same all-features suite first passed from fresh `/tmp` artifacts, then the literal four-command sequence passed immediately on committed `c0475df` through a temporary ignored target symlink before work continued. No history was rewritten.

The final Rust source and documentation tree passed the full workspace sequence:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`

Focused Rust evidence passed:

- Phase 7 protocol rigid-world suite: 26/26 tests.
- Rigid-world fixture boundary suite: 2/2 tests.
- Protocol schema presentation suite: 4/4 tests.
- Typed Phase 7 policy suite: 6/6 tests.
- Differential rigid-world integration suite: 46/46 tests.
- Rigid fixture workflow integration suite: 15/15 tests.
- Real-oracle invalid selector-child process regression: 1/1 test.

Cross-language and repository evidence passed:

- Fresh `oracle-debug` configure rebuilt the final reference executable and compiled protocol-test target.
- CTest reference protocol suite passed 1/1.
- Rigid compare and replay each matched all 9 required families under `phase7-v1` at local D2-supported authority.
- `cargo xtask docs check` verified all 5 Phase 7 document contracts.
- `cargo xtask inventory check` verified 177 compatibility rows.
- `cargo xtask check` passed package isolation for 69 entries, protocol schema and fixture presentation, documentation, inventory, upstream identity, and provenance.
- GSD schema-drift verification reported no drift and no blocker.
- `git diff --check 86c6be7..HEAD` and final pre-report `git diff --check` passed.

The first fresh C++ build exposed that the WR-21 test lambda captured a structured-binding name, which is a C++20 extension under the repository's C++17 target. Commit `75e1021` replaced that capture with ordinary pair bindings; the freshly rebuilt target and CTest then passed. No stale CTest result is counted as evidence.

The macOS loader intermittently stalled binaries executed from the repository target. Final workspace testing therefore used fresh Cargo artifacts under `/tmp` while retaining the real repository target for fixture roots and C++ provenance. A whole-target symlink was rejected after it correctly changed canonical fixture paths and caused provenance digest failures; rebuilding the stale xtask integration artifact and rerunning with the original fixture root produced a complete zero-exit workspace result.

The local CMake 3.27.9 and Apple Clang 21.0.0 differ from canonical CMake 4.3.3 and Clang 22.1.8, so successful local compare and replay remain D2-supported evidence rather than canonical D1 authority.

## Worktree and Residual Risk

No iteration-7 finding was skipped. `.planning/config.json` remained user-owned, unstaged, and unedited throughout this fix run. Its final observed SHA-256 is `440f14fa5b03113fe46105f252bace03fa84094e2b862c9ec1757a855fca5eba`; its Git blob hash is `621946b2b075747d8342124a8abb2226e77546ad`. This iteration-7 report intentionally remains uncommitted for the review workflow.

The final worktree is expected to contain only `.planning/config.json` and this report as unstaged modifications. Expensive scheduled fuzzing, sanitizer, randomized differential, canonical-toolchain, and wider platform campaigns remain later evidence lanes rather than blockers for these focused parser and comparator corrections.
