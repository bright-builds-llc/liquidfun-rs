---
phase: 01-oracle-provenance-and-repository-foundation
verified: 2026-07-10T04:49:57Z
status: passed
score: "15/15 must-haves verified"
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T04:49:57Z
lifecycle_validated: true
overrides_applied: 0
requirements_checked:
  - FND-01
  - FND-02
  - FND-03
  - FND-04
  - FND-05
  - FND-07
  - FND-08
  - COMP-01
  - COMP-02
  - TEST-09
  - DOCS-03
gaps: []
human_verification: []
---

# Phase 1: Oracle, Provenance, and Repository Foundation Verification

**Phase Goal:** Freeze the final upstream oracle and establish the licensed,
reproducible, Cargo-first repository foundation, architecture evidence, and
compatibility inventory before broad physics implementation.

**Verified:** 2026-07-10T04:49:57Z\
**Status:** passed\
**Re-verification:** No — initial goal-backward verification

## Goal Achievement

The five roadmap success criteria and the detailed PLAN truths reduce to the 15
observable must-haves below. The roadmap criteria are covered respectively by
truths 1-3, 7-9, 4/12/13, 10-12/14, and 6/15.

### Observable Truths

| # | Truth | Status | Evidence |
| ---: | --- | --- | --- |
| 1 | The canonical oracle is an immutable official commit whose release context, ancestry, and selection rationale are explicit. | VERIFIED | Gitlink, checkout, and `reference/upstream-lock.toml` agree on `7f20402173fd143a3988c921bc384459c6a858f2`; the submodule's `v1.1.0` tag object is `d15bcf…` and peels to `f38db7…`; ADR 0001 records the bounded delta and Box2D 2.3.0/revision-280 ancestry. |
| 2 | The upstream checkout is read-only and derived artifacts have a traceable provenance and alteration policy. | VERIFIED | `cargo xtask upstream verify` passed; `git -C third_party/liquidfun status --short` was empty; `.gitmodules` has no tracking branch; `reference/source-map.toml` defines five required fields and maps every Phase 1 upstream-informed local artifact. |
| 3 | License, notice, and alteration obligations are recorded before broad translation. | VERIFIED | `THIRD_PARTY_NOTICES.md`, `UPSTREAM.md`, and `reference/source-map.toml` identify LiquidFun/Box2D, GoogleTest, freeglut, root-license limits, and altered-source duties. Embedded notice content matches the pinned notices except for an immaterial final blank line; `cmp LICENSE crates/liquidfun/LICENSE` passed. |
| 4 | Ordinary Cargo operations select only the publishable Rust crate and never require C++ or repository-only inputs. | VERIFIED | Cargo metadata reports only `liquidfun` as a default member. A fresh `git archive` copy with `reference/` and `third_party/` removed passed build, test, rustdoc, package listing, and `cargo xtask check` in explicit Cargo-only mode. |
| 5 | The development toolchain and provisional publishable-crate MSRV are explicit and work. | VERIFIED | `rust-toolchain.toml` pins 1.97.0 with rustfmt/Clippy; `liquidfun` declares Rust 1.92; `cargo +1.92.0 check -p liquidfun --all-targets --all-features` passed. |
| 6 | Private orchestration has a substantive modular command seam without adding Phase 2 protocol code. | VERIFIED | `tools/xtask/src/main.rs` dispatches exact `upstream`, `inventory`, `provenance`, `package`, and aggregate `check` commands to focused modules using typed errors; source and diff scans found no Phase 2 protocol or broad physics implementation. |
| 7 | Contributors can verify, configure, and build the exact oracle through structured commands. | VERIFIED | Real `cargo xtask upstream verify`, `configure --preset oracle-debug`, and `build --preset oracle-debug` passed; `target/reference/oracle-debug/upstream/Box2D/libliquidfun.a` exists (4.3 MiB). Unknown preset and invalid aggregate arguments failed with the expected categorized diagnostics. |
| 8 | Legacy CMake adaptation and all build output stay outside the pinned tree. | VERIFIED | `tools/reference/CMakeLists.txt` sets CMake 3.25 and policy floor 3.5 before `add_subdirectory`; presets write below `target/reference/`; the submodule remained clean after configure/build. |
| 9 | Canonical presets are deterministic baseline configurations without fast-math or native-only CPU targeting. | VERIFIED | Preset and generated `compile_commands.json` scans found neither `-ffast-math` nor `-march=native`; debug/release/sanitizer presets are allowlisted and use Ninja. |
| 10 | Compatibility scope has one authoritative ledger plus conservative pinned-tree discovery coverage. | VERIFIED | `cargo xtask inventory check` passed with 177 rows. The ledger contains 16 subsystems, 59 public APIs, 8 source areas, 14 tests, 73 examples, and 7 build options; discovery contains all 161 mechanically discoverable non-subsystem entries. |
| 11 | Evidence dimensions remain independent and generated human documentation cannot drift silently. | VERIFIED | Every row contains the eight declared dimensions. Counts truthfully show 177 investigated/planned and zero implemented/unit-tested/differential/platform/difference/unsupported evidence. Check mode preserved SHA-256 `3520b25e…` for `COMPATIBILITY.md` and `11075ea3…` for `reference/discovery.json`, with no diff. |
| 12 | Provenance and package gates reject mismatches, traversal, missing notices/licenses, and consumer leakage. | VERIFIED | `cargo xtask provenance check` and `package verify` passed. Focused tests cover wrong revisions/hashes/generator commits, symlink escape, missing notices, traversal/absolute archive paths, native files, and the required matching license. The real seven-entry package built/tested outside the repository. |
| 13 | Contributors can discover transparent fast, package, inventory, provenance, and oracle workflows from the root. | VERIFIED | `just --list` and default `just` expose exactly nine thin recipes; `cargo xtask check` passed all applicable gates; recipe inspection shows direct Cargo/xtask calls without validation logic or error swallowing. |
| 14 | CI separates Cargo-only consumer proof from oracle proof and pins external actions with least privilege. | VERIFIED | `actionlint` passed. All `uses:` refs are 40-hex SHAs verified against the upstream action repositories; both workflows use `contents: read`, no secrets/write permissions, Cargo CI has `submodules: false` and no CMake/oracle commands, and oracle CI verifies upstream/provenance/inventory before configure/build. |
| 15 | Public architecture, testing, licensing, and maturity documentation is truthful and sufficient for focused next-phase planning. | VERIFIED | `ARCHITECTURE.md`, `TESTING.md`, `README.md`, `CONTRIBUTING.md`, `UPSTREAM.md`, the ADR, and generated compatibility report document dependency direction, Cargo/oracle separation, generated ownership, risk, and commands. README and crate docs explicitly say version 0.0.0 foundation with no physics behavior or parity claim. |

