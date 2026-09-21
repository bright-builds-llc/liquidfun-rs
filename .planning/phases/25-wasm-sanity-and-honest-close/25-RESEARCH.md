# Phase 25: WASM sanity and honest close - Research

**Researched:** 2026-09-21
**Domain:** Post-gate honesty documentation + existing Chromium playground smoke (no new physics, no WASM profiler)
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Six-scene proof
- **D-01:** Prove Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel still run with the existing `just web-player-smoke` Chromium suite (`bun scripts/web-build.ts player-smoke`). Do not add a Firefox/Safari matrix, a new visual suite, or a live Pages redeploy as this phase’s gate.
- **D-02:** A smoke failure is a regression to fix in shared stepping or the existing player. Do not rewrite scene builders or WASM copy lanes to force a pass, and do not change physics to make the browser look faster.

### WASM Dam Break timing comparison
- **D-03:** The optional Dam Break step-time note is a committed unreviewed sample versus previous WASM or native Rust only. Never versus `oracle-release` or any other C++ wall.
- **D-04:** If no prior WASM Dam Break step-time sample exists, cite native Rust ms/step from the unprofiled gate stamp and say “no prior WASM step-time sample.” Do not invent a browser-versus-C++ number.
- **D-05:** Do not add `Instant`, `step_profiled`, or samply to the cdylib. `MAX_ADVANCE_STEPS` stays 4. Do not lift the catch-up cap to fake realtime.

### Remaining-delta note placement
- **D-06:** Write the remaining Dam Break delta in the existing committed notes: `docs/native-performance-audit.md`, `docs/playground-dam-break-timing.md`, and a short honesty sentence in `BENCHMARKING.md`. Include the close ratio, suspected leftover causes already named, and what was not attempted (SIMD, Rayon, PGO, lifting `unsafe_code = "forbid"`, WASM-versus-C++, WASM engineering beyond this sanity check, Phase 12 sealed matrix, gold-plating leftover ~2% frames).
- **D-07:** The close number is the Phase 24 verification gate stamp `target/dam-break-perf/2026-09-21T20-38-50Z` (`rust_over_cpp_ratio` `2.956857456935513`, copied from that `pair.json` / `24-VERIFICATION.md`). The kernel-HEAD stamp `2026-09-21T20-36-30Z` (`2.8769953439599707`) may be named only as a sibling sample. Reconcile the audit and timing docs if they currently present the kernel-HEAD pair as the sole 3× number. Do not mint a new pair unless those stamps are missing.

### Public-claim boundary
- **D-08:** README and crates.io metadata do not gain a universal “Rust is N× slower/faster” sentence. Do not paste the ratio into README.
- **D-09:** `reference/performance/manifest.toml` `reviewed_reports` stays empty. Committed notes state that the playground pair is an unreviewed local canary, not a Phase 12 sealed public claim.

### Claude's Discretion
- Exact wording of the remaining-delta paragraphs, as long as the gate ratio, leftover causes, not-attempted list, and unreviewed label are present.
- Whether the optional WASM step-time is a one-line addition to the timing doc or a short sibling section.
- How the smoke run is cited in the phase summary (command, exit status, test count) without committing `target/web-build` logs.

### Deferred Ideas (OUT OF SCOPE)
- `PERF-WASM-ENG`: engineer WASM versus native stepping beyond this post-gate sanity check.
- SIMD, default Rayon, PGO, or lifting workspace `unsafe_code = "forbid"`.
- Filling `reference/performance/manifest.toml` or writing a public speed claim.
- Package publication or a release tag.
- Browser Dam Break versus `oracle-release`.
- An Instant profiler in the cdylib, or lifting `MAX_ADVANCE_STEPS` above 4.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-NOTES | Document remaining Dam Break delta after the gate (ratio, suspected leftover causes, what was not attempted) in committed notes; README and crates.io do not gain a universal “Rust is N×” claim. | Reconcile audit + timing docs to gate stamp `2026-09-21T20-38-50Z` ratio `2.956857456935513`; add remaining-delta / not-attempted paragraphs; short honesty sentence in `BENCHMARKING.md`; verify README / `crates/liquidfun/Cargo.toml` stay claim-free; leave `reviewed_reports = []`. |
| PERF-WASM | After PERF-GATE, visitor can still run the six playground scenes; record `just web-player-smoke` plus optional Dam Break step-time note versus previous WASM or native Rust, never versus `oracle-release`. | Reuse existing `just web-player-smoke` → `bun scripts/web-build.ts player-smoke` → `bun run test:player` (36 Chromium tests across four e2e files covering all six `SCENE_IDS`). No prior WASM step-time sample found → optional note cites gate native `ms_per_step` `1.053103` with explicit “no prior WASM step-time sample.” Do not touch `MAX_ADVANCE_STEPS` / cdylib Instant. |
</phase_requirements>

