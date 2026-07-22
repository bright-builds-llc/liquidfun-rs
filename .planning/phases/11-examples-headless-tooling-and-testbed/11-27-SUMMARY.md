---
phase: 11-examples-headless-tooling-and-testbed
plan: "27"
subsystem: accessible-semantic-comparison-testbed
tags: [macroquad, comparison, responsive-ui, accessibility, diagnostics]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "13"
    provides: Canonical renderer-neutral ComparisonModel with bounded semantic entries
  - phase: 11-examples-headless-tooling-and-testbed
    plans: ["25", "26"]
    provides: Passive dark shell, typed controller boundary, synchronized semantic viewport, and confined PNG capture
provides:
  - Canonical difference projection shared by overlay and side-by-side presentation
  - Responsive desktop, drawer, sheet, compact-notice, and minimum-window behavior
  - Accessible focus, target, contrast, selectable-value, and mismatch-announcement contracts
  - Retained bounded error states and deterministic diagnostic capture with explicit authority exclusions
affects: [phase11-testbed-integration, phase12-portability, phase12-release-audit]
tech-stack:
  added: []
  patterns:
    - Separate complete canonical overlay projection from the non-exact navigable difference list
    - Preserve controller, checkpoint, camera-world, and semantic selection identity across presentation changes
    - Reuse the retained renderer capability path for deterministic non-authoritative diagnostic capture
key-files:
  created:
    - crates/liquidfun-testbed/src/ui/differences.rs
    - crates/liquidfun-testbed/src/ui/inspector.rs
    - crates/liquidfun-testbed/src/ui/layout.rs
    - crates/liquidfun-testbed/src/ui/accessibility.rs
    - crates/liquidfun-testbed/src/screenshot.rs
    - crates/liquidfun-testbed/tests/visual_contract.rs
  modified:
    - crates/liquidfun-testbed/src/lib.rs
    - crates/liquidfun-testbed/src/main.rs
    - crates/liquidfun-testbed/src/ui.rs
key-decisions:
  - "Keep exact-match entries in the complete canonical overlay projection at 35 percent opacity while excluding them from navigable Difference rows."
  - "Represent backend availability, comparison mode, panel behavior, inspector state, and focus return as closed presentation enums with no simulation command path."
  - "Build visual diagnostics by reusing the verified confined Macroquad capability capture and attach immutable fixture hash, commit provenance, and explicit comparison/evidence exclusions."
patterns-established:
  - "Semantic diff projection: mode changes reuse identical canonical paths, policies, primitive keys, and synchronized camera state."
  - "Responsive identity retention: resize, DPI, drawers, and sheets mutate only local presentation metadata."
requirements-completed: [EXMP-04, EXMP-05]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T14:49:36Z
duration: 14 min
completed: 2026-07-22
---

# Phase 11 Plan 27: Accessible Semantic Comparison Testbed Summary

**The private testbed now presents one canonical semantic comparison through synchronized overlay or side-by-side modes with exact responsive, accessibility, retained-error, and diagnostic non-authority contracts.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-07-22T14:35:35Z
- **Completed:** 2026-07-22T14:49:36Z
- **Tasks:** 1
- **Files modified:** 9

## Accomplishments

- Added a canonical semantic projection that preserves paths, named policies, stable primitive keys, bounded values, and one synchronized camera across overlay and side-by-side presentation.
- Encoded exact match, within-policy, physics mismatch, Rust-only, and oracle-only states with the approved color plus icon/text/shape/stroke redundancy; exact matches render at 35% but do not pollute Difference navigation.
- Added canonical-path sorting, stable wrapping previous/next navigation, explicit ordinal announcements, and exact empty-difference handling.
- Added Run, Observe, Differences, and Provenance inspector tabs with exact operational copy, distinct harness and physics authority, bounded details, and retained last-valid checkpoint identity.
- Added every required breakpoint, 44px two-row compact controls, mutually exclusive drawers, full-window sheets, a once-per-session compact notice, minimum-window Close/About access, focus entry/return, and identity-preserving resize/DPI behavior.
- Added exact 44px target, 2px persistent focus ring, contrast, no-flashing, reduced-motion, selectable-value, normal focus-order, and concise focused-mismatch announcement contracts.
- Added the `--visual-contract-check` command, which reuses verified confined Macroquad PNG capture and emits deterministic fixture hash, commit provenance, regular-file evidence, and explicit screenshot exclusion from comparison and compatibility evidence.

