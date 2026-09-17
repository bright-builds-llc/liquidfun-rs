---
quick_id: 260916-quk
status: complete
date: 2026-09-17
---

# Hobby scope and optional qualification

## Delivered

- Recorded the owner's 2026-09-16 hobby-project direction in PROJECT-SCOPE.md and active guidance. Linux x64 and controlled benchmark hardware are no longer ordinary completion prerequisites.
- Deferred Phase 15 strict qualification in active state/roadmap, retaining its original evidence and optional acceptance semantics. PLAT-01 is deferred rather than falsely completed.
- Applied both follow-up choices: local checks plus one macOS Cargo CI job; cross-platform, oracle, safety, fuzz, coverage, performance and regression campaigns manual-only. Retained explicit Linux opt-in and manual fuzz build-only mode.
- Preserved managed Bright Builds workflows and strict evidence validators. No package was published or remote qualification dispatched.

## Verification

- Ordered final gates passed: cargo fmt --all; cargo clippy --all-targets --all-features -- -D warnings; cargo build --all-targets --all-features; cargo test --all-features.
- Expanded focused workflow suites: coverage 6, performance 9, platform 9, regression 14, Phase 11 workflow 5 (43 total). Earlier full docs_contract, platform_workflow and phase11_evidence_cli suites also passed; the platform suite passed again after the review correction.
- cargo xtask docs check, just markdown-check, Bright Builds checks, actionlint for eight changed workflows and git diff --check passed.
- Local final logs: target/quick-260916-quk/gates-02/. Executor detail: target/quick-260916-quk/task2-result.md.
- Independent AI review: REVIEW.md. Corrected stale documentation and active routing, plus normalized uppercase fuzz candidate SHA input.

## Decisions and limits

The user explicitly accepted manual-only expensive checks during implementation, expanding the initial Linux-only plan. Targeted active STATE/ROADMAP edits were necessary to represent deferral accurately; existing CLI progress commands cannot express this scope change. Historical records were preserved.

Simplification pass retained direct workflow predicates and existing commands; no new dependency or generic policy framework was introduced. Smaller release paperwork and durable API/MSRV promises remain discussion topics. Managed standards automation may still run on Linux, separately from physics testing. Strict qualification remains not release-ready; no evidence was relabeled passing.
