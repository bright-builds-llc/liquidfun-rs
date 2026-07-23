---
status: resolved
trigger: "Diagnose one Phase 11 UAT gap: after a previously failing oracle identity is corrected and a valid comparison is produced, Inspect and status surfaces must not continue showing the old identity mismatch."
created: 2026-07-22T22:41:03-05:00
updated: 2026-07-23T15:06:35Z
---

## Current Focus

hypothesis: `DesktopApp::refresh_comparison` records comparison failure in `maybe_error`, but its later `Ok(model)` branch does not clear or replace `maybe_error`; because `clear_comparison` also leaves `maybe_error` untouched, a failure survives scenario/settings changes and the later successful comparison.
test: Confirmed through complete producer/consumer tracing, UAT screenshots, comparator inspection, and expected-base comparison.
expecting: Confirmed: successful comparison and stale mismatch coexist because the desktop transition updates only one of two independent state fields.
next_action: Return the root-cause-only diagnosis; do not modify production code.

## Symptoms

expected: After a previously failing oracle identity is corrected and a valid comparison is produced, Inspect and status surfaces must not continue showing the old identity mismatch.
actual: Agent-controlled desktop UAT first produced an identity mismatch, then changed the selected scenario/settings until the native resolved SHA matched the supplied oracle. A live Rust-only comparison successfully appeared in Overlay and Side by side, but the old red identity-mismatch error remained visible.
errors: `checkpoint comparison identity mismatch: resolved_sha256`
reproduction: Test 4 in `.planning/phases/11-examples-headless-tooling-and-testbed/11-UAT.md`; screenshots are in gitignored `target/phase11-uat/exact-overlay.png` and `target/phase11-uat/rust-only-side-by-side-2.png`.
started: Discovered during Phase 11 UAT.

## Eliminated

- hypothesis: The comparator carries a prior identity error into a later successful model.
  evidence: `compare_canonical_checkpoints` is a stateless function that first validates current identities and then returns a newly constructed `ComparisonModel`; the model has no error field.
  timestamp: 2026-07-22T22:41:03-05:00

- hypothesis: The red mismatch is a stale render/cache artifact after application state was corrected.
  evidence: `draw_inspector` reads `self.maybe_comparison` and `self.maybe_error` directly on every draw. The fields genuinely coexist after the success branch because no transition clears the latter.
  timestamp: 2026-07-22T22:41:03-05:00

- hypothesis: An asynchronous operation restores the old mismatch after success.
  evidence: `DesktopApp::update` calls `refresh_comparison` synchronously after controller update/capture, and the source contains no deferred comparison-error writer. The only two `maybe_error` writes both assign `Some(...)`.
  timestamp: 2026-07-22T22:41:03-05:00

- hypothesis: Current unrelated dirty testbed edits introduced the missing reset.
  evidence: `git show d6df45155c623ab5bb9678c3aae20ab50e0fcf28:crates/liquidfun-testbed/src/bin/interactive.rs` shows the same success branch and `clear_comparison` omission at the expected base.
  timestamp: 2026-07-22T22:41:03-05:00

## Evidence