## Summary

Phase 25 is a **documentation + existing-smoke** close, not a new measurement or player feature phase. Phase 24 already closed the unprofiled Dam Break Medium pair ≤ 3×. Two exclusive stamps still exist on disk: the **gate** stamp `2026-09-21T20-38-50Z` (`rust_over_cpp_ratio` `2.956857456935513`, native `ms_per_step` `1.053103`) and the **kernel-HEAD sibling** `2026-09-21T20-36-30Z` (`2.8769953439599707`). Committed notes currently present the sibling as the sole “current 3× number”; Phase 25 must reconcile that labeling without minting a new pair.

No committed prior WASM Dam Break step-time sample exists under `docs/` or related honesty docs. Per D-04, the optional note should cite the gate stamp’s native Rust ms/step and state that no prior WASM sample exists—without adding browser Instant/`performance.now` profiling or comparing to C++. Six-scene proof is already encoded in `just web-player-smoke`; the phase only needs a fresh exit-0 run and a SUMMARY citation.

**Primary recommendation:** Split into two plans—(1) honesty/doc reconciliation for PERF-NOTES, (2) run-and-cite `just web-player-smoke` plus the optional one-line native-fallback WASM note for PERF-WASM—with zero physics, zero cdylib profiler, and zero `manifest.toml` changes.

## Standard Stack

### Core

| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| Existing `just web-player-smoke` | one-line `bun scripts/web-build.ts player-smoke` | Six-scene Chromium proof | Already the WEBTEST-01 / Phase 19 gate; D-01 forbids a new matrix `[VERIFIED: justfile + scripts/web-build.ts]` |
| Bun | 1.4.2 (local) | Orchestrates wasm regen, frontend build, Playwright | README pin; `web-build.ts` invokes `bun run test:player` `[VERIFIED: bun --version + README]` |
| Playwright Chromium | `@playwright/test` 1.63.0 | Browser assertions | `web/package.json` `test:player` + `browser:install` `[VERIFIED: web/package.json]` |
| Gate `pair.json` | stamp `2026-09-21T20-38-50Z` | Authoritative close ratio + native ms/step | D-07 / `24-VERIFICATION.md` `[VERIFIED: on-disk pair.json]` |

### Supporting

| Asset | Version / ID | Purpose | When to Use |
|-------|--------------|---------|-------------|
| `docs/native-performance-audit.md` | committed | Remaining-delta + leftover causes + Not found | PERF-NOTES body |
| `docs/playground-dam-break-timing.md` | committed | Unreviewed pair table + optional WASM note | PERF-NOTES + optional PERF-WASM note |
| `BENCHMARKING.md` | committed | Short honesty sentence; Phase 12 boundary | PERF-NOTES / D-09 |
| `reference/performance/manifest.toml` | `reviewed_reports = []` | Keep empty | D-09 verification only—do not edit contents except to confirm empty |
| `crates/liquidfun-wasm/src/session.rs` | `MAX_ADVANCE_STEPS = 4` | Catch-up cap invariant | Assert unchanged; do not edit |
| `web/src/physics/clock.ts` | `MAX_STEPS_PER_FRAME = 4` | JS mirror of catch-up cap | Assert unchanged |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Existing `just web-player-smoke` | New Firefox/Safari matrix or live Pages redeploy | Forbidden by D-01 |
| Cite gate native ms/step | Measure browser Dam Break with Instant / `performance.now` | Forbidden by D-05; PITFALLS warn Instant in cdylib |
| Gate stamp `20-38-50Z` | Mint a new pair | Forbidden unless stamps missing (they are present) |
| Fill `reviewed_reports` | Leave empty + honesty sentence | D-09 / REQUIREMENTS out-of-scope |

