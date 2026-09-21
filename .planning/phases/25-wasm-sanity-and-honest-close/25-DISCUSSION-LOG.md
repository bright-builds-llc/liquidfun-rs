# Phase 25: WASM sanity and honest close - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-21T22:11:59.430Z
**Phase:** 25-wasm-sanity-and-honest-close
**Mode:** Yolo
**Areas discussed:** Six-scene proof, WASM Dam Break timing comparison, Remaining-delta note placement, Public-claim boundary

---

[auto-select] Selected all gray areas: Six-scene proof, WASM Dam Break timing comparison, Remaining-delta note placement, Public-claim boundary.

No pending todos matched Phase 25 (`todo_count: 0`). Advisor mode off (no USER-PROFILE.md).

Carrying forward: native Dam Break is the only C++ comparison; WASM is a post-gate sanity check; `reviewed_reports` stays empty; no public “Rust is N×” claim; `MAX_ADVANCE_STEPS` stays 4.

---

## Six-scene proof

| Option | Description | Selected |
|--------|-------------|----------|
| Existing `just web-player-smoke` | Chromium suite already opens the six scenes against the built Rust WASM player | ✓ |
| New visual suite plus Firefox/Safari | Expand browser coverage beyond the v1.1 smoke gate | |
| Live Pages redeploy as the gate | Require a new hosted revision before the phase can close | |

**User's choice:** [auto] Existing `just web-player-smoke` (recommended default)
**Notes:** Roadmap success criterion names this recipe. A failure is a real regression fix, not a scene or copy-lane rewrite. D-01, D-02.

---

## WASM Dam Break timing comparison

| Option | Description | Selected |
|--------|-------------|----------|
| Previous WASM or native Rust only | Optional committed unreviewed step-time note; native gate stamp if no prior WASM sample exists | ✓ |
| Browser Dam Break versus `oracle-release` | Treat the playground as another C++ pair | |
| Skip the note entirely | Smoke only, with no step-time sentence | |

**User's choice:** [auto] Previous WASM or native Rust only (recommended default)
**Notes:** REQUIREMENTS PERF-WASM forbids `oracle-release`. No `Instant` / `step_profiled` / samply in the cdylib, and the catch-up cap stays 4. D-03, D-04, D-05.

---

## Remaining-delta note placement

| Option | Description | Selected |
|--------|-------------|----------|
| Refresh existing audit, timing, and BENCHMARKING notes | Quote the verification gate stamp and name leftover causes plus what was not attempted | ✓ |
| New README speed section | Put the ratio in the public readme | |
| crates.io description | Publish an N× claim on the crate | |

**User's choice:** [auto] Refresh existing audit, timing, and BENCHMARKING notes (recommended default)
**Notes:** Close number is `2026-09-21T20-38-50Z` ratio `2.956857456935513`. Kernel-HEAD `2026-09-21T20-36-30Z` is a sibling sample. Do not mint a new pair unless those stamps are missing. D-06, D-07.

---

## Public-claim boundary

| Option | Description | Selected |
|--------|-------------|----------|
| No public N× claim; empty `reviewed_reports`; explicit not-sealed sentence | Playground pair stays an unreviewed local canary | ✓ |
| Paste the ratio into README | Universal speed sentence | |
| Add a `reviewed_reports` row | Treat the canary as a Phase 12 sealed report | |

**User's choice:** [auto] No public N× claim; empty `reviewed_reports`; explicit not-sealed sentence (recommended default)
**Notes:** Matches PERF-NOTES and ROADMAP success criterion 3. D-08, D-09.

---

## Claude's Discretion

- Exact remaining-delta wording, as long as ratio, leftover causes, not-attempted list, and unreviewed label are present.
- Whether the WASM step-time note is one line or a short section in the timing doc.
- How the smoke run is cited without committing `target/web-build` logs.

## Deferred Ideas

- `PERF-WASM-ENG`: WASM-versus-native engineering beyond this sanity check.
- SIMD, default Rayon, PGO, or lifting `unsafe_code = "forbid"`.
- Filling `reviewed_reports`, a public speed claim, or package publication.
- Browser Dam Break versus `oracle-release`.
- Instant profiler in the cdylib, or lifting `MAX_ADVANCE_STEPS` above 4.