**Score:** 15/15 truths verified

## Required Artifacts

`gsd-tools verify artifacts` passed all 17 declared artifacts at the existence
and substance levels. Manual inspection and command execution verified their
wiring.

| Artifact | Expected | Status | Wiring evidence |
| --- | --- | --- | --- |
| `reference/upstream-lock.toml` | Immutable oracle identity | VERIFIED | Read by upstream/provenance checks; agrees with gitlink and checkout. |
| `docs/decisions/0001-oracle-selection.md` | Auditable oracle decision | VERIFIED | Linked from `UPSTREAM.md`; contains release/candidate delta. |
| `UPSTREAM.md` | Maintainer workflow and policy | VERIFIED | Commands executed successfully; identity matches lock. |
| `Cargo.toml` | Resolver-3 isolated workspace | VERIFIED | Cargo metadata selects only `liquidfun` by default. |
| `crates/liquidfun/Cargo.toml` | Publishable Cargo-only boundary | VERIFIED | Seven-entry package has no native/tooling inputs. |
| `tools/xtask/src/main.rs` | Thin typed dispatcher | VERIFIED | Dispatch and aggregate commands exercised directly. |
| `tools/reference/CMakeLists.txt` | External legacy-policy wrapper | VERIFIED | Real configure/build consumed it. |
| `tools/reference/CMakePresets.json` | Fixed out-of-tree presets | VERIFIED | Real `oracle-debug` configure/build consumed it. |
| `tools/xtask/src/upstream.rs` | Identity and CMake orchestration | VERIFIED | Real success path plus seven focused integration tests. |
| `reference/compatibility.json` | Authoritative compatibility ledger | VERIFIED | Strictly validated against discovery and generated report. |
| `reference/discovery.json` | Deterministic pinned-tree snapshot | VERIFIED | Inventory check reproduced expected bytes/coverage. |
| `COMPATIBILITY.md` | Generated human report | VERIFIED | Check mode confirmed current deterministic bytes. |
| `justfile` | Thin root workflow menu | VERIFIED | `just --list`, default `just`, and aggregate check passed. |
| `.github/workflows/ci.yml` | Cargo-only CI | VERIFIED | actionlint and explicit submodule/CMake/permission scan passed. |
| `.github/workflows/oracle.yml` | Canonical and portability oracle CI | VERIFIED | actionlint, exact tool assertions, ordering, permission, and action-pin scans passed. |
| `ARCHITECTURE.md` | Enforceable dependency direction | VERIFIED | Matches manifests, package, and command behavior. |
| `TESTING.md` | Exact verification tiers | VERIFIED | Documented local commands were executed successfully. |