**Installation:** None for the phase itself. Smoke already requires the documented playground toolchain (`wasm32-unknown-unknown`, `wasm-pack` 0.15.0, Bun, Chromium via `playwright install chromium`).

**Version verification:** Local host probed 2026-09-21: `bun 1.4.2`, `just 1.48.0`, `cargo 1.97.0`, `wasm-pack 0.15.0`, `wasm32-unknown-unknown` installed, Playwright package present under `web/node_modules`. `[VERIFIED: shell probes]`

## Architecture Patterns

### Recommended Project Structure (edit surface only)

```
docs/
├── native-performance-audit.md      # Reconcile gate stamp; remaining-delta section
├── playground-dam-break-timing.md   # Reconcile gate stamp; optional WASM note
BENCHMARKING.md                      # One honesty sentence (playground ≠ Phase 12)
reference/performance/manifest.toml  # Verify reviewed_reports = [] only
justfile                             # Do not change web-player-smoke alias
crates/liquidfun-wasm/src/session.rs # Assert MAX_ADVANCE_STEPS == 4; no Instant
README.md / crates/liquidfun/Cargo.toml  # Assert no “Rust is N×” claim; do not add
```

### Pattern 1: Copy numbers from `pair.json` / verification, never invent

**What:** Every public-facing ratio or ms/step in committed notes is copied from an on-disk `pair.json` field or from `24-VERIFICATION.md`.
**When to use:** All PERF-NOTES numeric claims.
**Example:** Gate stamp fields (verified this session):

```text
# Source: target/dam-break-perf/2026-09-21T20-38-50Z/pair.json
kind: unprofiled_pair
timing_authority: unprofiled_wall_clock
git_head: 89d3456406c3c79ed500192bca9491c707033647
rust.wall_ms: 631.861834
rust.ms_per_step: 1.053103
cpp.wall_ms: 213.693708
rust_over_cpp_ratio: 2.956857456935513
```

### Pattern 2: Smoke citation without committing logs

**What:** Run `just web-player-smoke`; record command, exit 0, and Playwright file/test counts in plan SUMMARY / phase verification. Do not `git add` `target/web-build/`.
**When to use:** PERF-WASM scene proof (D-01, Claude’s discretion on citation form).

### Pattern 3: Optional WASM step-time without measuring WASM

**What:** Because no prior WASM Dam Break step-time sample exists in committed docs, add a short unreviewed note that cites **native** gate `ms_per_step` and says “no prior WASM step-time sample.” Never vs C++.
**When to use:** Optional PERF-WASM timing sentence (D-03/D-04). Prefer a one-line or short sibling subsection under `docs/playground-dam-break-timing.md` (discretion).

### Anti-Patterns to Avoid

- **Presenting kernel-HEAD `20-36-30Z` as the sole gate number:** Current timing + audit PERF-GATE sections do this; reconcile to `20-38-50Z` and demote `20-36-30Z` to sibling. `[VERIFIED: docs/playground-dam-break-timing.md + docs/native-performance-audit.md]`
- **Stale “current 3× number is the first leftover admission” sentence** in the Phase 23 unprofiled table section (audit ~L69–70); fix during reconciliation. `[VERIFIED: docs/native-performance-audit.md + 24-VERIFICATION.md anti-pattern note]`
- **Adding Instant / `step_profiled` / samply to the wasm cdylib path:** `dam_break_bench` / `scene_spot` Instant usage is already `#[cfg(not(target_arch = "wasm32"))]`. Keep it that way. `[VERIFIED: crates/liquidfun-wasm/src/lib.rs]`
- **Lifting `MAX_ADVANCE_STEPS` or `MAX_STEPS_PER_FRAME` above 4.**
- **Pasting the ratio into README or crate `description`.**
- **Rewriting scenes / copy lanes to force smoke green** (D-02).
- **Minting a new Dam Break pair** when both stamps exist (D-07).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Six-scene browser proof | New Playwright suite / visual regression / Pages deploy | `just web-player-smoke` | Already covers six scenes + pointer/control + hidden-tab max-4 `[VERIFIED: 19-VERIFICATION.md + web/e2e/player.spec.ts]` |
| WASM Dam Break step time | Instant in cdylib / browser `performance.now` timer | Cite gate native `ms_per_step` + “no prior WASM sample” | D-04/D-05; PITFALLS forbid Instant-in-WASM as Dam Break pair `[CITED: .planning/research/PITFALLS.md]` |
| Remaining-delta doc | New markdown book or Phase 12 report | Edit existing three honesty docs | D-06 |
| Public speed claim | README blurb / crates.io description | Keep claim-free; empty `reviewed_reports` | D-08/D-09; REQUIREMENTS out-of-scope |
| Close ratio | Re-run pair / round ratio | Copy `2.956857456935513` from gate `pair.json` | D-07 |

