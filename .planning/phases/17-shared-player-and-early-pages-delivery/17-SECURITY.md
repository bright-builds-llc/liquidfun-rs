---
phase: 17-shared-player-and-early-pages-delivery
slug: 17-shared-player-and-early-pages-delivery
audited_at: 2026-09-17T12:31:28Z
status: verified
asvs_level: 1
block_on: high
security_enforcement: true
threats_total: 53
threats_closed: 53
threats_open: 0
unregistered_flags: 0
review_digest: a962555da21e1da79b860c37b003d29fa7f3d6683ef16db1b2beac2a151bf893
---

# Phase 17 Security Verification

> Per-phase security contract: threat register, accepted risks, and audit trail.

This audit verifies only the 53 threats declared in the eight Phase 17
`<threat_model>` blocks. It does not invent new threats. ASVS Level 1;
`block_on: high`. Pages delivery is hobby-scope website hosting and is not
Linux native qualification.

State B: no prior `17-SECURITY.md`. Accept dispositions from earlier plans
deferred controls to later same-phase plans; those later controls are now
present and the deferred entries are recorded in the accepted-risks log.

## Scope and method

Inspected the declared hash parser, catalog, clock, session, generation token,
player shell, chrome, build provenance, player smoke, Pages workflow, HOST-03
evidence, and independent review. Passing automation is supporting context
only. Repository guidance applied: `AGENTS.md`, `PROJECT-SCOPE.md`,
`17-CONTEXT.md` D-04 through D-18, and `17-UI-SPEC.md`.

