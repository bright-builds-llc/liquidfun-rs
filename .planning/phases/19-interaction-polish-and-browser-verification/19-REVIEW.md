---
phase: 19-interaction-polish-and-browser-verification
plan: "07"
reviewed: 2026-09-19T02:41:54Z
independent_reviewed: 2026-09-19T02:41:54Z
depth: deep
hosted_source_sha: d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e
host_evidence_docs_commit: df7b46c09d4527245795232f7314ca064716055c
local_head: e8ec966ccc02b261cc2818d1e0f759c85c51bc49
origin_main_at_review: e8ec966ccc02b261cc2818d1e0f759c85c51bc49
files_reviewed: 16
files_reviewed_list:
  - web/src/render/camera.ts
  - web/src/input/pointer.ts
  - web/src/input/canvas-pointer.ts
  - web/src/physics/session.ts
  - crates/liquidfun-wasm/src/session.rs
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/dam_break.rs
  - crates/liquidfun-wasm/src/scene/fountain.rs
  - crates/liquidfun-wasm/src/scene/float_or_sink.rs
  - crates/liquidfun-wasm/src/scene/color_mixer.rs
  - crates/liquidfun-wasm/src/scene/jelly_drop.rs
  - crates/liquidfun-wasm/src/scene/water_wheel.rs
  - web/e2e/player.spec.ts
  - web/e2e/player-helpers.ts
  - target/web-build/web-build.log
  - .planning/phases/19-interaction-polish-and-browser-verification/19-HOST-EVIDENCE.md
findings:
  critical: 0
  warning: 1
  info: 2
  total: 3
status: issues_found
decision: APPROVED
review_digest: a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef
reviewer_identity: 913947b5-d495-4432-969e-85ce94d31c48
reviewer_invocation_id: 994f3f64-d31c-4782-9a79-fac49258e7f3
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
---

# Phase 19 Independent AI Review

## Decision

**APPROVED.**

There are no Critical findings. WEBTEST-01 hosted facts, the six-scene
pointer mappings, and the independently recomputed digest all match the
fixed manifest. One Warning records a paused-canvas world/display gap that
does not break the Playing-path smoke or live Pages checks. Two Info items
record a leftover construction fail-open and a second-finger capture steal.
This review does not mark requirement completion in planning state files
and grants no package publication, tag, or release authority.

This is independent AI review, not human approval. Passing
`just web-player-smoke` is not this acknowledgment.

This document replaces the earlier `19-REVIEW.md` that named reviewer
`b19fb890-110f-4640-b0f2-a51ae36c2f38`. That identity is the Phase 19-07
implementing executor Task id and is not a valid independent
acknowledgment. Those identities are not this reviewer.

## Reviewer and exact scope

- `reviewer_identity`: Cursor independent AI review subagent
  (`gsd-code-reviewer`), Task/agent-store identity
  `913947b5-d495-4432-969e-85ce94d31c48`
- `reviewer_invocation_id`: `994f3f64-d31c-4782-9a79-fac49258e7f3`
- `reviewer_disclosure`: AI reviewer, not a human
- `model`: Cursor Grok 4.6
- `implementing_or_fixing_executor`: no
- `reviewed_at_utc`: `2026-09-19T02:41:54Z`
- `hosted_source_sha`: `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`
- `host_evidence_docs_commit`: `df7b46c09d4527245795232f7314ca064716055c`
- `local_head_at_review`: `e8ec966ccc02b261cc2818d1e0f759c85c51bc49`
- `origin/main_at_review`: `e8ec966ccc02b261cc2818d1e0f759c85c51bc49`

This reviewer is not implementing executor
`b19fb890-110f-4640-b0f2-a51ae36c2f38` and did not resume that agent or
orchestrator store `42be9631-eac3-48e3-81be-cadf14d02795`. The
implementing agent does not acknowledge or approve this digest.

The reviewer independently inspected the complete relevant Phase 19
pointer/camera/input/session/six-scene pointer mappings, player spec and
helpers, ignored smoke log, and `19-HOST-EVIDENCE.md`. Cross-file support
inspected but not hashed into the digest includes `web/src/App.tsx`,
`web/src/render/canvas.ts`, `web/src/app.css`, `web/src/catalog/scenes.ts`,
`crates/liquidfun-wasm/src/lib.rs`, `.github/workflows/pages.yml`,
`README.md`, and `TESTING.md`. Material guidance was `AGENTS.md`
Independent review (2026-09-16 owner policy), `PROJECT-SCOPE.md` hobby
scope, `19-CONTEXT.md` D-13 and D-14, `19-07-PLAN.md` threats
T-19-07-01 through T-19-07-06, and `19-REVIEW-MANIFEST.md`. Passing
`just web-player-smoke` or other automation is supporting context only
and is not this acknowledgment.

