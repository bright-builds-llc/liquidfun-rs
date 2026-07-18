---
phase: 09-particle-storage-lifecycle-and-coupling
reviewed: 2026-07-18T17:35:51Z
depth: standard
files_reviewed: 100
files_reviewed_list:
  - .codex/tasks/todo.md
  - .github/workflows/oracle.yml
  - COMPATIBILITY.md
  - Cargo.toml
  - TESTING.md
  - crates/liquidfun-differential/src/rigid_evidence/phase7.rs
  - crates/liquidfun-differential/src/rigid_world.rs
  - crates/liquidfun-differential/src/rigid_world/phase9.rs
  - crates/liquidfun-differential/src/rigid_world/phase9/comparator.rs
  - crates/liquidfun-differential/tests/fixtures/rigid_world/phase9/phase9-v1.json
  - crates/liquidfun-differential/tests/particle_oracle.rs
  - crates/liquidfun-differential/tests/particle_protocol.rs
  - crates/liquidfun-differential/tests/phase9_corpus.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase9.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/types.rs
  - crates/liquidfun-test-protocol/src/scenario/rigid_world/validation.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world/phase9.rs
  - crates/liquidfun/Cargo.toml
  - crates/liquidfun/src/arena.rs
  - crates/liquidfun/src/error.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/particle.rs
  - crates/liquidfun/src/particle/body_contact.rs
  - crates/liquidfun/src/particle/buffer.rs
  - crates/liquidfun/src/particle/contact.rs
  - crates/liquidfun/src/particle/definition.rs
  - crates/liquidfun/src/particle/editor.rs
  - crates/liquidfun/src/particle/force.rs
  - crates/liquidfun/src/particle/lifetime.rs
  - crates/liquidfun/src/particle/lifetime/tests.rs
  - crates/liquidfun/src/particle/proxy.rs
  - crates/liquidfun/src/particle/query.rs
  - crates/liquidfun/src/particle/statistics.rs
  - crates/liquidfun/src/particle/storage.rs
  - crates/liquidfun/src/particle/storage/editor_tests.rs
  - crates/liquidfun/src/particle/storage/identity.rs
  - crates/liquidfun/src/particle/storage/lane_inventory.rs
  - crates/liquidfun/src/particle/storage/lanes.rs
  - crates/liquidfun/src/particle/storage/permutation.rs
  - crates/liquidfun/src/particle/storage/permutation/tests.rs
  - crates/liquidfun/src/particle/storage/properties.rs
  - crates/liquidfun/src/particle/storage/properties/lifecycle_model.rs
  - crates/liquidfun/src/particle/storage/properties/permutation_model.rs
  - crates/liquidfun/src/particle/storage/validation.rs
  - crates/liquidfun/src/particle/view.rs
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/config.rs
  - crates/liquidfun/src/world/object.rs
  - crates/liquidfun/src/world/particle_coupling.rs
  - crates/liquidfun/src/world/particle_lifecycle.rs
  - crates/liquidfun/src/world/particle_object.rs
  - crates/liquidfun/src/world/query.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/particle_body_contacts.rs
  - crates/liquidfun/tests/particle_buffers.rs
  - crates/liquidfun/tests/particle_contacts.rs
  - crates/liquidfun/tests/particle_creation_eviction.rs
  - crates/liquidfun/tests/particle_definitions.rs
  - crates/liquidfun/tests/particle_forces_statistics.rs
  - crates/liquidfun/tests/particle_lifecycle.rs
  - crates/liquidfun/tests/particle_lifetimes.rs
  - crates/liquidfun/tests/particle_objects.rs
  - crates/liquidfun/tests/particle_permutation_coherence.rs
  - crates/liquidfun/tests/particle_queries.rs
  - crates/liquidfun/tests/particle_step_guards.rs
  - crates/liquidfun/tests/particle_views.rs
  - crates/liquidfun/tests/particle_zombie_authority.rs
  - deny.toml
  - protocol/schemas/scenario-v1.schema.json
  - protocol/schemas/trace-v1.schema.json
  - reference/artifacts/phase9/lifecycle-contact-witnesses.json
  - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
  - reference/compatibility.json
  - reference/source-map.toml
  - scripts/phase9-evidence.sh
  - target/phase9-evidence/phase9-canonical/identity.json
  - target/phase9-evidence/phase9-sanitizer/identity.json
  - target/phase9-evidence/run.json
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/phase9_lifecycle_contact_witness.cpp
  - tools/reference/src/rigid_world.cpp
  - tools/reference/src/rigid_world.hpp
  - tools/reference/src/rigid_world_decode.hpp
  - tools/reference/src/rigid_world_phase9_decode.hpp
  - tools/reference/src/rigid_world_phase9_execute.hpp
  - tools/reference/tests/sanitizer_scope.cmake
  - tools/xtask/src/inventory/validation.rs
  - tools/xtask/src/main.rs
  - tools/xtask/src/phase9_evidence.rs
  - tools/xtask/src/provenance.rs
  - tools/xtask/src/provenance/phase9_witness.rs
  - tools/xtask/src/upstream.rs
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/phase9_evidence_cli.rs
  - tools/xtask/tests/upstream_cli.rs
findings:
  critical: 1
  warning: 1
  info: 1
  total: 3
status: issues_found
---

# Phase 09: Code Review Report

**Reviewed:** 2026-07-18T17:35:51Z
**Depth:** standard
**Files Reviewed:** 100
**Status:** issues_found

