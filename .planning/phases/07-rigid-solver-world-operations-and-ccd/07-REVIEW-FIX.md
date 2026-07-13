---
phase: 07-rigid-solver-world-operations-and-ccd
review_path: .planning/phases/07-rigid-solver-world-operations-and-ccd/07-REVIEW.md
fixed_at: 2026-07-13T12:37:35Z
iteration: 2
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 07 Review Fixes — Iteration 2

The remaining WR-09 warning from review iteration 2 is fixed in commit `d2c7346`. Minimized-regression replay now proves that its closed typed transform report is the exact output shape of a complete deterministic reducer run and that the accepted transformations reconstruct the staged request byte-for-byte from the checked-in source.

## Finding

| Finding | Severity | Commit | Resolution |
| --- | --- | --- | --- |
| WR-09 | Warning | `d2c7346` | Deserializes a closed `RigidScenarioTransform` report, validates the complete deterministic attempt stream and accepted sequence, reapplies accepted transforms through the reducer's strict typed decoder, and requires canonical reconstructed bytes to equal staged `request.jsonl` before any review state write. |

## Implementation Evidence

- `RigidScenarioTransform` now derives closed `Deserialize` with unknown fields denied.
- Reducer output and replay share `apply_rigid_scenario_transform` and `canonical_rigid_request_bytes`.
- `reconstruct_complete_rigid_minimization` regenerates each state's deterministic candidate order, requires the recorded attempt prefix through every acceptance, requires the complete final rejected-attempt tail, and reapplies every accepted transform to the checked-in typed source.
- Replay rejects malformed transforms, invented or unrelated transforms, reordered transforms, excess duplicates, invalid applications, and any canonical byte disagreement before `review.toml` can be written.
- The positive minimized-regression workflow still reviews successfully.
- Four adversarial real-binary regressions rewrite the transform report and then recompute both `report_sha256` and `candidate_sha256`, proving rejection occurs at the transform-provenance boundary rather than the outer hash guard.

## Verification

The committed tree passed the exact required Rust pre-commit sequence in order:

- `cargo fmt --all`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo build --all-targets --all-features`
- `cargo test --all-features`

The full workspace verification also passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-targets --all-features`
- `cargo test --workspace --all-features`
- `cargo test -p liquidfun-differential --test rigid_fixture_workflow` — 15 passed
- `git diff --check`

The final diff was reviewed for unrelated changes. `.planning/config.json` remains the pre-existing user-owned modification, target artifacts were not staged, and this iteration-2 report intentionally remains uncommitted for the review workflow.

## Residual Risk

No iteration-2 finding was skipped. Expensive scheduled fuzzing, sanitizers, and randomized differential campaigns remain normal later evidence lanes rather than blockers for this focused replay trust-boundary correction.
