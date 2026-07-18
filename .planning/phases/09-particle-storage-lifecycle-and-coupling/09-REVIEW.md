---
phase: 09-particle-storage-lifecycle-and-coupling
reviewed: 2026-07-18T19:55:23Z
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
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 09: Code Review Report

**Reviewed:** 2026-07-18T19:55:23Z
**Depth:** standard
**Files Reviewed:** 100
**Status:** issues_found

## Summary

The iteration-3 standard review covered the same exact 100-file Phase 09 scope and inspected the new shared cross-run evaluator as a direct dependency of the in-scope validator. Commits `1e621f4` and `cb7397c` resolve both prior warnings in their principal behavior. The five cross-run witness families now have typed, branch-bound proof records with persisted payload paths and digests. Validation decodes those payloads, recomputes replay and D0 equality, debug/release equality, retained-mismatch signatures, semantic paths, and deliberate first divergence through the complete Phase 09 comparator. The corpus generator executes a second sanitizer oracle for sanitizer replay and independently resolves debug and release executables.

The prior authority issue is also resolved. Exactly four affected compatibility rows are demoted to `not_evidenced` with no platform references, the superseded run and artifact identities are explicitly denied and documented only as forensic history, generated `COMPATIBILITY.md` reflects the inventory, and the five deferred Phase 10 rows remain `not_evidenced` across all evidence dimensions.

CR-01 remains resolved: output and archive paths are checked component-by-component for symlinks, and the focused regression passed. IN-01 remains resolved: the obsolete constructor block is absent. One fail-closed evidence-integrity warning remains. The validator proves the contents referenced by each cross-run record, but it does not prove that payloads described as independent runs are distinct from the baseline or from one another.

The review applied the repo-local `AGENTS.md` guidance, `AGENTS.bright-builds.md`, `standards-overrides.md`, and the managed architecture, code-shape, verification, testing, and Rust standards.

## Warnings

### WR-01: Cross-run proof references can alias the same persisted result

**Files:** `crates/liquidfun-differential/src/rigid_world/phase9/evidence.rs:140-287`; `tools/xtask/src/phase9_evidence.rs:612-641`; `tools/xtask/src/phase9_evidence.rs:656-679`; `tools/xtask/tests/phase9_evidence_cli.rs:673-703`

**Issue:** The semantic evaluator verifies every referenced digest and recomputes each declared predicate, but neither it nor the manifest validator constrains the relationship among proof paths. A digest-recomputed manifest can point replay-native and replay-oracle at the baseline result paths, point debug and release at one shared oracle result, or point minimized and copied at one shared mutated result. `cross_run_payload_refs` and the expected-file-set construction deduplicate those aliases, so the artifact still satisfies the exact file set and all content predicates while no longer demonstrating independently executed or independently persisted results. The committed generator currently emits six appropriate proof files, so this is a validation-contract gap rather than evidence that the generated local corpus is wrong. The mutation regression also changes shared proof content; it does not exercise path aliasing or isolate the first-divergence record's path predicate from its shared minimization payload.

**Fix:** Define and validate a canonical proof-path topology per case. Require proof payloads below `cases/<case-id>/proofs/`, reject references to baseline request/result/comparison files, require replay-native and replay-oracle to be distinct paths, require debug and release to be distinct paths, and require minimized and copied to be distinct paths. Explicitly encode only intentional reuse, such as replay payloads also serving the D0 proof or minimized/copied payloads serving both mismatch proofs. Add digest- and identity-recomputed regressions that substitute baseline paths, alias each independent pair, and mutate only the first-divergence record's stored semantic path so each branch-specific predicate is reached.

## Verification

- The exact original scope remains 100 files; the new shared evaluator module was inspected only as the direct implementation dependency of the in-scope validator.
- Local schema-v3 evidence validation passed for the canonical and sanitizer directories with 7 cases, 58 bindings, all five typed proof records, and the complete comparator recomputed.
- The canonical and sanitizer semantic manifest digests match. Both identities include all six proof payloads. Sanitizer replay matches the sanitizer baseline oracle result, and the independently generated debug and release results match.
- `cargo test -p xtask --test phase9_evidence_cli` passed all 14 tests, including one digest-recomputed semantic mutation for each of the five cross-run proof families.
- `cargo test -p xtask --test inventory_cli` passed all 21 tests, including rejection of pre-WR-01 authority and protection of the five deferred Phase 10 rows.
- `cargo xtask inventory check` passed all 177 rows. Exactly the four reviewed Phase 09 rows are platform `not_evidenced`, their platform reference lists are empty, and generated `COMPATIBILITY.md` is consistent.
- `cargo xtask provenance check` passed.
- `cargo test -p liquidfun-differential --test phase9_corpus workflow_contract_rejects_symlinked_output_before_cleanup -- --exact` passed.
- `bash -n scripts/phase9-evidence.sh` and `git diff --check` passed before report creation. No source file was modified, no commit was created, and untracked `09-REVIEW-FIX.md` was preserved.

***

_Reviewed: 2026-07-18T19:55:23Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