## Summary

The standard-depth review covered the exact 100-file Tier 2 scope reconstructed from all 29 Phase 09 summary frontmatter blocks. Plans 09-25 through 09-29 materially close the two stale verification gaps: retained Phase 6-8 comparison now runs before particle comparison, the hermetic fake compile database is self-contained, the corpus carries 58 typed semantic bindings, replacement run `29652578231` is pinned while runs `29439515367`, `29583793056`, and `29625083184` plus artifact `8423580554` are denied, generated compatibility claims are deterministic, and all five Phase 10 rows remain `not_evidenced`.

One critical local path-safety defect remains in the evidence runner: a symlinked output path under `target/` reaches the destructive cleanup before any typed validation. One evidence-integrity warning also remains because the reusable validator does not independently resolve and evaluate most bindings against their exact action/checkpoint observation. The obsolete commented constructor block reported previously is still present.

The review applied the repo-local `AGENTS.md` guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the managed architecture, code-shape, verification, testing, and Rust standards.

## Critical Issues

### CR-01: Evidence cleanup follows symlinked output roots outside `target/`

**Files:** `scripts/phase9-evidence.sh:18-33`; `tools/xtask/src/phase9_evidence.rs:276-288`; `tools/xtask/src/phase9_evidence.rs:951-970`

**Issue:** The shell runner validates `output_dir` lexically, then calls `mkdir -p`, `rm -rf "$output_dir/cases"`, and writes logs before the Rust validator runs. Neither the shell guard nor the Rust `checked_relative_path`/`read_regular_file` helpers reject symlinks in ancestor components. A path such as `target/probe/canonical` may therefore be a symlink to an arbitrary directory; the cleanup resolves through it and recursively deletes that external directory's `cases` child. A safe review probe confirmed the defect: a marker under the symlink target was deleted before a deliberately failing fake `cargo` command ran. The same missing component walk lets exact-ref archive paths read through symlinked ancestors.

**Fix:** Resolve the repository and `target/` roots once, walk every existing path component with `symlink_metadata`, reject all symlink components, and require the canonical parent to remain beneath the canonical `target/` root before any `mkdir`, delete, read, or write. Create a new output directory without following links, then reopen/validate it before cleanup. Add regressions for a symlinked final output directory, a symlinked ancestor, and a symlinked archive ancestor; all must fail before changing the symlink target.

## Warnings

### WR-01: Exact-ref validation does not prove most witness bindings against their bound observations

**Files:** `tools/xtask/src/phase9_evidence.rs:488-539`; `tools/xtask/src/phase9_evidence.rs:589-657`; `tools/xtask/tests/phase9_evidence_cli.rs:384-415`

**Issue:** `validate_manifest` proves that action/checkpoint indices are in range and that the assertion enum declares the expected observation kind, but it never reconstructs the exact action-to-observation slot used by the corpus. `validate_semantic_outcomes` independently checks only collision energy and stuck candidates, and even those checks accept any statistics observation in the checkpoint rather than the observation bound by `action_index`. Finite/infinite/equal lifetime, strict contact, listener/filter, replay/minimization/first-divergence/D0/debug-release, and ordinary observed-semantic assertions are not evaluated against downloaded result values. The validator also trusts the stored `complete-comparison.json` value instead of rerunning the complete comparator over the decoded native/oracle pair.

An in-range wrong action or checkpoint binding can therefore pass after its witness and semantic-manifest digests are recomputed. The existing corruption test changes an index to `usize::MAX` without recomputing those digests, so it exercises digest/range rejection rather than exact semantic binding.

**Fix:** Move the corpus's action-to-observation resolver and semantic assertion evaluator into production code shared by corpus generation and `phase9-evidence`. Resolve each binding to its exact action, checkpoint, and observation; verify the actual observation variant; evaluate every assertion against both decoded results; and rerun `compare_complete_phase9_rigid_world_results` to derive the comparison outcome. Add digest-recomputed mutations for an in-range wrong action, wrong checkpoint, wrong observation, false lifetime/contact/listener/filter assertion, and divergent native/oracle result pair.

## Info

### IN-01: Obsolete constructor code remains commented out

**File:** `crates/liquidfun/src/particle/storage.rs:416-425`

**Issue:** A block-commented call to `Self::from_owned_lanes` remains between `commit_create` and the active constructor. It has no runtime effect but obscures the live construction path.

**Fix:** Delete the commented block; version control already preserves the prior implementation.

## Verification

- Reconstructed scope: 29 summaries, 331 extracted entries, 325 after planning exclusions, 131 missing/deleted paths removed, exactly 100 sorted unique existing files.
- `cargo xtask phase9-evidence validate --mode exact-ref ...` passed with all three historical/failed runs denylisted and reported 7 cases plus 58 semantic bindings.
- `cargo test -p xtask --test phase9_evidence_cli` passed 9 tests.
- `cargo test -p xtask --test inventory_cli` passed 21 tests.
- `cargo test -p liquidfun-differential --test phase9_corpus` passed 25 tests with 1 explicit regeneration test ignored.
- `cargo test -p liquidfun-differential --test particle_oracle` passed 13 tests.
- `cargo test -p liquidfun-differential --test particle_protocol` passed 25 tests.
- The safe symlink probe reproduced CR-01 and left the worktree clean after cleanup.
- `git diff --check` passed before report creation. No source file was modified and no commit was created.

***

_Reviewed: 2026-07-18T17:35:51Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