## TDD Evidence

- **RED:** `cargo test -p liquidfun-testbed --test visual_contract` failed only because the planned `screenshot` and `ui::{accessibility,differences,inspector,layout}` modules did not exist.
- **GREEN:** The focused target passes 13/13 tests across comparison-mode equivalence, canonical/empty navigation, redundant state cues, all breakpoints, once-only compact notice, modal focus return, identity retention, accessibility, state copy, bounded errors, and deterministic diagnostic capture.
- **REFACTOR:** Split full canonical overlay projection from non-exact Difference navigation after the simplification review caught exact matches being navigable; focused Clippy and all tests remained green.

The intentionally failing RED state was not committed because repository policy requires every tracked commit to follow the complete passing ordered Rust gate.

## Task Commits

1. **Task 1: Implement the full differences, responsive, accessibility, and error contract** - `90d38d2` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-testbed/src/ui/differences.rs` - Complete canonical projection, overlay/side-by-side/single-backend modes, non-exact Difference rows, synchronized camera, canonical ordering, navigation, and exact visual cues.
- `crates/liquidfun-testbed/src/ui/inspector.rs` - Four tabs, exact state copy, bounded error details, recovery actions, and retained checkpoint identity.
- `crates/liquidfun-testbed/src/ui/layout.rs` - Exact responsive breakpoints, panel modes, compact notice, minimum-window affordances, modal focus return, and identity-preserving resize/DPI state.
- `crates/liquidfun-testbed/src/ui/accessibility.rs` - Target, focus, contrast, motion, flashing, selectable-value, focus-order, and bounded mismatch-announcement contracts.
- `crates/liquidfun-testbed/src/screenshot.rs` - Deterministic diagnostic report over confined renderer artifacts with provenance/hash and explicit authority exclusions.
- `crates/liquidfun-testbed/tests/visual_contract.rs` - Thirteen focused UI-SPEC behavior tests and deterministic replay assertions.
- `crates/liquidfun-testbed/src/lib.rs`, `crates/liquidfun-testbed/src/ui.rs`, and `crates/liquidfun-testbed/src/main.rs` - Module wiring and the exact visual-contract command surface.

## Decisions Made

- Comparison mode is a presentation choice only. `DifferenceList` stores immutable `ComparisonEntry` references, and its mode switch cannot reread engine state, bind policies, or submit a controller command.
- The complete canonical projection includes exact matches for overlay fading, while only non-exact entries enter keyboard Difference navigation and ordinal announcements.
- Responsive layout owns no controller object. Its identity snapshot retains the synchronized camera, semantic selection, checkpoint, and controller identity while size and DPI remain local values.
- Error panels accept only bounded non-control text, distinguish recoverable scenario errors from harness failure, and retain only stable checkpoint identity rather than private records or raw stderr.
- Diagnostic capture reuses the Plan 24 capability renderer and path confinement. Its report explicitly sets both `contributes_to_comparison` and `contributes_to_evidence` to false.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Wired new modules and the required visual-contract command into package entrypoints**

- **Found during:** Task 1 GREEN implementation
- **Issue:** The primary Plan 27 file list omitted `lib.rs`, `ui.rs`, and `main.rs`, but the new modules could not compile or satisfy the plan's exact command without those entrypoint changes.
- **Fix:** Exported the new UI and screenshot modules and extended the existing private binary parser with the structured `--visual-contract-check --fixture ... --output ...` command.
- **Files modified:** `crates/liquidfun-testbed/src/lib.rs`, `crates/liquidfun-testbed/src/ui.rs`, `crates/liquidfun-testbed/src/main.rs`
- **Verification:** Focused testbed Clippy, package tests, exact CLI execution, and the ordered Rust gate passed.
- **Committed in:** `90d38d2`

**2. [Rule 1 - Bug] Removed exact matches from navigable Difference rows without weakening overlay projection**

- **Found during:** Task 1 simplification review
- **Issue:** The first GREEN model treated every exact-match comparison entry as a navigable difference, conflicting with the approved empty-differences state.
- **Fix:** Retained all canonical entries for overlay/side-by-side rendering while deriving the keyboard Difference list from non-exact states only.
- **Files modified:** `crates/liquidfun-testbed/src/ui/differences.rs`, `crates/liquidfun-testbed/tests/visual_contract.rs`
- **Verification:** A regression test proves the exact model retains 10 overlay entries, exposes zero Difference rows, and announces `No differences at this checkpoint`.
- **Committed in:** `90d38d2`

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug). **Impact on plan:** Both changes were required to compile the specified command and preserve the approved semantic difference contract; no renderer or simulation authority was added.

## Issues Encountered

- The known Phase 10 protocol lint correction remains fenced and uncommitted in the root worktree. It was not imported into this branch and was unnecessary for Plan 27's focused no-dependency Clippy or the mandated default-member ordered Rust gate.

## Security Verification

- Comparison presentation accepts only validated bounded `ComparisonModel` entries and preserves canonical paths, named policies, and stable semantic keys without private-engine reads.
- Layout, focus, camera, DPI, mode, and screenshot interactions expose no controller command, logical tick, or checkpoint creation path.
- Error and announcement fields reject empty, oversized, or control-bearing text; screenshot provenance accepts only lowercase public commit identities or literal `Unavailable`.
- Diagnostic output reuses target-confined, traversal/link-rejecting regular PNG generation and writes a bounded deterministic JSON report.
- Diagnostic reports explicitly cannot contribute to semantic comparison or compatibility evidence, and no raw records, pointers, private indices, stack traces, or unbounded stderr enter UI copy.
- Renderer and C++ dependencies remain private, unpublished, non-default, and absent from the production `liquidfun` dependency graph.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Verification Evidence

- Focused Plan 27 tests: 13/13 passed with `cargo test -p liquidfun-testbed --test visual_contract`.
- Existing visual regressions: `app_shell` 8/8, `capability` 2/2, and `controller_ui` 15/15 passed; complete testbed package tests passed.
- Focused testbed Clippy passed with `cargo clippy -p liquidfun-testbed --all-targets --all-features --no-deps -- -D warnings`.
- Exact plan command passed and reproduced the reviewed three PNG SHA-256 identities under `target/testbed-visual-contract`.
- Mandatory ordered gate passed with a fresh temporary `CARGO_TARGET_DIR`: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- `git diff --check`, stub-pattern scan, file-size review, threat-surface review, and root fenced-status verification passed.

## Requirements Status

Plan 11-27's `EXMP-04` and `EXMP-05` mappings are implemented and recorded in summary frontmatter. Shared requirement checkboxes remain for the orchestrator's Phase 11 integration flow because Plans 11-23, 11-28, and 11-29 are still incomplete on this isolated branch.

## User Setup Required

None - all comparison, responsive, accessibility, error, and diagnostic-capture checks are deterministic and headless.

## Next Phase Readiness

- Plan 11-28 can integrate the completed controller, viewport, responsive inspector, semantic differences, and deterministic diagnostic capture surfaces.
- Plan 11-29 can audit the complete testbed/evidence boundary with explicit pixel and timing authority exclusions.
- No Plan 11-27 blocker remains.

## Known Stubs

None - all created Plan 27 modules are wired to immutable comparison, viewport, layout, accessibility, or diagnostic-capture inputs; no placeholder data reaches rendering.

## Self-Check: PASSED

- Confirmed all six primary created artifacts exist and implementation commit `90d38d2` is present.
- Confirmed focused tests, exact CLI execution, complete testbed regressions, focused Clippy, and the complete ordered Rust gate pass.
- Confirmed the isolated branch is based exactly on `c595f0e598537a8033b8410bea3f3f38dc5b3b57` and the four fenced root edits remain untouched and absent from the task commit.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