## Findings

No Critical findings.

### WR-01: Paused canvas pointer mutates the world without redrawing

**File:** `web/src/input/canvas-pointer.ts:66-74` and `web/src/App.tsx:368-369`
**Issue:** `forwardScenePointer` accepts `playing` and `paused`. Pause,
reset, hide, and abandon correctly `cancel()` in-flight gestures, but a
new gesture after Pause still calls `pointerAction` and does not draw.
`presentOwnedFrame` always `nextFrame()`s, so there is no capture-only
redraw. A paused Dam Break drag or Float-or-Sink drop therefore changes
engine state while the canvas stays on the last playing frame until Play
or a labeled action. Labeled actions while paused do call
`presentOwnedFrame`.
**Fix:** Either ignore pointer kinds while `viewKind !== "playing"`, or
after an accepted paused pointer draw the current world without stepping
(add a capture-only present) so the canvas matches the mutation.

### IN-01: Jelly Drop construction still defaults unknown shape/softness tokens

**File:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs:77-82`
**Issue:** `build` still uses `and_then(JellyShape::parse)` /
`and_then(Softness::parse)` then `unwrap_or(Circle|Medium)`. Live
`apply_control` rejects unknown tokens. The browser only sends
allowlisted select values, so this is not visitor-facing today.
**Fix:** Fail closed with `SessionError::UnknownControl` when a present
token does not parse, matching Dam Break and Color Mixer.

### IN-02: A second `pointerdown` steals gesture state without canceling the first pointer

**File:** `web/src/input/pointer.ts:103-109`
**Issue:** `reducePointerEvent` always replaces `maybePointerId` on
`pointerdown` and never emits `cancel` for the previous id. D-04 deferred
multi-touch editing; a second finger can leave the first pointer captured
until browser auto-release. Single-pointer Playing paths used by the
smoke suite are unaffected.
**Fix:** If `state.maybePointerId` is already set, emit `cancel` and
release that id before capturing the new pointer.

## Required inspection outcomes

### Digest

The reviewer recomputed the fixed manifest independently from current
file bytes. For each of the 16 listed paths in the listed order, the
reviewer concatenated UTF-8 path bytes, one NUL byte, the lowercase
SHA-256 of the exact file bytes as ASCII hex, and one LF byte, then
SHA-256 hashed that complete concatenation.

All 16 per-file hashes match `19-REVIEW-MANIFEST.md`. The independently
recomputed digest is exactly:

`a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`

### Pointer pipeline and camera

`unprojectPoint` is the inverse of `projectPoint` over the same
`WORLD_BOUNDS` and CSS-pixel camera. `createCamera` rejects non-positive
or non-finite viewports. `cssPointFromClient` rejects non-finite client
or rect samples. `attachCanvasPointer` converts
`getBoundingClientRect()` CSS points through `maybeCamera`, which App
updates from the same CSS bounds in `ResizeObserver` via
`resizeCanvasBackingStore`. Device pixels stay in the backing store
(`setTransform(pixelRatio, …)`); physics receives world `f32`s.

`reducePointerEvent` captures on `pointerdown` and finishes with `up` or
`cancel` on `pointerup`, `pointercancel`, and `lostpointercapture`.
`preventDefault` runs only when capturing. `touch-action: none` is
scoped to `canvas` in `app.css`, not the document. App `cancel()` /
`detach()` run on pause, construction recreate, session dispose, hidden
document, canvas reassignment, abandon, and Solid `onCleanup`.

JS `SceneSession.pointerAction` allowlists kind and finite coordinates
before the generated call. Rust `SessionCore::apply_pointer` parses kind
and rejects non-finite coordinates with `InvalidPointer`. Unknown kinds
return `UnknownControl` without disposing. `ProofSession::pointer_action`
is the wasm-bindgen boundary.

### Six scene pointer mappings

Each scene implements exactly one canvas gesture through
`SceneHooks::apply_pointer`. Physics stay in the engine: no JavaScript
particle writes, no scripted pose track.

- Dam Break: down/move teleports the obstacle inside the basin clamp;
  up applies a wake impulse; cancel leaves the last pose. Labeled
  `drop-obstacle` still teleports.
- Fountain: down/move aims from the nozzle at `(0.0, 0.5)` with a ±τ/4
  clamp; up/cancel leave the last aim. Labeled `aim-angle` still applies
  live.
- Float or Sink: down drops the selected preset at world x and y `6.0`,
  clamped in x; later downs cap at four bodies; move/up/cancel do not
  add a body.
- Color Mixer: down/move store an origin; `on_advance` applies
  per-particle engine forces inside a radius; up/cancel clear leftover
  stir. Colors are not rewritten by the pointer path.
- Jelly Drop: down pokes at the world location; move/up/cancel do not
  apply a second impulse.
- Water Wheel: down/move aim the jet through native emitter velocity;
  up keeps the last aim; cancel restores the default. The wheel remains
  a pinned revolute hub without a motor.

`parse_scene_id` still allowlists only the six lowercase ids.
`parse_pointer_kind` allowlists `down|move|up|cancel`.

### Player spec, helpers, and local smoke

`web/e2e/player.spec.ts` covers production-base `/liquidfun-rs/` open,
pause/play/reset, catalog-or-hash open of all six scenes, unknown-hash
fallback, Dam Break → Fountain without a second WASM fetch, Gravity
Apply setting, hidden-tab max-4, one pointer gesture plus a labeled
control per scene, Dam Break `pointercancel` →
`data-last-pointer-kind=cancel`, resize-then-drag increasing
`data-pointer-accepted`, hidden-tab after a gesture, and a 375px
keyboard-focus plus page-scroll pass. Helpers use hash paths
`#/scene/{id}` for all six ids. Chromium-only config is asserted.
There is no Firefox or Safari claim.

