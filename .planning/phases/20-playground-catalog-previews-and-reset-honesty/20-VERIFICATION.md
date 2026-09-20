---
phase: 20-playground-catalog-previews-and-reset-honesty
verified: 2026-09-20T19:33:32Z
status: passed
score: 5/5 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 20-2026-09-20T17-36-31
generated_at: 2026-09-20T19:33:32Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 20: Playground catalog previews and Reset honesty Verification Report

**Phase Goal:** Visitors can browse six in-app demo entries with names, descriptions and visual previews, then Reset a playing scene to its documented initial physics and matching live-control labels.
**Verified:** 2026-09-20T19:33:32Z
**Status:** passed
**Re-verification:** No — initial verification

Provenance: `20-CONTEXT.md`, plans `20-01` through `20-06`, and matching SUMMARYs share `lifecycle_mode: yolo` and `phase_lifecycle_id: 20-2026-09-20T17-36-31`. No `direct-fallback` markers.

Must-haves are the four ROADMAP success criteria plus the Plan 06 independent-review truth. Plan 01–05 frontmatter truths restated those criteria and are folded into the evidence below; they did not reduce roadmap scope.

## Goal Achievement

The phase delivers the goal, not only completed tasks. The Kobalte `PlaygroundShell` still lists six hash-linked demos. Each `DemoNavigation` item now contains a token-only SVG, a visible `Static preview` caption, the catalog title, and the locked one-sentence description. Reset and Retry assign an empty construction bag, bump `resetGeneration`, remount `SceneControls` through Solid `Show keyed`, and rebuild WASM `build([])` defaults so preset selects show `DEFAULT_PRESET_VALUES` instead of a stale `pendingValue`. Play and pause freeze or resume without remounting those selects. Focused Chromium `test:player` covers sidebar/drawer previews and representative Reset label honesty. Independent AI review is bound to digest `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`.

### Observable Truths

| #   | Truth   | Status     | Evidence       |
| --- | ------- | ---------- | -------------- |
| 1   | The in-app catalog presents Dam Break, Fountain, Float or Sink, Color Mixer, Jelly Drop and Water Wheel with names, short descriptions and visual previews; each still opens the shared player through the existing hash route. | ✓ VERIFIED | `SCENES` has those six `ready: true` records with titles and locked descriptions. `sceneNavigationItems` maps them to `href: #/scene/{id}`. `DemoNavigation` renders one `<a class="demo-nav-link">` per item with `ScenePreview`, caption `Static preview`, title, then description — no nested Open button. `ScenePreview` is an exhaustive `SceneId` switch of classless inline SVG (`viewBox="0 0 160 90"`, `aria-hidden="true"`). No WASM, `innerHTML`, or `docs/assets/demos` WebP in `previews.tsx` / `DemoNavigation.tsx`. |
| 2   | The owner-approved responsive Kobalte / semantic-HTML shell remains; restoring previews must not revive the old card layout that broke narrow widths. | ✓ VERIFIED | `PlaygroundShell` still uses `@kobalte/core/dialog` `0.13.12` for the mobile drawer and mounts `DemoNavigation` in `.demo-sidebar` plus the `Demos` dialog. `web/src` has zero `.catalog-card`, `.catalog-grid`, or `CatalogNav`. `app.css` adds compact `.demo-nav-preview*` (`max-block-size: 72px`, `aspect-ratio: 16 / 9`) without a three-column card grid. `shell.spec.ts` asserts `.catalog-card` count `0` on desktop, sidebar-preview, and mobile-drawer paths. |
| 3   | Play, pause and Reset still rebuild or freeze the native world as documented, and live preset selects show that initial value after Reset instead of a stale pendingValue. | ✓ VERIFIED | `recreateScene` (Reset and Retry) assigns `constructionValues = {}`, then `setResetGeneration`, then `startScene`. `startScene` applies `constructionEntriesForScene` which emits `[]` for `{}`, so WASM rebuilds documented initials. `PlayerSceneChrome` remounts `SceneControls` with `<Show keyed>` on `sceneControlsIdentity(scene.id, resetGeneration)` (`"dam-break:0"` is non-empty). `PresetControl` snapshots `pendingValue` once from `initialPresetValue` / `DEFAULT_PRESET_VALUES`. `playScene` / `pauseScene` only change `view()`; they do not clear the bag or bump the remount key. Chromium proves Fountain emission-rate high→medium, Float or Sink body cork→wood, Color Mixer stir-speed fast→slow, Water Wheel jet-strength strong→medium, Dam Break water-amount large+Apply→medium, Color Mixer mix-strength gentle+Apply→strong, and play/pause keeping emission-rate high. |
| 4   | Focused Chromium smoke covers visible catalog previews and Reset label honesty for representative live presets. | ✓ VERIFIED | `web/package.json` `test:player` is `playwright test e2e/player.spec.ts e2e/demo-media-clock.spec.ts e2e/shell.spec.ts e2e/reset-honesty.spec.ts` (36 top-level tests plus one `demo-media-clock` describe case = 37). `shell.spec.ts` scopes six `Static preview` captions and six `svg[aria-hidden='true']` to `.demo-sidebar` at 1280×800, and checks one visible caption plus SVG in the `Demos` dialog at 390×812. `justfile` `web-player-smoke` runs `bun scripts/web-build.ts player-smoke`. Ignored `target/web-build/web-build.log` starts `start player-smoke` and ends `complete player-smoke` with `VITE_GIT_SHA=d33d3e2…`. `web/test-results/.last-run.json` is `"status": "passed"` and `"failedTests": []`. `.github/workflows/pages.yml` has no Playwright job. |
| 5   | A separate identified AI reviewer acknowledges the Phase 20 diff and smoke evidence at exact SHA-256 digest `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`; the implementing executor does not approve its own work. | ✓ VERIFIED | `20-REVIEW.md` binds that digest to reviewer `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a`, `reviewed_at: 2026-09-20T19:25:04Z`, `reviewer_disclosure: AI reviewer, not a human`, `implementing_or_fixing_executor: no`, decision **APPROVED**. This verifier independently concatenated the 13 listed files in 20-06 order and recomputed SHA-256; the digest and all 13 per-file hashes match. Hashed paths are unchanged after implementation HEAD `d33d3e2`. Passing smoke is not that acknowledgment. |

