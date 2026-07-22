---
phase: 11-examples-headless-tooling-and-testbed
plan: "15"
subsystem: headless-catalog-cli
tags: [catalog, cli, controller, replay, comparison, structured-argv, security]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "14"
    provides: Exact resolved-byte native and supervised C++ execution, replay, comparison, and failure classification
provides:
  - Closed list, inspect, run, replay, and compare commands for all reviewed catalog slugs
  - Bounded typed controller scripts for pause, resume, step, restart, scenario action, and semantic capture
  - Stable exit categories and separately labeled semantic, screenshot, profile, timing, physics, and harness output
affects: [phase11-testbed, phase11-regression, phase11-benchmarks, phase11-evidence]
tech-stack:
  added: []
  patterns:
    - Validate untrusted options in xtask before forwarding fixed structured argv to the differential binary
    - Keep semantic compatibility output distinct from diagnostic screenshots, profiles, and wall time
key-files:
  created:
    - crates/liquidfun-differential/src/catalog_command.rs
    - crates/liquidfun-differential/src/catalog_command/parse.rs
    - crates/liquidfun-differential/src/catalog_command/render.rs
    - tools/xtask/src/differential/catalog.rs
    - tools/xtask/tests/catalog_cli.rs
  modified:
    - crates/liquidfun-differential/src/main.rs
    - tools/xtask/src/differential.rs
    - tools/xtask/src/main.rs
    - justfile
key-decisions:
  - "Use two closed parser layers: xtask rejects invalid input before effects, while the differential binary independently reconstructs typed settings and controller commands."
  - "Forward every catalog invocation through structured argument vectors and propagate registered child exit codes without shell interpolation."
  - "Report semantic checkpoint and comparison fields separately from explicitly diagnostic screenshot, profile, and timing labels."
patterns-established:
  - "Catalog CLI boundary: stable slug, finite positive timestep, iterations 1 through 1024, named preset/profile/output, and at most 128 typed controller commands."
  - "Catalog exit contract: usage 64, scenario 65, settings 66, script 67, oracle unavailable 69, physics mismatch 2, and harness failure 3."
