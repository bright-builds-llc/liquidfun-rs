---
phase: 11-examples-headless-tooling-and-testbed
reviewed: 2026-07-22T16:31:07Z
depth: standard
files_reviewed: 181
files_reviewed_list:
  - .github/workflows/ci.yml
  - .github/workflows/oracle.yml
  - ARCHITECTURE.md
  - COMPATIBILITY.md
  - Cargo.lock
  - Cargo.toml
  - TESTING.md
  - UPSTREAM-CORPUS.md
  - crates/liquidfun-benchmarks/Cargo.toml
  - crates/liquidfun-benchmarks/benches/catalog.rs
  - crates/liquidfun-benchmarks/src/lib.rs
  - crates/liquidfun-benchmarks/tests/catalog_equivalence.rs
  - crates/liquidfun-differential/src/catalog_command.rs
  - crates/liquidfun-differential/src/catalog_command/parse.rs
  - crates/liquidfun-differential/src/catalog_command/render.rs
  - crates/liquidfun-differential/src/catalog_native.rs
  - crates/liquidfun-differential/src/catalog_native/capture.rs
  - crates/liquidfun-differential/src/catalog_native/executor.rs
  - crates/liquidfun-differential/src/comparison_model.rs
  - crates/liquidfun-differential/src/comparison_model/diff.rs
  - crates/liquidfun-differential/src/comparison_model/diff/primitives.rs
  - crates/liquidfun-differential/src/failure_bundle/catalog.rs
  - crates/liquidfun-differential/src/failure_bundle/catalog/replay.rs
  - crates/liquidfun-differential/src/fixtures.rs
  - crates/liquidfun-differential/src/fixtures/replay.rs
  - crates/liquidfun-differential/src/fixtures/replay/catalog.rs
  - crates/liquidfun-differential/src/lib.rs
  - crates/liquidfun-differential/src/main.rs
  - crates/liquidfun-differential/src/report.rs
  - crates/liquidfun-differential/src/rigid_world/phase10/native.rs
  - crates/liquidfun-differential/src/runner/catalog.rs
  - crates/liquidfun-differential/src/session.rs
  - crates/liquidfun-differential/src/session/backend.rs
  - crates/liquidfun-differential/src/session/state.rs
  - crates/liquidfun-differential/src/session/tests.rs
  - crates/liquidfun-differential/src/supervisor.rs
  - crates/liquidfun-differential/src/supervisor/catalog.rs
  - crates/liquidfun-differential/src/supervisor/catalog/protocol.rs
  - crates/liquidfun-differential/tests/catalog_failures.rs
  - crates/liquidfun-differential/tests/catalog_native.rs
  - crates/liquidfun-differential/tests/catalog_regressions.rs
  - crates/liquidfun-differential/tests/catalog_round_trip.rs
  - crates/liquidfun-differential/tests/comparison_model.rs
  - crates/liquidfun-differential/tests/fixtures/catalog/cases/particle-groups.jsonl
  - crates/liquidfun-differential/tests/fixtures/catalog/cases/queries-callbacks-mutations.jsonl
  - crates/liquidfun-differential/tests/fixtures/catalog/cases/rigid-joint-rope.jsonl
  - crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json
  - crates/liquidfun-differential/tests/headless_catalog.rs
  - crates/liquidfun-differential/tests/phase11_corpus.rs
  - crates/liquidfun-differential/tests/phase11_corpus/validation.rs
  - crates/liquidfun-test-protocol/Cargo.toml
  - crates/liquidfun-test-protocol/src/catalog.rs
  - crates/liquidfun-test-protocol/src/catalog/mapping.rs
  - crates/liquidfun-test-protocol/src/catalog/mapping/projection.rs
  - crates/liquidfun-test-protocol/src/catalog/model.rs
  - crates/liquidfun-test-protocol/src/catalog/model/identity.rs
  - crates/liquidfun-test-protocol/src/catalog/model/metadata.rs
  - crates/liquidfun-test-protocol/src/catalog/resolve.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/groups.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/joints.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/particles.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/queries_callbacks.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/rigid.rs
  - crates/liquidfun-test-protocol/src/catalog/scenarios/rope.rs
  - crates/liquidfun-test-protocol/src/catalog/wire.rs
  - crates/liquidfun-test-protocol/src/checkpoint.rs
  - crates/liquidfun-test-protocol/src/checkpoint/observation.rs
  - crates/liquidfun-test-protocol/src/checkpoint/primitive.rs
  - crates/liquidfun-test-protocol/src/codec.rs
  - crates/liquidfun-test-protocol/src/ids.rs
  - crates/liquidfun-test-protocol/src/lib.rs
  - crates/liquidfun-test-protocol/src/schema.rs
  - crates/liquidfun-test-protocol/src/schema/checkpoint.rs
  - crates/liquidfun-test-protocol/src/schema/rigid_world/phase10.rs
  - crates/liquidfun-test-protocol/src/schema/tests.rs
  - crates/liquidfun-test-protocol/tests/catalog_registry.rs
  - crates/liquidfun-test-protocol/tests/catalog_resolution.rs
  - crates/liquidfun-test-protocol/tests/checkpoint_protocol.rs
  - crates/liquidfun-testbed/CAPABILITY.md
  - crates/liquidfun-testbed/Cargo.toml
  - crates/liquidfun-testbed/src/app.rs
  - crates/liquidfun-testbed/src/app/state.rs
  - crates/liquidfun-testbed/src/capability.rs
  - crates/liquidfun-testbed/src/capability/render.rs
  - crates/liquidfun-testbed/src/controller_adapter.rs
  - crates/liquidfun-testbed/src/input.rs
  - crates/liquidfun-testbed/src/lib.rs
  - crates/liquidfun-testbed/src/main.rs
  - crates/liquidfun-testbed/src/screenshot.rs
  - crates/liquidfun-testbed/src/theme.rs
  - crates/liquidfun-testbed/src/ui.rs
  - crates/liquidfun-testbed/src/ui/about.rs
  - crates/liquidfun-testbed/src/ui/accessibility.rs
  - crates/liquidfun-testbed/src/ui/differences.rs
  - crates/liquidfun-testbed/src/ui/inspector.rs
  - crates/liquidfun-testbed/src/ui/layout.rs
  - crates/liquidfun-testbed/src/ui/overlays.rs
  - crates/liquidfun-testbed/src/ui/run_controls.rs
  - crates/liquidfun-testbed/src/ui/scenario_browser.rs
  - crates/liquidfun-testbed/src/ui/settings.rs
  - crates/liquidfun-testbed/src/ui/viewport.rs
  - crates/liquidfun-testbed/src/ui/viewport/draw.rs
  - crates/liquidfun-testbed/tests/app_shell.rs
  - crates/liquidfun-testbed/tests/capability.rs
  - crates/liquidfun-testbed/tests/controller_ui.rs
  - crates/liquidfun-testbed/tests/controller_ui/support.rs
  - crates/liquidfun-testbed/tests/visual_contract.rs
  - crates/liquidfun/Cargo.toml
  - crates/liquidfun/src/debug_draw.rs
  - crates/liquidfun/src/debug_draw/collector.rs
  - crates/liquidfun/src/debug_draw/collector/layers.rs
  - crates/liquidfun/src/debug_draw/collector/support.rs
  - crates/liquidfun/src/debug_draw/primitive.rs
  - crates/liquidfun/src/lib.rs
  - crates/liquidfun/src/particle/testdata/group-topology-witnesses.json
  - crates/liquidfun/src/world.rs
  - crates/liquidfun/src/world/diagnostics.rs
  - crates/liquidfun/src/world/observation.rs
  - crates/liquidfun/src/world/observation/profile.rs
  - crates/liquidfun/src/world/step.rs
  - crates/liquidfun/tests/debug_draw.rs
  - crates/liquidfun/tests/phase11_public_observability.rs
  - crates/liquidfun/tests/world_observations.rs
  - deny.toml
  - justfile
  - protocol/schemas/checkpoint-v1.schema.json
  - reference/artifacts/manifest.toml
  - reference/artifacts/phase10/group-topology-witnesses.provenance.json
  - reference/artifacts/phase11/exact-ref.json
  - reference/artifacts/phase11/scenario-mappings.json
  - reference/artifacts/phase9/lifecycle-contact-witnesses.provenance.json
  - reference/compatibility.json
  - reference/scenario-catalog.json
  - reference/upstream-corpus.json
  - scenarios/catalog/joint-rope-v1.json
  - scenarios/catalog/particle-group-v1.json
  - scenarios/catalog/rigid-stack-v1.json
  - scenarios/regressions/catalog-manifest.json
  - scripts/phase11-evidence.sh
  - target/phase11-evidence/phase11-canonical
  - target/phase11-evidence/phase11-sanitizer
  - target/phase11-evidence/run.json
  - tools/reference/CMakeLists.txt
  - tools/reference/adapter-inputs.txt
  - tools/reference/src/catalog_checkpoint.cpp
  - tools/reference/src/catalog_run.cpp
  - tools/reference/src/catalog_run_decode.cpp
  - tools/reference/src/catalog_run_session.cpp
  - tools/reference/src/main.cpp
  - tools/reference/src/protocol.cpp
  - tools/reference/tests/protocol_tests.cpp
  - tools/xtask/src/differential.rs
  - tools/xtask/src/differential/catalog.rs
  - tools/xtask/src/inventory.rs
  - tools/xtask/src/inventory/corpus.rs
  - tools/xtask/src/inventory/corpus/discovery.rs
  - tools/xtask/src/inventory/corpus/discovery/source.rs
  - tools/xtask/src/inventory/corpus/model.rs
  - tools/xtask/src/inventory/corpus/report.rs
  - tools/xtask/src/inventory/corpus/validation.rs
  - tools/xtask/src/inventory/validation.rs
  - tools/xtask/src/inventory/validation/phase11.rs
  - tools/xtask/src/main.rs
  - tools/xtask/src/package.rs
  - tools/xtask/src/package/metadata.rs
  - tools/xtask/src/phase11_evidence.rs
  - tools/xtask/src/phase11_evidence/authority.rs
  - tools/xtask/src/phase11_evidence/content.rs
  - tools/xtask/src/provenance/artifact.rs
  - tools/xtask/tests/catalog_cli.rs
  - tools/xtask/tests/corpus_closure.rs
  - tools/xtask/tests/corpus_discovery.rs
  - tools/xtask/tests/corpus_model.rs
  - tools/xtask/tests/fixtures/corpus/invalid-disposition.json
  - tools/xtask/tests/fixtures/corpus/valid-minimal.json
  - tools/xtask/tests/inventory_cli.rs
  - tools/xtask/tests/inventory_cli/phase11.rs
  - tools/xtask/tests/package_cli.rs
  - tools/xtask/tests/phase11_evidence_cli.rs
  - tools/xtask/tests/phase11_evidence_cli/workflow.rs
