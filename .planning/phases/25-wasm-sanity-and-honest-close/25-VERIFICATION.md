---
phase: 25-wasm-sanity-and-honest-close
verified: 2026-09-21T22:33:50Z
status: passed
score: 7/7 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-09-21T22-11-01
generated_at: 2026-09-21T22:33:50Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 25: WASM sanity and honest close Verification Report

**Phase Goal:** After the native 3× gate, a visitor can still run the six playground scenes, and a developer can read committed remaining-delta notes without a public “Rust is N×” claim.
**Verified:** 2026-09-21T22:33:50Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | After PERF-GATE, a visitor can still run Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel; developer records `just web-player-smoke` plus an optional Dam Break step-time note versus previous WASM or native Rust, never versus `oracle-release`. | ✓ VERIFIED | `25-02-SUMMARY.md` cites `just web-player-smoke` exit `0`, Playwright `37 passed` across four e2e files covering all six `SCENE_IDS`. Timing doc WASM note states never versus `oracle-release` / C++. Smoke not re-run; SUMMARY claim is present and self-consistent. |
| 2 | Developer can document the remaining Dam Break delta after the gate (ratio, suspected leftover causes, what was not attempted) in committed notes; README and crates.io do not gain a universal “Rust is N× slower/faster” claim. | ✓ VERIFIED | Audit **Remaining delta** + **Not attempted in this close** name ratio `2.956857456935513`, ~2% not gold-plated, SIMD/Rayon/PGO/`unsafe_code`/WASM-versus-C++/PERF-WASM-ENG/Phase 12 sealed. `README.md` / `crates/liquidfun/Cargo.toml` have no `2.956…` and no universal N× claim (`rg` empty). |
| 3 | `reviewed_reports` remains empty and committed benchmarking/audit notes state that the playground pair is not a Phase 12 sealed public claim. | ✓ VERIFIED | `reference/performance/manifest.toml` has `reviewed_reports = []`. `BENCHMARKING.md` honesty sentence: unreviewed local canary, not a Phase 12 sealed public claim. Audit/timing docs repeat unreviewed / not Phase 12 sealed. |
| 4 | Committed notes name Phase 24 gate stamp `2026-09-21T20-38-50Z` with `rust_over_cpp_ratio` `2.956857456935513` as the close number. | ✓ VERIFIED | Both `docs/playground-dam-break-timing.md` Current recorded sample and audit PERF-GATE leftover close lead with that stamp/ratio; walls match `target/dam-break-perf/2026-09-21T20-38-50Z/pair.json`. Stale “current 3× number is the first leftover” sentence absent. |
| 5 | Kernel-HEAD stamp `2026-09-21T20-36-30Z` with ratio `2.8769953439599707` is labeled sibling only, not the sole 3× number. | ✓ VERIFIED | Timing doc and audit use explicit sibling-only wording for that stamp/ratio; gate stamp remains the close number. |
| 6 | Optional WASM Dam Break step-time note cites native Rust `ms_per_step` `1.053103` from gate stamp `2026-09-21T20-38-50Z` and exact phrase `no prior WASM step-time sample`; never versus C++. | ✓ VERIFIED | Section present in `docs/playground-dam-break-timing.md` with all required phrases; no browser-invented ms/step; no `rust_over_cpp_ratio` attached to WASM wording. |
| 7 | `MAX_ADVANCE_STEPS` remains 4; no Instant / `step_profiled` / samply added to the WASM cdylib for this phase; smoke artifacts under `target/web-build` are not committed. | ✓ VERIFIED | `session.rs` still has `MAX_ADVANCE_STEPS: u32 = 4`; `clock.ts` `MAX_STEPS_PER_FRAME = 4`. Phase 25 commits touched docs/planning only (`63218cb`, `0ac325f`, `27fb752`, `d77501b`). Native-only `advance_profiled` / bench Instant paths pre-exist under `#[cfg(not(target_arch = "wasm32"))]` / diagnostic bins — not phase additions. `git status` shows no staged `target/web-build` / `*.json.gz`. |

