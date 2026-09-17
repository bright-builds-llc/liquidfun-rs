---
phase: 17-shared-player-and-early-pages-delivery
plan: "08"
requirement: HOST-03
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T12:25:24Z
---

# Phase 17 HOST-03 live Pages evidence

Ordinary non-force push of Phase 17 player and docs work to `main` on
`bright-builds-llc/liquidfun-rs`. Workflow identity rechecked before the
push: remote `git@github.com:bright-builds-llc/liquidfun-rs.git`, branch
`main`, workflow file `.github/workflows/pages.yml`, workflow name `Pages`.
No release tag and no npm or crates publication.

page_url: https://bright-builds-llc.github.io/liquidfun-rs/
source_sha: 50a15562b356ed941266eedddc636df3f76e7e7e
workflow_run_url: https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/35220721701
wasm_asset_url: https://bright-builds-llc.github.io/liquidfun-rs/assets/liquidfun_wasm_bg-BWMkHUXc.wasm
checked_at: 2026-09-17T12:25:24Z

## Deploy identity

- `origin/main` at check time: `50a15562b356ed941266eedddc636df3f76e7e7e`
- Pages run `35220721701` event: `push` on `main`
- `build-site` job `105199914174`: success in 2m13s
- `deploy-pages` job `105200623894`: success in 9s
- GitHub Pages deployment `6502560699` state: `success`
- Deployment `environment_url`: `https://bright-builds-llc.github.io/liquidfun-rs/`
- Live origin matches the expected GitHub Pages project URL.

## Live asset and hash-route checks

- `GET https://bright-builds-llc.github.io/liquidfun-rs/` returned HTTP 200
  HTML containing `liquidfun-rs playground`.
- `GET https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break`
  returned the same app-shell HTML containing `liquidfun-rs playground`.
- A no-cache GET of that hash URL still served the app shell
  (`<title>liquidfun-rs playground</title>` and `/liquidfun-rs/assets/`
  script/link tags).
- `GET https://bright-builds-llc.github.io/liquidfun-rs/assets/liquidfun_wasm_bg-BWMkHUXc.wasm`
  returned HTTP 200, `Content-Type: application/wasm` (not `text/html`),
  body length 674952, and WASM magic `\0asm`.

Authority: standing 2026-09-13 authorization in AGENTS.md for an ordinary
non-force push to `main`. Package publication remains unauthorized.
