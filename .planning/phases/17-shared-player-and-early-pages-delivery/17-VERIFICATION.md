---
phase: 17-shared-player-and-early-pages-delivery
verified: 2026-09-17T12:36:00Z
status: passed
score: 10/10 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T12:36:00Z
lifecycle_validated: true
overrides_applied: 0
human_verification: []
---

# Phase 17: Shared Player and Early Pages Delivery Verification Report

**Phase Goal:** Visitors can open, control and reload a working first scene on the actual GitHub Pages site, and new main pushes deliver a matching site/WASM artifact reliably.
**Verified:** 2026-09-17T12:36:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The hosted Dam Break playground exists, is wired through one Solid session owner, and is live at the recorded GitHub Pages origin. Roadmap success criteria, plan must-haves, and all seven requirement IDs are satisfied by code plus rechecked live evidence. Objective UAT was completed by the verifier from repo artifacts, unit tests, workflow inspection, and non-destructive live GETs.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | A visitor opens and reloads a stable scene URL under the real Pages project path with functioning JS/WASM assets; an unknown scene identifier provides a useful fallback. The deployed URL and source revision are recorded. | ✓ VERIFIED | Live `GET https://bright-builds-llc.github.io/liquidfun-rs/` is HTTP 200 HTML with `<title>liquidfun-rs playground</title>` and `/liquidfun-rs/assets/` script/css tags. Hosted JS `index-BBVknQKs.js` contains `Dam Break`, `Play scene`, `Open Dam Break`, `Scene not found`, and `liquidfun_wasm_bg-BWMkHUXc.wasm` under `/liquidfun-rs/assets/`. WASM URL returns `application/wasm`, 674952 bytes, magic `\0asm`. `17-HOST-EVIDENCE.md` records `page_url`, `source_sha` `50a15562b356ed941266eedddc636df3f76e7e7e`, run `35220721701`, and that WASM URL. `origin/main` is that SHA. Unknown hashes render `FallbackPanel` `Scene not found` plus `Open Dam Break` (`FallbackPanel.tsx`, `player.spec.ts`). |
| 2 | The shared SolidJS player runs a useful first scene, supports play/pause/reset to its documented initial state, and presents visible loading, actionable failures and a working retry/reset path. | ✓ VERIFIED | `App.tsx` constructs WASM only for `#/scene/dam-break` via `loadProofSession` + `createSceneSession`. `PlayerPanel` shows `Loading Dam Break…` / `Playing` / `Paused` / `Dam Break failed`, buttons `Play scene` / `Pause scene` / `Reset scene`, and `Retry scene` only on failure with the fixed alert copy. Play resumes without dispose; Pause freezes and clears the timestamp; Reset and Retry call `startDamBreak()` which disposes then recreates. Documented basin is 16×12 = 192 particles, gravity `(0, -10)`, cap 512 (`crates/liquidfun-wasm/src/scene.rs`). Playwright exercises loading → playing, pause/play, and reset restart (`web/e2e/player.spec.ts`). |
| 3 | Resetting, changing selection and leaving the player release prior worlds and animation resources; only the current session steps, hidden tabs do not accumulate catch-up debt, and stepping/emission stay bounded. | ✓ VERIFIED | One `maybeSession` and one rAF id. `abandonDamBreak` increments generation, cancels rAF, disconnects the observer, and `dispose()`s. Hash leave and `onCleanup` call it. Stale `loadProofSession` results are disposed via `isStaleGeneration`. Hidden-document callbacks clear `maybeLastTimestamp` and skip `nextFrame`. `acceptedStepCount` caps at 4; `nextFrame` forwards one `advance(n)` then one capture and rejects 0/5+ without generated advance. Dam Break has no continuous emitter; particle capture is capped at 512. Player spec asserts Fountain leave starts no second WASM and hidden-tab resume advances ≤ 4 steps. |
| 4 | Every push to main triggers the WASM and production-site build from the same checkout without path filters, and successful required build checks precede deployment of that assembled artifact. | ✓ VERIFIED | `.github/workflows/pages.yml` triggers on `push` to `main`, `pull_request`, and `workflow_dispatch` with no `paths:` filter. `build-site` runs `bun scripts/web-build.ts build` then uploads `web/dist`. `deploy-pages` has `needs: build-site` and `if: github.event_name == 'push' && github.ref == 'refs/heads/main'`. Deploy does not rebuild WASM. Pages run `35220721701` on `50a1556` concluded `success` (`build-site` and `deploy-pages`). |
| 5 | Deployment uses Pages configuration and ordinary Actions permissions without a personal token; overlapping pushes cannot leave an older revision as the final site, though superseded queued revisions may coalesce. | ✓ VERIFIED | Workflow-level permissions are `contents: read` only. `deploy-pages` uses job-scoped `pages: write` + `id-token: write`, environment `github-pages`, and SHA-pinned official Pages actions. No PAT, `secrets.*`, or `peaceiris`. Concurrency group is `${{ github.workflow }}-${{ github.ref }}` with `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}` (false on main). Latest Pages run is the recorded successful publish; no later failed/stale run is being treated as passing. |
| 6 | Catalog data lists Dam Break as the only ready scene and names Fountain, Float or Sink, Color Mixer, Jelly Drop, and Water Wheel as not ready. | ✓ VERIFIED | `web/src/catalog/scenes.ts` exports the six locked ids/titles; exactly one `ready: true` (`dam-break`). `CatalogNav` renders `SCENES` with `Ready` / `Not ready yet` and hash-only `href="#/scene/dam-break"`. Unit tests: `tests/scenes.test.ts`. |
| 7 | A hash string is parsed into empty, unknown, or an allowlisted lowercase hyphenated scene id without reading the DOM or window. | ✓ VERIFIED | Pure `maybeParseSceneRoute` allowlists `SCENE_IDS` only, keeps `#/scene` empty, and does not coerce case (`web/src/routing/hash.ts`). No `@solidjs/router`. `App.tsx` listens to `hashchange` and feeds `window.location.hash` into that parser. Unit tests: `tests/hash.test.ts`. |
| 8 | Missing version, commit, or build fields render as Unavailable; footer chrome always shows source, FOSS, maintainer, and provenance. | ✓ VERIFIED | `readBuildInfo` maps blank/missing fields to `Unavailable` and allowlists this repo's commit/Actions URLs (`web/src/build-info.ts`). `SiteFooter` renders `View source on GitHub`, `Free and open source`, `By Peter Ryszkiewicz`, and Version/Commit/Build. Unit tests: `tests/build-info.test.ts`. |
| 9 | A separate identified AI reviewer acknowledges the player-plus-delivery diff and evidence at an exact digest; the implementing agent does not approve its own work. | ✓ VERIFIED | `17-REVIEW.md` is independent AI review, not human approval. Reviewer identity `bb07ac37-0a71-4930-9eb1-30fe1a0ac188` / invocation `376b6b01-b9af-4b29-9b87-257377e254d9` is not implementing executor `df1401ce-d6bc-4fd9-9a2e-8540478600ea`. Acknowledgment is bound to digest `a962555da21e1da79b860c37b003d29fa7f3d6683ef16db1b2beac2a151bf893` at `2026-09-17T12:29:00Z`. Decision `APPROVED`. Working-tree drift is only an added `reviewer_invocation_id` frontmatter field. |
| 10 | Phase 16 forensic smoke remains opt-in and is not the default browser suite. | ✓ VERIFIED | `web/e2e/rust-wasm-proof.spec.ts` `test.skip`s unless `PHASE16_CLOSURE_ATTEMPT_DIR` is set. Default player smoke is `just web-player-smoke` → `bun scripts/web-build.ts player-smoke`. README and TESTING document that split. |