`target/web-build/web-build.log` starts `start player-smoke` and ends
`complete player-smoke` after `just web-player-smoke`, injecting local
`VITE_GIT_SHA=4fb01acb7da9532fb60237103794725c78f20ce7`. That log is
ignored local Chromium evidence, not the Pages deploy. Passing that
smoke is not this acknowledgment.

### HOST-EVIDENCE and live Pages

`19-HOST-EVIDENCE.md` records:

- `page_url:` `https://bright-builds-llc.github.io/liquidfun-rs/`
- `source_sha:` `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`
- `workflow_run_url:` `https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/35415816988`
- `wasm_asset_url:` `https://bright-builds-llc.github.io/liquidfun-rs/assets/liquidfun_wasm_bg-ILl9My4C.wasm`
- `checked_at:` `2026-09-19T02:34:24Z`

`source_sha` is 40 lowercase hex and is not
`50a15562b356ed941266eedddc636df3f76e7e7e`. All six `#/scene/` hashes
are listed: `dam-break`, `fountain`, `float-or-sink`, `color-mixer`,
`jelly-drop`, `water-wheel`.

This reviewer independently rechecked:

- `GET` the live origin: HTTP 200 HTML containing
  `liquidfun-rs playground`, `<title>liquidfun-rs playground</title>`,
  `/liquidfun-rs/assets/`, and `index-vHkA7iiG.js`.
- `GET` `index-vHkA7iiG.js`: contains
  `/liquidfun-rs/assets/liquidfun_wasm_bg-ILl9My4C.wasm`,
  `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`, and build `35415816988`.
- `GET` the recorded WASM URL: HTTP 200, `Content-Type: application/wasm`,
  body length `775572`, magic `\0asm`.
- No-cache GET of all six `#/scene/{id}` hashes: HTTP 200 app shell
  containing `liquidfun-rs playground` and `/liquidfun-rs/assets/`.
- Workflow run `35415816988`: `push` on `main`, `head_sha`
  `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`, `conclusion` success.
- `.github/workflows/pages.yml` contains no `playwright` job. Deploy
  uses job-scoped `pages: write` and `id-token: write`; no PAT.

Hash fragments are client-side; the six scene URLs share that app shell.
Headed Chromium `Playing` checks for Dam Break and Water Wheel remain
recorded evidence in `19-HOST-EVIDENCE.md`. This reviewer did not treat
those headed opens or passing `just web-player-smoke` as this
acknowledgment.

README and TESTING cite the same Pages origin and source
`d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`. They keep local
`just web-player-smoke` as the ordinary WEBTEST-01 gate and do not claim
Firefox, Safari, crate publication, or complete LiquidFun parity.