**Score:** 5/5 truths verified

### Required Artifacts

gsd-tools `verify artifacts` passed 13/13 planned paths across plans 20-01 through 20-06.

| Artifact | Expected    | Status | Details |
| -------- | ----------- | ------ | ------- |
| `web/src/player/runtime.ts` | `sceneControlsIdentity` plus Reset-honest construction-bag comment | ✓ VERIFIED | 120 lines. Interpolates `${sceneId}:${generation}`. Comment is `Last applied construction presets until Reset or scene switch clears the bag.` Empty bag still returns `[]`. |
| `web/tests/runtime.test.ts` | Empty-bag and identity unit tests | ✓ VERIFIED | Dam Break / Color Mixer empty bags; `"dam-break:0"` non-empty; Fountain/Color Mixer joins. |
| `web/tests/controls.test.ts` | Documented `initialPresetValue` cases | ✓ VERIFIED | Empty-bag defaults: emission-rate medium, body wood, stir-speed slow, jet-strength medium, water-amount medium, mix-strength strong. |
| `web/src/catalog/previews.tsx` | `ScenePreview` switch on `SceneId` | ✓ VERIFIED | 120 lines. Token constants `WATER`/`MIX_RED`/`RIGID`/`JELLY`/`CANVAS`. Six still geometries. No `catalog-preview` class. |
| `web/src/components/DemoNavigation.tsx` | Preview plus `Static preview` inside the hash link | ✓ VERIFIED | 40 lines. Child order preview → caption → title → description. Imports `ScenePreview`. |
| `web/src/app.css` | `.demo-nav-preview` and `.demo-nav-preview-caption` | ✓ VERIFIED | 628 lines (cap). Compact 72px 16:9 frame, 14px caption. |
| `web/src/App.tsx` | `resetGeneration` and `recreateScene` bag clear | ✓ VERIFIED | 617 lines (≤628). Starts generation at 1. Reset/Retry share `recreateScene`. |
| `web/src/components/PlayerSceneChrome.tsx` | Keyed `SceneControls` plus `SceneCredits` | ✓ VERIFIED | 41 lines. Credits stay outside `Show keyed`. No React `key=`. |
| `web/src/player/viewport.ts` | `prefersReducedMotion` and `isUsableViewport` | ✓ VERIFIED | Extracted helpers; App imports both. |
| `web/e2e/shell.spec.ts` | Scoped sidebar and drawer preview assertions | ✓ VERIFIED | 228 lines. Desktop six + mobile dialog. No global `page.getByText("Static preview")`. |
| `web/e2e/reset-honesty.spec.ts` | Live and construction Reset proofs plus play/pause negative | ✓ VERIFIED | 140 lines. Four live, two construction, one play/pause keep-dirty. |
| `web/package.json` | `test:player` allowlist includes `reset-honesty.spec.ts` | ✓ VERIFIED | Exact four-file Playwright list. `@kobalte/core` remains `0.13.12`. |
| `20-REVIEW.md` | Independent exact-digest acknowledgment | ✓ VERIFIED | AI reviewer `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a`. Digest matches live bytes. |

### Key Link Verification