## Key Link Verification

`gsd-tools verify key-links` passed all 11 declared links.

| From | To | Via | Status |
| --- | --- | --- | --- |
| `.gitmodules` | `reference/upstream-lock.toml` | Exact path/repository agreement | WIRED |
| `UPSTREAM.md` | ADR 0001 | Selection-rationale link | WIRED |
| `.cargo/config.toml` | `tools/xtask/Cargo.toml` | `cargo xtask` alias | WIRED |
| `Cargo.toml` | `crates/liquidfun/Cargo.toml` | Sole default member | WIRED |
| `tools/xtask/src/upstream.rs` | `reference/upstream-lock.toml` | Verify reads locked identity | WIRED |
| `tools/xtask/src/upstream.rs` | CMake presets | Exact preset allowlist/invocation | WIRED |
| `tools/xtask/src/inventory.rs` | `reference/compatibility.json` | Strict schema/evidence/coverage validation | WIRED |
| `tools/xtask/src/provenance.rs` | `reference/upstream-lock.toml` | Cross-record revision validation | WIRED |
| `justfile` | xtask dispatcher | Direct thin recipes | WIRED |
| Oracle workflow | upstream lock | Upstream/provenance gates before CMake | WIRED |
| `README.md` | `COMPATIBILITY.md` | Truthful maturity/evidence link | WIRED |

## Command Evidence

