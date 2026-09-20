---
phase: 20-playground-catalog-previews-and-reset-honesty
plan: "06"
reviewed: 2026-09-20T19:25:04Z
independent_reviewed: 2026-09-20T19:25:04Z
reviewed_at: 2026-09-20T19:25:04Z
depth: deep
local_head: 9c7d3223db675e76aa3e6b5555ad6c5070083cdb
phase_implementation_head: d33d3e2d04a6e18c2c951ae21a32775228190ba3
origin_main_at_review: a0f93557744c2631097426df33d76ac43020557e
files_reviewed: 13
files_reviewed_list:
  - web/src/catalog/previews.tsx
  - web/src/components/DemoNavigation.tsx
  - web/src/app.css
  - web/src/App.tsx
  - web/src/components/PlayerSceneChrome.tsx
  - web/src/player/runtime.ts
  - web/src/player/viewport.ts
  - web/tests/runtime.test.ts
  - web/tests/controls.test.ts
  - web/e2e/shell.spec.ts
  - web/e2e/reset-honesty.spec.ts
  - web/package.json
  - .planning/phases/20-playground-catalog-previews-and-reset-honesty/20-CONTEXT.md
findings:
  critical: 0
  warning: 0
  info: 0
  total: 0
status: clean
decision: APPROVED
digest: 7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f
review_digest: 7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f
reviewer_identity: d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a
reviewer_invocation_id: 89041aff-59bb-4b6f-a48f-38421c86532f
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
---

# Phase 20 Independent AI Review

## Decision

**APPROVED.**

There are no Critical, Warning, or Info findings. WEB-01 compact static
previews inside `DemoNavigation`, WEB-03 Reset label honesty, the
independently recomputed digest, and the inspected Chromium player-smoke
wrapper all match the Phase 20 contract. This review does not mark
requirement completion in planning state files and grants no package
publication, tag, release, or GitHub Pages deploy authority.

This is independent AI review, not human approval. Passing
`just web-player-smoke` is not this acknowledgment.

This document replaces the Task 1 smoke-evidence draft in
`20-REVIEW.md` that left reviewer identity and decision empty. That draft
was written by the implementing executor and is not a valid independent
acknowledgment. The implementing agent does not approve this digest.
The prior draft acknowledgment named parent conversation_id
`baf06cf6-34df-472a-973d-9449b6317970`. That identity is not this
reviewer.

## Reviewer and exact scope

- `reviewer_identity`: Cursor independent AI review subagent
  (`gsd-code-reviewer`), Task/agent-store identity
  `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a`
- `reviewer_invocation_id`: `89041aff-59bb-4b6f-a48f-38421c86532f`
- `reviewer_disclosure`: AI reviewer, not a human
- `model`: Cursor Grok 4.6
- `implementing_or_fixing_executor`: no
- `reviewed_at_utc`: `2026-09-20T19:25:04Z`
- `phase_implementation_head`: `d33d3e2d04a6e18c2c951ae21a32775228190ba3`
- `local_head_at_review`: `9c7d3223db675e76aa3e6b5555ad6c5070083cdb`
- `origin/main_at_review`: `a0f93557744c2631097426df33d76ac43020557e`

This reviewer is not implementing executor
`34df4636-2d9d-4686-9139-f3cf3c05a52b` and did not resume that agent or
its parent store. The prior draft acknowledgment named parent
conversation_id `baf06cf6-34df-472a-973d-9449b6317970`; that identity
is not this reviewer. The implementing agent does not acknowledge or
approve this digest.