Implementation files were not modified.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| Browser location hash → parser | Untrusted path tokens | Hash string → `SceneRoute` |
| Parser → player shell | Only allowlisted routes may construct WASM | `SceneRoute` / ready flag |
| Animation timestamp → clock | Untrusted elapsed values | `f64` elapsed → 0–4 steps |
| TypeScript owner → generated WASM | Only checked 1–4 counts may cross `advance` | Step budget |
| Exception objects → UI | Generated/browser errors must not become HTML | Fixed copy / optional DEV text |
| CI / git env → browser bundle | Provenance strings compiled into public JS | Version, SHA, build URL |
| GitHub Actions → Pages | OIDC token and `GITHUB_TOKEN` can publish | `web/dist` artifact |
| Review digest → acknowledgment | Only a separate reviewer may bind approval | Manifest digest |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-17-01-01 | S | `maybeParseSceneRoute` | mitigate | Allowlist `SCENE_IDS` only | closed |
| T-17-01-02 | T | Hash / scene strings | mitigate | `maybeRaw` never becomes `innerHTML` or `href` | closed |
| T-17-01-03 | I | Catalog readiness | mitigate | Ready flag is catalog data; WASM only for Dam Break | closed |
| T-17-01-04 | D | WASM session disposal | accept | Deferred to Plan 17-05; now `App.tsx` dispose | closed |
| T-17-01-05 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07; now `pages.yml` OIDC | closed |
| T-17-01-06 | T | Artifact integrity | accept | Deferred to Plans 17-03/17-07 same-checkout deploy | closed |
| T-17-02-01 | D | `acceptedStepCount` | mitigate | Cap 4 steps; reject non-finite elapsed | closed |
| T-17-02-02 | D | `SceneSession.nextFrame` | mitigate | Guard 1..=4; poison; one capture | closed |
| T-17-02-03 | T | WASM memory | mitigate | Copied arrays + `finally` free; no workers/views | closed |
| T-17-02-04 | I | Session disposal | mitigate | Idempotent `dispose()` and use-after-dispose tests | closed |
| T-17-02-05 | S | Hash-route handling | accept | Deferred to Plans 17-01/17-05 allowlist + dispose | closed |
| T-17-02-06 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07 | closed |
| T-17-02-07 | T | Artifact integrity | accept | Deferred to Plans 17-03/17-07 | closed |
| T-17-02-08 | T | XSS from scene/error strings | mitigate | Fixed `FAILED_MESSAGE` / `DISPOSED_MESSAGE` | closed |
| T-17-03-01 | T | Vite `base` | mitigate | Production/preview `/liquidfun-rs/`; dist gate | closed |
| T-17-03-02 | S | Footer URLs | mitigate | Allowlisted GitHub repo URLs only | closed |
| T-17-03-03 | T | XSS from scene/error strings | mitigate | Provenance is injected env, not scene hashes | closed |
| T-17-03-04 | T | Artifact integrity | mitigate | Same-checkout `just web-build`; deploy does not rebuild | closed |
| T-17-03-05 | D | WASM session disposal | accept | Deferred to Plan 17-05 | closed |
| T-17-03-06 | S | Hash-route handling | accept | Deferred to Plan 17-01 | closed |
| T-17-03-07 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07 | closed |
| T-17-04-01 | T | Fallback and error copy | mitigate | Fixed UI-SPEC text nodes; no `innerHTML` | closed |
| T-17-04-02 | S | Catalog hrefs | mitigate | Hash-only `#/scene/{id}` from catalog ids | closed |
| T-17-04-03 | S | Footer links | mitigate | Hard-coded + allowlisted URLs; `noopener noreferrer` | closed |
| T-17-04-04 | D | WASM session disposal | accept | Deferred to Plan 17-05 | closed |
| T-17-04-05 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07 | closed |
| T-17-04-06 | T | Artifact integrity | accept | Deferred to Plan 17-03 dist gate | closed |
| T-17-05-01 | D | Session / rAF ownership | mitigate | One session, generation token, cleanup dispose | closed |
| T-17-05-02 | D | Hidden-tab clock | mitigate | Skip while hidden; clear timestamp; max 4 | closed |
| T-17-05-03 | S | Hash-route handling | mitigate | Allowlist; WASM only for `dam-break` | closed |
| T-17-05-04 | T | XSS from scene/error strings | mitigate | Fixed copy; DEV `Details:` sliced text | closed |
| T-17-05-05 | T | WASM memory | mitigate | Copied frames + `dispose()`; no workers/views | closed |
| T-17-05-06 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07 | closed |
| T-17-05-07 | T | Artifact integrity | accept | Same-checkout `just web-build` | closed |
| T-17-06-01 | T | Production-base assets | mitigate | Smoke asserts WASM URL contains `/liquidfun-rs/` | closed |
| T-17-06-02 | S | Hash-route handling | mitigate | Unknown id → `Scene not found`; Open Dam Break | closed |
| T-17-06-03 | D | WASM session disposal | mitigate | Reset/leave restart or drop the step series | closed |
| T-17-06-04 | T | XSS from scene/error strings | mitigate | Spec asserts fixed headings/buttons | closed |
| T-17-06-05 | T | Artifact integrity | mitigate | Player smoke builds via `scripts/web-build.ts` | closed |
| T-17-06-06 | E | Pages OIDC / permissions | accept | Deferred to Plan 17-07 | closed |
| T-17-07-01 | E | Deploy permissions | mitigate | No PAT; job-scoped `pages`/`id-token` write | closed |
| T-17-07-02 | T | Artifact integrity | mitigate | `needs: build-site`; upload `web/dist` only | closed |
| T-17-07-03 | T | Older revision wins | mitigate | Per-ref concurrency; main does not cancel | closed |
| T-17-07-04 | S | Hash-route handling | accept | Workflow does not parse hashes | closed |
| T-17-07-05 | D | WASM session disposal | accept | Static host; client dispose remains 17-05 | closed |
| T-17-07-06 | T | XSS from scene/error strings | accept | No server-rendered scene strings | closed |
| T-17-07-07 | I | Qualification creep | mitigate | WASM + `web/` only; no oracle/sanitizer/coverage | closed |
| T-17-08-01 | T | Artifact integrity | mitigate | Recorded SHA/workflow; live WASM is not HTML | closed |
| T-17-08-02 | E | Pages OIDC / permissions | mitigate | Ordinary git credentials; workflow has no PAT | closed |
| T-17-08-03 | S | Hash-route handling | mitigate | `#/scene/dam-break` reload; unknown is client fallback | closed |
| T-17-08-04 | T | XSS from scene/error strings | mitigate | Reviewer inspected text-node copy; no `innerHTML` | closed |
| T-17-08-05 | D | WASM session disposal | mitigate | Reviewer inspected reset/leave/hidden-tab evidence | closed |
| T-17-08-06 | R | Self-review | mitigate | Separate AI reviewer + exact digest (D-18) | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Complete verification evidence