findings:
  critical: 0
  warning: 2
  info: 1
  total: 3
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-07-22T16:31:07Z
**Depth:** standard
**Files Reviewed:** 181
**Status:** issues_found

## Summary

The exact Tier-2 scope was derived from `key-files.created` and `key-files.modified` in all 29 Phase 11 summaries, filtered by the workflow exclusions and current path existence, then sorted and deduplicated. The resulting scope contains 181 entries, including the two ignored evidence directories named by Plan 11-22.

The review applied the repository-local guidance, Bright Builds sidecar, active standards override file, and the architecture, code-shape, verification, testing, and Rust standards. The Phase 11 implementation generally uses strong typed boundaries, bounded inputs, semantic identities, and focused regression coverage. Two correctness/security risks and one report-integrity inconsistency remain.

## Warnings

### WR-01: Renderer capability claims are disconnected from rendered output

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun-testbed/src/capability/render.rs:70-90`

**Issue:** Most evidence values used to pass the renderer capability matrix are assigned as fixed constants after drawing rather than measured from the generated frame. For example, contact points, normals, particle contacts, AABBs, profile names, overlay pairs, panel count, focus width, and dense rows remain at their passing values even if the corresponding draw calls are removed or broken. Two additional capabilities are unconditionally marked true in `capability/report.rs:243-251`. The current tests assert only the resulting pass flags and regular output files, so a visual regression can continue to authorize the renderer selection.

**Fix:** Build the scene from typed primitives that increment counters as they are actually emitted, or inspect the resulting pixels/semantic draw list for every claimed capability. Add a regression that removes or suppresses one required element and proves the matrix fails; alternatively pin and verify reviewed frame hashes in the executable contract.

### WR-02: Package extraction does not consume the archive instance that was validated

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/tools/xtask/src/package.rs:107-125`