| Command | Result |
| --- | --- |
| `gsd-tools verify lifecycle 1 --expect-id 1-2026-07-10T02-00-42 --expect-mode yolo --require-plans` | Passed before report creation; context, 5 plans, and 5 summaries valid. |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed. |
| `cargo build --workspace --all-targets --all-features` | Passed. |
| `cargo test --workspace --all-features` | Passed: 29 tests plus doctests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps` | Passed. |
| `cargo +1.92.0 check -p liquidfun --all-targets --all-features` | Passed. |
| `cargo xtask inventory check` | Passed: 177 rows. |
| `cargo xtask upstream verify` | Passed at `7f204…`; local noncanonical versions were labeled as warnings. |
| `cargo xtask provenance check` | Passed with zero canonical artifact records. |
| `cargo xtask package verify` | Passed: seven entries built/tested outside the repository. |
| `cargo xtask check` | Passed all applicable initialized-oracle checks. |
| `cargo xtask upstream configure --preset oracle-debug` | Passed. |
| `cargo xtask upstream build --preset oracle-debug` | Passed. |
| Cargo-only archive copy without `reference/` or `third_party/` | Build, test, rustdoc, package listing, and aggregate Cargo-only mode all passed. |
| `actionlint .github/workflows/ci.yml .github/workflows/oracle.yml` | Passed. |
| `mdformat --check` on all Phase 1 public/generated docs | Passed. |
| `git diff --check` and generated-file `git diff --exit-code` | Passed. |

## Requirements Coverage

| Requirement | Status | Evidence |
| --- | --- | --- |
| FND-01 | SATISFIED | Exact repository/revision/release/ancestry in lock, ADR, and `UPSTREAM.md`. |
| FND-02 | SATISFIED | Branchless submodule, documented initialize/verify/full-SHA update, clean identity gate. |
| FND-03 | SATISFIED | External CMake/Ninja wrapper, real local build, and canonical Linux plus manual macOS/Windows workflow definitions. |
| FND-04 | SATISFIED | Source map, notice classes, alteration summaries, and preserved third-party notices. |
| FND-05 | SATISFIED | Cargo-only repository-copy and packaged-crate proofs without upstream/reference/C++. |
| FND-07 | SATISFIED | Nine thin root recipes over documented Cargo/xtask commands. |
| FND-08 | SATISFIED | Lock/git/artifact/hash/generated/package checks and negative regression fixtures fail closed. |
| COMP-01 | SATISFIED | 177-row authoritative ledger covers all declared Phase 1 kinds and 161 discovered entries. |
| COMP-02 | SATISFIED | Eight independent evidence dimensions are present on every row. |
| TEST-09 | SATISFIED | Fast local aggregate/affected checks and separated oracle/manual expensive lanes are documented and wired. |
| DOCS-03 | SATISFIED | `UPSTREAM.md` records identity, ancestry, patches, build, licenses/notices, and intentional updates. |

No Phase 1 requirement mapped in `REQUIREMENTS.md` is orphaned from the plans.

## Anti-Patterns and Disconfirmation Pass

- No blocker anti-patterns were found: no `unwrap()`, shell-evaluated command
  construction, mutable action refs, public C++ build script/default feature,
  fast-math/native CPU flags, hidden parity claim, or dirty upstream content.
- The crate's explicit “no physics behavior yet” statement is truthful Phase 1
  scope, not a hidden implementation stub; all 177 compatibility rows expose
  the unimplemented state.
- The inventory check proves declared structural coverage, not semantic C++ API
  completeness. The report states that limitation and later phases own semantic
  implementation/differential evidence.
- Module tests cover the important failure categories. The aggregate `check`
  command does not separately duplicate every initialized-oracle failure test,
  but its fail-fast `?` wiring is direct and each delegated gate has focused
  negative coverage.

## Platform Runtime Evidence

The local real build passed on macOS ARM64 using CMake 3.27.9, Ninja 1.13.2,
and Apple Clang 21.0.0; xtask correctly labeled CMake/Clang as noncanonical.
The canonical Linux workflow asserts CMake 4.3.3, Ninja 1.13.2, and Clang
22.1.8. The Windows workflow installs and asserts the same CMake/Ninja pins and
enters the Visual Studio x64 developer environment before running the identical
gates. Static workflow validation passed, but `gh run list` returned no runs
because the local branch is ahead of the remote and has not been pushed.

This is not a human-verification item: no subjective judgment is required. It
is recorded as an honest runtime-evidence limitation. Phase 1 establishes the
reproducible commands and CI paths; the compatibility ledger continues to show
zero `platform_validated` physics rows, and no Linux/Windows runtime result is
claimed by this report.

## Human Verification Required

None. All Phase 1 goal claims are objective repository, command, package, or
build properties and were verified non-destructively.

## Gaps Summary

No blocking gaps. The phase goal is achieved without broad physics behavior or
an unsupported parity/production claim.

_Verified: 2026-07-10T04:49:57Z_\
_Verifier: the agent (gsd-verifier)_
