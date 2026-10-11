---
id: SEED-006
status: dormant
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone needs a cleanup or filler phase, or touches particle performance, the WASM build, or web tests
scope: Medium
---

# SEED-006: v1.4 performance and tooling leftovers

## Why This Matters

Small follow-ups left by v1.4 Scenario Performance. None is urgent, but together they make a tidy filler phase and keep the native loop clean. The list is kept in `.planning/PROJECT.md` under "Next Milestone":
- A presortedness test to recover particles' A2r cost
- A7 per-fixture validation
- The world backup clone
- Emission reallocations
- The grouped creation load time
- The wasm32 native-only bins
- A release-only warning
- The stale `web-smoke` spec
- The filtered scene-spot summary line

## When to Surface

**Trigger:** The next milestone needs a cleanup or filler phase, or touches particle performance, the WASM build, or web tests

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- A cleanup, hardening, or filler phase is wanted
- Work touches particle hot paths, emission, or world cloning
- Work touches the WASM crate's binaries or the web smoke tests

## Scope Estimate

**Medium** — one or two phases. Performance items must keep all scene fingerprints unchanged and use the v1.4 ABBA keep rule.

## Breadcrumbs

- `.planning/PROJECT.md` — "Next Milestone" list (source of truth)
- `.planning/milestones/v1.4-ROADMAP.md` — v1.4 context and decisions
- `docs/benchmarks/scene-survey.md` — before/after scene timing tables
- `crates/liquidfun-wasm/src/bin/` — native-only bins
