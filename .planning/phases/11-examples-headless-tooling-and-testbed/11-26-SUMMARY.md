---
phase: 11-examples-headless-tooling-and-testbed
plan: "26"
subsystem: controller-driven-semantic-testbed
tags: [macroquad, controller, debug-draw, input, deterministic-capture]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "25"
    provides: Passive visual shell and presentation-only state boundary
  - phase: 11-examples-headless-tooling-and-testbed
    plans: ["07", "09", "13"]
    provides: Typed session commands, semantic debug primitives, and deterministic capture contracts
provides:
  - Single-flight typed controller adapter with closed-state enabledness projections
  - Semantic debug-primitive viewport with synchronized pointer-anchored camera and presentation-only selection
  - Exact run controls, keyboard routing, validated staged settings, and independent diagnostic overlays
  - Confined PNG screenshot paths with explicit non-authority labeling
affects: [phase11-testbed-differences, phase12-portability]
tech-stack:
  added: []
  patterns:
    - Map every authoritative interaction to a typed SessionCommand before submission
    - Keep camera, hover, selection, overlays, panels, and screenshots presentation-only
    - Render immutable DebugPrimitive values through a validated semantic display list
key-files:
  created:
    - crates/liquidfun-testbed/src/controller_adapter.rs
    - crates/liquidfun-testbed/src/input.rs
    - crates/liquidfun-testbed/src/ui/viewport.rs
    - crates/liquidfun-testbed/src/ui/viewport/draw.rs
    - crates/liquidfun-testbed/src/ui/run_controls.rs
    - crates/liquidfun-testbed/src/ui/settings.rs
    - crates/liquidfun-testbed/src/ui/overlays.rs
    - crates/liquidfun-testbed/tests/controller_ui.rs
    - crates/liquidfun-testbed/tests/controller_ui/support.rs
  modified:
    - Cargo.lock
    - crates/liquidfun-testbed/Cargo.toml
    - crates/liquidfun-testbed/src/lib.rs
    - crates/liquidfun-testbed/src/ui.rs
key-decisions:
  - "Validate controller actions with the pure session transition before admitting a single in-flight typed submission."
  - "Project DebugPrimitive values into stable-keyed semantic layers before Macroquad drawing so renderer frames cannot acquire simulation authority."
  - "Keep accepted settings immutable until all staged fields parse and validate, then apply them only through ApplySettingsAndRestart."
  - "Confine diagnostic screenshots beneath target, reject absolute, traversal, symlink, non-file, and non-PNG destinations, and label them as non-authoritative."
patterns-established:
  - "Controller boundary: UI affordances produce typed actions whose validity and enabledness derive from closed SessionState values."
  - "Semantic viewport: immutable primitives become bounded display-list entries; local camera and selection mutations never produce controller effects."
