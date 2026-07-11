---
phase: 05-shapes-and-collision-foundation
plan: "07"
subsystem: collision-differential-evidence
tags: [rust, cpp, protocol, collision, differential-testing, determinism]
requires:
  - phase: 05-shapes-and-collision-foundation
    plan: "03"
    provides: checked distance cache and source-ordered GJK
  - phase: 05-shapes-and-collision-foundation
    plan: "04"
    provides: seven supported manifold pair families and reversals
  - phase: 05-shapes-and-collision-foundation
    plan: "05"
    provides: dynamic tree, broad phase, and refilter behavior
  - phase: 05-shapes-and-collision-foundation
    plan: "06"
    provides: checked TOI kernels and bounded diagnostics
provides:
  - typed accepted/rejected collision declarations and result outcomes
  - topology-safe semantic cache replay with exact source precedence
  - fail-closed registry and fixed corpus for all 78 required witness families
  - complete native Rust and pinned C++ collision-probe execution
  - debug/release comparison, replay, and D0 two-run byte identity
affects: [05-08-compatibility-signoff, phase-6-rigid-world]
tech-stack:
  added: []
  patterns: [declaration-first comparison, semantic cache fingerprint, closed witness registry, three-TU oracle identity]
key-files:
  created:
    - crates/liquidfun/src/collision/differential/cache_replay.rs
    - crates/liquidfun/src/collision/distance/replay.rs
    - crates/liquidfun-test-protocol/src/scenario/collision_probe/witness_registry.rs
    - crates/liquidfun-differential/src/collision_probe/support.rs
    - tools/reference/src/collision_probe_shapes.hpp
    - tools/reference/src/collision_probe_spatial.hpp
  modified:
    - crates/liquidfun-differential/src/collision_probe.rs
    - crates/liquidfun-differential/src/collision_evidence.rs
    - crates/liquidfun-differential/src/supervisor/collision_probe.rs
    - tools/reference/src/collision_probe.cpp
    - protocol/fixtures/accepted/collision-probe-request.jsonl
key-decisions:
  - "Validate each engine result against the declared expected outcome before comparing engines."
  - "Replay cache state only from semantic shape-child fingerprints, checked support pairs, and exact metric bits."
  - "Keep C++ compile-command identity at exactly collision_probe.cpp, math_probe.cpp, and protocol_bits.cpp while splitting implementation into included cohesive headers."
requirements-completed:
  - COLL-02
  - COLL-03
  - COLL-04
  - COLL-05
  - COLL-06
  - COLL-07
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 5-2026-07-11T14-53-25
generated_at: 2026-07-11T23:30:00Z
completed: 2026-07-11
---

# Phase 5 Plan 07: Closed Collision Differential Evidence Summary

**The fixed Phase 5 corpus now validates all 78 witness families through identical native Rust and pinned C++ semantic operations, including rejected shapes, topology-safe cache replay, complete pair/tree/broad-phase/TOI handling, and byte-identical D0 replay.**

## Accomplishments

- Added required `CollisionWitnessFamily` and `CollisionExpectedOutcome` fields to every case, with fail-closed missing-family, deletion, unknown-family, and operation/outcome validation.
- Split the bounded protocol into cohesive result, type, validation, tree-validation, shape-validation, test, and witness-registry modules while preserving existing public paths.
- Made accepted shape decoding call the production `liquidfun` constructors and retained only bounded raw shape candidates for declared rejected construction cases.
- Added declaration-first comparator logic: Rust must match the declaration, then C++ must match it, before aligned engine comparison begins.
- Added tagged accepted/rejected supervisor decoding without weakening record, sequence, policy, horizon, collection, timeout, or reset validation.
- Added semantic proxy fingerprints and a feature-gated cache replay seam with exact A/B fingerprint, count, A-index, B-index, duplicate, metric, single-point, ratio, epsilon precedence.
- Added exact cold, Used, one-point Used, Reset, Rejected, and compound-precedence evidence in the fixed corpus.
- Completed native and C++ execution for all four shape kinds, checked chain children, all seven pair families and reversals, real dynamic-tree and broad-phase operations, and all TOI shape-child combinations.
- Preserved exactly three result-affecting C++ compile commands while adding every result-affecting adapter source/header to the content identity manifest.
- Split large Rust and C++ integration surfaces into cohesive child modules without changing protocol versions, adding a second harness, or widening the default public engine API.

## Task Commit

1. **Task 5: Close rejected-shape, cache-replay, and required-family D-22 evidence gaps** — `df6181f` (`feat`)

## Verification

- All 11 feature-gated cache replay precedence tests passed.
- Production circle, edge, polygon, and chain validation boundary test passed.
- Protocol expected-rejection, 78-family completeness, per-family deletion, policy, schema, fixture, and byte-stability tests passed.
- All 10 collision differential integration tests passed, including declaration disagreement and cache outcome/precedence cases.
- Native result-source identity mutation tests passed for the executor, support child, differential adapter, cache replay children, and production collision sources.
- Xtask required-family preflight and exact three-translation-unit compile-database identity tests passed.
- `cargo check -p liquidfun --no-default-features`, no-default compile-fail doctests, warning-denied default docs, default feature-tree exclusion, and package absence scans passed.
- `cargo xtask package verify` built and tested the 50-entry package outside the repository.
- `cargo xtask provenance check` passed for pinned revision `7f20402173fd143a3988c921bc384459c6a858f2`.
- Oracle debug and release configure/build plus CTest passed under strict C++ warnings.
- Debug and release comparison each matched all 78 ordered cases under `phase5-v1`.
- Debug replay matched all 78 ordered cases and two determinism runs were byte-identical.
- Mandatory `cargo fmt --all`, strict all-target/all-feature Clippy, all-target/all-feature build, and full all-feature tests passed in order.
- `git diff --check`, policy wildcard, feature-tree, private-storage vocabulary, package-isolation, and generated-artifact checks passed.

## Evidence Authority

- Local Apple Clang 21.0.0 and CMake 3.27.9 differ from the canonical Clang 22.1.8 and CMake 4.3.3 pins, so the successful local cross-language comparison remains truthful supported evidence rather than a false D1 promotion.
- D0 authority is established by two byte-identical debug oracle runs over the complete fail-closed fixed corpus.

## Public Surface

- `differential-internals` remains non-default and is enabled only by private workspace differential tooling.
- Ordinary `liquidfun` dependency resolution, docs, and packaged consumer builds do not enable the replay adapter.
- No raw proxy, simplex, cache mutation, node coordinate, packed contact key, pointer identity, or C++ runtime dependency entered the public engine API.

## Next Plan Readiness

- Plan 05-08 can consume the complete fixed evidence, compatibility disposition, and truthful D0/D2 authority without reopening D-22 witness gaps.
- Phase 6 can build rigid-world contact lifecycle on the now differentially demonstrated Phase 5 substrate.

## Self-Check: PASSED

- Task commit `df6181f` exists.
- Every required witness family occurs in the checked-in request and deleting any one fails before oracle execution.
- Debug/release comparison, replay, determinism, package, provenance, CTest, and mandatory Rust gates all passed.
- `.planning/STATE.md`, `.planning/config.json`, and `05-07-PLAN.md` remained outside the Task 5 commit.
- No push was performed.

***

_Phase: 05-shapes-and-collision-foundation_
_Completed: 2026-07-11_
