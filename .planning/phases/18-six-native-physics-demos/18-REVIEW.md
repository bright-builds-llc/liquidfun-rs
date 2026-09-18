---
phase: 18-six-native-physics-demos
plan: "10"
reviewed: 2026-09-18T05:45:27Z
independent_reviewed: 2026-09-18T05:45:27Z
depth: deep
candidate: 1a69ff3bc1b03039818bd621983cfbe394fdd244
local_head: 504d720f9e59ea8331e698c23de043fee60b6590
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
review_digest: 1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94
reviewer_identity: f5a42aa5-af9e-47d2-93c3-2e2c9312809d
reviewer_invocation_id: 019d874b-897c-4e37-a0f4-ac39fcd3468f
reviewer_disclosure: AI reviewer, not a human
implementing_or_fixing_executor: no
---

# Phase 18 Independent AI Review

## Decision

**APPROVED.**

There are no Critical findings and zero unresolved Warning findings. Prior
WR-01 (UI Cork versus constructed Wood) is resolved at commit `504d720`:
`DEFAULT_PRESET_VALUES.body` is now `"wood"` and `FloatOrSinkHooks` still
constructs `BodyPreset::Wood`. One leftover Info records a fail-open
construction-token fallback in Jelly Drop. This review does not mark
requirement completion in planning state files.

This document replaces the earlier `18-REVIEW.md` that named reviewer
`a8b9e495-ffa1-4be6-875e-fb733345fabc` and digest
`22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c`. Those
identities are not this reviewer.

## Reviewer and exact scope

- `reviewer_identity`: Cursor independent AI review subagent
  (`gsd-code-reviewer`), Task/agent-store identity
  `f5a42aa5-af9e-47d2-93c3-2e2c9312809d`
- `reviewer_invocation_id`: `019d874b-897c-4e37-a0f4-ac39fcd3468f`
- `reviewer_disclosure`: AI reviewer, not a human
- `model`: Cursor Grok 4.6
- `implementing_or_fixing_executor`: no
- `reviewed_at_utc`: `2026-09-18T05:45:27Z`
- `task_1_commit`: `1a69ff3bc1b03039818bd621983cfbe394fdd244`
- `wr01_fix_commit`: `504d720f9e59ea8331e698c23de043fee60b6590`
- `local_head_at_review`: `504d720f9e59ea8331e698c23de043fee60b6590`
- `origin/main_at_review`: `dc23318ce4a460d46022d749db5f24386d66c92b`

This reviewer is not implementing executor `eddc4c2f-39ef-460d-99ad-782c4871b074`
and did not resume that agent or parent store
`69e3a7c4-0536-4c8b-8b83-fd5bb2f407e4`. The implementing agent does not
acknowledge or approve this digest.

The reviewer independently inspected the complete relevant Phase 18 six-scene
diff and evidence: the six native scene modules, factory `scene.rs`, catalog,
controls, credits, `App.tsx` session wiring, `player.spec.ts`, ignored smoke
log, and README/TESTING honesty. Cross-file support inspected but not hashed
into the digest includes `web/src/catalog/links.ts`,
`web/src/catalog/previews.tsx`, `web/src/player/runtime.ts`,
`crates/liquidfun-wasm/src/session.rs`, and `crates/liquidfun-wasm/src/lib.rs`.
Material guidance was `AGENTS.md` Independent review (2026-09-16 owner policy),
`PROJECT-SCOPE.md` hobby scope, `18-CONTEXT.md` D-16 and D-17, `18-10-PLAN.md`,
and `18-REVIEW-MANIFEST.md`. Passing `just web-player-smoke` or other
automation is supporting context only and is not this acknowledgment.

## Findings and resolutions

No Critical findings. No unresolved Warning findings.

### WR-01 resolved: Float or Sink labeled default now matches Wood construction