The reviewer independently inspected the complete relevant Phase 20
catalog-preview and Reset-honesty implementation (plans 20-01 through
20-05) ending at `d33d3e2`, plus the Task 1 smoke-evidence commit
`9c7d322` that does not change the 13 hashed source bytes. Cross-file
support inspected but not hashed into the digest includes
`web/src/components/SceneControls.tsx`,
`web/src/components/scene-controls.ts`,
`web/src/components/PlaygroundShell.tsx`, `web/e2e/player.spec.ts`,
`web/e2e/player-helpers.ts`, `web/e2e/demo-media-clock.spec.ts`,
`web/src/catalog/scenes.ts`, `web/src/player/navigation.ts`,
`.github/workflows/pages.yml`, `target/web-build/web-build.log`, and
`web/test-results/.last-run.json`. Material guidance was `AGENTS.md`
Independent review (2026-09-16 owner policy), `PROJECT-SCOPE.md` hobby
scope, `20-CONTEXT.md` D-01 through D-11, and `20-06-PLAN.md` threats
T-20-06-01 through T-20-06-06. Passing `just web-player-smoke` or other
automation is supporting context only and is not this acknowledgment.

## Findings

No Critical findings. No Warning findings. No Info findings.

## Required inspection outcomes

### Digest

The reviewer recomputed the digest independently from current file bytes
using this exact command (concatenated file bytes in this listed order,
then SHA-256):

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

All 13 per-file SHA-256 hashes match the implementer-recorded values.
`git diff d33d3e2 --` on those 13 paths is empty, so HEAD `9c7d322` source
bytes match phase implementation HEAD `d33d3e2`. The independently
recomputed digest is exactly:

`7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`

Per-file SHA-256 (supporting, not the digest):

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

### WEB-01 catalog previews

`ScenePreview` is token-only inline SVG (`viewBox="0 0 160 90"`) for the
six allowlisted ids. It does not import WASM, Canvas, or
`docs/assets/demos/*.webp`. Each illustration is still (no CSS animation,
no SMIL). `DemoNavigation` places the SVG in `.demo-nav-preview`, then a
visible `Static preview` caption, then title and description, inside the
existing hash `<a href="#/scene/{id}">`. Sidebar and mobile drawer share
that component through `PlaygroundShell`. There is no `.catalog-card`,
`.catalog-grid`, or CatalogNav in `web/src`. `app.css` adds compact
`.demo-nav-preview` / `.demo-nav-preview-caption` rules (`max-block-size:
72px`, `aspect-ratio: 16 / 9`) without restoring a three-column card grid.

`web/e2e/shell.spec.ts` still asserts `.catalog-card` count is `0` on
desktop, sidebar preview, and mobile-drawer paths. Desktop proof is
scoped to `.demo-sidebar` and expects six exact `Static preview` captions
plus six `svg[aria-hidden='true']`. Mobile proof opens the `Demos` dialog
and checks one visible caption plus one visible `svg[aria-hidden='true']`,
then asserts no horizontal overflow.

### WEB-03 Reset honesty

`recreateScene` assigns `constructionValues = {}`, then bumps
`resetGeneration`, then `startScene`. Play and pause only change
`view()`; they do not clear the bag or bump the remount key.
`PlayerSceneChrome` remounts `SceneControls` through Solid `Show keyed`
on `sceneControlsIdentity(sceneId, resetGeneration)`. `SceneCredits`
stay mounted across that remount. `PresetControl` still initializes
`pendingValue` once from `initialPresetValue` /
`DEFAULT_PRESET_VALUES`, so the remount is what makes live and
construction selects honest after Reset.

`constructionEntriesForScene` emits no `applyControl` rows for an empty
bag, so Reset rebuilds the native documented initial world rather than
replaying last construction presets. Hash scene-switch and `abandonScene`
also clear the bag. Retry shares `recreateScene`.

`web/e2e/reset-honesty.spec.ts` proves live `toHaveValue` recovery
(Fountain emission-rate high→medium, Float or Sink body cork→wood, Color
Mixer stir-speed fast→slow, Water Wheel jet-strength strong→medium),
construction recovery after Apply setting (Dam Break water-amount
large→medium, Color Mixer mix-strength gentle→strong), and the play/pause
negative (Fountain emission-rate stays high).
`web/package.json` `test:player` lists `e2e/reset-honesty.spec.ts`.

### Player smoke