### Plan 17-01 — catalog and hash parser

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-01-01 | S | mitigate | CLOSED | `maybeParseSceneRoute` matches the second path token only against `SCENE_IDS` (`hash.ts:25-30`). Unknown tokens stay `{ kind: "unknown" }` and never become a `SceneId`. No `@solidjs/router`, `createRouter`, or `history.push` exists under `web/src`. Tests cover empty, unknown, Dam-Break case, and fountain (`hash.test.ts`). |
| T-17-01-02 | T | mitigate | CLOSED | `maybeRaw` is stored on unknown routes only (`hash.ts:6,22,27`) and is never interpolated into `href` or HTML. Unknown fallback uses fixed `Scene not found` copy (`FallbackPanel.tsx:4-6,26-28`). No `innerHTML` exists under `web/src`. |
| T-17-01-03 | I | mitigate | CLOSED | Readiness lives on catalog records; only Dam Break is `ready: true` (`scenes.ts:18-24,31-34`). Parser returns `{ kind: "scene" }` for known not-ready ids; `App` constructs WASM only when `route.id === "dam-break"` (`App.tsx:47-49,313-326,469-480`). |
| T-17-01-04 | D | accept | CLOSED | Accepted at parse-only time. Plan 17-05 now disposes on route change (`App.tsx:183-193,403-412,434-438`). Logged below. |
| T-17-01-05 | E | accept | CLOSED | Accepted before any workflow. Plan 17-07 now pins OIDC and forbids a PAT (`pages.yml:9-10,64-69`). Logged below. |
| T-17-01-06 | T | accept | CLOSED | Accepted before dist/deploy. Plans 17-03/17-07 bind same-checkout `web-build` to upload (`web-build.ts:213-226`; `pages.yml:43-57,59-61`). Logged below. |

### Plan 17-02 — clock and session

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-02-01 | D | mitigate | CLOSED | `acceptedStepCount` returns 0 for non-finite or non-positive elapsed time and caps at 4 (`clock.ts:6-13`). Tests cover NaN/±Infinity, non-positive, 1–3 ticks, and oversized finite values (`clock.test.ts:29-79`). |
| T-17-02-02 | D | mitigate | CLOSED | `isAcceptedStepCount` requires a safe integer in 1..=4; violation poisons and throws the fixed failure message (`session.ts:25-31,57-60`). Legal 1/2/4 counts use one `advance` and one `capture` (`session.test.ts:199-237`); 0 and 5 never call generated advance (`session.test.ts:239-258`). |
| T-17-02-03 | T | mitigate | CLOSED | Temporary frame is freed in `finally` (`session.ts:65-69`). Parser returns JS-owned typed arrays and does not retain the Rust wrapper (`frame.ts:114-185`). No `WebAssembly.Memory`, `.buffer`, pointer, worker, or zero-copy path exists under `web/src/physics` or `App.tsx`. |
| T-17-02-04 | I | mitigate | CLOSED | `dispose()` is idempotent (`session.ts:76-83`). Tests assert exact-once free and use-after-dispose rejection with the fixed message (`session.test.ts:169-197`). |
| T-17-02-05 | S | accept | CLOSED | Accepted; later allowlist + route-change dispose are present. Logged below. |
| T-17-02-06 | E | accept | CLOSED | Accepted; later OIDC workflow is present. Logged below. |
| T-17-02-07 | T | accept | CLOSED | Accepted; later same-checkout deploy is present. Logged below. |
| T-17-02-08 | T | mitigate | CLOSED | Session throws only `FAILED_MESSAGE` / `DISPOSED_MESSAGE` and swallows generated exception text (`session.ts:7-8,53-73`). Tests assert those fixed strings after generated `"unbounded generated detail"` errors (`session.test.ts:135-167`). |

