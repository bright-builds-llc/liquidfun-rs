---
generated_by: gsd-phase-researcher
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T02:12:00.000Z
phase: 01-oracle-provenance-and-repository-foundation
status: complete
---

# Phase 1 Research: Oracle, Provenance, and Repository Foundation

## Research Summary

Phase 1 should establish four deep seams rather than begin broad physics work:

1. one immutable, explained upstream oracle;
2. one Cargo-first publishable crate isolated from C++ tooling;
3. one private orchestration shell for reproducible upstream builds and checks; and
4. one machine-readable compatibility/provenance model that generates human documentation.

The prior project research remains sound, but a direct official-repository audit changes the default oracle recommendation. The annotated `v1.1.0` tag object is `d15bcf1879144bf2a4c8ebcc73f6418186756fb2` and peels to release commit `f38db7c627c3dc5ec879d726e16fa5a12ad6e478`. Official repository HEAD is `7f20402173fd143a3988c921bc384459c6a858f2`. The post-release range contains material native C++ changes, including a particle-group split memory fix, particle/fixture collision-filtering behavior and tests, `b2GrowableBuffer` corrections, and warning/build fixes. Phase 1 should therefore adopt `7f204…` through a checked-in ADR that records the release baseline, exact delta, and reason the official post-release commit is the more complete behavioral oracle.

## Requirement Coverage Strategy

| Requirement | Planning implication |
| --- | --- |
| FND-01, DOCS-03 | `UPSTREAM.md`, oracle ADR, and a machine-readable lock must agree on repository, revision, release context, ancestry, patches, licenses, and update steps. |
| FND-02 | Use an exact Git submodule gitlink plus `xtask upstream verify` and an intentional update workflow; never track a branch. |
| FND-03 | Use a repository-owned CMake wrapper and presets with Ninja; validate a library/test smoke on contributor platforms without modifying upstream. |
| FND-04 | Add an alteration/source map and provenance-bearing reference manifest schema before translation begins. |
| FND-05 | Make `crates/liquidfun` the only default member and prove an unpacked package builds/tests with upstream and reference data unavailable. |
| FND-07 | Keep `justfile` recipes thin and make them call Cargo or `cargo xtask` commands whose implementation and errors are inspectable. |
| FND-08 | CI must cross-check gitlink, lock, generated artifacts, toolchain pin, inventory, and packaged contents. |
| COMP-01, COMP-02 | Use an authoritative machine-readable inventory with stable IDs and independent evidence dimensions; generate `COMPATIBILITY.md`. |
| TEST-09 | Provide a fast aggregate verification path and isolate expensive oracle/platform lanes behind explicit commands and CI jobs. |

## Oracle Decision and Provenance

### Verified official references

| Item | Verified value |
| --- | --- |
| Canonical repository | `https://github.com/google/liquidfun.git` |
| `v1.1.0` annotated tag object | `d15bcf1879144bf2a4c8ebcc73f6418186756fb2` |
| `v1.1.0` peeled commit | `f38db7c627c3dc5ec879d726e16fa5a12ad6e478` |
| Recommended final oracle | `7f20402173fd143a3988c921bc384459c6a858f2` |
| Release date context | `v1.1.0` tag: 2014-07-16; recommended official commit: 2018-01-10 |
| Upstream license/notice files | `liquidfun/Box2D/License.txt`, `liquidfun/NOTICE`, plus vendored `googletest` and `freeglut` notices/licenses |

The ADR should make the selection auditable:

- compare `f38db7…..7f204…` by native source, build files, tests/examples, generated JavaScript, and documentation;
- distinguish native behavior changes from bindings/docs-only changes;
- record the particle-group split memory fix and particle/fixture filtering addition as material reasons;
- confirm Box2D 2.3.0/revision-280 ancestry using release notes and pinned-tree records;
- state that the selected oracle is an official repository commit, not a moving branch;
- declare upstream code read-only and all compatibility adaptations external unless a hashed patch is approved;
- record the tag object and peeled commit separately so future tooling cannot confuse them.

Use a small machine-readable lock such as `reference/upstream-lock.toml` with exact keys:

```toml
schema_version = 1
repository = "https://github.com/google/liquidfun.git"
revision = "7f20402173fd143a3988c921bc384459c6a858f2"
release_tag = "v1.1.0"
release_commit = "f38db7c627c3dc5ec879d726e16fa5a12ad6e478"
submodule_path = "third_party/liquidfun"
patch_set = "none"
```

`xtask upstream verify` should fail if the submodule gitlink, submodule checkout, lock revision, `UPSTREAM.md`, or artifact manifests disagree. `xtask upstream update --revision <40-hex>` should only prepare a reviewable change; it must not choose a moving ref or silently regenerate evidence.

## Repository and Build Architecture

### Recommended structure