gsd-tools `verify key-links` auto-verified 11/13. The two failures are tool `to:` path matching, not missing wiring (same class of false negative as Phase 19 escaped-regex links). Manual grep confirms both.

| From | To  | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `runtime.ts` | `scenes.ts` | `sceneControlsIdentity` interpolates `SceneId` | WIRED | Tool pass. `SceneId` import + `${sceneId}:${generation}`. |
| `controls.test.ts` | `scene-controls.ts` | `initialPresetValue` | WIRED | Tool pass. |
| `DemoNavigation.tsx` | `previews.tsx` | `ScenePreview sceneId={item.id}` inside hash `<a>` | WIRED | Tool pass. |
| `previews.tsx` | `scenes.ts` | `switch` on `SceneId` / `case "dam-break"` | WIRED | Tool false negative (`to` file path not a string). File imports `SceneId` from `./scenes` and switches all six ids. |
| `App.tsx` | `PlayerSceneChrome.tsx` | PlayerPanel children render chrome | WIRED | Tool pass. |
| `PlayerSceneChrome.tsx` | `runtime.ts` | `Show keyed` + `sceneControlsIdentity` | WIRED | Tool pass. |
| `App.tsx` | `App.tsx` | `recreateScene` clears bag then `setResetGeneration` | WIRED | Tool pass. Order is `{}` then bump then `startScene`. |
| `shell.spec.ts` | `.demo-sidebar` | six `Static preview` | WIRED | Tool pass. |
| `shell.spec.ts` | dialog named Demos | mobile drawer preview | WIRED | Tool false negative (`to` is not a file). Test uses `getByRole("dialog", { name: "Demos" })`. |
| `package.json` | `reset-honesty.spec.ts` | `test:player` list | WIRED | Tool pass. |
| `reset-honesty.spec.ts` | Reset scene | `toHaveValue` documented initial | WIRED | Tool pass. |
| `20-REVIEW.md` | digest | SHA-256 + reviewer identity | WIRED | Tool pass. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `DemoNavigation` title/description | `item.title` / `item.description` | `SCENES` via `sceneNavigationItems` | Six locked catalog strings, not empty placeholders | ✓ FLOWING |
| `DemoNavigation` preview | `item.id` → `ScenePreview` | Exhaustive `SceneId` SVG switch | Token-only stills; no WASM/WebP | ✓ FLOWING |
| `PresetControl` select | `pendingValue` | `initialPresetValue(control, maybeValues)` at remount | After Reset, `maybeValues` is `{}` so `DEFAULT_PRESET_VALUES` | ✓ FLOWING |
| Reset native world | `constructionEntriesForScene(scene, constructionValues)` | Empty bag after `recreateScene` | `[]` → `startScene` does not replay last construction presets | ✓ FLOWING |
| Hash open | `item.href` | `` `#/scene/${scene.id}` `` | Existing hash router; sidebar click in `shell.spec.ts` opens Fountain | ✓ FLOWING |

`pendingValue` is a one-time `createSignal` snapshot. That is why the keyed remount exists; it is not a hollow prop. Credits stay outside the remount so they do not reset with selects.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Review digest still matches current bytes | Concatenated SHA-256 of the 13 manifest files in 20-06 order | `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f` | ✓ PASS |
| No `.catalog-card` / `.catalog-grid` / `CatalogNav` in `web/src` | `rg` under `web/src` | Zero matches | ✓ PASS |
| `test:player` lists shell + reset-honesty | `web/package.json` | Both files present; no firefox/webkit | ✓ PASS |
| `App.tsx` stays at or under 628 lines | `wc -l web/src/App.tsx` | 617 | ✓ PASS |
| Local player-smoke completed | Read `target/web-build/web-build.log` and `web/test-results/.last-run.json` | `complete player-smoke`; `"status": "passed"` | ✓ PASS |
| `just web-player-smoke` exists | `justfile` | `bun scripts/web-build.ts player-smoke` | ✓ PASS |
| No Pages Playwright job | `rg playwright .github/workflows/pages.yml` | Empty | ✓ PASS |

Step 7b did not start Playwright or a server. The ignored smoke log is committed local evidence from HEAD `d33d3e2` / 20-06, not a fresh run in this verification. Allowlisted specs declare 37 Chromium tests (`player` 13 + `shell` 9 + `reset-honesty` 7 + `demo-media-clock` 8 including the capture-preconditions describe case).

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| WEB-01 | 20-02, 20-04, 20-06 | A visitor can browse six demo entries with names, short descriptions and previews, then open a selected demo in a shared player. | ✓ SATISFIED | Six `SCENES` + `DemoNavigation` hash links with SVG stills and `Static preview`; Chromium sidebar/drawer proofs; `.catalog-card` remains 0. REQUIREMENTS.md still says “demo cards”; Phase 20 D-04 / SC2 forbids restoring that layout. Intent is met by list items in the Kobalte shell. REQUIREMENTS.md already Complete. |
| WEB-03 | 20-01, 20-03, 20-05, 20-06 | A visitor can play, pause and reset the selected demo to its documented initial state. | ✓ SATISFIED | Play/pause freeze or resume without remounting selects. Reset/Retry clear the construction bag, remount keyed controls, and rebuild WASM defaults. Chromium live + construction `toHaveValue` proofs plus play/pause keep-dirty. REQUIREMENTS.md already Complete. |