### Plan 17-03 — production base and provenance

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-03-01 | T | mitigate | CLOSED | Vite production/preview `base` is `/liquidfun-rs/` (`vite.config.ts:6`). `assertProductionAssetPaths` requires `/liquidfun-rs/assets/` in `index.html` and at least one `.wasm` file (`web-build.ts:213-226`). |
| T-17-03-02 | S | mitigate | CLOSED | Commit URLs are built only from a 40-hex SHA plus a hard-coded repo origin (`build-info.ts:17-44`). Build URLs must match this repository's Actions run pattern (`build-info.ts:19-20,47-55`). Hash-route input is never used. |
| T-17-03-03 | T | mitigate | CLOSED | Provenance is injected env (`web-build.ts:205-210`; `pages.yml:44-48`). Footer renders labels as text nodes (`SiteFooter.tsx:46-68`). |
| T-17-03-04 | T | mitigate | CLOSED | `build-site` runs `bun scripts/web-build.ts build` and uploads `web/dist` (`pages.yml:43-57`). `deploy-pages` only runs `actions/deploy-pages` after `needs: build-site` (`pages.yml:59-77`). |
| T-17-03-05 | D | accept | CLOSED | Accepted; later player dispose is present. Logged below. |
| T-17-03-06 | S | accept | CLOSED | Accepted; later hash allowlist is present. Logged below. |
| T-17-03-07 | E | accept | CLOSED | Accepted; later OIDC deploy is present. Logged below. |

### Plan 17-04 — presentational chrome

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-04-01 | T | mitigate | CLOSED | Fallback and player copy are constants rendered as text children (`FallbackPanel.tsx:1-48`; `PlayerPanel.tsx:15-35,83-88`). Optional DEV `Details:` is sliced to 240 characters and rendered in a `<p>` (`App.tsx:109-117`; `PlayerPanel.tsx:85-87`). `rg innerHTML web/src` is empty. |
| T-17-04-02 | S | mitigate | CLOSED | Catalog links are `#/scene/{id}` from `SCENES` ids; Dam Break is the literal `#/scene/dam-break` (`CatalogNav.tsx:38-46`). Fallback Open Dam Break is the same hash (`FallbackPanel.tsx:44`). No origin-absolute `/#/scene/` or path `/scene/` navigation. |
| T-17-04-03 | S | mitigate | CLOSED | Source and maintainer hrefs are constants; provenance links use `readBuildInfo()` allowlisted URLs; all use `rel="noopener noreferrer"` (`SiteFooter.tsx:5-6,23,38-43`). |
| T-17-04-04 | D | accept | CLOSED | Accepted; later one-session owner is present. Logged below. |
| T-17-04-05 | E | accept | CLOSED | Accepted; later OIDC workflow is present. Logged below. |
| T-17-04-06 | T | accept | CLOSED | Accepted; later `/liquidfun-rs/` dist gate is present. Logged below. |

### Plan 17-05 — shared player shell

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-05-01 | D | mitigate | CLOSED | One `maybeSession`, generation increment on start/abandon, `isStaleGeneration` discard with immediate dispose, and `onCleanup` → `abandonDamBreak` (`App.tsx:138-151,183-193,298-351,434-438`; `generation.ts:1-9`). |
| T-17-05-02 | D | mitigate | CLOSED | Hidden-document rAF clears `maybeLastTimestamp` and does not step (`App.tsx:213-217`). Visibility changes and pause/play also clear the timestamp (`App.tsx:379,391,414-416`). Step budget is `acceptedStepCount` (`App.tsx:238-244`). |
| T-17-05-03 | S | mitigate | CLOSED | Route comes from `maybeParseSceneRoute`. Player panel and `loadProofSession` run only for Dam Break; other hashes render `FallbackPanel` (`App.tsx:47-49,403-412,421-432,468-480`). |
| T-17-05-04 | T | mitigate | CLOSED | Failure UI uses fixed `FAILURE_COPY`. DEV details are prefixed `Details:` and sliced; production returns `undefined` (`App.tsx:109-117`; `PlayerPanel.tsx:31-32,82-88`). No `innerHTML` in `App.tsx`. |
| T-17-05-05 | T | mitigate | CLOSED | Player still consumes `createSceneSession` copied frames and `dispose()`. No workers or zero-copy views added. |
| T-17-05-06 | E | accept | CLOSED | Accepted; later OIDC workflow is present. Logged below. |
| T-17-05-07 | T | accept | CLOSED | Player still builds through same-checkout `just web-build`. Logged below. |

