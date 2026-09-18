---
gsd_state_version: 1.0
milestone: v1.1
milestone_name: Web Playground
status: executing
stopped_at: Phase 18 context gathered
last_updated: "2026-09-18T03:39:20.747Z"
last_activity: 2026-09-17
progress:
  total_phases: 4
  completed_phases: 2
  total_plans: 12
  completed_plans: 12
  percent: 100
---

# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-09-17)

**Core value:** Deliver a useful, independent Rust physics library for enjoyable experimentation, with honest limitations and a lightweight native development loop.
**Current focus:** Phase 17 — Shared Player and Early Pages Delivery

## Current Position

Phase: 18
Plan: Not started
Status: Ready to execute
Last activity: 2026-09-17

Progress: [██████████] 100% of currently planned v1.1 plans complete.

The prior 16 phases and 252 plans remain archived history. New work starts at Phase 16. The owner confirmed six interactive demos using our Rust engine via WASM, playful catalog/player controls and automatic GitHub Pages deployment from main.

## Performance Metrics

| Plan | Duration | Tasks | Files |
| --- | --- | --- | --- |

## Accumulated Context

| Phase 16 P01 | 13 min | 3 tasks | 7 files |
| Phase 16 P02 | 7 min | 2 tasks | 14 files |
| Phase 16 P03 | 6 min | 2 tasks | 7 files |
| Phase 16 P04 | 50min | 3 tasks | 9 files |
| Phase 17-shared-player-and-early-pages-delivery P01 | 2 min | 2 tasks | 4 files |
| Phase 17 P02 | 2 min | 2 tasks | 4 files |
| Phase 17 P03 | 2 min | 2 tasks | 4 files |
| Phase 17 P04 | 3min | 2 tasks | 7 files |
| Phase 17 P05 | 7min | 2 tasks | 9 files |
| Phase 17-shared-player-and-early-pages-delivery P06 | 6min | 2 tasks | 7 files |
| Phase 17 P07 | 2min | 2 tasks | 1 files |

### Decisions