**File:** `web/src/components/scene-controls.ts:13` and
`crates/liquidfun-wasm/src/scene/float_or_sink.rs:128`
**Issue:** Prior independent review found `DEFAULT_PRESET_VALUES.body` was
`cork` while `FloatOrSinkHooks` constructed `body_preset: BodyPreset::Wood`.
`constructionEntriesForScene` in `web/src/player/runtime.ts` only reapplies
`recreates: true` presets, and catalog `body` is a `runtimePreset` (`recreates:
false`), so the labeled first-load value was never sent to the engine.
**Resolution:** Independently confirmed from current source after commit
`504d720`. `DEFAULT_PRESET_VALUES.body` is `"wood"`.
`FloatOrSinkHooks` still constructs `BodyPreset::Wood` (density `0.6`). The
first `Drop body` without changing the select now drops wood while the labeled
value is Wood. Live `apply_control("body", …)` still allowlists cork/wood/stone
and applies without recreate.

### IN-01: Jelly Drop construction silently defaults unknown shape/softness tokens

**File:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs:74-80`
**Issue:** `build` uses `and_then(JellyShape::parse)` /
`and_then(Softness::parse)` then `unwrap_or(Circle|Medium)`. Dam Break and
Color Mixer fail closed on unknown construction tokens
(`SessionError::UnknownControl`). Live `apply_control` for Jelly Drop still
rejects unknown tokens. The only browser caller sends allowlisted select
values, so this is not a visitor-facing bug today.
**Fix:** Mirror Dam Break: map a present-but-unparsed shape or softness token
to `SessionError::UnknownControl`.

## Required inspection outcomes

### Digest

The reviewer recomputed the fixed manifest independently. For each of the 16
listed paths in the listed order, the reviewer concatenated UTF-8 path bytes,
one NUL byte, the lowercase SHA-256 of exact file bytes as ASCII hex, and one
LF byte, then SHA-256 hashed that complete concatenation.

Fifteen per-file hashes still match the prior manifest. Only
`web/src/components/scene-controls.ts` changed, to
`ee28e631fa1c4aafad283adb28b49436211400a7b7a69639addfdd9df9630241`.
The independently recomputed digest is exactly:

`1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`

### Native scenes and factory

Each of the six modules constructs a real `liquidfun::World` with gravity,
fixtures or particle groups, and allowlisted controls. There is no scripted
pose track, fake buoyancy, rendering-only color blend advertised as mixing, or
motor-driven Water Wheel.

- Dam Break builds the documented basin, water grid, and dynamic circle.
  Water amount and gravity return `ControlEffect::Recreated`. Obstacle
  actions stay live.
- Fountain emits a lifetime-capped stream (`maximum_count` 320,
  `destruction_by_age`) and applies emission/launch/aim live.
- Float or Sink builds a 15×12 water pool and drops cork/wood/stone densities
  through engine fixtures. Sequential cork-then-stone densities are tested.
  Construction default remains Wood; the UI default now matches.
- Color Mixer creates two `COLOR_MIXING` groups; mix-strength recreates,
  stir-speed applies live. Tests prove Off keeps color lanes and Strong
  changes them.
- Jelly Drop builds an `ELASTIC | SPRING` group on two bars; shape/softness
  recreate; `poke-jelly` applies an in-engine impulse.
- Water Wheel pins a revolute hub with motor disabled (`is_motor_enabled()`
  false, motor speed and torque bits `0`) and captures paddle segments from
  the live body transform. Jet/emission apply live. Emission-off leaves the
  angle nearly unchanged.

`parse_scene_id` allowlists only the six lowercase ids (`dam-break`,
`fountain`, `float-or-sink`, `color-mixer`, `jelly-drop`, `water-wheel`) and
returns `UnknownScene` otherwise. `build_scene` matches that enum exhaustively.
`ProofSession::new` parses the id before `SessionCore::create`.

### Catalog, controls, and credits

`SCENES` marks all six `ready: true`. Cards render static SVG previews
(`web/src/catalog/previews.tsx`, caption `Static preview`) and hash-only
`Open` links (`#/scene/{id}`). No WASM world is started per card. No
not-ready chips remain.

Construction presets (`recreates: true`) show the reset sentence and require
`Apply setting`. Runtime presets apply on select change and omit the hint.
Actions stay live.

Credits render `Scene source`, `View scene source`, and `Third-party notices`.
Implementation hrefs are host-locked through
`https://github.com/bright-builds-llc/liquidfun-rs/blob` and an allowlisted
`crates/liquidfun-wasm/src/scene/*.rs` path that rejects `..` and absolute
paths. Inspiration may cite the LiquidFun showcase, pinned Faucet, or particle
guide; those links are not presented as the running implementation. `web/src`
has no `innerHTML`. Scene and error strings are fixed text nodes. Development
diagnostics stay prefixed and length-capped in `maybeDevelopmentDetails`.