```text
Cargo.toml
Cargo.lock
rust-toolchain.toml
justfile
.cargo/config.toml
.gitmodules
crates/liquidfun/
  Cargo.toml
  src/lib.rs
tools/xtask/
  Cargo.toml
  src/main.rs
  src/upstream.rs
  src/inventory.rs
  src/package.rs
tools/reference/
  CMakeLists.txt
  CMakePresets.json
reference/
  upstream-lock.toml
  compatibility.toml
  discovery.json
  artifacts/
    manifest.toml
third_party/liquidfun/          # exact read-only submodule
docs/decisions/
  0001-oracle-selection.md
  0002-build-orchestration.md
UPSTREAM.md
ARCHITECTURE.md
COMPATIBILITY.md                # generated
TESTING.md
scripts/verify-package.sh
.github/workflows/ci.yml
.github/workflows/oracle.yml
```

Start with workspace `members = ["crates/liquidfun", "tools/xtask"]`, `default-members = ["crates/liquidfun"]`, resolver 3, shared lints, and one lockfile. `crates/liquidfun` must have no tooling path dependency, C++ `build.rs`, submodule access, reference-data include, default renderer, or protocol crate.

Pin the development toolchain to Rust 1.97.0 with `rustfmt` and `clippy`; declare `rust-version = "1.92"` on the publishable crate as a provisional MSRV. Use Edition 2024. The complete MSRV contract remains a later release responsibility, but the foundation should already compile the skeleton at both pins.

### CMake/Ninja wrapper

The wrapper owns modern policy and options before `add_subdirectory` of the pinned upstream tree. Do not edit upstream CMake files merely to satisfy CMake 4. The wrapper should:

- require a documented local CMake floor (research recommends 3.25) and validate the canonical CI pin 4.3.3;
- set `CMAKE_POLICY_VERSION_MINIMUM=3.5` externally for the legacy project;
- default to Ninja and out-of-tree builds under `target/reference/`;
- disable examples/testbed targets in the ordinary oracle-library smoke unless explicitly selected;
- provide named debug, release, and sanitizer-ready presets without fast-math or `-march=native` in canonical builds;
- keep stdout/stderr behavior and the Phase 2 oracle executable out of the public crate.

Canonical Linux reference artifacts should record Clang/LLVM 22.1.8, target triple, preset, optimization level, and flags. macOS Apple Clang and Windows MSVC/clang-cl are portability checks and must not overwrite canonical data.

## Compatibility Inventory and Generated Evidence

Use `reference/compatibility.toml` as the authoritative curated ledger and `reference/discovery.json` as a deterministic snapshot of mechanically discoverable upstream scope. A checker should require every discovered item to be mapped or explicitly excluded with a rationale; it must not claim that simple C++ parsing proves semantic completeness.

Each row needs these fields:

- `id`: stable, namespaced ID such as `particle.system.split-groups`;
- `kind`: subsystem, public API, source area, test, example, or build option;
- `upstream_path` and optional `upstream_symbol`;
- `applicability`: applicable or reviewed exclusion with rationale;
- `rust_target`: crate/module/API destination or `unassigned`;
- `provenance_ref` and applicable license/notice class;
- independent booleans or structured records for investigated, planned, implemented, unit tested, differentially validated, platform validated, documented difference, and intentionally unsupported;
- evidence links with commit, command, artifact hash, and platform where applicable.

`COMPATIBILITY.md` is generated from the ledger and must never be edited by hand. `cargo xtask inventory check` should verify schema, stable-ID uniqueness, discovery coverage, evidence-state consistency, provenance links, and a clean generated diff.

Reference artifact manifests should record:

```text
schema version
artifact path and SHA-256
generator revision
oracle revision
scenario/protocol version when applicable
CMake preset
compiler identity and target
build flags
source/provenance and notice references
review status
```

Tests may verify manifests but must not silently regenerate checked-in evidence.

## Licensing and Alteration Policy

The current root MIT `LICENSE` applies to original project work but does not by itself satisfy upstream obligations. Phase 1 should add third-party notices and a source/alteration register before translated material lands. For every derived source, test, scenario, or reference artifact, record local path, upstream commit/path, derivation kind, alteration summary, and notice class.

Preserve the upstream Box2D/LiquidFun notice text and mark altered source representations as required by the zlib-style license. Keep vendored GoogleTest/freeglut licenses scoped to developer-only upstream content, and confirm they are excluded from `cargo package`. Do not make a final derivative-work licensing claim beyond the reviewed evidence; surface any unresolved legal ambiguity explicitly.

## Concrete Commands and CI Lanes