- Prove a real Rust WASM browser step first, then deploy a shared SolidJS player early before expanding the six scenes.
- Approved scenes: Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel; all physics comes from this Rust engine.
- Prefer a private typed package, owned bulk frames, one active session and bounded simulation time/resources. Native Cargo consumers remain isolated.
- Every main push triggers a same-checkout site/WASM build and Pages delivery with latest-main protection. This is web delivery, not Linux native qualification.
- [Phase 16]: Keep liquidfun as the sole default workspace member and depend outward from the private wrapper.
- [Phase 16]: Copy five validated bounded numeric lanes into JavaScript-owned typed arrays instead of exposing WASM memory.
- [Phase 16]: Reject invalid advance counts and step-index overflow before invoking the engine.
- [Phase 16]: Record compilation and ES-module import only; real Chromium execution remains Plan 16-04.
- [Phase 16]: Regenerate ignored wasm-pack output from the current checkout before frontend verification or build.
- [Phase 16]: Defer the complete Vite build until Plan 16-03 supplies renderer and application entrypoints.
- [Phase 16]: Poison the TypeScript session owner after any advance, capture, parse, or frame-cleanup failure.
- [Phase 16]: Keep world fitting and y-axis inversion in one pure camera module while Canvas effects consume validated bulk arrays.
- [Phase 16]: Represent loading, running, failure, and disposed as a tagged union carrying only valid frame observations.
- [Phase 16]: Advance exactly one Rust frame per animation callback and derive browser proof attributes from consecutive Rust frame lanes.
- [Phase 16]: Allocate every smoke attempt before execution so failures remain immutable forensic records.
- [Phase 16]: Use Canvas pixel SHA-256 changes and attached PNG bytes alongside Rust movement counters.
- [Phase 16]: Redraw only the last Rust frame on resize and disconnect resize effects before terminal cleanup.
- [Phase 16]: Bind implementation, declarations, logs, metadata, and PNG bytes into one independent review digest.
- [Phase 17]: Keep #/scene and #/scene/ as empty so later UI can show the empty-hash fallback instead of unknown copy. — D-06 requires empty and unknown hashes to stay distinct useful states.
- [Phase 17]: Do not lowercase hash tokens; Dam-Break stays unknown with maybeRaw preserved. — Canonical ids are lowercase hyphenated tokens. Coercing case would hide invalid shared URLs.
- [Phase 17]: Known not-ready ids parse as scene; only dam-break is ready in catalog data. — Readiness is catalog metadata, not parser output, so later player code cannot construct a WASM world from a parsed scene kind alone.
- [Phase 17]: Keep acceptedStepCount pure: no performance.now, document.hidden, leftover accumulator, or rAF. — D-12 caps accepted wall-clock delta at four 1/60-second steps. Hidden-tab pause and leftover catch-up belong to the later player shell.
- [Phase 17]: Guard 1..=4 in TypeScript before generated advance; poison invalid counts without calling advance. — Rust already rejects 0 and 5+. TypeScript must not cross the WASM boundary with an illegal count or leak generated exception text.
- [Phase 17]: Forward one advance(n) plus one capture instead of calling nextFrame four times. — Four nextFrame calls would capture four times. WASM-04 needs one animation callback to step 1-4 ticks and still capture once.
- [Phase 17]: Production and preview Vite base is /liquidfun-rs/; local serve stays /. — Local vite serve must keep root hrefs; only production and preview-of-production need the GitHub Pages project prefix.
- [Phase 17]: Short commit labels are the first 12 lowercase hex characters of a full SHA. — Operability and UI-SPEC allow 7-12 characters; 12 stays unique while remaining readable in footer chrome.
- [Phase 17]: Commit and build URLs are accepted only on https://github.com/bright-builds-llc/liquidfun-rs/. — Footer links must not be built from hash-route input or arbitrary env strings; javascript: and off-host URLs stay undefined.
- [Phase 17]: just web-build fails unless dist/index.html contains /liquidfun-rs/assets/ and a .wasm file exists. — A root-base Vite build would 404 WASM on GitHub Pages; fail closed before any deploy job.
- [Phase 17]: Record D-09 as a Phase 17 semantic-HTML plus one scoped CSS exception; revisit MysticUI/Tailwind on 2026-12-17. — Pages delivery must not be blocked by adopting MysticUI or Tailwind in this thin slice.
- [Phase 17]: Keep the Scenes catalog label as a styled paragraph so the player or fallback heading remains the only h2. — 17-UI-SPEC allows only one h2 on a given view; the catalog label is not that heading.
- [Phase 17]: Leave App.tsx on the Phase 16 proof shell; Plan 17-05 mounts these presentational panels. — This plan owns chrome only and must not wire the WASM session.
- [Phase 17]: Reuse loadProofSession/ProofSession as named Dam Break instead of renaming the generated class. — wasm-pack output stays ignored; the existing basin constructor already matches the documented Dam Break reset constants.
- [Phase 17]: Extract observeFrame helpers so App.tsx stays under the 628-line file trigger. — The plan allowed a split; frame observation is not session ownership and can stay a pure helper.
- [Phase 17]: Allow explicit undefined on optional chrome props for exactOptionalPropertyTypes. — Solid call sites pass absent current-scene and failure-details values; widening the optional type unblocked typecheck without extra JSX branches.
- [Phase 17]: Player smoke is a web-build player-smoke mode that reuses the existing build and does not allocate a Phase 16 closure attempt. — D-17 local proofs must not run the Phase 16 forensic closure matrix. just web-smoke stays the opt-in forensic path that still sets PHASE16_CLOSURE_ATTEMPT_DIR.
- [Phase 17]: Hidden-tab proof uses Object.defineProperty plus a MutationObserver so the first resume frame is measured, not a later poll. — document.hidden is read-only. Playwright poll can sample a later animation callback and see a step delta of 5, which is not a catch-up storm. The first data-step-index mutation is the next observed frame.
- [Phase 17]: Every main and pull_request run builds WASM plus web/dist from the same checkout with no path filters. — HOST-01 requires a same-checkout site+WASM build on every main push so a Rust-only change still ships updated physics.
- [Phase 17]: Deploy uses job-scoped pages: write plus id-token: write and the github-pages environment; no PAT. — HOST-02 forbids a personal token; write permissions stay on deploy-pages so pull requests keep contents: read only.
- [Phase 17]: cancel-in-progress is false on main so an in-flight deploy finishes while queued revisions may coalesce. — Cancelling a mid-flight Pages deploy can strand a newer revision; official starter keeps the in-progress run and coalesces queued main SHAs.

### Pending Todos

No new milestone todos captured. Phase 17 is ready for discussion and planning.

### Blockers/Concerns

- Chromium closure attempt 10 is current for source `80d4d7b`: all 16 source-bound checks pass, strengthened validators and bounded failure records are exercised, and separate AI review approves digest `7b63ca2e7f1580a8a509e8265aca968667bc8443b9055651e285991a0cd3ff38`. Attempt 8 remains historical.
- Pages setup/access and final URL remain unverified; Phase 17 verifies deployment and project-subpath assets early.
- Floating, elastic and wheel scene stability require modest visual experiments in Phase 18; no fake physics or silently replaced approved scenes.

## Retained Context

- Local checks plus one macOS CI job; expensive qualification stays optional/manual.
- No package publication or release tag. The v1.0 label identifies planning history only.
- PLAT-01, PLAT-05 and DOCS-09 remain deferred in archived requirements. Strict certification remains not release-ready.
- Before eventual publication, verify the declared Rust 1.92 minimum and obtain separate publication authority. Current GUI behavior was not verified by the hobby wrap-up.
- Historical decisions and completion metrics: `.planning/milestones/v1.0-STATE.md` and `.planning/MILESTONES.md`.
- Three historical debug records remain preserved; Phase 15 completion classifies their later evidence without claiming new fixes.

## Session Continuity

Last session: 2026-09-18T03:39:20.743Z
Stopped at: Phase 18 context gathered
Resume file: .planning/phases/18-six-native-physics-demos/18-CONTEXT.md
