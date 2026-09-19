---
phase: 19-interaction-polish-and-browser-verification
plan: "07"
requirement: WEBTEST-01
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 19-2026-09-19T01-44-48
generated_at: 2026-09-19T02:34:24Z
---

# Phase 19 WEBTEST-01 live Pages evidence

Ordinary non-force push of Phase 19 interaction polish to `main` on
`bright-builds-llc/liquidfun-rs`. Workflow identity rechecked before the
push: remote `git@github.com:bright-builds-llc/liquidfun-rs.git`, branch
`main`, workflow file `.github/workflows/pages.yml`, workflow name `Pages`.
No release tag and no npm or crates publication. Playwright was not added
to the Pages workflow.

page_url: https://bright-builds-llc.github.io/liquidfun-rs/
source_sha: d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e
workflow_run_url: https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/35415816988
wasm_asset_url: https://bright-builds-llc.github.io/liquidfun-rs/assets/liquidfun_wasm_bg-ILl9My4C.wasm
checked_at: 2026-09-19T02:34:24Z

## Deploy identity

- `origin/main` at check time: `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`
- `source_sha` is 40 lowercase hex and is not
  `50a15562b356ed941266eedddc636df3f76e7e7e`
- Pages run `35415816988` event: `push` on `main`
- `build-site` job `105824154862`: success in 1m52s
- `deploy-pages` job `105824434619`: success in 7s
- GitHub Pages deployment `6536197830` state: `success`
- Deployment `environment_url`: `https://bright-builds-llc.github.io/liquidfun-rs/`
- Live origin matches the expected GitHub Pages project URL.
- Footer provenance on the live page shows commit `d3d8688dabba` and
  build `35415816988`.

## Live asset and hash-route checks

- `GET https://bright-builds-llc.github.io/liquidfun-rs/` returned HTTP 200
  HTML containing `liquidfun-rs playground` and `/liquidfun-rs/assets/`.
- `GET https://bright-builds-llc.github.io/liquidfun-rs/assets/index-vHkA7iiG.js`
  contains `/liquidfun-rs/assets/liquidfun_wasm_bg-ILl9My4C.wasm`.
- `GET https://bright-builds-llc.github.io/liquidfun-rs/assets/liquidfun_wasm_bg-ILl9My4C.wasm`
  returned HTTP 200, `Content-Type: application/wasm` (not `text/html`),
  body length 775572, and WASM magic `\0asm`.
- No-cache GET of each gallery hash still served the app shell
  (`<title>liquidfun-rs playground</title>` and `/liquidfun-rs/assets/`
  script/link tags):
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break`
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/fountain`
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/float-or-sink`
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/color-mixer`
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/jelly-drop`
  - `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/water-wheel`
- Headed Chromium open of
  `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break`
  reached status `Playing`, heading `Dam Break`, title
  `Dam Break · liquidfun-rs playground`, and `data-step-index=1`.
- Headed Chromium open of
  `https://bright-builds-llc.github.io/liquidfun-rs/#/scene/water-wheel`
  reached status `Playing`, heading `Water Wheel`, title
  `Water Wheel · liquidfun-rs playground`, `data-step-index=1`, and an
  enabled Pause control. This proves hosted JS/WASM execute for at least
  two scenes, not only the app-shell HTML.
- These checks do not claim Firefox or Safari coverage.

Authority: standing 2026-09-13 authorization in AGENTS.md for an ordinary
non-force push to `main`. Package publication remains unauthorized.
