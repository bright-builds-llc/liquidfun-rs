---
phase: 01-oracle-provenance-and-repository-foundation
plan: "05"
subsystem: repository-integration
tags: [just, github-actions, ci, documentation, package-isolation, oracle]

requires:
  - phase: 01-oracle-provenance-and-repository-foundation/03
    provides: Verified upstream identity and repository-owned CMake/Ninja oracle build
  - phase: 01-oracle-provenance-and-repository-foundation/04
    provides: Deterministic inventory, provenance, and package-isolation gates
provides:
  - Ordered aggregate repository checks with an explicit Cargo-only mode
  - Thin root command menu for Rust, package, inventory, provenance, and oracle workflows
  - Separated least-privilege Cargo-only and oracle CI with full-SHA action pins
  - Truthful architecture, testing, status, and contributor contracts for the Phase 1 scaffold
affects: [phase-02, ci, contributor-workflow, architecture, testing, release-evidence]

tech-stack:
  added: [just command menu, GitHub Actions Cargo and oracle workflows]
  patterns: [Cargo-only consumer lane, explicit oracle lane, full-SHA actions, evidence-gated maturity]

key-files:
  created:
    - justfile
    - .github/workflows/ci.yml
    - .github/workflows/oracle.yml
    - ARCHITECTURE.md
    - TESTING.md
  modified:
    - tools/xtask/src/main.rs
    - README.md
    - CONTRIBUTING.md
    - UPSTREAM.md

key-decisions:
  - "Run package isolation in every aggregate mode, but skip inventory, upstream, and provenance only when the oracle tree is genuinely absent or empty; a non-empty invalid checkout fails hard."
  - "Keep Cargo CI submodule-free and place live pinned-tree inventory/provenance validation in the oracle workflow, while exercising the inventory checker itself in Cargo-only CI."
  - "Use only full-SHA checkout/cache actions with read-only permissions; install canonical oracle tools through versioned, hash-checked commands."
  - "Report the crate as a version 0.0.0 foundation scaffold until compatibility rows gain independent implementation and validation evidence."

patterns-established:
  - "Transparent command surface: just recipes expose visible Cargo or xtask commands and contain no validation logic."
  - "CI trust separation: consumer jobs never initialize or invoke C++, while oracle jobs verify identity and provenance before CMake."
  - "Public maturity follows COMPATIBILITY.md evidence instead of roadmap intent or successful compilation."

requirements-completed:
  - FND-03
  - FND-05
  - FND-07
  - FND-08
  - TEST-09
  - DOCS-03
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 1-2026-07-10T02-00-42
generated_at: 2026-07-10T04:16:34Z

duration: 19 min
completed: 2026-07-10
---

# Phase 1 Plan 05: CI, Workflow, and Documentation Integration Summary

**Transparent local commands, isolated Cargo/oracle CI, and evidence-backed public documentation now make the Phase 1 foundation reproducible without implying physics maturity**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-10T03:57:26Z
- **Completed:** 2026-07-10T04:16:34Z
- **Tasks:** 4
- **Files modified:** 9

## Accomplishments

- Added `cargo xtask check` and nine visible `just` recipes, including an
  explicit Cargo-only skip and hard failure for initialized oracle mismatches.
- Added submodule-free Cargo quality/platform/MSRV/package jobs and distinct
  canonical/manual oracle jobs with read-only permissions and full-SHA action
  pins.
- Published enforceable native-Rust/oracle dependency direction, exact testing
  tiers, generated-file ownership, and a scaffold-only README status tied to
  the 177-row compatibility report.
- Proved the real oracle build and both repository-copy and unpacked-package
  consumer isolation locally.

## Task Commits

Each task was committed atomically:

1. **Task 1: Finish aggregate checks and thin recipes** - `4bfc427` (`feat`)
2. **Task 2: Add separated Cargo-only and oracle CI** - `44291a7` (`ci`)
3. **Task 3: Publish truthful architecture and contributor guidance** - `e0988f3` (`docs`)
4. **Task 4: Verify and simplify the complete foundation** - `6cc2566` (`ci`)

## Final Verification Matrix