- timestamp: 2026-07-22T22:41:03-05:00
  checked: `git merge-base HEAD d6df45155c623ab5bb9678c3aae20ab50e0fcf28`
  found: The merge base is exactly `d6df45155c623ab5bb9678c3aae20ab50e0fcf28`.
  implication: The investigation is running on the expected base lineage; no branch or reset action is needed.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: Phase 11 UAT Test 4 and project state.
  found: The UAT records a valid live Rust-only comparison visible in both comparison modes while the earlier red `checkpoint comparison identity mismatch: resolved_sha256` remained visible.
  implication: Rendering comparison data succeeded; the failure is specifically in lifecycle replacement of previously recorded diagnostic state.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: Common bug-pattern checklist.
  found: The symptom maps directly to State Management candidates: dual source of truth, stale render, stale handler state, or invalid transition.
  implication: Trace state ownership and transitions before considering rendering or oracle computation defects.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: `DesktopApp` state and `refresh_comparison` in `crates/liquidfun-testbed/src/bin/interactive.rs`.
  found: Comparison state (`maybe_comparison`) and the diagnostic (`maybe_error`) are independent fields. Failure writes `maybe_comparison = None` and `maybe_error = Some(...)`; success writes only `maybe_comparison = Some(model)` and resets focused difference.
  implication: A successful comparison can coexist with the earlier error unless another lifecycle path explicitly clears `maybe_error`.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: `clear_comparison`, scenario selection, restart/settings actions, and Inspect rendering.
  found: Scenario selection and relevant controller actions call `clear_comparison`, but that method clears only `maybe_comparison` and `maybe_compared_identity`. Inspect independently renders `maybe_comparison` and then renders any `maybe_error` as `Last bounded error`.
  implication: The exact UAT state—live comparison plus stale red mismatch—is an allowed and deterministic state transition, not a stale-render or asynchronous timing effect.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: Every `maybe_error` mutation and comparison lifecycle reference in the testbed binary and integration tests.
  found: Production code has two writes, both assigning `Some(...)`; there is no `None`, `take`, or `replace` operation anywhere. No test references `refresh_comparison`, `Last bounded error`, or comparison identity mismatch in the interactive desktop lifecycle.
  implication: Once any bounded error is recorded, the field is permanent for the process lifetime, and the failure→success behavior lacks a regression guard.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: `target/phase11-uat/exact-overlay.png` and `target/phase11-uat/rust-only-side-by-side-2.png`.
  found: Both screenshots simultaneously show active `Comparison: RustOnly` content with focused `debug_primitives.0.presence` differences and the red `Last bounded error: checkpoint comparison identity mismatch: resolved_sha256`; the second screenshot shows the same coexistence in Side by side mode.
  implication: The UAT report is directly corroborated. The live model is present and usable while the prior diagnostic remains, matching the independent-field source trace.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: Renderer-neutral comparator implementation and comparison-model tests.
  found: `compare_canonical_checkpoints` validates only the current pair and returns a new `ComparisonModel` on success. `ComparisonModel` contains comparison identity/state/entries but no retained error. Existing tests cover comparison outcomes and comparator errors, not desktop failure→success lifecycle replacement.
  implication: Comparator behavior is correct for the observed successful Rust-only model; the stale diagnostic is introduced solely by the desktop state application layer.

- timestamp: 2026-07-22T22:41:03-05:00
  checked: Expected-base `DesktopApp::refresh_comparison`, `clear_comparison`, and error assignment at commit `d6df45155c623ab5bb9678c3aae20ab50e0fcf28`.
  found: The base has the same behavior as the worktree: success sets `maybe_comparison` and focused difference only; failure sets `maybe_error`; reset clears only comparison and compared identity.
  implication: The defect is an original invalid state transition in the desktop comparison lifecycle, independent of the current shared dirty changes.

## Resolution

root_cause: `DesktopApp` models current comparison and last bounded error as independent fields but never retires an error. `refresh_comparison` assigns `maybe_error = Some(...)` on identity failure, while its later success branch assigns only `maybe_comparison = Some(model)`; `clear_comparison` likewise does not clear the error. Inspect renders both fields independently, so failure → scenario/settings reset → successful comparison deterministically produces a live model plus the stale red mismatch.
fix: Plan 11-30 introduced one compiled `DesktopDiagnostics` owner for comparison and generic diagnostic channels; success and reset retire only the comparison-scoped error.
verification: Three compiled lifecycle regressions, the ordered Rust gate, and agent-controlled Overlay and Side-by-side UAT passed without the stale `resolved_sha256` error.
files_changed:
  - crates/liquidfun-testbed/src/bin/interactive.rs
  - crates/liquidfun-testbed/tests/comparison_lifecycle.rs
