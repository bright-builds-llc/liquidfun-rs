---
phase: 11-examples-headless-tooling-and-testbed
plan: "25"
subsystem: passive-visual-testbed-shell
tags: [macroquad, ui, accessibility, catalog, provenance]
requires:
  - phase: 11-examples-headless-tooling-and-testbed
    plan: "24"
    provides: Selected private Macroquad renderer and passive adapter boundary
provides:
  - Presentation-only app state with typed SessionCommand effects
  - Exact dark theme, typography, spacing, focus, contrast, and five-region layout contracts
  - Stable catalog browser and exact resolved-run identity presentation
  - Safe bounded source, license, maintainer, and build provenance chrome
affects: [phase11-testbed-controls, phase11-testbed-differences, phase12-portability]
tech-stack:
  added: []
  patterns:
    - Borrow controller, comparison, and run identity as immutable inputs while AppState owns presentation only
    - Derive searchable rows from the checked typed catalog and select by stable slug/version/seed
    - Restrict external links to fixed HTTPS disclosures or validated lowercase commit identities
key-files:
  created:
    - crates/liquidfun-testbed/src/app.rs
    - crates/liquidfun-testbed/src/app/state.rs
    - crates/liquidfun-testbed/src/theme.rs
    - crates/liquidfun-testbed/src/ui/about.rs
    - crates/liquidfun-testbed/src/ui/scenario_browser.rs
    - crates/liquidfun-testbed/tests/app_shell.rs
  modified:
    - crates/liquidfun-testbed/src/lib.rs
    - crates/liquidfun-testbed/src/ui.rs
key-decisions:
  - "Keep AppState limited to camera, panels, scroll, focus, catalog filters/selections, semantic focus, and screenshot presentation; effects leave the shell only as typed SessionCommand values."
  - "Project searchable rows from ScenarioCatalog in canonical order and retain stable slug, version, seed support, eligibility, and exact resolved SHA-256 instead of treating display titles as identity."
  - "Represent source, license, upstream notices, OpenLinks, and commits as allowlisted HTTPS links with visible copy fallbacks; sanitize and bound every optional provenance value before display."
patterns-established:
  - "Passive shell: borrowed semantic inputs flow into product chrome while mutable local state cannot advance or reinterpret a run."
  - "Non-color state cues: compact status text is always paired with a distinct glyph and exact accessible theme metadata."
requirements-completed: [EXMP-04, EXMP-05, EXMP-06]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 11-2026-07-21T21-10-55
generated_at: 2026-07-22T07:05:00Z
duration: 19 min
completed: 2026-07-22
---

# Phase 11 Plan 25: Passive Visual Testbed Shell Summary

**The private testbed now has an exact accessible dark shell, stable keyboard catalog browser, and bounded provenance chrome without taking simulation or comparison authority.**

## Performance

- **Duration:** 19 min
- **Started:** 2026-07-22T06:46:00Z
- **Completed:** 2026-07-22T07:05:00Z
- **Tasks:** 1
- **Files modified:** 8

## Accomplishments

- Added a pure responsive five-region layout with the exact 1280px contract: 48px app bar, 280px scenario rail, 640px fluid viewport, 360px inspector, and 64px control strip.
- Locked the approved 4/8/16/24/32/48/64 spacing, 4/8 radii, four font sizes and line heights, two font weights, 44px target, 2px focus ring, WCAG contrast metadata, and every exact dark/state color.
- Kept mutable `AppState` limited to camera, panels, scroll, focus, filters, stable presentation selections, and screenshot options; session/comparison/run values remain immutable inputs and effects are only typed `SessionCommand` submissions.
- Projected all reviewed catalog definitions into canonical searchable keyboard rows with display title, category, stable slug/version, seed support, Rust/oracle/visual eligibility, and 44px targets.
- Added exact operational empty/loading/error/status copy and a complete resolved-run identity view with catalog/generator versions, seed, settings, and resolved-byte SHA-256.
- Added normal-chrome source, separate Rust/upstream license truth, Peter Ryszkiewicz/OpenLinks attribution, version, linked commit, target/profile/toolchain/protocol/adapter/run/oracle/evidence fields, literal `Unavailable` fallbacks, and safe copyable HTTPS links.

## TDD Evidence

- **RED:** `CARGO_TARGET_DIR=/tmp/liquidfun-rs-phase11-11-25 cargo test -p liquidfun-testbed --test app_shell` failed on the deliberately absent `app`, `theme`, `ui`, and `app/state.rs` contracts.
- **GREEN:** The focused target passes 8/8 tests covering exact theme/layout/copy, stable catalog and run identity, presentation-only state, typed command effects, safe URLs, sanitation, and literal fallbacks.
- **REFACTOR:** Theme, shell, local state, About, and catalog browsing were split into cohesive modules; focused all-target/all-feature Clippy passes with warning denial.