### Pages, publication, and hobby scope

Hobby scope in `PROJECT-SCOPE.md` applies: local proof plus truthful
hosted URL/revision, not Linux qualification or a browser matrix. This
review grants no package publication, tag, release, or further Pages
deploy authority.

## Fixed review manifest

The 16 manifest entries and hashes in `19-REVIEW-MANIFEST.md` were
independently rehashed from current file bytes. Every hash matches. The
digest is
`a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`.

Manifest entry count: `16`.

## Threat review

- **T-19-07-01 mitigated:** fresh `19-HOST-EVIDENCE.md` bound to
  `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`, not `50a1556`.
- **T-19-07-02 mitigated:** independently confirmed
  `Content-Type: application/wasm` and `\0asm`.
- **T-19-07-03 mitigated:** all six allowlisted hashes are recorded;
  unknown hash remains `Scene not found` in the player spec.
- **T-19-07-04 mitigated:** this separate AI reviewer binds approval to
  the exact digest below. Passing `just web-player-smoke` alone is not
  this acknowledgment. The prior `19-REVIEW.md` that used implementing
  executor identity `b19fb890-110f-4640-b0f2-a51ae36c2f38` is not this
  acknowledgment.
- **T-19-07-05 mitigated:** recorded deploy is an ordinary `push` on
  `main`; existing Pages job permissions; no PAT in `pages.yml`.
- **T-19-07-06 accepted:** publication remains unauthorized; this review
  creates no tag and publishes no crate.

## Digest acknowledgment

`review_digest`:
`a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`

The independent result exactly matched all 16 current file entries and
the required digest.

> I, the Cursor independent AI review subagent identified above as
> Task/agent identity `913947b5-d495-4432-969e-85ce94d31c48`,
> independently inspected the listed Phase 19 implementation-and-evidence
> bytes and explicitly acknowledge exact digest
> `a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`
> as the content I inspected and approve. I am an AI reviewer, not a
> human. I am not the implementing or fixing executor.

Any implementation, player, evidence, or listed-test byte change
invalidates this acknowledgment and requires a new fixed manifest,
digest, and independent review.

This review grants no package publication, tag, release, Pages deploy, or
other release authority. Passing `just web-player-smoke` alone is not this
acknowledgment.

## GSD Source Code Review

**Reviewed:** 2026-09-19T02:41:54Z
**Depth:** deep
**Files Reviewed:** 16
**Status:** issues_found

### Summary

Deep independent review confirmed the shared CSS-bound camera unproject,
Pointer Events capture/cancel pipeline, WASM `pointer_action` allowlist,
and one native gesture per scene. Live Pages at
`https://bright-builds-llc.github.io/liquidfun-rs/` serves the recorded
WASM as `application/wasm` with `\0asm` from workflow `35415816988` at
source `d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`. The independently
recomputed digest is
`a297f33179b980d55d4ba64378f93edaf842ed5990b43a6eda6bbbba5f9c6fef`.
WR-01 notes paused pointer mutations without a redraw. IN-01 and IN-02
are leftover fail-open construction and second-finger capture steal.

The fixed manifest, independent AI review, approval, and exact digest
acknowledgment above are the review decision. This is not human approval.

### Warnings

#### WR-01: Paused canvas pointer mutates the world without redrawing

**File:** `web/src/input/canvas-pointer.ts:66-74`
**Issue:** Pointer input is accepted while paused and does not present a
frame, so the canvas can disagree with engine state until Play.
**Fix:** Ignore pointer while paused, or draw without stepping after a
paused pointer.

### Info

#### IN-01: Jelly Drop unknown construction tokens default silently

**File:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs:77-82`
**Issue:** Unknown shape/softness presets fall back to Circle/Medium
instead of `UnknownControl`.
**Fix:** Fail closed like Dam Break and Color Mixer.

#### IN-02: Second pointerdown steals without canceling the first pointer

**File:** `web/src/input/pointer.ts:103-109`
**Issue:** A new `pointerdown` overwrites `maybePointerId` without
emitting `cancel` for the previous id.
**Fix:** Cancel and release the prior captured pointer first.

---

_Reviewed: 2026-09-19T02:41:54Z_
_Reviewer: Cursor AI (gsd-code-reviewer), Task/agent identity 913947b5-d495-4432-969e-85ce94d31c48_
_Depth: deep_

GSD_SOURCE_CODE_REVIEW_COMPLETE