**Score:** 10/10 truths verified

### Required Artifacts

gsd-tools `verify artifacts` passed 25/25 planned paths. Manual L2/L3 checks below.

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `web/src/catalog/scenes.ts` | Six locked ids, titles, ready flags | ✓ VERIFIED | Exists, substantive, imported by hash parser, CatalogNav, App |
| `web/src/routing/hash.ts` | Pure `#/scene/{id}` parser | ✓ VERIFIED | Allowlists `SCENE_IDS`; used by `App.tsx` |
| `web/src/physics/clock.ts` | `acceptedStepCount` max 4 | ✓ VERIFIED | Pure; used by `App.tsx` before `nextFrame` |
| `web/src/physics/session.ts` | `nextFrame(1..=4)` then one capture | ✓ VERIFIED | `generatedSession.advance(stepCount)` then `captureFrame()` |
| `web/vite.config.ts` | Production/preview base `/liquidfun-rs/` | ✓ VERIFIED | Dev serve stays `/` |
| `web/src/build-info.ts` | Provenance + `Unavailable` | ✓ VERIFIED | Consumed by `SiteFooter` |
| `scripts/web-build.ts` | VITE_* inject + dist gate | ✓ VERIFIED | Requires `/liquidfun-rs/assets/` and a `.wasm` |
| `standards-overrides.md` | D-09 semantic CSS exception | ✓ VERIFIED | Records Phase 17 skip of MysticUI/Tailwind |
| `web/src/components/CatalogNav.tsx` | Honest six-name list | ✓ VERIFIED | Mounted from `App.tsx` |
| `web/src/components/FallbackPanel.tsx` | Empty/unknown/not-ready + Open Dam Break | ✓ VERIFIED | Mounted for non-Dam-Break routes |
| `web/src/components/PlayerPanel.tsx` | Play/Pause/Reset/Retry chrome | ✓ VERIFIED | Wired to play/pause/recreate handlers |
| `web/src/components/SiteFooter.tsx` | Source/FOSS/maintainer/provenance | ✓ VERIFIED | Mounted on every view |
| `web/src/app.css` | Evolved Phase 16 dark tokens | ✓ VERIFIED | No Tailwind config under `web/` |
| `web/src/player/generation.ts` | Stale-load tokens | ✓ VERIFIED | Used in `startDamBreak` |
| `web/src/App.tsx` | One-session playground shell | ✓ VERIFIED | 486 lines; hash, clock, loader, dispose, visibility |
| `web/e2e/player.spec.ts` | Production-base D-17 proofs | ✓ VERIFIED | Dam Break, fallback, Fountain leave, hidden-tab |
| `web/playwright.config.ts` | Origin baseURL + `/liquidfun-rs/` preview | ✓ VERIFIED | `baseURL` `http://127.0.0.1:4173`; webServer URL includes `/liquidfun-rs/` |
| `justfile` | `web-player-smoke` | ✓ VERIFIED | Distinct from opt-in `web-smoke` |
| `.github/workflows/pages.yml` | SHA-pinned build + OIDC deploy | ✓ VERIFIED | Two jobs; no path filters |
| `17-HOST-EVIDENCE.md` | Live URL/SHA/run/wasm | ✓ VERIFIED | Rechecked live; not stale |
| `17-REVIEW.md` | Independent exact-digest ack | ✓ VERIFIED | Separate AI reviewer |
| `README.md` / `TESTING.md` | Playground URL and smoke split | ✓ VERIFIED | `#/scene/dam-break`, `/liquidfun-rs/`, `web-player-smoke` |

