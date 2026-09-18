---
phase: 18-six-native-physics-demos
plan: "10"
reviewed: 2026-09-18T05:39:01Z
independent_reviewed: 2026-09-18T05:39:01Z
depth: deep
candidate: 1a69ff3bc1b03039818bd621983cfbe394fdd244
local_head: 1a69ff3bc1b03039818bd621983cfbe394fdd244
origin_main_at_review: dc23318ce4a460d46022d749db5f24386d66c92b
files_reviewed: 18
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene/dam_break.rs
  - crates/liquidfun-wasm/src/scene/fountain.rs
  - crates/liquidfun-wasm/src/scene/float_or_sink.rs
  - crates/liquidfun-wasm/src/scene/color_mixer.rs
  - crates/liquidfun-wasm/src/scene/jelly_drop.rs
  - crates/liquidfun-wasm/src/scene/water_wheel.rs
  - crates/liquidfun-wasm/src/scene.rs
  - web/src/catalog/scenes.ts
  - web/src/components/CatalogNav.tsx
  - web/src/components/SceneControls.tsx
  - web/src/components/scene-controls.ts
  - web/src/components/SceneCredits.tsx
  - web/src/components/scene-credits.ts
  - web/src/App.tsx
  - web/e2e/player.spec.ts
  - target/web-build/web-build.log
  - README.md
  - TESTING.md
findings:
  critical: 0
  warning: 0
  info: 1
  total: 1
status: issues_found
decision: APPROVED
review_digest: 22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c
reviewer_identity: eddc4c2f-39ef-460d-99ad-782c4871b074
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
---

# Phase 18 Independent AI Review

## Decision

**APPROVED.**

There are no unresolved Critical or Warning findings and zero high-severity
findings. One Info item records a labeled Float or Sink default mismatch that
does not fake physics, break the six-id allowlist, or invalidate local smoke
evidence. This review does not mark requirement completion in planning state
files.

## Reviewer and exact scope

- `reviewer_identity`: Cursor independent AI review subagent
  (`gsd-code-reviewer`), agent-store identity
  `eddc4c2f-39ef-460d-99ad-782c4871b074`
- `reviewer_disclosure`: AI reviewer, not a human
- `model`: Cursor Grok 4.6
- `implementing_or_fixing_executor`: no
- `reviewed_at_utc`: `2026-09-18T05:39:01Z`
- `task_1_commit`: `1a69ff3bc1b03039818bd621983cfbe394fdd244`
- `local_head_at_review`: `1a69ff3bc1b03039818bd621983cfbe394fdd244`
- `origin/main_at_review`: `dc23318ce4a460d46022d749db5f24386d66c92b`

This reviewer is not implementing executor `gsd-executor` on the main working
tree and did not resume that agent or parent store
`69e3a7c4-0536-4c8b-8b83-fd5bb2f407e4`. The implementing agent does not
acknowledge or approve this digest.

The reviewer independently inspected the complete relevant Phase 18 six-scene
diff and evidence: the six native scene modules, factory `scene.rs`, catalog,
controls, credits, `App.tsx` session wiring, `player.spec.ts`, ignored smoke
log, and README/TESTING honesty. Material guidance was `AGENTS.md` Independent
review (2026-09-16 owner policy), `PROJECT-SCOPE.md` hobby scope, `18-CONTEXT.md`
D-16 and D-17, `18-10-PLAN.md`, and `18-UI-SPEC.md`. Passing
`just web-player-smoke` or other automation is supporting context only and is
not this acknowledgment.

## Findings and resolutions

No Critical or Warning findings.

### IN-01: Float or Sink UI default is cork; constructed world default is wood

**File:** `web/src/components/scene-controls.ts:13` and
`crates/liquidfun-wasm/src/scene/float_or_sink.rs:128`
**Issue:** `DEFAULT_PRESET_VALUES.body` is `cork`, so the first-load Body
select shows Cork. `FloatOrSinkHooks` constructs `BodyPreset::Wood`. A first
`Drop body` without changing the select therefore drops wood (density `0.6`)
while the labeled value is Cork. Live `apply_control("body", …)` still
allowlists cork/wood/stone and applies without recreate.
**Fix:** Align the labeled default with the constructed default, preferably
`wood` in `DEFAULT_PRESET_VALUES` to match 18-03 discretion, or construct
`BodyPreset::Cork` if the UI default is intentional.

