# Phase 35: Speed Up the Slowest Scenes - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-07
**Phase:** 35-speed-up-the-slowest-scenes
**Mode:** Yolo (recommended answers auto-selected)
**Areas discussed:** Targets, Where fixes go, Behavior preservation, Measurement and keep rule, Plan shape and records

---

## Targets

| Option | Description | Selected |
|--------|-------------|----------|
| Survey top five plus optional browser observation | Native ranking is authoritative; browser stutter adds scenes only if seen | ✓ |
| Top three only | Smaller scope, but misses washing-machine and particles | |
| All scenes above 1 ms/step | Eight scenes; drifts toward the shelved all-scene campaign | |

**Choice:** Top five, liquid-tumbler first; refresh the before run on current HEAD.

## Where fixes go

| Option | Description | Selected |
|--------|-------------|----------|
| Shared engine hot paths first | One fix can speed several particle-heavy scenes | ✓ |
| Per-scene fixes only | Isolated, but duplicates work and misses the common solver cost | |
| Lower authored settings (iterations, counts) | Out of scope per REQUIREMENTS | |

**Choice:** Shared engine first; scene code only for scene-specific per-step work.

## Behavior preservation

| Option | Description | Selected |
|--------|-------------|----------|
| Bit-identical end-state fingerprint for all 25 scenes | Objective, cheap, catches float reordering | ✓ |
| Existing tests only | May miss subtle trajectory changes | |
| Tolerance-based comparison | Allows float drift; weaker "unchanged behavior" guarantee | |

**Choice:** Fingerprint in survey tooling; float-changing edits are rejected.

## Measurement and keep rule

| Option | Description | Selected |
|--------|-------------|----------|
| After median below before min, no regressions past before max, ≥3 runs, samply profiles | Uses the survey's own spread as noise | ✓ |
| Fixed percentage threshold | Arbitrary; ignores per-scene noise | |
| Criterion microbenchmarks | New stack; Out of Scope | |

**Choice:** Survey-based keep rule plus a `--scene` filter for targeted runs.

## Plan shape and records

| Option | Description | Selected |
|--------|-------------|----------|
| Tooling + profiles plan first, then fix plans grouped by hot path | Matches shared-solver reality | ✓ |
| Strictly one plan per scene | Roadmap default; awkward when scenes share a hot path | |

**Choice:** Planner may group by hot path; records go in phase-local `35-PROFILES.md`.

## Claude's Discretion

- Fingerprint hash and extra state fields.
- Target order after liquid-tumbler and plan grouping.
- Specific optimization techniques within the behavior and safety rules.

## Deferred Ideas

- Browser frame-timing harness / WASM profiling.
- Opt-in parallel or SIMD stepping.
- Whole-catalog before/after and doc notes (Phase 36).