No orphaned Phase 20 requirements. Plans claimed only WEB-01 and WEB-03. REQUIREMENTS.md maps those two IDs to Phase 20. No later-phase deferral of these truths: Phase 21 is leftover cleanup (`loadProofSession`, dead `FallbackPanel` branches, scene file-lengths), not catalog previews or Reset labels.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `web/src/components/SceneControls.tsx` | 29-31 | `pendingValue` initialized once from `initialPresetValue` | ℹ️ Info | Intentional. Without keyed remount this would stay stale after Reset; Plan 03 wires that remount. |
| `web/src/player/runtime.ts` | 60-66 | `return []` in `constructionEntriesForScene` | ℹ️ Info | Filter for non-construction controls and missing bag keys, not a stub. Empty `{}` is the Reset contract. |
| `deferred-items.md` | — | Pre-existing `tools/xtask/tests/upstream_cli.rs` file-lengths | ℹ️ Info | Out of Phase 20 catalog/Reset scope. Scene-module lengths remain Phase 21. |

No TODO/FIXME/placeholder stubs in Phase 20 catalog, chrome, runtime, or e2e files. No `innerHTML`. No README WebP used as in-app previews. No React `key=` on `SceneControls`.

### Human Verification Required

None. Agent-performed simple UAT covered the objective checkpoints below. Subjective preview-art taste is not a separate gate beyond the UI-SPEC geometries, `Static preview` caption, and Chromium visibility counts.

### Agent UAT (objective checkpoints)

| Checkpoint | result | verified_by | evidence |
| ---------- | ------ | ----------- | -------- |
| Six in-app entries show name, description, static preview, and hash open | pass | agent | `SCENES` six titles/descriptions; `DemoNavigation` child order; `href="#/scene/{id}"`; `shell.spec.ts` six sidebar captions/SVGs |
| Kobalte shell kept; `.catalog-card` count 0 | pass | agent | `PlaygroundShell` Dialog drawer; `rg` zero card/grid/CatalogNav in `web/src`; three `toHaveCount(0)` asserts |
| Reset restores documented live and construction select labels | pass | agent | Keyed remount + empty bag in `App.tsx` / `PlayerSceneChrome.tsx`; `reset-honesty.spec.ts` seven cases |
| Play/pause do not fake a Reset of live selects | pass | agent | `playScene`/`pauseScene` omit bag/generation; Fountain keep-high test |
| Focused Chromium player-smoke includes preview + Reset specs | pass | agent | `test:player` allowlist; wrapper log complete; `.last-run.json` passed |
| Independent AI review bound to exact digest | pass | agent | `20-REVIEW.md` identity `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a`; verifier-recomputed digest match |

### Confirmation-bias notes (not gaps)

1. The desktop preview test counts six captions and six SVGs inside `.demo-sidebar` and does not name all six titles in that test. Titles and hash navigation are wired from `SCENES` through `DemoNavigation`; `player.spec.ts` still opens each native scene from desktop navigation.
2. D-10 representative live presets are emission-rate, body, stir-speed, and jet-strength. Launch-speed, aim-angle, emission, gravity, shape, and softness share the same keyed remount and `DEFAULT_PRESET_VALUES` path but have no dedicated Playwright row.
3. Roadmap SC3 says play/pause/Reset “rebuild the native world.” D-07 and the code freeze or resume on play/pause and rebuild only on Reset/Retry. The play/pause keep-dirty test is the intended honesty split, not a missing rebuild.
4. `20-06-SUMMARY.md` and the orchestrator reported 37 passed. This verifier counted 37 `test(` cases on the `test:player` allowlist, including `demo-media-clock`’s capture-preconditions describe case. The wrapper log does not embed Playwright’s per-test count; `.last-run.json` is the machine pass record.

### Gaps Summary

No actionable gaps. Phase 20 achieved the goal: visitors can browse six in-app demo entries with names, descriptions, and static visual previews in the existing Kobalte shell, then Reset a playing scene to documented initial physics with matching live-control labels.

---

_Verified: 2026-09-20T19:33:32Z_
_Verifier: Claude (gsd-verifier)_