requirements-completed: [RIGD-10, EXMP-04, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T14:25:26Z
duration: 27 min
completed: 2026-07-22
---

# Phase 11 Plan 26: Controller-Driven Semantic Testbed Summary

**The private testbed now routes authoritative interactions through closed controller commands while rendering bounded semantic geometry with synchronized camera, validated settings, overlays, and diagnostic-only screenshots.**

## Performance

- **Duration:** 27 min
- **Started:** 2026-07-22T13:58:46Z
- **Completed:** 2026-07-22T14:25:26Z
- **Tasks:** 1
- **Files modified:** 13

## Accomplishments

- Added a single-flight `ControllerAdapter` that maps select, run, pause, one-step, restart, checkpoint capture, settings restart, and scenario actions to typed `SessionCommand` values after pure transition validation.
- Derived exact control labels and enabledness from every closed `SessionState`, including `Run Scenario`, `Resume`, `Pause`, `Step Once`, `Restart`, `Capture Checkpoint`, and the distinct `Session paused` versus `Particle system pause action` copy.
- Added exact global shortcut routing for Space, Right, R, C, slash, F, brackets, 1-4, Home, question mark, Escape, and bounded scenario keys, with every global action suppressed during field editing.
- Added staged settings that parse on commit, retain the previous accepted values after invalid input, enforce finite positive timesteps and iteration counts from 1 through 1024, and enable `Apply & Restart` only for a valid change.
- Projected every `DebugPrimitive` variant into stable-keyed shapes, joints, contacts, normals, particle contacts, broad-phase bounds, centers, statistics, and diagnostic-profile layers without exposing engine storage.
- Added bounded pointer-anchored zoom, pan, reset-to-bounds, synchronized comparison cameras, stable semantic hover/selection, exact selection accent, and a 400ms tooltip threshold without emitting simulation effects.
- Confined PNG screenshots beneath the target tree, rejected path escape and link hazards, and exposed the exact diagnostic non-authority clarification.

## TDD Evidence

- **RED:** `CARGO_TARGET_DIR=/tmp/liquidfun-11-26-target-75882 cargo test -p liquidfun-testbed --test controller_ui` failed only because the new controller adapter, input, viewport, run-control, settings, and overlay modules did not yet exist.
- **GREEN:** The focused target passes 15/15 tests covering all controller actions, closed-state projections, exact shortcuts, edit suppression, settings fields and errors, one-tick stepping, zero-tick presentation actions, every semantic layer, synchronized camera, stable selection, overlays, and screenshot confinement.
- **REFACTOR:** Split Macroquad drawing from semantic viewport projection and test support from behavior assertions, then passed focused deny-warnings Clippy and the complete ordered Rust gate.

The intentionally failing RED state was not committed because repository policy requires every tracked commit to follow the complete passing ordered Rust gate.

## Task Commits

1. **Task 1: Wire all interaction through the controller and semantic viewport** - `26ecb84` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-testbed/src/controller_adapter.rs` - Typed action mapping, pure transition validation, single-flight admission, and closed-state enabledness.
- `crates/liquidfun-testbed/src/input.rs` - Exact keyboard routing, field-edit suppression, presentation effects, and bounded scenario shortcuts.
- `crates/liquidfun-testbed/src/ui/viewport.rs` - Semantic display-list projection, stable keys, bounded camera, synchronized comparison views, selection, hover, and screenshot confinement.
- `crates/liquidfun-testbed/src/ui/viewport/draw.rs` - Imperative Macroquad drawing shell for validated semantic entries.
- `crates/liquidfun-testbed/src/ui/run_controls.rs` - Exact dynamic control labels, tooltips, and enabledness.
- `crates/liquidfun-testbed/src/ui/settings.rs` - Staged settings parsing, exact guidance, accepted-value retention, and Apply & Restart action.
- `crates/liquidfun-testbed/src/ui/overlays.rs` - Independent semantic layers, shortcut groups, and diagnostic profile labeling.
- `crates/liquidfun-testbed/tests/controller_ui.rs` - Fifteen behavior tests spanning the complete interaction and presentation contract.
- `crates/liquidfun-testbed/tests/controller_ui/support.rs` - Deterministic session, geometry, and temporary-output fixtures.
- `crates/liquidfun-testbed/Cargo.toml` and `Cargo.lock` - Direct private path dependency on renderer-neutral `liquidfun` debug primitives.
- `crates/liquidfun-testbed/src/lib.rs` and `crates/liquidfun-testbed/src/ui.rs` - New private-package module exports.

## Decisions Made

- Controller projection and action admission both derive from the same closed session transition model, preventing enabled controls from disagreeing with authoritative command validity.
- Camera, hover, selection, overlays, help, fullscreen, panels, and screenshots return presentation-only effects or no effect; only explicitly authoritative controls can construct a controller action.
- Viewport validation rejects non-finite or unreasonably large geometry before display-list creation, camera scale is clamped to 5-400 pixels per meter, and semantic identity comes from stable primitive keys rather than pointers or storage indices.
- Settings retain independent staged text, last accepted values, and exact per-field errors. No partial settings object can cross the controller boundary.
- Diagnostic timing profiles remain visibly excluded from compatibility authority, matching their observation semantics.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired the new modules and renderer-neutral primitive dependency into the private testbed package**

- **Found during:** Task 1 GREEN implementation
- **Issue:** The plan's primary file list omitted the manifest, lockfile, and module entrypoints required for the new controller/input/UI modules to compile and for the viewport to consume `liquidfun::DebugPrimitive` directly.
- **Fix:** Added a private path dependency from `liquidfun-testbed` to `liquidfun`, updated the lockfile, and exported the new modules from `lib.rs` and `ui.rs`.
- **Files modified:** `Cargo.lock`, `crates/liquidfun-testbed/Cargo.toml`, `crates/liquidfun-testbed/src/lib.rs`, `crates/liquidfun-testbed/src/ui.rs`
- **Commit:** `26ecb84`

## Issues Encountered

- A broader package-scoped dependency Clippy invocation exposes a pre-existing `clippy::match_same_arms` warning in `crates/liquidfun-test-protocol/src/scenario/rigid_world/result/phase10/prefix.rs`. The same failure was reproduced in a separate clean worktree at exact base `f2ec4a03`, proving it is not introduced by Plan 11-26. That file is fenced for Plan 11-22 ownership and remained untouched. The plan-mandated default-member Clippy gate passes, and Plan 11-26's own package targets pass with `--no-deps` and warning denial.

## Security Verification

- Shortcut routing is closed and bounded, global shortcuts are suppressed during editing, controller submissions are single-flight, and invalid state transitions fail before effectful submission.
- Settings reject non-finite or non-positive timesteps and iteration values outside 1-1024 while retaining the previous accepted values.
- Semantic primitives are validated for finite bounded geometry, selected by stable keys, and rendered without raw pointers, private indices, engine storage, or simulation ownership.
- Camera scale is clamped, display-list inputs are bounded, and local camera/selection/panel actions cannot tick or capture the simulation.
- Screenshot output is confined beneath target, restricted to PNG files, and rejects absolute paths, traversal, symlinks, non-files, and escaping parents.
- `cargo tree -p liquidfun --edges normal` contains neither Macroquad nor `liquidfun-testbed`; renderer capabilities remain private, unpublished, and outside the production dependency graph.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Verification Evidence

- Focused controller/UI tests: 15/15 passed with `cargo test -p liquidfun-testbed --test controller_ui`.
- Focused Plan 11-26 Clippy: `cargo clippy -p liquidfun-testbed --all-targets --all-features --no-deps -- -D warnings` passed.
- Mandatory ordered gate passed with `CARGO_TARGET_DIR=/tmp/liquidfun-11-26-target-75882`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- Production dependency isolation, stub-pattern scan, and `git diff --check` passed.
- Baseline package-scoped dependency Clippy issue was independently reproduced from exact base `f2ec4a03`; no fenced root edit was touched, staged, or committed.

## Requirements Status

Plan 11-26's `RIGD-10`, `EXMP-04`, and `EXMP-05` mappings are implemented and recorded in summary frontmatter. Global requirement checkboxes remain unchanged until the owning Phase 11 integration flow closes their complete scopes.

## User Setup Required

None - controller, viewport, settings, overlay, and filesystem-boundary tests are deterministic and headless.

## Next Phase Readiness

- Plan 11-27 can layer semantic difference presentation onto the stable viewport keys, synchronized cameras, and immutable display-list model.
- Plan 11-22 must integrate its fenced protocol cleanup before package-wide dependency Clippy can pass from the shared branch; Plan 11-26 requires no change for that prerequisite.
- No Plan 11-26 blocker remains.

## Self-Check: PASSED

- Confirmed all nine created artifacts exist and implementation commit `26ecb84` is present.
- Confirmed focused tests and Clippy, complete ordered Rust gate, production dependency isolation, stub scan, and diff check pass.
- Confirmed the branch is based exactly on `f2ec4a03abb852518fdbd52e9efd7c3cde43b03e` and the four fenced root edits remain untouched and absent from both commits.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
