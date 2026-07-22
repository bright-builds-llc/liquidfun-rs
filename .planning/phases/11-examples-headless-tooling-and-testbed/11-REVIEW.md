---
phase: 11-examples-headless-tooling-and-testbed
reviewed: 2026-07-22T18:24:17Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - TESTING.md
  - crates/liquidfun-differential/src/session.rs
  - crates/liquidfun-differential/src/session/tests.rs
  - crates/liquidfun-testbed/CAPABILITY.md
  - crates/liquidfun-testbed/Cargo.toml
  - crates/liquidfun-testbed/src/bin/interactive.rs
  - crates/liquidfun-testbed/src/capability.rs
  - crates/liquidfun-testbed/src/interactive.rs
  - crates/liquidfun-testbed/src/lib.rs
  - crates/liquidfun-testbed/src/ui.rs
  - crates/liquidfun-testbed/src/ui/protocol_viewport.rs
  - crates/liquidfun-testbed/tests/capability.rs
  - crates/liquidfun-testbed/tests/interactive.rs
  - tools/xtask/tests/phase10_evidence_cli.rs
findings:
  critical: 0
  warning: 0
  info: 2
  total: 2
status: issues_found
---

# Phase 11: Post-Fix Code Review Report

**Reviewed:** 2026-07-22T18:24:17Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

This standard review extended the prior Phase 11 capability review with the interactive gap-closure implementation, its live catalog/session integration, canonical protocol renderer, production launcher, documentation, and the differential session changes that let reviewed settings materialize through every supported step-action family. The review applied the repository-local guidance, Bright Builds sidecar, active standards override file, and the architecture, code-shape, frontend UI, testing, verification, and Rust standards.

No Critical or Warning findings remain. The implementation now keeps simulation ownership in the typed controller/session boundary; refreshes semantic captures during running; rejects sub-clock-resolution timesteps without advancing; bounds and validates checkpoint-file input; keys comparison caching by resolved identity and checkpoint; presents every comparison state truthfully; and keeps responsive drawer hit-testing aligned with visible regions.

EXMP-04 wiring now exposes contact, particle-contact, and broad-phase layers plus explicitly non-authoritative diagnostics. EXMP-05 wiring now presents canonical Rust/oracle values and named policies, supports synchronized overlay and side-by-side modes, and derives faded matches, backend-specific solid/dashed cues, and focused mismatch halos from immutable comparison entries and stable primitive keys. The `?` production bridge opens a bounded local shortcut overlay without submitting simulation commands.

The initial review findings for unbounded input, stale or falsely exact comparison presentation, incomplete responsive input regions, stale running captures, zero-duration timesteps, and missing EXMP-04/EXMP-05 production wiring were corrected before this report was finalized. The prior capability-report self-digest inconsistency remains informational, and the launcher smoke test still checks source-token presence rather than runtime behavior.

## Info

### IN-01: Persisted capability report records a null digest while the returned report records a hash

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun-testbed/src/capability.rs:178-194`

**Issue:** The report is serialized and written while `report_sha256` is `None`, then the in-memory object is updated with the hash of those bytes. The persisted JSON therefore contains `"report_sha256": null`, while the returned object records the persisted payload hash. The field describes two different serialized report contracts.

**Fix:** Define the digest contract explicitly. Either omit the self-digest field and publish a companion digest, or hash a clearly named canonical payload that excludes the digest field and then write the final report containing that payload digest. Add a test that reads the persisted report and compares it with the returned contract.

### IN-02: Launcher wiring smoke test proves token presence, not interactive behavior

**File:** `/Users/peterryszkiewicz/Repos/liquidfun-rs/crates/liquidfun-testbed/tests/interactive.rs:11-40`

**Issue:** `production_launcher_wires_the_live_catalog_controller_and_renderer` loads the launcher with `include_str!` and succeeds when selected source substrings exist. Dead, unreachable, or incorrectly connected calls would satisfy the assertions. The test did not detect the hidden-drawer hit region, stale comparison cache, inert difference shortcuts, or incomplete comparison rendering found and fixed during this review.

**Fix:** Keep this as a lightweight packaging smoke check, but move production event routing and presentation transitions into a renderer-independent state reducer. Exercise drawer visibility, shortcut help, comparison-mode/focus transitions, and controller-command emission through behavioral integration tests. Retain focused pure renderer tests for comparison style/key mapping.

## Verification Evidence

- `cargo check -p liquidfun-testbed -p liquidfun-differential --all-targets --all-features` passed on the corrected interactive/session snapshot.
- `cargo check -p liquidfun-testbed --all-targets` passed after the final comparison-style and shortcut-help corrections.
- `git diff --check` passed after the final corrections.
- Focused renderer unit coverage now exercises all eight protocol primitive variants, ordering/style preservation, layer filtering, invalid geometry, viewport bounds, and comparison style separation.
- Focused interactive integration coverage now exercises shared-catalog selection, canonical capture, fixed-time cadence, settings re-resolution, and fail-closed sub-clock-resolution timesteps.
- The companion Phase 11 security audit reports 174/174 controls secured with no open finding after the bounded-loader and comparison-state fixes.
- `cargo test -p liquidfun-testbed --all-features` passed: 6 library tests, 0 launcher tests, 8 `app_shell` tests, 2 capability tests, 15 `controller_ui` tests, 6 interactive tests, 13 `visual_contract` tests, and 0 doctests.

***

_Reviewed: 2026-07-22T18:24:17Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
