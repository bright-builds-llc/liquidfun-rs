---
phase: 02-semantic-protocol-and-oracle-round-trip
plan: "13"
subsystem: contributor-differential-orchestration
tags: [rust, xtask, just, allowlist, cargo-only, differential, provenance]

requires:
  - phase: 02-semantic-protocol-and-oracle-round-trip
    provides: Reviewed empty-world evidence, process supervision, replay, minimization, and confined fixture lifecycle from Plans 02-01 through 02-12
  - phase: 01-oracle-provenance-and-repository-foundation
    provides: Pinned upstream identity, private xtask shell, package isolation, and thin just workflow conventions
provides:
  - Allowlisted xtask compare, replay, minimize, and fixture lifecycle entrypoints
  - Exact structured child invocation with propagated nonzero status and diagnostics
  - Thin direct just aliases for one-shot, reuse, replay, minimize, stage, review, and promote
  - Read-only protocol schema, fixture, package, and artifact checks in Cargo-only mode
affects: [02-14, contributor-workflows, differential-evidence, cargo-only-ci, release-audit]

tech-stack:
  added: []
  patterns: [parse-before-effects command boundaries, canonical structured arguments, injectable executable test seams, cargo-only artifact provenance]

key-files:
  created:
    - tools/xtask/src/differential.rs
    - tools/xtask/tests/differential_cli.rs
    - tools/xtask/tests/fixtures/fake_differential_tool.rs
  modified:
    - tools/xtask/src/main.rs
    - tools/xtask/src/provenance.rs
    - justfile

key-decisions:
  - "Parse every differential command into a closed canonical invocation before upstream verification or child execution."
  - "Require upstream identity verification only for oracle-dependent compare, replay, minimize, and fixture-stage commands; review and promote remain C++-independent lifecycle operations."
  - "Invoke the private differential binary through a repository-owned Cargo command in normal use and reserve the executable override for command-level test injection."
  - "Keep Cargo-only aggregate checks useful by validating protocol presentations, fixtures, package isolation, and artifact provenance without requiring a checked-out C++ submodule."

patterns-established:
  - "Contributor boundary: xtask accepts named scenarios, three presets, three session profiles, and fixed option shapes, then emits canonical runner arguments."
  - "Thin facade: just recipes contain one visible cargo xtask differential command and no validation, loops, conditionals, or swallowed status."

requirements-completed:
  - COMP-05
  - COMP-08
  - COMP-09
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 2-2026-07-10T04-59-34
generated_at: 2026-07-10T11:23:36Z

duration: 10 min
completed: 2026-07-10
---

# Phase 2 Plan 13: Safe Differential Contributor Entrypoints Summary

**Contributors can now run the complete Phase-2 comparison and evidence lifecycle through closed xtask commands and transparent just aliases while Cargo-only checks remain independent of the C++ checkout.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-07-10T11:12:53Z
- **Completed:** 2026-07-10T11:23:36Z
- **Tasks:** 1
- **Files modified:** 6

## Accomplishments

- Added a strict differential xtask boundary for compare, replay, minimize, fixture stage, fixture review, and fixture promote with canonical structured arguments and no caller-provided executable, output, destination, or exact-request path.
- Added 10 command-level tests proving exact arguments, scenario/preset/profile allowlists, arbitrary-path and extra-option rejection, fixture metadata forwarding, and child status/stderr propagation through a separately compiled fake runner.
- Added seven discoverable just recipes whose bodies are direct `cargo xtask differential ...` lines, with shell-safe quoting for lifecycle metadata.
- Extended aggregate checks so protocol schemas, fixtures, package isolation, and reviewed artifact provenance run read-only even when `third_party/liquidfun` is not initialized.

## Task Commits

The TDD task was committed in its RED and GREEN stages:

1. **Task 1 RED: Specify safe differential entrypoints** - `5175682` (`test`)
2. **Task 1 GREEN: Expose safe differential workflows** - `36b345e` (`feat`)

## Files Created/Modified

- `tools/xtask/src/differential.rs` - Closed command parser, upstream gate, canonical runner invocation, failure propagation, and protocol check shell.
- `tools/xtask/tests/differential_cli.rs` - Command-level acceptance and rejection coverage with isolated repository fixtures.
- `tools/xtask/tests/fixtures/fake_differential_tool.rs` - Structured invocation recorder and deterministic failing child.
- `tools/xtask/src/main.rs` - Differential dispatch and full/Cargo-only aggregate check routing.
- `tools/xtask/src/provenance.rs` - Artifact-only provenance entrypoint for a repository without an initialized submodule.
- `justfile` - Thin one-shot, reuse, replay, minimize, stage, review, and promote aliases.

