---
phase: 01-oracle-provenance-and-repository-foundation
plan: "04"
subsystem: compatibility-governance
tags: [inventory, provenance, packaging, serde, sha256]

requires:
  - phase: 01-oracle-provenance-and-repository-foundation/01
    provides: Immutable upstream oracle, machine-readable identity lock, source-map policy, and third-party notices
  - phase: 01-oracle-provenance-and-repository-foundation/02
    provides: Cargo-first workspace, publishable crate boundary, and private xtask dispatcher
provides:
  - Authoritative 177-row compatibility ledger with eight independent evidence dimensions
  - Deterministic 161-entry pinned-tree discovery snapshot and generated human report
  - Read-only provenance validation across git, revisions, source maps, artifacts, hashes, notices, and generator commits
  - Traversal-safe packaged-crate inspection plus out-of-repository Rust 1.92 build and test proof
affects: [01-05, phase-02, compatibility, provenance, packaging, ci, release-evidence]

tech-stack:
  added: [serde 1.0, serde_json 1.0, toml 0.9, sha2 0.10, flate2 1.1, tar 0.4]
  patterns: [strict boundary schemas, allowlisted deterministic discovery, explicit generation, inspect-before-extract archives]

key-files:
  created:
    - reference/compatibility.json
    - reference/discovery.json
    - reference/artifacts/manifest.toml
    - COMPATIBILITY.md
    - tools/xtask/src/inventory/discovery.rs
    - tools/xtask/src/inventory/report.rs
    - tools/xtask/src/inventory/validation.rs
    - tools/xtask/tests/inventory_cli.rs
    - tools/xtask/tests/provenance_cli.rs
    - tools/xtask/tests/package_cli.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - reference/source-map.toml
    - tools/xtask/Cargo.toml
    - tools/xtask/src/main.rs
    - tools/xtask/src/inventory.rs
    - tools/xtask/src/provenance.rs
    - tools/xtask/src/package.rs

key-decisions:
  - "Treat compatibility.json as the authoritative curated ledger, discovery.json as the conservative structural omission detector, and COMPATIBILITY.md as generated presentation only."
  - "Keep discovery bounded to 59 public headers, 8 implementation-bearing source areas, 14 unit-test sources, 73 examples, and 7 CMake options at Phase 1 granularity."
  - "Make inventory check read-only; only explicit discover and generate commands may rewrite tracked evidence surfaces."
  - "Inspect every tar entry before extraction, reject non-file entry types and repository/native leakage, and build the unpacked crate outside the repository with Rust 1.92.0."

patterns-established:
  - "Independent evidence: investigated, planned, implemented, unit tested, differentially validated, platform validated, documented difference, and intentionally unsupported never collapse into one maturity state."
  - "Generated-evidence gate: canonical bytes, selected-oracle revision, coverage, hashes, notices, and review status fail closed with categorized diagnostics."
  - "Consumer archive gate: validate path confinement and content allowlists before writing any archive entry."

requirements-completed:
  - FND-04
  - FND-05
  - FND-08
  - COMP-01
  - COMP-02
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T03:53:41Z

duration: 34 min
completed: 2026-07-10
---

# Phase 1 Plan 04: Compatibility Inventory and Evidence Gates Summary

**A pinned-tree compatibility ledger now exposes every declared Phase 1 scope item and evidence gap while provenance and package checks fail closed on wrong-oracle or consumer-leakage risks.**

## Performance

- **Duration:** 34 min
- **Started:** 2026-07-10T03:18:59Z
- **Completed:** 2026-07-10T03:53:41Z
- **Tasks:** 4
- **Files modified:** 19

## Accomplishments

- Mapped the pinned oracle into 177 stable compatibility rows: 16 subsystems, 59 public headers, 8 source areas, 14 unit-test sources, 73 examples, and 7 build options.
- Added deterministic allowlisted discovery, strict unknown-field-denying schemas, evidence dependency checks, omission detection, and a 441-line generated human report.
- Bound lock, gitlink, checkout, compatibility/discovery identities, source mappings, artifact hashes, notices, generator commits, and review status into one read-only provenance check.
- Proved the six-file published archive contains no repository tooling or native source, then built and tested its unpacked contents outside the repository using Rust 1.92.0.

## Inventory Evidence Snapshot

| Kind | Rows |
| --- | ---: |
| Subsystem | 16 |
| Public API/header | 59 |
| Source area | 8 |
| Upstream unit test | 14 |
| Example/testbed scenario | 73 |
| CMake build option | 7 |
| **Total** | **177** |

| Evidence dimension | Evidenced | Not evidenced |
| --- | ---: | ---: |
| Investigated | 177 | 0 |
| Planned | 177 | 0 |
| Implemented | 0 | 177 |
| Unit tested | 0 | 177 |
| Differentially validated | 0 | 177 |
| Platform validated | 0 | 177 |
| Documented difference | 0 | 177 |
| Intentionally unsupported | 0 | 177 |

These gaps are intentional and truthful: Plan 01-04 inventories and governs evidence but does not implement physics or the Phase 2 semantic protocol.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define and populate compatibility/discovery schemas** - `42a831f` (feat)
2. **Task 2: Implement inventory generation and validation** - `5a2d142` (feat)
3. **Task 3: Implement provenance and packaged-crate isolation checks** - `4813cda` (feat)
4. **Task 4: Add focused validator and CLI regression tests** - `aaf9b98` (test)

## Files Created/Modified