**Score:** 7/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `docs/native-performance-audit.md` | Gate-stamp reconciliation, Remaining delta, Not attempted | ✓ VERIFIED | 304 lines; contains `2.956857456935513`, sibling demotion, Remaining delta, PERF-WASM-ENG |
| `docs/playground-dam-break-timing.md` | Gate sample table + WASM note | ✓ VERIFIED | Lead stamp `20-38-50Z`; WASM section with `no prior WASM step-time sample` |
| `BENCHMARKING.md` | Unreviewed canary / not Phase 12 sealed honesty | ✓ VERIFIED | Exact substrings `unreviewed local canary` and `not a Phase 12 sealed` |
| `reference/performance/manifest.toml` | Empty reviewed_reports only | ✓ VERIFIED | `reviewed_reports = []` unchanged |
| `crates/liquidfun-wasm/src/session.rs` | Catch-up cap invariant | ✓ VERIFIED | `MAX_ADVANCE_STEPS: u32 = 4`; not modified this phase |
| `justfile` | Existing `web-player-smoke` alias | ✓ VERIFIED | Recipe present; wires to `bun scripts/web-build.ts player-smoke` |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `docs/playground-dam-break-timing.md` | `target/dam-break-perf/2026-09-21T20-38-50Z/pair.json` | copied ratio / walls / ms_per_step | ✓ WIRED | Manual confirm: doc has `2.956857456935513` and `1.053103`; pair.json matches. (`gsd-tools verify key-links` false-negative from double-escaped regex.) |
| `docs/native-performance-audit.md` | `24-VERIFICATION.md` / gate stamp | gate vs sibling labeling | ✓ WIRED | Gate `20-38-50Z` promoted; sibling `20-36-30Z` demoted |
| `BENCHMARKING.md` | `reference/performance/manifest.toml` | unreviewed canary / empty reviewed_reports | ✓ WIRED | Honesty sentence + empty manifest |
| `justfile` | `scripts/web-build.ts` | `web-player-smoke` → player-smoke | ✓ WIRED | Alias unchanged |
| `session.rs` | `web/src/physics/clock.ts` | catch-up cap 4 | ✓ WIRED | Both still 4 |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Timing Current sample table | walls / ratio / ms_per_step | gate `pair.json` | Yes — values match live stamp | ✓ FLOWING |
| WASM step-time note | native `ms_per_step` `1.053103` | same gate `pair.json` | Yes — native citation only; no invented browser sample | ✓ FLOWING |
| Audit PERF-GATE leftover close | close ratio / walls | gate `pair.json` | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Gate ratio present in honesty docs | `rg` for `2.956857456935513` in audit + timing | Matches; pair.json confirms | ✓ PASS |
| Claim-free public surface | `rg` N× / ratio in README + Cargo.toml | No matches | ✓ PASS |
| Empty reviewed_reports | `rg 'reviewed_reports = \[\]'` | Match | ✓ PASS |
| Smoke citation (no re-run) | Inspect `25-02-SUMMARY.md` | Exit 0; 37 passed; six scenes named | ✓ PASS |
| Catch-up cap | `rg MAX_ADVANCE_STEPS` / `MAX_STEPS_PER_FRAME` | Both = 4 | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| PERF-NOTES | 25-01 | Document remaining Dam Break delta; no universal README/crates.io N× claim | ✓ SATISFIED | Audit Remaining delta + Not attempted; BENCHMARKING honesty; claim-free README/Cargo.toml; empty manifest |
| PERF-WASM | 25-02 | Six scenes still run; `just web-player-smoke` + optional WASM step-time note versus WASM/native only, never `oracle-release` | ✓ SATISFIED | SUMMARY smoke exit 0 / 37 passed / six scenes; WASM note with required phrases |

No orphaned Phase 25 requirements: REQUIREMENTS.md maps only PERF-NOTES and PERF-WASM to Phase 25; both appear in plan frontmatter.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `BENCHMARKING.md` | ~168 | “placeholder” in illustrative public-claim template | ℹ️ Info | Intentional claim-template prose, not a stub implementation |
| `crates/liquidfun-wasm/src/session.rs` | ~127–146 | `step_profiled` under `#[cfg(not(target_arch = "wasm32"))]` | ℹ️ Info | Pre-existing native diagnostic path; not added by Phase 25; WASM cdylib advance path unchanged |

No blocker stubs in phase-modified docs.

### Human Verification Required

None. Roadmap success criteria are satisfied by committed honesty docs plus recorded Chromium smoke evidence; visual/browser feel beyond the existing automated suite is out of phase scope (D-01).

### Gaps Summary

No gaps. Phase goal achieved: six playground scenes remain smoke-proven, remaining-delta notes are committed against gate stamp `2026-09-21T20-38-50Z` / ratio `2.956857456935513`, and the public claim boundary (empty `reviewed_reports`, no README/crate N× sentence) holds.

---

_Verified: 2026-09-21T22:33:50Z_
_Verifier: Claude (gsd-verifier)_