**Key insight:** The phase closes honesty gaps, not performance gaps. Almost all work is surgical doc edits plus one existing smoke command.

## Common Pitfalls

### Pitfall 1: Quoting the sibling stamp as the gate
**What goes wrong:** Timing and audit docs still treat `2026-09-21T20-36-30Z` / `2.876…` as “the” PERF-GATE number.
**Why it happens:** That pair was the first ≤3 closing kernel stamp; verification later named `20-38-50Z` as the official gate after a same-HEAD docs commit.
**How to avoid:** Lead with `20-38-50Z` / `2.956857456935513`; label `20-36-30Z` as sibling only.
**Warning signs:** Docs that omit `20-38-50Z` or call sibling “the 3× number.”

### Pitfall 2: Inventing a browser-versus-C++ number
**What goes wrong:** Optional WASM note compares playground to `oracle-release`.
**Why it happens:** Habit from the native Dam Break pair.
**How to avoid:** Only previous WASM or native Rust; with no WASM sample, cite native gate ms/step and say so (D-03/D-04).
**Warning signs:** Any `rust_over_cpp_ratio` attached to WASM/browser wording.

### Pitfall 3: “Fixing” smoke by changing physics or copy caps
**What goes wrong:** Scene builders, particle counts, or frame-copy lanes change to green the suite.
**Why it happens:** Smoke failure after shared-path waves.
**How to avoid:** Treat failure as a real regression in shared stepping or existing player (D-02). Do not lift the 4-step cap.
**Warning signs:** Diffs under `scene/*.rs` or `MAX_ADVANCE_STEPS` / `MAX_STEPS_PER_FRAME`.

### Pitfall 4: Committing smoke artifacts or filling the manifest
**What goes wrong:** `target/web-build/` logs or a filled `reviewed_reports` land in git.
**Why it happens:** Desire to “preserve evidence.”
**How to avoid:** Cite smoke in SUMMARY only; leave `reviewed_reports = []`.
**Warning signs:** `git status` shows `target/web-build` or non-empty `reviewed_reports`.

### Pitfall 5: README gains an N× sentence while “documenting honesty”
**What goes wrong:** Ratio pasted into README “for transparency.”
**Why it happens:** Misreading PERF-NOTES as a public claim.
**How to avoid:** Keep README/Cargo.toml claim-free (D-08); put numbers only in the existing unreviewed docs.
**Warning signs:** README diff mentioning `2.95` or “slower/faster.”

## Code Examples

### Exact smoke invocation chain

```bash
# Source: justfile + scripts/web-build.ts (player-smoke branch)
just web-player-smoke
# → bun scripts/web-build.ts player-smoke
# → verifyTools → regenerateWasm → runFrontendBuild
# → bun run browser:install
# → bun run test:player
# test:player = playwright test \
#   e2e/player.spec.ts \
#   e2e/demo-media-clock.spec.ts \
#   e2e/shell.spec.ts \
#   e2e/reset-honesty.spec.ts
```

**Test counts (committed suite):** 13 + 7 + 9 + 7 = **36** Playwright tests. `[VERIFIED: rg -c '^test(' on those four files]`

