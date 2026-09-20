---
generated_by: gsd-discuss-phase
lifecycle_mode: yolo
phase_lifecycle_id: 21-2026-09-20T19-45-21
generated_at: 2026-09-20T19:47:12.688Z
---

# Phase 21: Playground leftover cleanup - Context

**Gathered:** 2026-09-20
**Status:** Ready for planning
**Mode:** Yolo

<domain>
## Phase Boundary

Remove dead playground chrome and proof leftovers, and keep the named WASM scene modules under the Bright Builds file-length gate, without changing public scene behavior. This phase closes the v1.1 milestone-audit leftover items: unused `loadProofSession`, the Phase 16 opt-in forensic spec versus ordinary product smoke, dead `FallbackPanel` empty/not-ready branches, and scene file-length debt on `color_mixer.rs`, `dam_break.rs`, and `water_wheel.rs`.

This is leftover cleanup with no milestone requirement reassignment. Do not add scenes, controls, gestures, catalog layout, Reset-honesty work, a design-system migration, a new Pages URL, crate/npm publication, Linux qualification, or Dam Break headless speed versus pinned C++.

</domain>

<decisions>
## Implementation Decisions

### Unused proof helper and forensic spec

- **D-01:** Delete unused `loadProofSession` from `web/src/physics/loader.ts`. The live player already constructs worlds through `loadSceneSession(sceneId)`. Do not re-wire the Dam Break wrapper into `App.tsx`, tests, or the forensic spec.
- **D-02:** Keep `web/e2e/rust-wasm-proof.spec.ts` as an explicit opt-in behind `PHASE16_CLOSURE_ATTEMPT_DIR` / `just web-smoke`. Do not fold Phase 16 Dispose-session, PNG-hash, or closure-attempt forensics into ordinary `just web-player-smoke`.
- **D-03:** Do not rewrite the forensic spec onto current Play/Pause/Reset chrome in this phase. Product proof remains the existing Chromium player suite. Document that `just web-smoke` is historical Phase 16 chrome, not the v1.1 product gate.

### Dead FallbackPanel branches

- **D-04:** Remove `FallbackPanel` `empty` and `not-ready` kinds, copy, and `App.tsx` `fallbackProps` branches that cannot be reached from a real visitor path. All six catalog ids are `ready: true`, and empty hashes already normalize to `#/scene/dam-break`.
- **D-05:** Keep the unknown-hash fallback as the real useful path: `kind: "unknown"` still shows Scene not found, the locked explanation, and `Open Dam Break`. Do not leave a blank canvas for `#/scene/not-a-scene`.
- **D-06:** Do not undo `normalizeSceneRoute` empty-hash replacement to Dam Break. Parser `kind: "empty"` may remain as an internal parse result; it must not render empty FallbackPanel chrome.

### Scene file-length cleanup

- **D-07:** Named scene modules `color_mixer.rs`, `dam_break.rs`, and `water_wheel.rs` must satisfy Bright Builds `file-lengths` (628 physical lines). Live scout already shows them under the gate after `tests.rs` extraction (346 / 360 / 426). Confirm that, and split further only if a named file is over the gate again.
- **D-08:** Any split uses the existing `foo.rs` plus `foo/` layout. Extract tests or helpers; do not change public scene behavior, controls, particle recipes, gravity, joints, pointer magnitudes, or catalog/WASM ids.
- **D-09:** `tools/xtask/tests/upstream_cli.rs` (642 lines) is the current remaining `file-lengths` finding and is not one of the three named scene files. Bring it under the gate in this phase only if phase verification requires `bun scripts/bright-builds-check.ts file-lengths` or `all` to exit 0. Prefer a `foo.rs` plus `foo/` test split over a TSV exception. Do not change xtask CLI behavior.

### Verification boundary

- **D-10:** Prove leftover cleanup with existing local checks: loader/fallback unit tests, `cargo test -p liquidfun-wasm` when scene modules move, Bright Builds `file-lengths` on the named files (and `all` / `file-lengths` exit 0 if that is the phase gate), and `just web-player-smoke` after player/fallback edits. Do not add Firefox/Safari/WebKit, screenshot-hash oracles, Playwright on Pages CI, Linux qualification, or Dam Break C++ timing.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work. No crate/npm publish, no release tag, and no new GitHub Pages URL.

### Claude's Discretion

- Exact helper filenames and test-module splits inside the locked `foo.rs` plus `foo/` shape, provided scene behavior and file-length gates hold.
- Whether unknown fallback props stay a dedicated union member or a single-kind component after empty/not-ready removal.
- Exact documentation wording that marks `just web-smoke` as historical opt-in forensic chrome.
- Exact Playwright assertions for unknown-hash fallback, provided D-05 remains true and empty/not-ready chrome is gone.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Active milestone scope

- `.planning/PROJECT.md` — v1.1 playground goal; Phase 21 leftover cleanup remains; no requirement IDs.
- `.planning/REQUIREMENTS.md` — 22/22 requirements already complete; this phase has no requirement reassignment.
- `.planning/ROADMAP.md` § Phase 21 — goal, success criteria, and out-of-scope Dam Break C++ timing.
- `.planning/v1.1-MILESTONE-AUDIT.md` — unused-proof, dead-FallbackPanel, and scene file-length leftovers this phase closes.
- `PROJECT-SCOPE.md` — hobby completion; this phase does not revive Linux qualification or package publication.

### Prior locked playground contracts

