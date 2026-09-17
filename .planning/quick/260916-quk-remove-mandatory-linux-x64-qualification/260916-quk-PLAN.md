---
phase: quick-260916-quk
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - AGENTS.md
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
  - .planning/phases/15-re-establish-candidate-evidence/15-CONTEXT.md
  - .planning/phases/15-re-establish-candidate-evidence/15-DISCUSSION-LOG.md
  - README.md
  - RELEASE.md
  - TESTING.md
  - .github/workflows/ci.yml
  - .github/workflows/oracle.yml
  - tools/xtask/tests/platform_workflow.rs
  - tools/xtask/tests/phase11_evidence_cli/workflow.rs
autonomous: true
requirements: [PLAT-01, DOCS-09]
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-260916-quk-hobby-scope
generated_at: 2026-09-17T00:22:15Z
must_haves:
  truths:
    - Ordinary hobby development does not require Linux x64 qualification or a dedicated controlled Linux runner.
    - Pull requests and ordinary pushes retain macOS and Windows smoke checks without launching the Linux Cargo quality job or Ubuntu matrix branch.
    - Linux checks and oracle evidence remain explicitly selectable and retain truthful strict validation.
    - Missing strict evidence remains missing; historical frozen evidence is preserved.
  artifacts:
    - path: .github/workflows/ci.yml
      provides: Optional Linux checks and default macOS/Windows smoke matrix
    - path: .github/workflows/oracle.yml
      provides: Manually selected oracle producers
    - path: .planning/PROJECT.md
      provides: Active hobby and experimental project scope
  key_links:
    - from: .github/workflows/ci.yml
      to: tools/xtask/tests/platform_workflow.rs
      via: Contract tests for opt-in Linux routing
    - from: .github/workflows/oracle.yml
      to: tools/xtask/tests/phase11_evidence_cli/workflow.rs
      via: Explicit manual evidence-phase routing tests
---

## Plan revision — accepted owner choices

The owner subsequently chose local checks plus one hosted CI job and optional manual expensive checks. Task 2 now uses one default macOS Cargo smoke job and converts all repository-owned expensive validation schedules to manual dispatch, preserving manual capabilities and strict validators. Update corresponding workflow contract tests. Managed Bright Builds automation is outside this change. Earlier references to retaining Windows smokes and pending schedule decisions are superseded by this revision.

<objective>
Apply the owner's explicit removal of mandatory Linux x64 qualification and describe liquidfun-rs as a fun, experimental hobby project. Preserve the strict qualification machinery as an optional, accurately labeled tool contract.

Scope is bounded: broader removal of expensive schedules, other platform guarantees, and release-procedure simplification remain proposals awaiting discussion. This plan does not declare release readiness or authorize publication.
</objective>

<context>
@AGENTS.md
@AGENTS.bright-builds.md
@standards-overrides.md
@standards/core/verification.md
@standards/core/testing.md
@standards/languages/rust.md
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/phases/15-re-establish-candidate-evidence/15-CONTEXT.md
@.github/workflows/ci.yml
@.github/workflows/oracle.yml

The local instructions require truthful evidence, native Cargo independence, ordered Rust verification, Markdown checks, and preservation of failed records. Existing tests inspect workflow job conditions, action pins, isolation, artifact names, and success-only evidence uploads. Reuse those contracts. Discovery is Level 0: existing workflow and documentation patterns, no new dependency.

Decision coverage: D-Q01 hobby/experimental description → Task 1 (full); D-Q02 Linux qualification and dedicated runner optional for ordinary completion → Tasks 1–2 (full); D-Q03 preserve strict validators and historical evidence → Tasks 1–2 (full). These are quick-task labels for the current owner instruction, not replacements for historical Phase 15 decision IDs.

Tasks have disjoint ownership and may proceed concurrently. The parent owns documentation and planning updates; the executor owns the two workflow files and two workflow test files. Agents are not alone in the checkout: preserve concurrent edits. Parent-owned state CLI, task/lesson records, commits, and publication are outside this plan's file ownership.
</context>