### Key Link Verification

gsd-tools reported five false-negative key links because escaped regex and non-file `from:` values. Manual inspection found each pattern.

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| `web/src/routing/hash.ts` | `web/src/catalog/scenes.ts` | `SCENE_IDS` allowlist | ✓ WIRED | Import + `SCENE_IDS.find` |
| `web/src/catalog/scenes.ts` | `dam-break` | Only Dam Break `ready: true` | ✓ WIRED | Line 19 |
| `web/src/physics/session.ts` | `generatedSession.advance` | One `advance(stepCount)` then capture | ✓ WIRED | `session.ts:63-64`; tool regex over-escaped |
| `web/src/physics/loader.ts` | `web/vite.config.ts` | `liquidfun_wasm_bg.wasm?url` inherits base | ✓ WIRED | `loader.ts:4`; production base prefixes that URL |
| `scripts/web-build.ts` | `web/dist/index.html` | Fail without `/liquidfun-rs/assets/` | ✓ WIRED | `assertProductionAssetPaths` |
| `CatalogNav.tsx` | `#/scene/dam-break` | Hash-only href | ✓ WIRED | Never `/#/scene/` |
| `SiteFooter.tsx` | `readBuildInfo` | Provenance labels | ✓ WIRED | |
| `FallbackPanel.tsx` | `Open Dam Break` | UI-SPEC CTA | ✓ WIRED | `href="#/scene/dam-break"` |
| `App.tsx` | `maybeParseSceneRoute` | `hashchange` | ✓ WIRED | |
| `App.tsx` | `dispose()` | Reset/leave/cleanup | ✓ WIRED | `disposeOwnedSession` / stale-load dispose; tool regex over-escaped |
| `App.tsx` | `acceptedStepCount` | Before `nextFrame(n)` | ✓ WIRED | |
| `App.tsx` | `loadProofSession` | Dam Break only | ✓ WIRED | |
| `player.spec.ts` | `/liquidfun-rs/#/scene/dam-break` | Playwright goto | ✓ WIRED | |
| `playwright.config.ts` | `127.0.0.1:4173/liquidfun-rs/` | Preview webServer | ✓ WIRED | `playwright.config.ts:45`; tool regex over-escaped |
| `pages.yml` | `scripts/web-build.ts` | `bun scripts/web-build.ts build` | ✓ WIRED | |
| `deploy-pages` | `build-site` | `needs: build-site` | ✓ WIRED | Job-to-job; tool looked for a file named `deploy-pages` |
| `17-HOST-EVIDENCE.md` | `github.io/liquidfun-rs` | Recorded live URL | ✓ WIRED | Rechecked 200 |
| `README.md` | `#/scene/dam-break` | Documented first scene | ✓ WIRED | |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| `App.tsx` | `route` / player `view` | `maybeParseSceneRoute(window.location.hash)` + `loadProofSession` WASM frames | Yes — live hash and generated Rust session | ✓ FLOWING |
| `PlayerPanel.tsx` | `status` / canvas frames | `playerStatus(view)` and `drawRenderFrame` of `nextFrame` output | Yes — copied typed arrays from Rust | ✓ FLOWING |
| `CatalogNav.tsx` | `SCENES` | Locked catalog module | Yes — static catalog is the product data | ✓ FLOWING |
| `FallbackPanel.tsx` | empty/unknown/not-ready copy | Route kind + `maybeSceneById` | Yes — parser/catalog, not a stub | ✓ FLOWING |
| `SiteFooter.tsx` | version/commit/build | `readBuildInfo()` from injected `VITE_*` | Yes — CI env or local git/timestamp; missing → `Unavailable` | ✓ FLOWING |
| Hosted `index-BBVknQKs.js` | scene/control copy + wasm URL | Production Vite build of the same sources | Yes — live bundle contains player strings and hashed wasm | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Catalog/hash/clock/session/generation/provenance unit tests | `cd web && bun run test:unit -- tests/scenes.test.ts tests/hash.test.ts tests/clock.test.ts tests/generation.test.ts tests/build-info.test.ts tests/session.test.ts` | 6 files, 38 tests passed in 106ms | ✓ PASS |
| Live app shell | `curl -sL https://bright-builds-llc.github.io/liquidfun-rs/` | HTTP 200; title `liquidfun-rs playground`; `/liquidfun-rs/assets/` JS+CSS | ✓ PASS |
| Live WASM asset | GET recorded `wasm_asset_url` | HTTP 200; `Content-Type: application/wasm`; 674952 bytes; magic `\0asm` | ✓ PASS |
| Live player bundle | GET `index-BBVknQKs.js` | Contains Dam Break controls, fallback copy, and wasm filename | ✓ PASS |
| Pages deploy of recorded SHA | `gh run view 35220721701` | `Pages` push on `main`, head `50a1556`, conclusion `success` | ✓ PASS |
| Recorded SHA is current `origin/main` | `git rev-parse origin/main` | `50a15562b356ed941266eedddc636df3f76e7e7e` | ✓ PASS |
| Implementation drift vs deployed SHA | `git diff --stat origin/main -- web/src .github/workflows/pages.yml scripts/web-build.ts` | Empty — player/workflow bytes match the live revision | ✓ PASS |
| Chromium player smoke | `just web-player-smoke` | ? SKIP — starts a preview server; local spec exists and SUMMARY recorded exit 0; not re-run here | ? SKIP |