requirements-completed: [EXMP-02, EXMP-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T04:22:19Z
duration: 35 min
completed: 2026-07-21
---

# Phase 11 Plan 15: Bounded Headless Catalog Commands Summary

**All 43 reviewed scenarios now have a renderer-free, injection-resistant command surface for discovery, typed control, exact replay, and native/C++ semantic comparison.**

## Performance

- **Duration:** 35 min
- **Started:** 2026-07-22T03:47:19Z
- **Completed:** 2026-07-22T04:22:19Z
- **Tasks:** 1
- **Files modified:** 19

## Accomplishments

- Added deterministic `catalog list`, structured human/JSON inspection, native run, exact D0 replay, and supervised Rust/C++ compare commands keyed by stable scenario slug.
- Added finite positive timestep validation, iteration bounds, allowlisted oracle presets/session profiles/output modes, optional seed resolution, and bounded typed scripts covering pause, resume, step, restart, scenario action, and capture.
- Preserved semantic checkpoint IDs, resolved identity, controller progress, comparison outcome, and stable exit categories while explicitly excluding screenshots, timing, and profile values from compatibility claims.
- Added command-level confinement tests proving canonical argument forwarding, injection rejection, closed option shapes, script limits, and exit-code propagation without starting the effectful runner on invalid input.

## TDD Evidence

- **RED:** `cargo test -p xtask --test catalog_cli` compiled after a pre-existing lint blocker was repaired, then failed 6/6 because the top-level `catalog` command was not registered.
- **GREEN:** The focused target passes 7/7 across list/inspect, run/replay/compare, typed scripts, setting bounds, unknown values, injection attempts, the 128-command cap, stable catalog exits, and child exit propagation.
- **REFACTOR:** The binary command was split into orchestration, closed parsing, and rendering modules; xtask retains one separate closed forwarding parser, and all implementation files remain within the repository's file-size guidance.

The intentionally failing RED state was not committed because repository policy requires each commit to follow a completely passing ordered Rust gate.

## Task Commits

1. **Rule 3: Restore strict workspace lint gate** - `addffab` (fix)
1. **Rule 3: Preserve catalog joint semantics in the C++ adapter** - `2fa2cf0` (fix)
1. **Task 1: Add the bounded headless catalog command surface** - `761a012` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-differential/src/catalog_command.rs` - Catalog discovery, native control, exact replay, oracle comparison, and request resolution.
- `crates/liquidfun-differential/src/catalog_command/parse.rs` - Closed settings, preset, profile, output, seed, script, error, and exit-code domain parsing.
- `crates/liquidfun-differential/src/catalog_command/render.rs` - Human and JSON reports with explicit semantic and diagnostic labels.
- `crates/liquidfun-differential/src/main.rs` - Minimal catalog dispatch beside the existing differential command surface.
- `tools/xtask/src/differential/catalog.rs` - Pre-effect allowlist and canonical structured-argv construction.
- `tools/xtask/src/differential.rs` and `tools/xtask/src/main.rs` - Catalog routing and stable child exit propagation.
- `tools/xtask/tests/catalog_cli.rs` - Fake-runner confinement, validation, injection, and exit-category regression coverage.
- `justfile` - Thin fixed aliases for list, inspect, run, replay, and compare.
- `tools/reference/src/catalog_joint.cpp` and related adapter files - Typed upstream joint construction and complete fail-closed mutation dispatch discovered while validating the real compare path.

## Decisions Made

- Kept xtask and binary validation independent. Invalid contributor input is rejected before child startup, while direct binary callers still cannot bypass domain validation.
- Used only structured process arguments. Controller scripts remain one bounded value and are parsed into typed commands; shell syntax, paths, and unregistered IDs are rejected.
- Made semantic evidence labels explicit and kept screenshot, diagnostic profile, and wall-time wording visibly non-compatibility-bearing.
- Propagated registered catalog and differential child statuses through xtask so automation can distinguish usage, scenario, settings, script, oracle, physics, and harness outcomes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored the strict workspace lint gate**

- **Found during:** Task 1 RED compilation and the required full-workspace gate
- **Issue:** A missing committed import blocked compilation, and two existing xtask modules failed the repository's deny-warnings Clippy gate through an oversized validator, needless borrows, uninlined format args, and no-effect metadata bindings.
- **Fix:** Restored the required import, split the oversized validator into cohesive helpers, tightened metadata validation, and resolved the strict lints without changing command semantics.
- **Files modified:** `crates/liquidfun-differential/src/failure_bundle/catalog.rs`, `tools/xtask/src/inventory/validation/phase9.rs`, `tools/xtask/src/phase9_evidence.rs`
- **Verification:** The exact ordered full-workspace fmt, deny-warnings Clippy, build, and test gate passes.
- **Committed in:** `addffab`

**2. [Rule 3 - Blocking] Preserved reviewed joint kinds in the catalog oracle**

- **Found during:** Real `catalog compare joint-distance-behavior` verification
- **Issue:** The Plan 11-12 C++ session created every catalog joint as revolute and accepted only revolute mutations, causing a valid distance-joint request to become `MalformedRecord` with `unknown joint mutation`.
- **Fix:** Added closed-slug construction for all 11 upstream joint kinds, real gear topology, full typed fail-closed mutation dispatch, adapter-input provenance, and C++ plus Rust integration regressions.
- **Files modified:** `tools/reference/src/catalog_joint.cpp`, `catalog_joint.hpp`, `catalog_run_session.cpp`, `CMakeLists.txt`, `adapter-inputs.txt`, `protocol_tests.cpp`, and `crates/liquidfun-differential/tests/catalog_round_trip.rs`
- **Verification:** Debug CTest passes 1/1; ASan/UBSan protocol and sanitizer-scope CTests pass 2/2; the Rust real-oracle regression passes; the real compare now reaches a semantic physics mismatch with exit 2 rather than a malformed-record harness failure.
- **Committed in:** `2fa2cf0`

**Total deviations:** 2 auto-fixed blocking issues.
**Impact on plan:** Both fixes were necessary to run the required strict gate and prove the real command path. They restored existing contracts without adding a renderer, dependency, alternate protocol, or broader public engine surface.

## Issues Encountered

- An attempted eleven-scenario convenience sweep entered a shared nested Cargo rebuild and was stopped; the focused C++ type/mutation regression, sanitizer lane, Rust integration regression, and real distance compare provide the required evidence without relying on that incidental sweep.
- The shared worktree contained four unrelated fenced edits. They remained unstaged and uncommitted by this plan.

## Security Verification

- Xtask rejects unregistered options, scenarios, presets, profiles, outputs, invalid settings, overlong scripts, path-like IDs, and shell syntax before the runner starts.
- The differential binary repeats closed validation before resolution or process startup and translates scripts only into typed bounded controller commands.
- Child execution uses structured argv, retains existing process limits and provenance checks, and preserves fixed stable exit categories.
- Tests prove rejected injection strings cannot create the fake runner marker and that 129 commands fail before effects.
- Semantic evidence, screenshots, diagnostic profiles, timing, physics mismatches, and harness failures remain visibly distinct.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Requirements Status

Plan 11-15's `EXMP-02` and `EXMP-03` mappings are implemented in the headless operational surface and retained in summary frontmatter. Their global requirement checkboxes remain unchanged until the later benchmark and optional visual consumers prove the complete cross-consumer requirement scope.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plans 11-16 and later can invoke one stable renderer-free catalog surface for regression, minimization, benchmark, and optional visual workflows.
- Every reviewed slug is discoverable and inspectable, while exact replay and compare retain the same resolved-byte authority established by Plan 11-14.
- No blocker remains for the next incomplete Phase 11 plan.

## Self-Check: PASSED

- Confirmed the five new command/parser/render/test modules exist and commits `addffab`, `2fa2cf0`, and `761a012` are present.
- Confirmed focused xtask tests pass 7/7 and real catalog listing emits all 43 reviewed definitions in stable slug order.
- Confirmed real inspect, native run, exact D0 replay, typed controller script, and supervised compare paths execute headlessly with the expected semantic labels and exit outcomes.
- Confirmed oracle-debug and oracle-asan-ubsan CTests pass after the typed joint repair.
- Confirmed the exact ordered `cargo fmt --all`, full-workspace deny-warnings Clippy, all-targets build, and all-features test gate passes with `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-15`.
- Confirmed the four fenced pre-existing edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-21*