- `.planning/phases/16-rust-wasm-browser-bridge/16-CONTEXT.md` — original forensic proof; product player replaced Dispose-session chrome.
- `.planning/phases/17-shared-player-and-early-pages-delivery/17-CONTEXT.md` — D-06 useful unknown fallback; D-11 one-session ownership; D-17 product smoke vs opt-in `just web-smoke`.
- `.planning/phases/18-six-native-physics-demos/18-CONTEXT.md` — six ready native scenes; do not change recipes or control sets here.
- `.planning/phases/19-interaction-polish-and-browser-verification/19-CONTEXT.md` — focused Chromium product smoke; `deferred-items.md` recorded the original scene file-length debt.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` — empty-hash Dam Break normalization and Kobalte shell stay; leftover cleanup was deferred here.
- `.planning/phases/19-interaction-polish-and-browser-verification/deferred-items.md` — original `color_mixer.rs` / `dam_break.rs` / `water_wheel.rs` file-length note.
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/deferred-items.md` — remaining `tools/xtask/tests/upstream_cli.rs` file-length finding.

### Live code and tests

- `web/src/physics/loader.ts` — unused `loadProofSession` wrapper around `loadSceneSession("dam-break")`.
- `web/e2e/rust-wasm-proof.spec.ts` — opt-in Phase 16 forensic spec; skips unless `PHASE16_CLOSURE_ATTEMPT_DIR` is set.
- `web/src/components/FallbackPanel.tsx` — empty / unknown / not-ready copy; only unknown is a real visitor path.
- `web/src/App.tsx` — `fallbackProps` still constructs empty and not-ready; live route uses `normalizeSceneRoute` then `loadSceneSession`.
- `web/src/routing/hash.ts` — empty hashes replace to `#/scene/dam-break`; unknown hashes stay unknown.
- `crates/liquidfun-wasm/src/scene/{color_mixer,dam_break,water_wheel}.rs` — named scene modules already using `foo.rs` plus `foo/tests.rs`.
- `tools/xtask/tests/upstream_cli.rs` — current Bright Builds `file-lengths` finding (642 lines).
- `web/e2e/player.spec.ts` and `web/e2e/shell.spec.ts` — existing unknown-hash and empty-hash normalization coverage.
- `TESTING.md` and `README.md` — document `just web-player-smoke` vs opt-in `just web-smoke`.

### Repository standards

- `AGENTS.md`, `AGENTS.bright-builds.md`, and `standards-overrides.md` — hobby scope, standing iteration authorization, independent AI review; Kobalte/skipLibCheck stay; no design-system migration.
- `standards/core/architecture.md` — functional-core/imperative-shell around routing and loader decisions.
- `standards/core/code-shape.md` — 628-line file trigger; `foo.rs` plus `foo/` splits; `maybe_` naming.
- `standards/core/frontend-ui.md` — keep dark default and existing source/provenance chrome; no new visual system.
- `standards/core/testing.md` and `standards/core/verification.md` — focused tests and repo-native verification before commit.
- `standards/languages/rust.md` — `foo.rs` plus `foo/` for touched multi-file modules.
- `standards/languages/typescript-javascript.md` — SolidJS/Bun defaults; Kobalte exception remains in `standards-overrides.md`.
- `LICENSE` — MIT; truthful free-and-open-source copy is allowed.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `loadSceneSession(sceneId)` is the live constructor; deleting `loadProofSession` does not change WASM init.
- `FallbackPanel` already has locked unknown copy and `Open Dam Break`. Keep that path; drop the other two kinds.
- Named scene modules already extracted `tests.rs` under `scene/{name}/` and are under 628 lines.
- `just web-player-smoke` is the product Chromium suite; `just web-smoke` still allocates a Phase 16 closure attempt.

### Established Patterns

- Empty hashes parse as `kind: "empty"` then `normalizeSceneRoute` replaces them with Dam Break. Do not revive empty FallbackPanel by undoing that.
- All six `SCENES` are `ready: true`, so not-ready FallbackPanel is defensive dead copy.
- Scene modules follow `foo.rs` plus `foo/` for tests. Further splits should match that, not `mod.rs`.
- Forensic Playwright skips unless `PHASE16_CLOSURE_ATTEMPT_DIR` is set, so ordinary player smoke never runs it.

### Integration Points

- Remove `loadProofSession` from `loader.ts` and any docs that still present it as the Dam Break constructor.
- Narrow `FallbackPanelProps` and `App.tsx` `fallbackProps` to unknown (plus any remaining internal empty parse that must not render).
- Confirm Bright Builds `file-lengths` on the three named scene files; split `upstream_cli.rs` only if the phase gate requires `file-lengths` to exit 0.
- Keep `web/e2e/player.spec.ts` unknown-hash coverage; do not add a browser matrix.

</code_context>

<specifics>
## Specific Ideas

- ROADMAP success criteria are explicit: remove or use `loadProofSession`; fold or keep the forensic spec with no unused helper; remove or reach dead FallbackPanel branches; named scene files under `file-lengths` without behavior change.
- Phase 17 already documented that `just web-smoke` expects retired Dispose-session chrome if invoked. This phase does not revive that chrome.
- Live file-length scout on 2026-09-20: named scene files pass; `tools/xtask/tests/upstream_cli.rs` is the only current finding.

</specifics>

<deferred>
## Deferred Ideas

- Playground Dam Break headless speed versus pinned C++ — out of v1.1 definition of done.
- Rewriting Phase 16 forensic smoke onto current player chrome — new evidence tooling, not leftover deletion.
- MysticUI/Tailwind/design-system migration — revisit 2026-12-17 per `standards-overrides.md`.
- Crate/npm publication, release tags, and a new GitHub Pages URL — separately authorized.
- Linux qualification, sanitizers, coverage, and a broader browser matrix — optional hobby-scope checks.

</deferred>

---

*Phase: 21-playground-leftover-cleanup*
*Context gathered: 2026-09-20*