| Surface | Command or evidence | Result |
| --- | --- | --- |
| Rust format | `cargo fmt --all --check` | Passed |
| Rust default-member lint/build/test | Clippy with denied warnings, all-target/all-feature build, all-feature tests | Passed in required order |
| Rust full workspace | Workspace Clippy/build/test across `liquidfun` and `xtask` | Passed; 25 xtask unit/integration tests plus crate/doctests |
| Documentation | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` | Passed |
| Provisional MSRV | `cargo +1.92.0 check -p liquidfun --all-targets --all-features` | Passed |
| Inventory | `cargo xtask inventory check` | Passed with 177 rows; generated report and discovery bytes unchanged |
| Upstream | `cargo xtask upstream verify` | Passed for exact revision `7f20402173fd143a3988c921bc384459c6a858f2`; submodule clean |
| Provenance | `cargo xtask provenance check` | Passed with zero Phase 1 artifact records |
| Package | `cargo xtask package verify` | Passed; six-entry archive inspected, unpacked, built, and tested outside the repository |
| Real oracle | `cargo xtask upstream configure/build --preset oracle-debug` | Passed using local CMake 3.27.9, Ninja 1.13.2, and Apple Clang 21.0.0 |
| Aggregate/menu | `just --list`, `just`, and `just check` | Passed; exactly nine public recipes and all applicable checks succeeded |
| Cargo-only repository copy | Copy without `third_party/` or `reference/`, then Cargo build/test/package list | Passed |
| Unpacked consumer archive | Rust 1.92 locked build/test plus forbidden-content scan | Passed without tooling, reference, native source, CMake, or build script leakage |
| Workflow integrity | `actionlint`, YAML parse, action-pin allowlist, permission/secret/submodule separation checks | Passed |
| Markdown and diff | `mdformat --check`, `git diff --check`, generated-file diff, upstream status | Passed |

## CI and Documentation Integration

- `.github/workflows/ci.yml` never initializes submodules or invokes CMake. It
  runs Linux workspace quality and package isolation, a three-platform default
  feature matrix, and the Rust 1.92 `liquidfun` check.
- `.github/workflows/oracle.yml` initializes the exact recursive submodule,
  asserts canonical CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8 identities,
  verifies upstream/provenance/inventory before CMake, and keeps macOS/Windows
  portability jobs manual and non-canonical.
- `ARCHITECTURE.md` prohibits runtime C++ delegation and C++ discovery through
  a `liquidfun` build script or default feature, while preserving renderer and
  Cargo independence.
- `TESTING.md`, `README.md`, and `CONTRIBUTING.md` share the same exact commands,
  evidence ownership, deterministic no-retry policy, and explicit foundation
  maturity.

## Simplification Review

- Kept the `justfile` as direct Cargo/xtask invocations with no loops,
  conditionals, CMake logic, or swallowed errors.
- Kept aggregate orchestration in one typed Rust path rather than duplicating
  validation in recipes or workflows.
- Replaced a proposed 1.9 GB canonical LLVM archive cache with the official
  apt.llvm.org installer pinned by SHA-256, retaining the exact Clang identity
  assertion while materially reducing CI transfer and cache cost.
- Left platform-specific tool setup visible in its owning workflow; extracting
  it at this size would add indirection without reducing policy duplication.
- Confirmed the final diff contains no physics implementation, Phase 2 protocol,
  default C++ feature, consumer build script, or generated-evidence rewrite.

## Decisions Made

- A missing or empty submodule selects explicit Cargo-only mode. Any non-empty
  path is treated as initialized so corrupted or mismatched checkouts fail
  inventory/upstream validation rather than silently skipping.
- Live inventory drift requires the pinned tree and therefore belongs in oracle
  CI; Cargo CI tests the inventory checker's regression behavior without
  weakening the consumer/submodule separation.
- Canonical Linux uses exact tool assertions. Native macOS and Windows compiler
  jobs are portability evidence only and cannot publish canonical artifacts.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced stale upstream build placeholders**

- **Found during:** Task 3 (truthful contributor documentation)
- **Issue:** `UPSTREAM.md` still described `cargo xtask upstream` as a future
  placeholder even though the verified configure/build commands already exist.
- **Fix:** Replaced the placeholder with exact upstream/provenance/configure/build
  commands, local floors, canonical identities, and the real output boundary.
- **Files modified:** `UPSTREAM.md`
- **Verification:** All documented commands ran successfully against the clean
  pinned submodule; Markdown checks passed.
- **Committed in:** `e0988f3`

**Total deviations:** 1 auto-fixed (1 bug).
**Impact on plan:** The correction was necessary for public command accuracy and
stayed entirely within the Phase 1 documentation boundary.

## Issues Encountered

None affecting the implementation. A negative shell assertion initially used
zsh's reserved `status` variable; renaming the local variable allowed the
intended invalid-checkout failure proof to run successfully.

## Residual Platform Evidence

- This macOS host lacks canonical CMake 4.3.3 and Clang 22.1.8, so the real
  local oracle build used supported local-floor tools (CMake 3.27.9, Ninja
  1.13.2, Apple Clang 21.0.0) and passed with expected identity warnings.
- The exact canonical Linux tool installation/assertion and the Ubuntu/Windows
  platform runs cannot execute on this host. Their workflows pass static YAML,
  `actionlint`, permission, pin, ordering, and command checks; runtime evidence
  remains for the named GitHub Actions jobs.
- Manual macOS and Windows oracle portability jobs intentionally publish no
  canonical artifacts. No platform result is claimed before those jobs run.

## User Setup Required

None - no external service configuration required. Contributors who run oracle
work must initialize the documented public submodule and install the listed
local CMake/Ninja/C++ prerequisites.

## Next Phase Readiness

- Phase 1 now has frozen oracle identity, provenance/license policy, Cargo-only
  isolation, a real external oracle build, deterministic compatibility
  inventory, CI separation, and truthful contributor documentation.
- Phase 2 can design the private semantic process protocol against these stable
  boundaries without adding C++ to the published crate.
- All 177 compatibility rows still correctly show no native implementation,
  unit-test, differential, or platform evidence; later plans must update them
  independently as behavior lands.

## Self-Check: PASSED

- All nine key created/modified files exist and four atomic `01-05` task commits
  are present in git history.
- Summary lifecycle metadata matches Plan 01-05 and all six requirement IDs are
  copied verbatim.
- Protected orchestrator-owned `.planning/config.json`, `.planning/STATE.md`,
  and `.planning/ROADMAP.md` remain unstaged and uncommitted.

***

*Phase: 01-oracle-provenance-and-repository-foundation*
*Completed: 2026-07-10*