## Decisions Made

- Kept option parsing and allowlist decisions pure and effect-free; an invalid scenario, preset, profile, path-like option, duplicate, or extra option fails before upstream verification or child launch.
- Used canonical argument construction instead of forwarding contributor input verbatim, making the runner boundary auditable and preventing alternate command shapes from leaking through xtask.
- Used inherited child stdout/stderr and explicit nonzero status checks so machine reports remain visible and wrappers cannot convert a harness or physics failure into success.
- Split full provenance from artifact-only provenance only at the checkout identity boundary: Cargo-only mode still validates the lock, source map, artifact content, generator revision, notice, review, and trace metadata.

## Verification Evidence

- TDD RED: all 10 new command tests failed with `unknown command differential` before implementation.
- `cargo test -p xtask --test differential_cli` passes all 10 structured invocation, allowlist, path-rejection, and failure-propagation cases.
- `cargo test -p xtask --all-features` passes all 50 xtask unit and command tests.
- One-shot compare, two-request reuse, reviewed-trace replay, and minimize all return semantic Match through `cargo xtask differential`.
- `just --list` exposes all seven new recipes; recipe inspection and dry-run show one direct xtask line with shell-safe quoted metadata.
- `cargo xtask check` passes with the initialized oracle and from a fresh detached Cargo-only worktree whose submodule directory has no entries.
- Warning-denied full-workspace Clippy, all-target build, all-feature tests, and warning-denied rustdoc pass.
- The required ordered pre-commit sequence passed before both task commits: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added an artifact-only provenance seam for Cargo-only checks**

- **Found during:** Task 1 aggregate-check design
- **Issue:** Full provenance intentionally requires the checked-out upstream revision, but Plan 02-13 also requires reviewed artifact provenance to remain checked when the submodule is absent. The plan's file list did not include the owning provenance module.
- **Fix:** Added a narrow `check_artifacts` entrypoint that retains lock, source-map, generator, content, notice, review, and trace validation while omitting only checkout identity. Full provenance remains unchanged for initialized repositories.
- **Files modified:** `tools/xtask/src/provenance.rs`, `tools/xtask/src/main.rs`
- **Verification:** Full `cargo xtask check` reports complete provenance; the fresh Cargo-only worktree reports artifact provenance and passes without any submodule entry.
- **Committed in:** `36b345e`

**2. [Rule 1 - Bug] Synchronized stale human-readable GSD progress**

- **Found during:** Plan metadata update
- **Issue:** `state update-progress` and `roadmap update-plan-progress 02` returned the correct 95% and 13/14 disk-derived values but left the tracked body progress at 89% and 12/14.
- **Fix:** Updated only the stale human-readable state progress bar and Phase-2 roadmap row to match the successful GSD tool results.
- **Files modified:** `.planning/STATE.md`, `.planning/ROADMAP.md`
- **Verification:** Thirteen Phase-2 summaries exist; state frontmatter reports 18/19 and 95%, and the roadmap reports 13/14.
- **Committed in:** Plan metadata commit

***

**Total deviations:** 2 auto-fixed (1 blocking integration seam, 1 metadata correctness bug)
**Impact on plan:** The narrow extra seam is required by the explicit Cargo-only acceptance criterion and does not weaken full provenance or expand accepted artifact classes; the metadata correction keeps tracking truthful.

## Issues Encountered

- A detached worktree contains the empty registered submodule directory even when the submodule is not initialized. The acceptance check was corrected to test for directory entries, matching xtask's production Cargo-only detection.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Ready for Plan 02-14 to document and wire the now-stable contributor commands into final Phase-2 verification surfaces.
- Contributor input cannot select unregistered scenarios, presets, profiles, executables, output paths, destinations, or exact request files through xtask.
- Full and Cargo-only aggregate modes both have demonstrated read-only validation paths with explicit labels and propagated failures.

## Self-Check: PASSED

- All six task-owned source, test, and just paths exist.
- Both task commits `5175682` and `36b345e` exist and exclude the pre-existing `.planning/config.json` change.
- Summary lifecycle metadata and all three requirement IDs match Plan 02-13 exactly.
- Full and Cargo-only aggregate checks, real oracle commands, focused tests, and workspace gates pass.

***

_Phase: 02-semantic-protocol-and-oracle-round-trip_
_Completed: 2026-07-10_