**Issue:** `inspect_archive` opens and validates one archive handle, returns only its path list, and then `extract_archive` reopens the pathname. If that file is replaced between the two operations, extraction and subsequent build/test can consume bytes that never passed the entry-type, path, content, count, or size checks. This undercuts the verifier's fail-closed archive boundary and is especially relevant because an explicit archive pathname can be supplied through `LIQUIDFUN_XTASK_TEST_PACKAGE_ARCHIVE`.

**Fix:** Open the archive once and bind validation to extraction, or copy/read the archive into an immutable bounded temporary file/byte buffer, validate those exact bytes, rewind, and extract from the same object. A regression should replace the source path after validation and prove the extracted bytes cannot change.

## Info

### IN-01: Persisted capability report records a null digest while the returned report records a hash

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun-testbed/src/capability.rs:178-194`

**Issue:** The report is serialized and written while `report_sha256` is `None`, then the in-memory object is updated with the hash of those bytes. Current generated files therefore contain `"report_sha256": null`, while a caller that serializes the returned value sees a different report with a populated field. The field does not provide a consistent persisted integrity contract.

**Fix:** Define the digest contract explicitly. Either omit the self-digest field from the report and publish a companion digest, or hash a clearly named canonical payload that excludes the digest field and then write the final report containing that payload digest. Add a test that reads the persisted report and compares it with the returned contract.

***

_Reviewed: 2026-07-22T16:31:07Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_