- `reference/compatibility.json` - authoritative stable-ID ledger with all eight evidence records on every row.
- `reference/discovery.json` - exact selected-revision structural snapshot with deterministic kind/path/symbol ordering.
- `reference/artifacts/manifest.toml` - empty schema-versioned artifact registry requiring hashes, generator/oracle identity, build identity, notices, and review state.
- `COMPATIBILITY.md` - generated report with oracle identity, evidence legend, per-kind tables, state counts, and explicit gaps.
- `tools/xtask/src/inventory.rs` plus `inventory/` - thin command shell around strict parsing, pure validation, bounded discovery, and deterministic rendering.
- `tools/xtask/src/provenance.rs` - cross-record identity, hash, notice, and generator-commit validation.
- `tools/xtask/src/package.rs` - `.crate` inspection, traversal/content rejection, confined extraction, and independent MSRV verification.
- `tools/xtask/tests/{inventory_cli,provenance_cli,package_cli}.rs` - bounded Arrange/Act/Assert regression fixtures independent of canonical inputs and Cargo cache state.
- `reference/source-map.toml` - provenance mappings for every new upstream-informed inventory/report artifact.

## Decisions Made

- Testbed headers are examples rather than upstream unit tests; the 14 `*Tests.cpp` files under `Unittests` remain a separate test kind, and `HelloWorld.cpp` is the 73rd example.
- Source-area granularity is the eight implementation-bearing directories under `Box2D/Box2D`, not every C++ source file. Public API granularity remains every discovered header.
- Symlinks inside allowlisted upstream roots are never followed. They are skipped while the real files under the declared discovery patterns remain exhaustively inventoried.
- `flate2` and `tar` are private tooling dependencies because safe entry-type/path inspection must happen before extraction; the published `liquidfun` crate retains no normal dependencies.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Safely skipped pinned-tree symlinks during discovery**

- **Found during:** Task 2 canonical discovery run.
- **Issue:** The first walker rejected a symlink in the upstream unit-test assets, blocking all discovery even though following it would violate path-confinement policy.
- **Fix:** Skip symlink entries without following them; continue scanning only regular files and directories beneath allowlisted roots.
- **Files modified:** `tools/xtask/src/inventory/discovery.rs`.
- **Verification:** Two canonical discovery runs produced SHA-256 `11075ea393d85eb5be26e337ecb8c8cac54e049ccda0464b7b662a84b01b32c8` and all 161 entries remained covered.
- **Committed in:** `5a2d142`.

**2. [Rule 2 - Missing Critical] Registered new upstream-informed artifacts in the source map**

- **Found during:** Task 2 provenance review.
- **Issue:** The planned file list omitted `reference/source-map.toml`, but repository policy requires every upstream-informed inventory, discovery, manifest, or generated report to carry an explicit derivation record.
- **Fix:** Added four source-map records with exact revision, upstream scope, derivation kind, alteration/no-copy summary, and notice class.
- **Files modified:** `reference/source-map.toml`.
- **Verification:** `cargo xtask provenance check` validates each mapped local path and exact selected revision.
- **Committed in:** `5a2d142`.

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical). **Impact:** Both changes enforce the planned threat model without expanding beyond Phase 1 inventory/provenance/package scope.

## Verification Evidence

- Required Rust sequence passed: `cargo fmt --all`, Clippy with denied warnings, all-target build, and all-feature tests.
- Explicit workspace variants also passed so private xtask targets were covered despite `liquidfun` being the only default workspace member.
- Named integration targets passed: 5 inventory tests, 5 provenance tests, and 4 package tests; the full workspace also retained all 7 upstream CLI tests.
- `cargo xtask inventory check`, `cargo xtask provenance check`, and `cargo xtask package verify` each passed twice.
- Check-mode commands preserved `COMPATIBILITY.md` and `reference/discovery.json`; report SHA-256 remained `3520b25ee4aa8241220bada4a02f046f30e7f020c2b59d7cacbe291c66ed4d92`.
- `cargo package -p liquidfun --allow-dirty --list` reported only `.cargo_vcs_info.json`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, and `src/lib.rs`.
- `git diff --exit-code -- COMPATIBILITY.md reference/discovery.json` and `git diff --check` passed after verification.

## Issues Encountered

- Plain workspace commands honor `default-members = ["crates/liquidfun"]`, so the repository-required command sequence alone does not compile private xtask targets. The same sequence was additionally run with `--workspace`, and the named CLI test targets were run explicitly.
- Workspace Clippy found local formatting-only issues in the new generator/tests on the first full pass; they were corrected, then the complete ordered verification sequence passed from the beginning.

## Residual Limitations

- The artifact manifest is intentionally empty in Phase 1. Hash, notice, review, and generator-commit validation are proven with bounded fixtures and will apply to canonical artifacts once later phases add them.
- Discovery is conservative at the declared structural granularity; it does not pretend simple filename or CMake parsing establishes semantic API completeness.
- All 177 rows remain unimplemented and not differentially or platform validated. The generated report makes those gaps explicit rather than implying maturity.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Ready for `01-05-PLAN.md` to wire inventory, provenance, and package checks into thin contributor aliases and split CI lanes.
- Ready for Phase 2 to design the semantic protocol against a frozen, mechanically governed oracle scope.
- No inventory-coverage, stale-report, wrong-revision, artifact-integrity, archive-traversal, or consumer-leakage blocker remains.

## Self-Check: PASSED

- All key ledger, report, xtask, and regression-test files exist.
- Four atomic `01-04` task commits are present in git history.
- Summary lifecycle metadata matches Plan 01-04, and all five requirement IDs are copied verbatim.
- Protected orchestrator-owned `.planning/config.json`, `.planning/STATE.md`, and `.planning/ROADMAP.md` remain unstaged and uncommitted.

***

_Phase: 01-oracle-provenance-and-repository-foundation_
_Completed: 2026-07-10_