### Agent-Performed Simple UAT

Objective checkpoints only. No subjective visual review.

| Checkpoint | result | verified_by | evidence |
| ---------- | ------ | ----------- | -------- |
| Live Pages HTML is the playground app shell | pass | agent | `GET https://bright-builds-llc.github.io/liquidfun-rs/` HTTP 200, title and `/liquidfun-rs/assets/` tags, 2026-09-17T12:33:32Z |
| Hosted WASM is a real wasm body, not HTML | pass | agent | Recorded asset URL HTTP 200, `application/wasm`, 674952, `\0asm` |
| Hosted JS is the Dam Break player, not the Phase 16 proof | pass | agent | Bundle contains `Play scene`, `Open Dam Break`, `Scene not found`; no `Dispose session` in `web/src` |
| Recorded HOST-03 run is the latest successful Pages publish | pass | agent | Run `35220721701` success; `gh run list --workflow Pages --limit 5` shows only that run |
| Workflow statically matches HOST-01/02 | pass | agent | No `paths:`; OIDC job perms; `needs: build-site`; main concurrency does not cancel in-flight deploys |
| Independent review acknowledgment exists and is not self-approval | pass | agent | Digest `a962555d…151bf893`, reviewer `bb07ac37-…`, implementing executor excluded |

### Requirements Coverage