The intentionally failing RED state was not committed because repository policy requires every commit to follow the complete passing ordered Rust gate.

## Task Commits

1. **Task 1: Implement the app shell, design tokens, scenario browser, and About panel** - `67d5e50` (feat)

**Plan metadata:** committed separately with this summary.

## Files Created/Modified

- `crates/liquidfun-testbed/src/app.rs` - Responsive shell layout, borrowed semantic inputs, typed effects, and exact status text/glyphs.
- `crates/liquidfun-testbed/src/app/state.rs` - Presentation-only camera, panels, scroll, focus, filters, stable selection, and screenshot state.
- `crates/liquidfun-testbed/src/theme.rs` - Exact spacing, typography, radii, palette, target, focus, contrast, and motion tokens.
- `crates/liquidfun-testbed/src/ui.rs` - Exact operational copy, app-bar projection, and resolved-run identity view.
- `crates/liquidfun-testbed/src/ui/about.rs` - Bounded provenance values and safe fixed/commit HTTPS disclosures with copy fallback.
- `crates/liquidfun-testbed/src/ui/scenario_browser.rs` - Canonical typed-registry projection, bounded search, keyboard focus, and stable selection.
- `crates/liquidfun-testbed/src/lib.rs` - Public private-package module exports.
- `crates/liquidfun-testbed/tests/app_shell.rs` - Pure shell, catalog, provenance, accessibility, and ownership-boundary tests.

## Decisions Made

- Mutable testbed state contains only presentation concerns. `ReadOnlyAppInputs` borrows comparison and exact run identity, copies the closed session state, and cannot execute work; `AppEffect` can only carry a typed `SessionCommand` to the external owner.
- The browser consumes the checked `ScenarioCatalog` directly, preserves registry order, searches title/slug/category for presentation, and returns only the stable slug/version/seed key. Seed support is verified against the typed resolver rather than copied from projection JSON.
- Every fixed disclosure URL is an exact HTTPS constant. Commit links are constructed only from 7–40 lowercase hexadecimal characters, and every platform-open intent retains the same visible copyable URL.
- Optional provenance is trimmed, ASCII/control checked, and capped at 192 bytes. Invalid or absent values become the literal `Unavailable` instead of guessed data, zeroes, or hidden fields.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Rust 1.97 does not yet permit trait-based `Ord::min` in this `const fn` layout path. The unsupported expression was replaced with an explicit constant-compatible branch without changing behavior.
- Focused Clippy identified strict float comparisons in tests, one redundant closure, and identical marker arms. Tests now use epsilon comparison and the marker match combines identical closed states.

## Security Verification

- Catalog search rejects control characters and values over 256 bytes before filtering; rows come only from the validated typed registry.
- Provenance text is bounded to 192 bytes, rejects non-ASCII/control text, and never exposes raw records, private indices, pointers, stderr, or stack traces.
- External destinations are fixed source/license/upstream/OpenLinks HTTPS targets plus a validated repository commit path; every action exposes a visible copy fallback.
- App state contains no session controller, comparison model, backend, engine storage, frame-time, or logical-tick field. Source-level regression checks fail if those ownership types appear.
- The testbed remains unpublished and non-default, and this plan added no dependency or production-crate edge.
- No unresolved high-severity OWASP ASVS L1 or STRIDE finding remains.

## Verification Evidence

- RED failure captured against missing shell modules and state source.
- Focused shell tests: 8/8 passed.
- Focused testbed Clippy: `cargo clippy -p liquidfun-testbed --all-targets --all-features -- -D warnings` passed.
- Acceptance scan confirms `app/state.rs` contains none of the prohibited ownership types or render-loop fields.
- `git diff --check` passed.
- Mandatory ordered gate passed with `/tmp/liquidfun-rs-phase11-11-25`: `cargo fmt --all`, full all-target/all-feature deny-warnings Clippy, all-target/all-feature build, and all-feature test.

## Requirements Status

Plan 11-25's `EXMP-04`, `EXMP-05`, and `EXMP-06` mappings are implemented and recorded in summary frontmatter. Global requirement checkboxes remain unchanged until all Phase 11 visual and evidence plans close their full scopes.

## User Setup Required

None - all shell, catalog, and provenance tests are pure and headless.

## Next Phase Readiness

- Plan 11-26 can layer camera, validated settings, overlay controls, and controller-command affordances onto the passive shell without changing ownership.
- Plan 11-27 can consume the exact theme/state markers and immutable comparison model for semantic difference presentation.
- No blocker remains for the remaining Phase 11 visual plans.

## Self-Check: PASSED

- Confirmed all six primary created artifacts exist and implementation commit `67d5e50` is present.
- Confirmed focused tests and Clippy, source ownership scan, diff check, and complete ordered Rust gate pass.
- Confirmed the four fenced pre-existing edits remain unstaged and uncommitted.

*Phase: 11-examples-headless-tooling-and-testbed*
*Completed: 2026-07-22*