Human-facing commands should be thin aliases for these stable entrypoints:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
cargo xtask inventory check
cargo xtask provenance check
cargo xtask package verify
```

Before Phase 1 commits, follow the repository Rust order with formatting first, then Clippy, build-all-targets, and all-feature tests. Prefer a repo-owned aggregate `just check` or `cargo xtask check` once it exists, but keep the underlying commands documented.

Required CI separation:

- **Cargo-only quality:** no submodule checkout and no CMake install; format, Clippy, build, test, docs, package listing, unpacked package build/test.
- **MSRV skeleton:** build/test `liquidfun` at Rust 1.92 without tooling crates.
- **Provenance/inventory:** validate lock/gitlink/docs/manifests, generated report cleanliness, and package exclusions.
- **Oracle smoke:** initialize the exact submodule, validate pinned tools, configure/build the wrapper, and run a named upstream smoke target on Linux; add macOS/Windows portability jobs as sustainable.
- **Scheduled/manual:** full upstream tests, sanitizers, expensive inventory regeneration, and later randomized/differential work.

Pin GitHub Actions by full commit SHA. Friendly major comments are documentation, not security pins.

## Planning Decomposition

The phase is large enough for dependency-aware plans:

1. oracle ADR, exact submodule pin, provenance lock, license/notice/source-map policy;
2. Cargo workspace, toolchain/MSRV skeleton, private `xtask`, and package isolation;
3. CMake wrapper/presets plus upstream verify/configure/build commands;
4. inventory schema/discovery/generator and compatibility report;
5. CI, `justfile`, contributor documentation, architecture/testing records, and end-to-end verification.

Plans that share root manifests, `xtask`, or documentation should not be placed in the same parallel wave without explicit file separation.

## Threat Model

Phase 1 is infrastructure-heavy. Plans should include threats for supply-chain pin drift, malicious or accidental submodule replacement, generated-artifact tampering, unreviewed source patches, package leakage of third-party code/data, command injection through revision/preset inputs, and CI action drift. Mitigate with immutable 40-hex revisions, allowlisted preset names, structured process invocation rather than shell concatenation, hash checks, package-content assertions, and full-SHA action pins. No high-severity threat may remain open at verification.

## Validation Architecture

### Fast deterministic checks

- Unit-test pure parsing and validation for the upstream lock, compatibility rows, manifest hashes, tool-version output, and 40-hex revision inputs using Arrange/Act/Assert.
- Add negative fixtures for wrong revision, duplicate inventory IDs, missing discovery mapping, illegal evidence combinations, stale generated Markdown, invalid artifact hashes, and forbidden package entries.
- Keep orchestration thin: unit-test command construction and validation separately from subprocess execution.

### Integration checks

- Clone/package smoke in a temporary directory with `third_party/` and `reference/` unavailable.
- Verify a deliberately mismatched lock or gitlink fails with an actionable diagnostic.
- Configure and build the pinned upstream through the wrapper in `target/reference/`.
- Regenerate `COMPATIBILITY.md` in check mode and require a clean diff.
- Inspect the packaged crate archive, then build and test the unpacked archive with the provisional MSRV and pinned development compiler where practical.

### Phase completion evidence

Run, in order:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
cargo xtask upstream verify
cargo xtask inventory check
cargo xtask provenance check
cargo xtask package verify
cmake --workflow --preset oracle-debug
git diff --exit-code
```

Where a canonical tool is unavailable locally, do not falsify completion: record the exact missing prerequisite and rely on a named CI lane only when local guidance explicitly permits it. Phase verification must inspect actual files and command results, not summaries alone.

## Pitfalls to Prevent

- Pinning only the tag name instead of the peeled commit and selected post-release SHA.
- Treating the earlier research candidate as final without an ADR explaining material deltas.
- Patching the submodule in place or letting CMake compatibility leak into the consumer build.
- Adding C++ work to `liquidfun/build.rs` or a default Cargo feature.
- Treating generated `COMPATIBILITY.md` as the source of truth.
- Collapsing investigated, implemented, and validated into one status.
- Allowing tests to regenerate reference artifacts silently.
- Claiming root MIT licensing alone accounts for translated or vendored upstream material.
- Hiding platform/tool detection in opaque `just` recipes or shell strings.
- Running same-wave plans that modify the same root manifests or `xtask` modules.

## Primary Sources

- Official repository and immutable refs: `https://github.com/google/liquidfun`
- Official `v1.1.0` tag: `https://github.com/google/liquidfun/releases/tag/v1.1.0`
- Recommended official commit: `https://github.com/google/liquidfun/commit/7f20402173fd143a3988c921bc384459c6a858f2`
- Official comparison range: `https://github.com/google/liquidfun/compare/v1.1.0...7f20402173fd143a3988c921bc384459c6a858f2`
- Pinned-tree Box2D license: `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/License.txt`
- Pinned-tree notice: `https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/NOTICE`
- Existing project sources: `.planning/research/STACK.md`, `.planning/research/ARCHITECTURE.md`, `.planning/research/FEATURES.md`, `.planning/research/PITFALLS.md`, and `.planning/research/SUMMARY.md`.

## RESEARCH COMPLETE

Phase 1 can be planned without further open-ended research. The only deliberately implementation-time validations are platform-specific CMake/Ninja/compiler behavior, the exact conservative inventory discovery surface, and any legal ambiguity surfaced while reconciling third-party notices.
