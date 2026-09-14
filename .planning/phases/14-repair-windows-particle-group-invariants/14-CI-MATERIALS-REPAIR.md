---
phase: 14-repair-windows-particle-group-invariants
type: ci-prerequisite-repair
status: locally-verified
---

# Cargo-only materials contract repair

## Failure boundary

Read-only inspection after Cargo CI run `34809888461` identified two later suites that unconditionally read the omitted native checkout. `producer_records_the_full_scoped_materials_identity` called `witness_materials_identity` on the real repository; `target_scoped_materials` called `resolve_declared_materials` on that same root before constructing its isolated fixture. Both implementations read declared file contents, including `third_party/liquidfun/liquidfun/Box2D/Box2D/CMakeLists.txt`, while Cargo CI explicitly uses `submodules: false`.

This is a statically established missing-input boundary, not a claim that these later tests ran in the failed CI attempt. Full native content validation remains required in its native evidence workflows.

## Bounded correction

- The Phase 13 test now uses a confined synthetic manifest with build-rule, generated-input, header, source, and compile-definition entries. Separate tests prove a known complete digest, unrelated-file exclusion, order/duplicate normalization, content sensitivity for every file-bearing kind, and missing-file rejection.
- The known synthetic SHA-256 was independently calculated with Python `hashlib`: sort distinct `(kind, identity)` pairs, then hash each UTF-8 kind and identity and applicable file bytes prefixed by an eight-byte big-endian length. Its digest is `9fca1c0db33ce9124311acd1cbff697c4e8f255e2dc5f78eae9435684cc112d4`.
- A separate checked-manifest test retains schema, target, preset, allowed-kind, and all 176 distinct identity assertions without claiming the native bytes are available or accepted.
- The Phase 9 test retains its existing self-contained derivation, unrelated/scoped mutation, and Git binding assertions; only its preceding real-native-root read and redundant broad count assertion are removed.

Production hashing/validation, checked manifests, canonical receipts, native source, and acceptance authority are unchanged. No test is skipped or ignored. Synthetic fixture success does not validate any real canonical digest or complete Phase 15 acceptance.

## Verification handoff

Source inspection and an independent Python digest/count check completed. The executor's `target/phase14-platform/local-attempt-20260914-isolation01/gate-02.log` records passing acceptance 31/31, evidence 24/24 and Phase 9 provenance 2/2 under empty native-source and repository-target overlays. Ordered fmt, workspace-wide Clippy, core build and all-feature core tests also pass (1,012 tests). No Cargo execution or commit was performed by the editing agent, avoiding contention with executor-owned gates. Fresh same-SHA complete Cargo CI remains required by Plan 14-04; local synthetic checks do not certify native evidence.