### App session

`App.tsx` owns one session. `startScene` increments generation, cancels the
pending frame, and disposes the prior world before load. `abandonScene` does
the same and clears construction presets. Hash changes to another ready id
clear construction values and call `startScene`. Stale generation disposes the
unused loaded session. Reset calls `startScene` without clearing
`constructionValues`, then reapplies last construction presets through
`constructionEntriesForScene`. Hidden-document callbacks clear the accumulated
timestamp; `acceptedStepCount` remains the four-step cap. Page copy says
experimental Rust physics through WebAssembly.

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
local Chromium evidence, not a Pages deploy. Passing that smoke is not this
acknowledgment.

### Docs honesty

README and TESTING state the playground has six native scenes and that
`just web-player-smoke` proves open/reset/switch locally. They keep the
existing hosted Dam Break URL as a visitor link and say a new Pages URL is
not required for local proof. They do not claim crate publication, complete
LiquidFun parity, pigment chemistry, live card previews, or WEBTEST-01.

### Pages, publication, and hobby scope

`.github/workflows/pages.yml` last changed in Phase 17
(`7dfafdf9baf90b2bf6a53005ab3f3c978b3c0250`) and is untouched by task-1 commit
`1a69ff3`, WR-01 fix `504d720`, and this review. This review grants no package
publication, tag, release, or Pages deploy authority. Hobby scope in
`PROJECT-SCOPE.md` applies: local proof and truthful docs, not Linux
qualification or a new live URL gate.

## Fixed review manifest

The 16 manifest entries and hashes in `18-REVIEW-MANIFEST.md` were
independently rehashed from current file bytes. Fifteen hashes are unchanged.
`web/src/components/scene-controls.ts` is now
`ee28e631fa1c4aafad283adb28b49436211400a7b7a69639addfdd9df9630241`. The new
digest is
`1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`.

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
`1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`

The independent result exactly matched all 16 current file entries and the
required new digest.

> I, the Cursor independent AI review subagent identified above as Task/agent
> identity `f5a42aa5-af9e-47d2-93c3-2e2c9312809d`, independently inspected the
> listed Phase 18 implementation-and-evidence bytes and explicitly acknowledge
> exact digest
> `1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`
> as the content I inspected and approve. I am an AI reviewer, not a human.
> I am not the implementing or fixing executor.

Any implementation, catalog, player, evidence, or listed-test byte change
invalidates this acknowledgment and requires a new fixed manifest, digest, and
independent review.

This review grants no package publication, tag, release, Pages deploy, or
other release authority. Passing `just web-player-smoke` alone is not this
acknowledgment.

## GSD Source Code Review

**Reviewed:** 2026-09-18T05:45:27Z
**Depth:** deep
**Files Reviewed:** 18
**Status:** issues_found

### Summary

Deep re-review after commit `504d720` confirmed the six native scenes, factory
allowlist, catalog controls and credits, one-session App wiring, player spec,
local smoke log, and playground docs. The independently recomputed digest is
`1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94`. Prior
WR-01 is resolved: UI default `wood` matches constructed `BodyPreset::Wood`.
IN-01 still notes Jelly Drop construction token fail-open.

The fixed manifest, independent AI review, approval, and exact digest
acknowledgment above are the review decision. This is not human approval.

### Info

#### IN-01: Jelly Drop unknown construction tokens default silently

**File:** `crates/liquidfun-wasm/src/scene/jelly_drop.rs:74-80`
**Issue:** Unknown shape/softness presets fall back to Circle/Medium instead
of `UnknownControl`.
**Fix:** Fail closed like Dam Break and Color Mixer.

---

_Reviewed: 2026-09-18T05:45:27Z_
_Reviewer: Cursor AI (gsd-code-reviewer), Task/agent identity f5a42aa5-af9e-47d2-93c3-2e2c9312809d_
_Depth: deep_

GSD_SOURCE_CODE_REVIEW_COMPLETE
