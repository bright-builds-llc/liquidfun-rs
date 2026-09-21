---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 25-2026-09-21T22-11-01
generated_at: 2026-09-21T22:11:59.430Z
---

# Phase 25: WASM sanity and honest close - Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

After the native 3× gate, a visitor can still run the six playground scenes, and a developer can read committed remaining-delta notes without a public “Rust is N×” claim.

This phase closes PERF-NOTES and PERF-WASM. It records `just web-player-smoke` plus an optional Dam Break step-time note versus previous WASM or native Rust. It does not compare browser Dam Break to `oracle-release`, put an Instant profiler in the cdylib, lift the 4-step catch-up cap, fill `reviewed_reports`, or publish a crate.

</domain>

<decisions>
## Implementation Decisions

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

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase scope and locked milestone policy
- `.planning/ROADMAP.md` — Phase 25 goal and success criteria; no Instant profiler in the cdylib, no lifted 4-step catch-up, no browser-versus-C++ comparison.
- `.planning/REQUIREMENTS.md` — `PERF-NOTES` and `PERF-WASM` (this phase); `PERF-WASM-ENG` is out of scope.
- `.planning/PROJECT.md` — Native is the C++ comparison; WASM is a post-gate sanity check; no public “Rust is N×” claim. Records gate stamp `2026-09-21T20-38-50Z` and ratio `2.956857456935513`.
- `.planning/phases/24-shared-hot-path-waves-through-3/24-CONTEXT.md` — Deferred PERF-NOTES / PERF-WASM here; stop gold-plating after the gate.
- `.planning/phases/24-shared-hot-path-waves-through-3/24-VERIFICATION.md` — Authoritative gate pair versus kernel-HEAD sibling sample.
- `PROJECT-SCOPE.md` — Hobby scope; honest limitations; no package publication from this phase.

### Honesty and measurement
- `BENCHMARKING.md` — Exploratory Dam Break pair is an unreviewed local sample, not a Phase 12 reviewed report.
- `docs/native-performance-audit.md` — Named leftover causes and **Not found** list; currently cites the kernel-HEAD pair and must be reconciled to the gate stamp.
- `docs/playground-dam-break-timing.md` — Locked Medium recipe and unreviewed pair table; same reconciliation.
- `reference/performance/manifest.toml` — `reviewed_reports` must stay empty.
- `reference/performance/policy.json` — `timing_authority: unprofiled_wall_clock`.

### Existing playground seams
- `justfile` — `web-player-smoke` is a one-line alias to `bun scripts/web-build.ts player-smoke`.
- `crates/liquidfun-wasm/src/session.rs` — `MAX_ADVANCE_STEPS` remains 4.
- `.planning/phases/19-interaction-polish-and-browser-verification/19-VERIFICATION.md` — What `just web-player-smoke` already covers for the six scenes.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `just web-player-smoke` already builds the Rust WASM player and runs the Chromium six-scene suite.
- Phase 24 verification already records the unprofiled gate pair and the kernel-HEAD sibling.
- `docs/native-performance-audit.md` already lists landed leftover kernels and the **Not found** closer list.
- `BENCHMARKING.md` already forbids treating the playground pair as a Phase 12 reviewed report.

### Established Patterns
- `just` stays a one-line printer; orchestration stays in existing scripts.
- Public performance numbers are copied from `pair.json` or verification records and labeled unreviewed.
- `liquidfun` stays bitflags-only. The cdylib does not gain a profiler.
- `reviewed_reports` stays an empty list.

### Integration Points
- Note edits land in `docs/native-performance-audit.md`, `docs/playground-dam-break-timing.md`, and `BENCHMARKING.md`.
- Scene proof reuses `just web-player-smoke`. Do not add player features to satisfy this phase.
- README and crate metadata stay free of a universal speed claim.

</code_context>

<specifics>
## Specific Ideas

- Gate close number to quote: `target/dam-break-perf/2026-09-21T20-38-50Z`, `rust_over_cpp_ratio` `2.956857456935513`. Kernel-HEAD sibling only: `2026-09-21T20-36-30Z`, `2.8769953439599707`.
- The audit’s current “PERF-GATE leftover close” section presents the kernel-HEAD pair as the 3× number. Phase 25 corrects that labeling without running a new pair.
- Six scenes stay the v1.1 set: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel.

</specifics>

<deferred>
## Deferred Ideas

- `PERF-WASM-ENG`: engineer WASM versus native stepping beyond this post-gate sanity check.
- SIMD, default Rayon, PGO, or lifting workspace `unsafe_code = "forbid"`.
- Filling `reference/performance/manifest.toml` or writing a public speed claim.
- Package publication or a release tag.
- Browser Dam Break versus `oracle-release`.
- An Instant profiler in the cdylib, or lifting `MAX_ADVANCE_STEPS` above 4.

</deferred>

---

*Phase: 25-wasm-sanity-and-honest-close*
*Context gathered: 2026-09-21*