### Plan 17-06 — local player smoke

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-06-01 | T | mitigate | CLOSED | `openDamBreakPlaying` requires a `.wasm` request whose URL contains `/liquidfun-rs/` (`player.spec.ts:63-65`). Preview `baseURL` is `/liquidfun-rs/` (`playwright.config.ts:45`). |
| T-17-06-02 | S | mitigate | CLOSED | Unknown hash asserts heading `Scene not found` and Open Dam Break navigates to `#/scene/dam-break` (`player.spec.ts:148-157`). |
| T-17-06-03 | D | mitigate | CLOSED | Reset requires a lower step index than the prior series (`player.spec.ts:140-145`). Leaving for Fountain drops the live canvas, sets `data-playback=fallback`, and does not request a second WASM (`player.spec.ts:160-176`). |
| T-17-06-04 | T | mitigate | CLOSED | Spec asserts fixed headings and buttons (`Scene not found`, `Play scene`, `Pause scene`, `Reset scene`), not raw exception HTML (`player.spec.ts:117-157`). |
| T-17-06-05 | T | mitigate | CLOSED | `just web-player-smoke` runs `bun scripts/web-build.ts player-smoke`, which regenerates WASM and builds from the current checkout before Playwright (`web-build.ts:517-523`; `justfile:26-27`). |
| T-17-06-06 | E | accept | CLOSED | Accepted; later deploy job is present. Logged below. |

### Plan 17-07 — Pages workflow

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-07-01 | E | mitigate | CLOSED | Workflow top-level `permissions: contents: read` (`pages.yml:9-10`). Deploy job is `pages: write` + `id-token: write` (`pages.yml:66-69`). No `secrets.*`, PAT, or extra token input. Actions are SHA-pinned. |
| T-17-07-02 | T | mitigate | CLOSED | `deploy-pages` `needs: build-site` and publishes the uploaded `web/dist` (`pages.yml:53-61,73-77`). Deploy does not rebuild WASM. |
| T-17-07-03 | T | mitigate | CLOSED | Concurrency group is `${{ github.workflow }}-${{ github.ref }}`; `cancel-in-progress` is false on `main` (`pages.yml:15-17`). |
| T-17-07-04 | S | accept | CLOSED | Workflow does not parse hashes; the built player allowlists ids. Logged below. |
| T-17-07-05 | D | accept | CLOSED | Static Pages host; disposal remains client behavior. Logged below. |
| T-17-07-06 | T | accept | CLOSED | No server-rendered scene strings. Logged below. |
| T-17-07-07 | I | mitigate | CLOSED | Pages runner installs Rust/wasm-pack/Bun and runs `bun scripts/web-build.ts build` only (`pages.yml:32-49`). No oracle, sanitizer, coverage, or performance gates. Hobby scope: website delivery is not Linux qualification. |

### Plan 17-08 — live evidence and independent review

| Threat ID | Category | Disposition | Verification evidence |
| --- | --- | --- | --- |
| T-17-08-01 | T | mitigate | CLOSED | `17-HOST-EVIDENCE.md` records `source_sha` `50a15562b356ed941266eedddc636df3f76e7e7e`, workflow `35220721701`, and WASM URL with `Content-Type: application/wasm` plus `\0asm` magic, not HTML. |
| T-17-08-02 | E | mitigate | CLOSED | Evidence records an ordinary non-force push; `pages.yml` still has no PAT. Review T-17-08-02 confirms OIDC Pages permissions. |
| T-17-08-03 | S | mitigate | CLOSED | HOST-03 fetched `#/scene/dam-break` and still received the app shell. Unknown hashes are a client fallback (`FallbackPanel.tsx`; `player.spec.ts:148-157`). Independent review restated the same bound. |
| T-17-08-04 | T | mitigate | CLOSED | Separate reviewer inspected text-node error/fallback copy and recorded no source `innerHTML` (`17-REVIEW.md` Threat review). |
| T-17-08-05 | D | mitigate | CLOSED | Separate reviewer inspected reset/leave dispose and hidden-tab bounds in `App.tsx`, `clock.ts`, `session.ts`, and `player.spec.ts` (`17-REVIEW.md` Threat review). |
| T-17-08-06 | R | mitigate | CLOSED | Reviewer identity `bb07ac37-0a71-4930-9eb1-30fe1a0ac188` is not implementing executor `df1401ce-d6bc-4fd9-9a2e-8540478600ea`. Acknowledgment is bound to digest `a962555da21e1da79b860c37b003d29fa7f3d6683ef16db1b2beac2a151bf893` at `2026-09-17T12:29:00Z` and discloses AI review, not human approval. |