<tasks>
<task type="auto">
  <name>Task 1: Record the hobby scope and optional strict qualification policy</name>
  <files>AGENTS.md, .planning/PROJECT.md, .planning/REQUIREMENTS.md, .planning/phases/15-re-establish-candidate-evidence/15-CONTEXT.md, .planning/phases/15-re-establish-candidate-evidence/15-DISCUSSION-LOG.md, README.md, RELEASE.md, TESTING.md</files>
  <action>Per D-Q01–D-Q03, update the current project description and prominent README/RELEASE/TESTING guidance to distinguish ordinary hobby development from opt-in strict qualification. Record the owner's dated scope change in Phase 15 context and discussion: mandatory Linux x64 qualification and the dedicated controlled runner are removed as ordinary completion prerequisites; PLAT-01 is deferred by scope change, never completed by missing evidence. Add current policy outside the managed Bright Builds block and reconcile the active project description. Preserve historical 0f5 frozen-candidate records and explain that this scope change permits new source work without retroactively changing that evidence. Keep existing 21/19 strict qualification/report semantics and applicable strict prose intact. Record expensive schedule reductions, other platform guarantees, and release simplification as unresolved proposals only. Apply targeted edits and format only non-GSD Markdown with the configured Python 3.13/mdformat toolchain; never format .planning with mdformat.</action>
  <verify><automated>just markdown-check; cargo test -p xtask --test docs_contract</automated></verify>
  <done>Active scope and top-level guidance consistently identify hobby development and optional Linux qualification; PLAT-01 is deferred; strict evidence is neither fabricated nor relabeled passing; broader decisions remain pending.</done>
</task>

<task type="auto">
  <name>Task 2: Make Linux CI and oracle producers explicit manual choices</name>
  <files>.github/workflows/ci.yml, .github/workflows/oracle.yml, tools/xtask/tests/platform_workflow.rs, tools/xtask/tests/phase11_evidence_cli/workflow.rs</files>
  <action>Per D-Q02–D-Q03, add workflow_dispatch input run_linux_checks with boolean type and default false to Cargo CI. Gate Linux quality on workflow_dispatch plus that input. Use a GitHub Actions expression/fromJSON matrix so ordinary PR/push and manual false run macos-15/windows-2025, while manual true also includes ubuntu-24.04. Preserve the existing commands and submodule-free isolation. Make oracle Linux producers manual-only, retaining every selectable evidence_phase mode and its matching job conditions; remove automatic oracle triggers rather than leaving a scheduled workflow with no intended jobs. Preserve existing manual macOS/Windows oracle routes, pinned actions, permissions, same-run identity, failure diagnostics, success-only evidence uploads, and strict validators. Update focused tests to assert event/input routing, default false, both matrix alternatives, manual phase selection, and preservation of isolation and evidence boundaries. Keep unrelated workflows and release registry/attestation code unchanged. Avoid inventing Linux success or dispatching qualification runs as part of this task.</action>
  <verify><automated>cargo test -p xtask --test platform_workflow; cargo test -p xtask --test phase11_evidence_cli workflow; actionlint .github/workflows/ci.yml .github/workflows/oracle.yml</automated></verify>
  <done>Normal CI contains only the retained macOS/Windows smoke matrix; explicit manual opt-in restores Linux Cargo checks; oracle modes are manual and remain strictly validated; routing regression tests and actionlint pass.</done>
</task>
</tasks>

<threat-model>
## Trust Boundaries

Workflow inputs select runners and evidence modes; public documentation describes validation status.

| Threat ID | Category | Component | Disposition | Mitigation Plan |
| --- | --- | --- | --- | --- |
| T-Q-01 | T | Workflow routing | mitigate | Use typed boolean input, closed existing mode choices, explicit event predicates, and routing tests. |
| T-Q-02 | R | Evidence and public claims | mitigate | Preserve exact identity and strict validator contracts; mark scope removal/deferment explicitly instead of claiming evidence passed. |
</threat-model>

<verification>
Run focused workflow/documentation checks and actionlint, then the required Rust sequence in order: cargo fmt --all; cargo clippy --all-targets --all-features -- -D warnings; cargo build --all-targets --all-features; cargo test --all-features. Use the parent's working native environment and dedicated target directory. Each failed command stops its sequence and must be diagnosed. Also run just markdown-check, bun scripts/bright-builds-check.ts all, and git diff --check. Review the complete diff for unrelated workflow changes or altered evidence. Do not waive failed verification, fabricate a qualification pass, or perform commits from this bounded planner task.
</verification>

<success-criteria>
The owner's explicit hobby/Linux policy is implemented, workflow routing is regression-tested, historical evidence and opt-in strict acceptance retain their meaning, and pending broader scope choices are clearly identified for discussion.
</success-criteria>

<output>
Parent records completion evidence in .planning/quick/260916-quk-remove-mandatory-linux-x64-qualification/260916-quk-SUMMARY.md after implementation and verification.
</output>