**Six scenes exercised:** `SCENE_IDS` = `dam-break`, `fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, `water-wheel` via `opens each native scene…` and `plays each scene, accepts one pointer gesture…` in `web/e2e/player.spec.ts`. `[VERIFIED: web/src/catalog/scenes.ts + player.spec.ts]`

### Catch-up cap (must remain 4)

```rust
// Source: crates/liquidfun-wasm/src/session.rs
pub(crate) const MAX_ADVANCE_STEPS: u32 = 4;
```

```typescript
// Source: web/src/physics/clock.ts
export const MAX_STEPS_PER_FRAME = 4;
```

### Suggested doc edit map (planner task checklist)

| File | Exact work |
|------|------------|
| `docs/playground-dam-break-timing.md` | Replace “Current recorded sample” lead stamp with `2026-09-21T20-38-50Z` table (`wall_ms` 631.861834 / 213.693708, ratio `2.956857456935513`, `git_head` `89d3456…`). Keep historical pointers. Name `20-36-30Z` as sibling. Add optional WASM note (native ms/step `1.053103` + “no prior WASM step-time sample”). |
| `docs/native-performance-audit.md` | In **PERF-GATE leftover close**, promote `20-38-50Z` as the close/gate number; keep landed leftover kernel list; add **remaining delta** paragraph (ratio still ~2.96×, leftover ~2% frames not gold-plated, suspected leftover causes already named in Not found / ranking text). Fix stale L69–70 “current 3× number is the first leftover admission.” Expand **Not found / not attempted** to include Phase 25 deferrals (WASM-vs-C++, PERF-WASM-ENG, SIMD/Rayon/PGO/`unsafe_code`, Phase 12 sealed matrix). |
| `BENCHMARKING.md` | Add one short honesty sentence under Exploratory local diagnosis: playground Dam Break pair is an unreviewed local canary, not a Phase 12 sealed public claim; `reviewed_reports` remains empty. Do not paste wall-ms. |
| `reference/performance/manifest.toml` | Verify only: `reviewed_reports = []`. |
| `README.md`, `crates/liquidfun/Cargo.toml` | Verify only: no universal speed claim (description is already claim-free). |
| `crates/liquidfun-wasm/src/session.rs` | Verify only: `MAX_ADVANCE_STEPS == 4`. |

### Suggested remaining-delta content (wording discretionary)

Must include:

1. Close ratio `2.956857456935513` from gate stamp `2026-09-21T20-38-50Z`.
1. Sibling-only mention of `2026-09-21T20-36-30Z` / `2.8769953439599707`.
1. Suspected leftovers already named: stop gold-plating ~2% residual frames after D-06; audit **Not found** list (SIMD-first, Rayon, PGO, lifting `unsafe_code`, WASM vs C++, Phase 12 matrix).
1. Not attempted in this close: SIMD, default Rayon, PGO, lifting `unsafe_code = "forbid"`, WASM-versus-C++, WASM engineering beyond sanity (`PERF-WASM-ENG`), Phase 12 sealed matrix, gold-plating leftover ~2% frames.
1. Unreviewed / not Phase 12 sealed public claim.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Phase 23 ~327× unprofiled pair as audit identity | Gate ≤3× unprofiled pair | Phase 24 | Close number is ~2.96×, not 327× |
| Kernel-HEAD stamp presented as sole 3× | Gate stamp `20-38-50Z` is close number; sibling optional | Phase 24 verification / Phase 25 docs | Doc reconciliation required |
| WASM Instant in bridge for “parity timing” | Post-gate smoke + optional note vs WASM/native only | Milestone policy | No cdylib profiler |

**Deprecated/outdated:**

- Calling the first leftover admission pair “the current 3× number” in the Phase 23 section of the audit.
- Treating playground Dam Break numbers as Phase 12 reviewed reports.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Fresh `just web-player-smoke` will exit 0 on current HEAD without physics/player edits | PERF-WASM / Environment | If red, Phase 25 must diagnose shared stepping / existing player (D-02)—not rewrite scenes. Research did not re-run smoke. `[ASSUMED]` |

**If this table is empty:** N/A — one assumed claim (A1) needs live confirmation during execution.

All other factual claims in this research were verified from on-disk stamps, committed docs, or source.

## Open Questions

1. **Does the optional WASM note need any browser measurement at all?**
   - What we know: D-04 explicitly allows citing native gate ms/step when no prior WASM sample exists; no prior sample found in committed docs.
   - What's unclear: Nothing material—locked.
   - Recommendation: One-line native-fallback note; do not build a browser timer.

2. **Should smoke run before or after doc edits?**
   - What we know: Docs do not depend on smoke logs; smoke does not depend on doc text.
   - What's unclear: Preference only.
   - Recommendation: Plan 01 docs (PERF-NOTES), Plan 02 smoke + optional WASM note citation (PERF-WASM), so a smoke failure does not block honesty edits—or reverse if the planner wants smoke evidence mentioned inside the timing doc. Either order is fine; prefer docs-first so honesty close is not blocked by flaky browser infra.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| bun | `web-player-smoke` | ✓ | 1.4.2 | — |
| just | recipe alias | ✓ | 1.48.0 | Invoke `bun scripts/web-build.ts player-smoke` directly |
| cargo / rustc 1.97 | WASM regen | ✓ | 1.97.0 | — |
| wasm-pack | WASM package | ✓ | 0.15.0 | — |
| wasm32-unknown-unknown | WASM target | ✓ | installed | `rustup target add …` |
| Playwright Chromium | `test:player` | ✓ (install step in recipe) | 1.63.0 package present | `bun run browser:install` inside player-smoke |
| Gate stamp `20-38-50Z` | PERF-NOTES numbers | ✓ | on disk | Only then mint new pair (D-07) |
| Kernel sibling `20-36-30Z` | Sibling labeling | ✓ | on disk | — |

**Missing dependencies with no fallback:** None identified for this host.

**Missing dependencies with fallback:** None.

**Step 2.6 note:** Smoke was **not** executed during research (A1). Execution must run it once.

## Security Domain

> Docs + existing local browser smoke only. No new auth, network APIs, or crypto.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes (existing) | Scene id / pointer kind already validated in `ProofSession` / `SessionCore`; do not weaken for smoke |
| V6 Cryptography | no | — |

### Known Threat Patterns for this phase

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Misleading public performance claim | Elevation / Tampering (reputation) | Keep README claim-free; empty `reviewed_reports`; unreviewed labels (D-08/D-09) |
| XSS via instruction copy changes | Tampering | Do not rewrite scene/copy lanes (D-02); existing figcaption assertions remain |
| Accidental commit of profiling dumps | Information disclosure | Do not commit `target/web-build` or `target/dam-break-perf` |

## Smallest Plan Split (prescriptive)

| Plan | Requirement | Tasks (high level) | Touches code? |
|------|-------------|--------------------|---------------|
| **25-01** | PERF-NOTES (+ D-06/D-07/D-08/D-09) | Reconcile audit + timing to gate stamp; remaining-delta + not-attempted; BENCHMARKING honesty sentence; verify README/Cargo.toml/manifest empty | Docs only |
| **25-02** | PERF-WASM (+ D-01–D-05) | Run `just web-player-smoke`; cite exit 0 + 36 tests in SUMMARY; add optional one-line WASM note (native ms/step + “no prior WASM sample”); assert `MAX_ADVANCE_STEPS` still 4 | Docs one-liner + smoke run; no Rust/player feature edits |

Do not add a third plan for Pages, Firefox, Instant, or a new pair.

## Sources

### Primary (HIGH confidence)

- `.planning/phases/25-wasm-sanity-and-honest-close/25-CONTEXT.md` — locked decisions
- `.planning/REQUIREMENTS.md` — PERF-NOTES, PERF-WASM
- `.planning/phases/24-shared-hot-path-waves-through-3/24-VERIFICATION.md` — gate vs sibling stamps
- `target/dam-break-perf/2026-09-21T20-38-50Z/pair.json` — live gate fields
- `target/dam-break-perf/2026-09-21T20-36-30Z/pair.json` — sibling fields
- `justfile`, `scripts/web-build.ts`, `web/package.json`, `web/e2e/*.spec.ts` — smoke chain
- `docs/native-performance-audit.md`, `docs/playground-dam-break-timing.md`, `BENCHMARKING.md` — current wording gaps
- `crates/liquidfun-wasm/src/{lib,session}.rs` — Instant cfg + `MAX_ADVANCE_STEPS`
- `.planning/phases/19-interaction-polish-and-browser-verification/19-VERIFICATION.md` — prior smoke coverage

### Secondary (MEDIUM confidence)

- `.planning/research/PITFALLS.md` — Instant-in-WASM / catch-up cap warnings `[CITED]`

### Tertiary (LOW confidence)

- A1: smoke still green without edits `[ASSUMED]`

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — existing recipes and pins verified on disk
- Architecture: HIGH — edit map and anti-patterns verified against current docs/source
- Pitfalls: HIGH — match locked decisions and Phase 24 verification notes

**Research date:** 2026-09-21
**Valid until:** 2026-10-21 (docs/smoke phase; re-check only if gate stamps deleted or smoke suite rewritten)