## Required inspection outcomes

### Digest

The reviewer recomputed the fixed manifest independently. For each of the 16
listed paths in the listed order, the reviewer concatenated UTF-8 path bytes,
one NUL byte, the lowercase SHA-256 of exact file bytes as ASCII hex, and one
LF byte, then SHA-256 hashed that complete concatenation.

Every per-file hash matched `18-REVIEW-MANIFEST.md`. The independently
recomputed digest is exactly:

`22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c`

### Native scenes and factory

Each of the six modules constructs a real `liquidfun::World` with gravity,
fixtures or particle groups, and allowlisted controls. There is no scripted
pose track, fake buoyancy, or motor-driven Water Wheel.

- Dam Break builds the documented basin, water grid, and dynamic circle.
  Water amount and gravity return `ControlEffect::Recreated`. Obstacle
  actions stay live.
- Fountain emits a lifetime-capped stream (`maximum_count` 320,
  `destruction_by_age`) and applies emission/launch/aim live.
- Float or Sink builds a 15×12 water pool and drops cork/wood/stone densities
  through engine fixtures. Sequential cork-then-stone densities are tested.
- Color Mixer creates two `COLOR_MIXING` groups; mix-strength recreates,
  stir-speed applies live. Tests prove Off keeps color lanes and Strong
  changes them.
- Jelly Drop builds an `ELASTIC | SPRING` group on two bars; shape/softness
  recreate; `poke-jelly` applies an in-engine impulse.
- Water Wheel pins a revolute hub with motor disabled and captures paddle
  segments from the live body transform. Jet/emission apply live.