All seven Phase 17 IDs appear in PLAN frontmatter. REQUIREMENTS.md maps no extra Phase 17 IDs.

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| WASM-04 | 17-02, 17-05, 17-06 | Reset/leave releases the prior world; bounded stepping, emission, and hidden-tab handling prevent unbounded catch-up | ✓ SATISFIED | Clock/session 1–4, generation dispose, hidden-tab clear, particle cap 512, player spec |
| WEB-02 | 17-01, 17-04, 17-05, 17-06 | Share/reload a stable Pages demo URL; unknown id returns a useful fallback | ✓ SATISFIED | Hash parser, catalog/fallback chrome, live `/liquidfun-rs/#/scene/dam-break`, unknown-hash spec |
| WEB-03 | 17-05, 17-06 | Play, pause, and reset to the documented initial state | ✓ SATISFIED | Player handlers + 192-particle constructor + Playwright pause/play/reset |
| WEB-06 | 17-04, 17-05, 17-06 | Visible loading, useful failure, working retry/reset instead of a blank canvas | ✓ SATISFIED | Loading overlay/status, failure alert + Retry, Reset recreate; canvas fallback text |
| HOST-01 | 17-07 | Every main push builds WASM + site from the same checkout, then deploys after required checks | ✓ SATISFIED | `pages.yml` + successful run `35220721701` |
| HOST-02 | 17-07 | Pages/OIDC permissions, no PAT; overlapping pushes cannot leave an older completed site | ✓ SATISFIED | Job-scoped OIDC; main `cancel-in-progress` false |
| HOST-03 | 17-03, 17-06, 17-08 | Live site loads JS/WASM under the project base; URL and source revision recorded | ✓ SATISFIED | Evidence file + independent live recheck of HTML/JS/WASM |

Orphaned requirements: none.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| `web/e2e/player.spec.ts` | — | No forced-failure / `Retry scene` click | ℹ️ Info | Retry is wired in `App.tsx`/`PlayerPanel.tsx` to the same recreate path as Reset; WEB-06 is implemented. Coverage gap only. |
| gsd-tools key-link output | — | Five `verified: false` from over-escaped regex / job-name `from` | ℹ️ Info | Manual grep found every intended pattern. Not a product gap. |
| Dam Break scene | — | No continuous emitter | ℹ️ Info | WASM-04 emission bound is vacuously held by a non-emitting basin plus particle cap 512. Fountain emission is Phase 18. |

No `TODO`/`FIXME`/`innerHTML`/`Dispose session`/`placeholder` product stubs under `web/src`.

### Confirmation-bias notes

1. **Partial automated coverage:** Retry/failure is presentationally and imperatively wired but not clicked in Playwright. Not a failed truth.
2. **Misleading-looking tool result:** `verify key-links` failed several links that exist in source. Treat those as tool limitations.
3. **Error path:** Invalid `nextFrame` counts poison without calling generated `advance` and are unit-tested. Hosted failure UI uses fixed copy; generated exception text is not rendered in production.

### Human Verification Required

None. Visual polish, pointer/accessibility, and six-scene smoke belong to Phase 19 and are not Phase 17 success criteria. Objective hosted-shell, WASM, workflow, and control-wiring checks were completed by the agent.

### Gaps Summary

No actionable gaps. Later milestone phases own the remaining five physics demos (Phase 18) and pointer/accessibility/six-scene hosted smoke (Phase 19); those are out of this phase's contract, not failed must-haves.

---

_Verified: 2026-09-17T12:36:00Z_
_Verifier: Cursor AI (gsd-verifier)_
