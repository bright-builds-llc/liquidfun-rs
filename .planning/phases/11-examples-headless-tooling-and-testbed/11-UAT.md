---
status: complete
phase: 11-examples-headless-tooling-and-testbed
source:
  - 11-01-SUMMARY.md
  - 11-02-SUMMARY.md
  - 11-03-SUMMARY.md
  - 11-04-SUMMARY.md
  - 11-05-SUMMARY.md
  - 11-06-SUMMARY.md
  - 11-07-SUMMARY.md
  - 11-08-SUMMARY.md
  - 11-09-SUMMARY.md
  - 11-10-SUMMARY.md
  - 11-11-SUMMARY.md
  - 11-12-SUMMARY.md
  - 11-13-SUMMARY.md
  - 11-14-SUMMARY.md
  - 11-15-SUMMARY.md
  - 11-16-SUMMARY.md
  - 11-17-SUMMARY.md
  - 11-18-SUMMARY.md
  - 11-19-SUMMARY.md
  - 11-20-SUMMARY.md
  - 11-21-SUMMARY.md
  - 11-22-SUMMARY.md
  - 11-23-SUMMARY.md
  - 11-24-SUMMARY.md
  - 11-25-SUMMARY.md
  - 11-26-SUMMARY.md
  - 11-27-SUMMARY.md
  - 11-28-SUMMARY.md
  - 11-29-SUMMARY.md
started: 2026-07-23T03:12:51Z
updated: 2026-07-23T03:38:08Z
---

# Phase 11 User Acceptance Testing

## Current Test

[testing complete]

## Tests

### 1. Headless catalog and deterministic controls

expected: Named or seeded reviewed scenarios run through the renderer-neutral catalog with deterministic pause, restart, step, action, capture, replay, and benchmark semantics.
result: pass
verified_by: agent
evidence: "The Phase 11 verifier records 91/91 must-haves and the ordered Rust 1.97.0 gate passed after the latest testbed fixes: cargo fmt --all; cargo clippy --all-targets --all-features -- -D warnings; cargo build --all-targets --all-features; cargo test --all-features."

### 2. Desktop visual and complete input flow

expected: The interactive testbed opens with readable approved styling and the reviewed scenario catalog. Search, row selection, Run/Pause/Step/Restart/Capture, settings, scenario actions, layer controls, pointer selection, zoom, pan, reset, keyboard focus, About, Help, and responsive/minimum-window controls behave visibly without UI errors or losing the selected scenario.
result: pass

### 3. Particle scenario presentation and diagnostics

expected: Select `particle-contacts-and-coupling`, click Run, and wait for completion. The viewport must show two particle markers and the exact historical-checkpoint notice. Inspect must show distinct Displayed `(last drawable)` and Latest `(empty after teardown)` checkpoint rows, `P:2`, retained/collected primitive counts of `2/2`, particle draw count `2`, all nine layer categories, `layers 9/9` before toggles, and numeric diagnostic FPS. Repeating with `particle-aabb-query-controls` must also show two retained particles and the same particle/primitive counts after Run.
result: pass

### 4. Live Rust/oracle comparison presentation

expected: Launch with `--oracle-checkpoint PATH` using validated matching inputs. Overlay must default to solid `R` Rust versus dashed `O` oracle geometry and label pixels as diagnostic-only. Side by side must preserve synchronized zoom, pan, and focused primitive. Inspect difference rows must expose canonical paths and explicit Rust, Oracle, and Policy values. Exact, within-policy, mismatch, Rust-only, and oracle-only states must use the documented green check, amber diamond plus policy, persistent red, orange `R`, and purple `O` cues without fabricated zero values. If no validated matching checkpoint is available, this test is blocked.
result: issue
reported: "Agent-controlled desktop UAT matched the selected native resolved SHA to the supplied oracle checkpoint and obtained a live Rust-only comparison in both Overlay and Side by side. The prior red `checkpoint comparison identity mismatch: resolved_sha256` error remained visible after the later comparison succeeded, contradicting the active comparison state."
severity: major

## Summary

total: 4
passed: 3
issues: 1
pending: 0
skipped: 0
blocked: 0

## Gaps

- truth: "A successful live Rust/oracle comparison replaces any prior identity-mismatch error with the current comparison state."
  status: failed
  reason: "Agent-controlled UAT observed that a stale `checkpoint comparison identity mismatch: resolved_sha256` error remained visible after a later comparison succeeded."
  severity: major
  test: 4
  root_cause: ""
  artifacts: []
  missing: []
  debug_session: ""