`parse_scene_id` allowlists only the six lowercase ids (`dam-break`,
`fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, `water-wheel`) and
returns `UnknownScene` otherwise. `build_scene` matches that enum exhaustively.

### Catalog, controls, and credits

`SCENES` marks all six `ready: true`. Cards render static SVG previews with
caption `Static preview` and hash-only `Open` links (`#/scene/{id}`). No
not-ready chips remain.

Construction presets (`recreates: true`) show the reset sentence and require
`Apply setting`. Runtime presets apply on select change and omit the hint.
Actions stay live.

Credits render `Scene source`, `View scene source`, and `Third-party notices`.
Implementation hrefs are host-locked through
`https://github.com/bright-builds-llc/liquidfun-rs/blob` and an allowlisted
`crates/liquidfun-wasm/src/scene/*.rs` path. Inspiration may cite the LiquidFun
showcase, pinned Faucet, or particle guide; those links are not presented as
the running implementation. `web/src` has no `innerHTML`. Scene and error
strings are fixed text nodes. Development diagnostics stay prefixed and
length-capped in `maybeDevelopmentDetails`.

### App session

`App.tsx` owns one session. `startScene` increments generation, cancels the
pending frame, and disposes the prior world before load. `abandonScene` does
the same and clears construction presets. Hash changes to another ready id
clear construction values and call `startScene`. Reset calls `startScene`
without clearing `constructionValues`, then reapplies last construction
presets through `constructionEntriesForScene`. Hidden-document callbacks
clear the accumulated timestamp; `acceptedStepCount` remains the four-step
cap.

### Player spec and smoke evidence

`web/e2e/player.spec.ts` loops the six catalog ids for catalog-or-hash open,
`Playing`, title, `Scene source` / `View scene source` / `Third-party notices`,
and Reset near-zero `data-step-index`. Dam Break Gravity shows the reset hint
and `Apply setting`, then returns to Playing. Fountain switch restarts
`data-step-index` without a second WASM fetch. Unknown hash still shows
`Scene not found` plus `Open Dam Break`. Hidden-tab catch-up stays on Dam
Break only and asserts a max-4 step jump. The spec contains no
`onPointer`, `pointerdown`, or `WEBTEST-01` coverage and no
`Fountain is not ready yet` assertion.

`target/web-build/web-build.log` starts `start player-smoke` and ends
`complete player-smoke` after `just web-player-smoke`. That log is ignored
local Chromium evidence, not a Pages deploy.

### Docs honesty

README and TESTING state the playground has six native scenes and that
`just web-player-smoke` proves open/reset/switch locally. They keep the
existing hosted Dam Break URL as a visitor link and say a new Pages URL is
not required for local proof. They do not claim crate publication, complete
LiquidFun parity, pigment chemistry, live card previews, or WEBTEST-01.

### Pages, publication, and hobby scope

`.github/workflows/pages.yml` last changed in Phase 17
(`7dfafdf9baf90b2bf6a53005ab3f3c978b3c0250`) and is untouched by task-1 commit
`1a69ff3` and this review. This review grants no package publication, tag,
release, or Pages deploy authority. Hobby scope in `PROJECT-SCOPE.md` applies:
local proof and truthful docs, not Linux qualification or a new live URL gate.

## Fixed review manifest

The 16 manifest entries and hashes in
`18-REVIEW-MANIFEST.md` were independently rehashed from current file bytes and
matched exactly.

Manifest entry count: `16`.

## Threat review

- **T-18-10-01 mitigated:** player spec still asserts production-base
  `/liquidfun-rs/` WASM URLs.
- **T-18-10-02 mitigated:** unknown hash remains `Scene not found`; only the
  six lowercase allowlisted ids play.
- **T-18-10-03 mitigated:** Dam Break → Fountain requires a restarted
  `data-step-index` series; `startScene` / `abandonScene` dispose the prior
  world.
- **T-18-10-04 mitigated:** headings, `Apply setting`, and credit link names
  are fixed text; no source `innerHTML` for scene or error strings.
- **T-18-10-05 mitigated:** this separate AI reviewer binds approval to the
  exact digest below. Passing `just web-player-smoke` alone is not this
  acknowledgment.
- **T-18-10-06 accepted:** no Pages deploy in this plan; existing workflow
  unchanged; no PAT.

## Digest acknowledgment

`review_digest`:
`22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c`

The independent result exactly matched all 16 candidate entries and the
required digest.

> I, the Cursor independent AI review subagent identified above as agent-store
> identity `eddc4c2f-39ef-460d-99ad-782c4871b074`, independently inspected the
> listed Phase 18 implementation-and-evidence bytes and explicitly acknowledge
> exact digest
> `22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c`
> as the content I inspected and approve.

Any implementation, catalog, player, evidence, or listed-test byte change
invalidates this acknowledgment and requires a new fixed manifest, digest, and
independent review.

This review grants no package publication, tag, release, Pages deploy, or
other release authority. Passing `just web-player-smoke` alone is not this
acknowledgment.

## GSD Source Code Review

**Reviewed:** 2026-09-18T05:39:01Z
**Depth:** deep
**Files Reviewed:** 18
**Status:** issues_found

### Summary

Deep review of the Phase 18 six native scenes, factory allowlist, catalog
controls and credits, one-session App wiring, player spec, local smoke log,
and playground docs found no Critical or Warning issues. The independently
recomputed digest matches the required value. One Info item notes that Float
or Sink's labeled Body default is cork while the constructed world default is
wood.

The fixed manifest, independent AI review, approval, and exact digest
acknowledgment above are the review decision.

### Info

#### IN-01: Float or Sink labeled default vs constructed default

**File:** `web/src/components/scene-controls.ts:13`
**Issue:** UI default `cork` does not match Rust construction default `Wood`.
**Fix:** Use the same default on both sides.

---

_Reviewed: 2026-09-18T05:39:01Z_
_Reviewer: Cursor AI (gsd-code-reviewer), agent-store identity eddc4c2f-39ef-460d-99ad-782c4871b074_
_Depth: deep_

GSD_SOURCE_CODE_REVIEW_COMPLETE