---

## Accepted Risks Log

These entries were `accept` in an earlier plan because that plan did not own
the control. The later same-phase mitigation is now present. They do not
resurface as open threats.

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-17-01 | T-17-01-04 | Parse-only plan had no session. Plan 17-05 now disposes on route change, reset, retry, and cleanup. | Phase 17 threat model; later mitigate T-17-05-01 | 2026-09-17 |
| AR-17-02 | T-17-01-05, T-17-02-06, T-17-03-07, T-17-04-05, T-17-05-06, T-17-06-06 | No workflow in those plans. Plan 17-07 now uses OIDC `id-token: write` + `pages: write` and no PAT. | Phase 17 threat model; later mitigate T-17-07-01 | 2026-09-17 |
| AR-17-03 | T-17-01-06, T-17-02-07, T-17-04-06, T-17-05-07 | No Pages artifact in those plans. Plans 17-03/17-07 now bind same-checkout `web/dist` to deploy. | Phase 17 threat model; later mitigate T-17-03-04 / T-17-07-02 | 2026-09-17 |
| AR-17-04 | T-17-02-05, T-17-03-06 | No hash parser in those plans. Plan 17-01 allowlists ids; Plan 17-05 disposes on non-Dam-Break routes. | Phase 17 threat model; later mitigate T-17-01-01 / T-17-05-03 | 2026-09-17 |
| AR-17-05 | T-17-03-05, T-17-04-04 | No player lifecycle in chrome/build plans. Plan 17-05 owns the one session. | Phase 17 threat model; later mitigate T-17-05-01 | 2026-09-17 |
| AR-17-06 | T-17-07-04 | Workflow does not parse hashes. Residual risk accepted; client allowlist remains T-17-01-01 / T-17-05-03. | Phase 17 Plan 17-07 | 2026-09-17 |
| AR-17-07 | T-17-07-05 | GitHub Pages is a static host and cannot dispose WASM. Residual risk accepted; client dispose remains T-17-05-01. | Phase 17 Plan 17-07 | 2026-09-17 |
| AR-17-08 | T-17-07-06 | Deploy publishes static assets only. Residual XSS risk accepted at the host; client text-node copy remains T-17-04-01 / T-17-05-04. | Phase 17 Plan 17-07 | 2026-09-17 |

No transferred risks.

---

## Threat flags and unregistered flags

None of the eight summaries contains a `## Threat Flags` section.

Plan 17-06 recorded two auto-fixed player-start bugs (0×0 first layout and
hash-return stuck on loading). Those are implementation corrections mapped to
T-17-05-01 / T-17-06-03, not new unregistered attack surface.

Plan 17-08 recorded required reviewer independence (D-18), mapped to
T-17-08-06.

## Unregistered flags

None.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-09-17 | 53 | 53 | 0 | gsd-security-auditor (State B, first audit) |

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-09-17 — mitigation verification only; not
independent implementation review and not package-release authority.

## SECURED

**Phase:** 17 — Shared Player and Early Pages Delivery
**Threats Closed:** 53/53
**Threats Open:** 0
**ASVS Level:** 1

All declared mitigations are present in the implemented player, build,
workflow, HOST-03 evidence, and independent digest review. Accepted earlier
plan deferrals are logged against the later controls that now close them.
