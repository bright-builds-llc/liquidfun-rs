---
phase: 15-re-establish-candidate-evidence
plan: 13
subsystem: documentation
tags: [hobby-scope, experimental-release, contributor-guidance]
requires:
  - phase: 14-repair-windows-particle-group-invariants
    provides: Retained particle-group regression protection
provides:
  - Accepted experimental scope and compatibility policies
  - Short native package-preparation checklist
affects: [15-14, release-preparation]
tech-stack:
  added: []
  patterns: [Existing documentation and verification commands]
key-files:
  created: []
  modified: [PROJECT-SCOPE.md, AGENTS.md, standards-overrides.md, .planning/PROJECT.md, README.md, CONTRIBUTING.md, TESTING.md, BENCHMARKING.md, RELEASE.md]
key-decisions:
  - Preserve generated COMPATIBILITY.md unchanged; explain its optional strict meaning through current public guidance.
  - Keep exact docs-contract headings and strict non-ready markers while separating experimental preparation.
requirements-completed: [HOBBY-01, HOBBY-02]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 15-hobby-2026-09-17T00-43-37
generated_at: 2026-09-17T01:02:00Z
duration: 7min
completed: 2026-09-17
---

# Phase 15 Plan 13: Experimental scope and preparation Summary

**Accepted hobby policies now govern contributor guidance, with a six-step experimental preparation checklist using existing native and package checks.**

## Performance

- Started: 2026-09-17T00:55:00Z (approximately, executor dispatch)
- Completed: 2026-09-17T01:02:00Z
- Tasks: 3
- Files modified: 9 documentation files; generated compatibility report preserved

## Accomplishments

- Reconciled policy proposals into accepted best-effort platforms, incremental parity, experimental APIs, manual expensive checks and deferred durable guarantees. Compiler metadata remains unchanged.
- Clarified ordinary macOS CI, local checks and optional strict evidence; labeled retained Macroquad discussion historical and made no new GUI claim.
- Added exact regression, representative headless and package verification commands, implementation-bound macOS CI, truthful limitations/notices and identified independent review. Version selection, tags, publication and minimum-compiler verification remain separate pre-publication work.

## Task Commits

1. Task 1 — `838335d`: adopt experimental project policies.
1. Task 2 — `18d18b0`: align contributor guidance with hobby scope.
1. Task 3 — `466e9c1`: add experimental package preparation checklist.

## Verification and independent review

Each task passed the ordered `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features` sequence before commit. Markdown, managed checks and `git diff --check` passed. `cargo xtask docs check` passed; `cargo test -p xtask --test docs_contract` passed 43 tests. Logs are under `target/phase15-hobby/task1`, `task2` and `task3`.

Separate AI reviewer `/root/review15_hobby_docs` inspected each full task diff and evidence, with actual acknowledgments in `target/phase15-hobby/review-task-{1,2,3}.md`:

| Task | Review time UTC | Acknowledged content-set SHA-256 |
| --- | --- | --- |
| 1 | 2026-09-17T00:58:30.291353+00:00 | `1dc663a816d2bc94a568a798b6ee5c00a146ed8639ec1bb0b5cd7acaf4a89fd1` |
| 2 | 2026-09-17T01:00:28.157842+00:00 | `c58193ec10589816b60fcca7da3d0f408c2b1cec5c752fbf49990c4104bfcc02` |
| 3 | 2026-09-17T01:01:09.265602+00:00 | `8df14d5b3834b1d36bc82efce9e8adb7291f4c96e5216fbbec6292263e84e2c0` |

Content sets hash sorted repo-relative path, NUL, lowercase file SHA-256, LF entries. The task commits preserve those reviewed file bytes. This is independent AI review, not human approval or publication authority.

## Deviations from Plan

**[Rule 2 — ownership constraint] Preserve the generated compatibility report.** Task 2 listed COMPATIBILITY.md, but the report is entirely generated and contributor policy forbids manual edits. Kept it byte-for-byte unchanged and explained the strict-profile interpretation in README and scope guidance. No generator or validator change was necessary; docs contracts pass. Parent agreed with this bounded adjustment.

## Issues Encountered

Independent review caught the old full-parity backlog still labeled Active; Task 1 now labels it historical ambitions outside current acceptance. Docs verification rejected a renamed RELEASE heading, so the required `Versioning and MSRV` heading was retained with explicit strict-profile context. Both issues were corrected before task commits.

## Simplification and safety

Used existing documents and commands instead of introducing tooling, a registry or a certification framework. No runtime, dependency, compiler pin, managed-block, generated-report or strict-validator changes. No new trust-boundary surface or functional stubs were introduced. Existing illustrative benchmark placeholders remain explicitly non-claiming examples.

Guidance applied: AGENTS.md, Bright Builds sidecar, overrides, verification/testing/Rust standards, writing-for-agents, current active lessons and D-10 through D-21. Parent orchestrator owns STATE, ROADMAP and requirements updates to avoid shared-file conflicts.

## Next Phase Readiness

Ready for Plan 15-14 to execute the preparation checks, gather the actual implementation commit's macOS result and record completion. This plan does not claim package preparation, GUI validation or strict qualification has passed.

## Self-Check: PASSED

All nine modified documents exist and the three task commits are present. COMPATIBILITY.md, Cargo.toml and rust-toolchain.toml remain unchanged from the plan baseline. Required strict `not release-ready` markers are retained.