The reviewer inspected `target/web-build/web-build.log`. It starts
`start player-smoke` and ends `complete player-smoke`. Injected
`VITE_GIT_SHA=d33d3e2d04a6e18c2c951ae21a32775228190ba3` and
`VITE_BUILD_ID=2026-09-20T19:12:34.860Z`. `PHASE16_CLOSURE_ATTEMPT_DIR`
is not referenced. `web/test-results/.last-run.json` records
`"status": "passed"` and `"failedTests": []`. The wrapper log does not
embed Playwright's per-test count; the Task 1 draft reported 37 Chromium
passes. That automation result is supporting context only and is not this
acknowledgment.

### Pages, publication, and hobby scope

`rg -n "playwright" .github/workflows/pages.yml` is empty. Deploy remains
job-scoped `pages: write` and `id-token: write` with no PAT. `git tag -l`
is empty; this review creates no tag. `web/package.json` stays
`"private": true`. This review claims no Firefox/Safari/WebKit matrix,
no Linux qualification, no Dam Break C++ timing, no new GitHub Pages URL,
and no package publication. Hobby scope in `PROJECT-SCOPE.md` applies.

## Threat review

- **T-20-06-01 mitigated:** this separate AI reviewer
  `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a` binds approval to the exact
  digest below. The implementing executor
  `34df4636-2d9d-4686-9139-f3cf3c05a52b` did not write this
  acknowledgment. Parent conversation_id
  `baf06cf6-34df-472a-973d-9449b6317970` is not this reviewer.
- **T-20-06-02 mitigated:** player-smoke was re-run against implementation
  HEAD `d33d3e2`; hashed source bytes at review HEAD `9c7d322` are
  identical.
- **T-20-06-03 mitigated:** this document discloses AI reviewer, not a
  human, and is not human approval.
- **T-20-06-04 accepted:** publication remains unauthorized; this review
  creates no tag and publishes no crate.
- **T-20-06-05 accepted:** this plan does not deploy; no PAT and no force
  push.
- **T-20-06-06 mitigated:** `test:player` still lists
  `e2e/reset-honesty.spec.ts` and `e2e/shell.spec.ts`.

## Digest acknowledgment

`review_digest`:
`7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`

The independent result exactly matched all 13 current file entries and
the required digest.

> I, the Cursor independent AI review subagent identified above as
> Task/agent identity `d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a`,
> independently inspected the listed Phase 20 implementation-and-evidence
> bytes and explicitly acknowledge exact digest
> `7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`
> as the content I inspected and approve. I am an AI reviewer, not a
> human. I am not the implementing or fixing executor.

Any implementation, catalog, player, evidence, or listed-test byte change
invalidates this acknowledgment and requires a new digest and independent
review.

This review grants no package publication, tag, release, Pages deploy, or
other release authority. Passing `just web-player-smoke` alone is not this
acknowledgment.

## GSD Source Code Review

**Reviewed:** 2026-09-20T19:25:04Z
**Depth:** deep
**Files Reviewed:** 13
**Status:** clean

### Summary

Deep independent review confirmed compact static SVG `ScenePreview`
entries inside existing `DemoNavigation` (sidebar and drawer), captioned
`Static preview`, with `.catalog-card` still forbidden; Reset clearing
`constructionValues` and remounting keyed `SceneControls` so live and
construction selects show `DEFAULT_PRESET_VALUES`; play/pause leaving
those selects mounted; Chromium `shell.spec.ts` and
`reset-honesty.spec.ts` on `test:player`; and no Playwright job in
`pages.yml`. The independently recomputed digest is
`7e94fe28c89308f16cb00a0e30543bf447b9cc7ac468e1f6b32c50aa4096535f`.
No source-code issues were found.

All reviewed files meet the Phase 20 quality bar for WEB-01 and WEB-03.
This is not human approval.

---

_Reviewed: 2026-09-20T19:25:04Z_
_Reviewer: Cursor AI (gsd-code-reviewer), Task/agent identity d82e8f2b-d7a5-4ad9-af0c-3a60f618cc6a_
_Depth: deep_

GSD_SOURCE_CODE_REVIEW_COMPLETE
