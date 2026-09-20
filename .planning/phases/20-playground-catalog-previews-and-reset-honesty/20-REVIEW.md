---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "06"
draft: task-1-smoke-evidence
local_head: d33d3e2d04a6e18c2c951ae21a32775228190ba3
smoke_command: just web-player-smoke
smoke_exit: 0
smoke_tests_passed: 37
vite_git_sha: d33d3e2d04a6e18c2c951ae21a32775228190ba3
vite_build_id: 2026-09-20T19:12:34.860Z
digest: 7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f
# reviewer_identity, reviewed_at, reviewer_disclosure, implementing_or_fixing_executor,
# and decision are left for Task 2 independent gsd-code-reviewer.
---

# Phase 20 smoke evidence (Task 1 draft)

This file is a Task 1 draft written by the implementing executor. It is **not**
an independent review acknowledgment. Passing `just web-player-smoke` is not
an acknowledgment. The implementing executor does not approve this work.

A separate identified `gsd-code-reviewer` must inspect the complete Phase 20
diff and this smoke evidence, independently recompute the digest, and replace
this draft with the bound acknowledgment.

## Smoke

Command (PHASE16_CLOSURE_ATTEMPT_DIR unset):

```bash
just web-player-smoke
```

Exit: `0`

Playwright: `37 passed` (Chromium). `test:player` ran
`e2e/player.spec.ts`, `e2e/demo-media-clock.spec.ts`, `e2e/shell.spec.ts`,
and `e2e/reset-honesty.spec.ts`.

Ignored log `target/web-build/web-build.log` starts with
`start player-smoke` and ends with `complete player-smoke`. Injected
`VITE_GIT_SHA=d33d3e2d04a6e18c2c951ae21a32775228190ba3`.

Allowlist check:

```bash
rg -n "e2e/reset-honesty.spec.ts|e2e/shell.spec.ts" web/package.json
```

Matched both files on `web/package.json` line 15.

Pages workflow:

```bash
rg -n "playwright" .github/workflows/pages.yml
```

No matches. No Playwright job was added. No git tag was created. No push,
package publish, or HOST-EVIDENCE.

## Digest command

Concatenated exact file bytes in the 20-06-PLAN.md listed order, then SHA-256.
All 13 paths exist on disk. None skipped.

```bash
cat \
  web/src/catalog/previews.tsx \
  web/src/components/DemoNavigation.tsx \
  web/src/app.css \
  web/src/App.tsx \
  web/src/components/PlayerSceneChrome.tsx \
  web/src/player/runtime.ts \
  web/src/player/viewport.ts \
  web/tests/runtime.test.ts \
  web/tests/controls.test.ts \
  web/e2e/shell.spec.ts \
  web/e2e/reset-honesty.spec.ts \
  web/package.json \
  .planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md \
  | shasum -a 256
```

Result (lowercase 64-hex):

`7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`

## Per-file SHA-256 (supporting, not the digest)

- `web/src/catalog/previews.tsx` `8a2e755a5f1d69009f70a6f5683525613beacba559382e309afa695e79393545`
- `web/src/components/DemoNavigation.tsx` `041cb21e017b1ad4866fef78865dac1862a5908d163e67293c0c667d2e424d40`
- `web/src/app.css` `ce529928f26b94cd43d9caa9985dd7312ac65656efe5d7473a3533e6e717d2d9`
- `web/src/App.tsx` `d95e393f3f91501e3ee9524422a6d637c29b1aa4aa08545aba42328e127e9f41`
- `web/src/components/PlayerSceneChrome.tsx` `1b07f253addc239f4c72d996b68ef1af2ae90ccb464e322344a2b36478c5324c`
- `web/src/player/runtime.ts` `d1f1182de8e88a3cc886c23ca38dea8c384d50860febdb1f0b7bde8687d1dfab`
- `web/src/player/viewport.ts` `279f0e916ec070c11629bb16328508aa7cbcce4a1d8a30f865fce16abd26e285`
- `web/tests/runtime.test.ts` `37a975beaad27843ab366fc508313749eea262ac9eed62cecfbb69ac5096a095`
- `web/tests/controls.test.ts` `d61c78f1320dbd85be24e5e0e61e946628662b6673eb275ec31348763c95774d`
- `web/e2e/shell.spec.ts` `9752ffa0bf1d799ff9eda29824bf83396b267f2470e93613cc0985f466002262`
- `web/e2e/reset-honesty.spec.ts` `5021e65e53d534de107fce908e556528936af5d76512200b1bf5b90f6e84ace8`
- `web/package.json` `d18baa1f372cae139f4ae1a13d4dac44f346acb752bca57a6921edb8568d2d87`
- `.planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md` `b571e53fcf8634b2fb5f088f0bb20464f3ff6ee9a7fe30113f588966af2e945a`

Manifest entry count: `13`.

## WEB-01 / WEB-03 criteria for the independent reviewer

- **WEB-01:** A visitor can browse six demo cards with names, short descriptions
  and previews, then open a selected demo in a shared player. Phase 20 restored
  compact static SVG previews inside Kobalte `DemoNavigation` (sidebar + drawer),
  captioned `Static preview`, without restoring `.catalog-card`.
- **WEB-03:** A visitor can play, pause and reset the selected demo to its
  documented initial state. Phase 20 remounts keyed `SceneControls` and clears
  the construction bag on Reset so live and construction selects show
  `DEFAULT_PRESET_VALUES`. Play/pause must not remount live selects.

This draft grants no package publication, tag, release, or Pages deploy
authority. Do not claim Firefox or Safari coverage.
