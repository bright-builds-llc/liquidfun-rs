---
phase: 28-interaction-seams
verified: 2026-09-22T14:58:00Z
status: passed
score: 5/5 must-haves verified
generated_by: gsd-verifier
lifecycle_mode: yolo
phase_lifecycle_id: 28-2026-09-22T12-56-07
generated_at: 2026-09-22T14:58:00Z
lifecycle_validated: true
overrides_applied: 0
---

# Phase 28: Interaction seams Verification Report

**Phase Goal:** Visitors can run Soup, Soup Stirrer, Impulse, Wave Machine, and Theo Jansen using native destroy-in-shape, group impulse, and live revolute motors (no JS physics fakes).
**Verified:** 2026-09-22T14:58:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

The phase goal holds in the live tree. `SCENE_IDS` is sixteen ready entries (eleven prior plus `soup`, `soup-stirrer`, `impulse`, `wave-machine`, `theo-jansen`). Native seams are present: public `World::destroy_particles_in_shape`, shared `soup_family` carve, Impulse whole-group `apply_particle_force_range` / `apply_particle_linear_impulse_range`, Wave Machine / Theo Jansen `set_revolute_motor_speed` from session step / live controls. Catalog, Vitest, capture plans, and Chromium smoke cover open/play/pause/reset plus Stirrer/Impulse/Theo gestures. No JS physics fakes found under `web/src` for these scenes.

Objective UAT (catalog order, controls, factory allowlist, native API wiring, e2e watch-first vs interactive split, recorded smoke) was agent-verified. No checkpoint required human judgment under hobby scope and Phase 26/27 VERIFICATION precedence (recognizable ports gated by Chromium smoke + focused unit tests, not headed visual review).

Pre-existing `cork_finishes_above_stone_after_the_same_native_steps` failure (cork y ≈ 0.883) is **out of scope**: same failure at `7d4092c` before phase 28; phase 28 scenes do not touch Float or Sink. Code review `28-REVIEW.md` (`issues_found`, 0 critical / 2 warnings) is advisory only and does not block goal achievement.

### Observable Truths

| # | Truth | Status | Evidence |
| --- | ------- | ---------- | -------------- |
| 1 | Visitor can watch Soup: a basin of liquid holds floating solid bits. | ✓ VERIFIED | `soup_family.rs` basin + water group + circle + two boxes + three edge noodles + carve via `destroy_particles_in_shape` before return. `soup.rs` watch-first `BuiltScene`; `SceneId::Soup => soup::build`. Catalog `ready: true`, empty controls, `WATCH_FIRST_HINT`. Focused liquidfun test `destroy_particles_in_shape_clears_pocket_under_circle`. No `with_destruction_by_age` on Soup family. E2E opens `#/scene/soup`. Smoke: 38/38 player-smoke. |
| 2 | Visitor can watch Soup Stirrer: a paddle keeps stirring that soup, and the visitor can free the paddle from its rail or put it back. | ✓ VERIFIED | `soup_stirrer.rs` composes `soup_family::`, paddle carve, `PrismaticJointDef` + `Option<JointId>` toggle via `destroy_joint`/`create_joint`, `on_advance` stir with `in_soup` + `MAX_STIR_SPEED` guards. Action `toggle-paddle-rail` and in-AABB pointer share `toggle_paddle_rail`. Unit tests: toggle×2, Reset remount, pointer path. Catalog action label `Toggle paddle rail`. E2E `POINTER_CONTROL["soup-stirrer"]`. |
| 3 | Visitor can click or tap Impulse and shove the whole particle blob. | ✓ VERIFIED | `impulse.rs` chain-loop box + one group; shove uses full `member_ids` with `apply_particle_force_range` / `apply_particle_linear_impulse_range`; outside-box no-op. Live `push-mode` force/impulse → `ControlEffect::Live`. Unit tests: momentum, hit-test, live preset, source asserts. Catalog preset Push force/impulse. E2E Impulse click + Push control. |
| 4 | Visitor can watch Wave Machine rock on its own and slosh the water inside. | ✓ VERIFIED | `wave_machine.rs` four-wall tank, revolute `with_motor(true, …)`, `on_advance` sets `MOTOR_SPEED_SCALE * time.cos() * PI` (0.05·cos(t)·π). Tests lock sim-time formula and reject Water Wheel motor-off pattern. Watch-first catalog empty controls. `MAX_ADVANCE_STEPS` remains 4 in `session.rs`. E2E opens `#/scene/wave-machine`. |
| 5 | Visitor can watch Theo Jansen walk under a particle load and can reverse its motor. | ✓ VERIFIED | `theo_jansen.rs` ground/walls, chassis/legs, soft `DistanceJointDef` `with_frequency(10.0)` + `with_damping_ratio(0.5)`, `FilterData` groupIndex `-1`, motorized revolute, particle slab load. Live `motor-direction` forward/reverse flips `set_revolute_motor_speed` sign → `ControlEffect::Live`. Tests in `theo_jansen/tests.rs`. Catalog Motor direction forward/reverse. E2E Theo Jansen gesture; shell last drawer link Theo Jansen. |

**Score:** 5/5 truths verified (roadmap success criteria = ACT-01…ACT-05)

### Supporting plan truths (also verified)

| Truth | Status | Evidence |
| ----- | ------ | -------- |
| Native destroy-in-shape + soup_family + Soup; MAX_ADVANCE_STEPS=4; no destruction-by-age on Soup | ✓ | `particle.rs` public helper + test; `soup_family` carve; `session.rs` `MAX_ADVANCE_STEPS: u32 = 4` |
| Soup Stirrer reuses soup_family; prismatic toggle×2; pointer ≡ action | ✓ | Compose + joint toggle + shared path tests |
| Impulse group shove + live push-mode; outside no-op | ✓ | Range APIs + Live control + tests |
| Wave Machine sim-time motor; not Water Wheel motor-off | ✓ | `set_revolute_motor_speed` in `on_advance`; source asserts |
| Theo soft legs + live motor reverse; not welded rigid polygons | ✓ | Distance soft joints + Live preset; no weld of legs |
| SCENE_IDS append five ids; controls/hints match UI-SPEC; PAGE_SUMMARY sixteen; Static previews; pinned 7f204021…; no `.catalog-card` | ✓ | `scenes.ts` / `previews.tsx` / `runtime.ts` / shell e2e |
| Vitest sixteen contract; SCENE_CAPTURE_PLANS sync | ✓ | `scenes.test.ts` length 16; `model.ts` five new plans |
| Chromium smoke five new + gestures; eleven regression; sixteen Static previews; `.catalog-card` 0 | ✓ | Plan 08 SUMMARY + `web-build.log` `complete player-smoke` + `.last-run.json` passed |

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | ----------- | ------ | ------- |
| `crates/liquidfun/src/world/particle_object/particle.rs` | Public destroy-in-shape | ✓ VERIFIED | `destroy_particles_in_shape` + focused pocket test (~354 lines) |
| `crates/liquidfun-wasm/src/scene/soup_family.rs` | Shared Soup builder | ✓ VERIFIED | Basin/solids/carve; returns `ground` (~271 lines) |
| `crates/liquidfun-wasm/src/scene/soup.rs` | Watch-first Soup | ✓ VERIFIED | `build` + tests; composes soup_family (~234 lines) |
| `crates/liquidfun-wasm/src/scene/soup_stirrer.rs` | Paddle + prismatic + stir | ✓ VERIFIED | Toggle, guards, pointer, tests (~462 lines) |
| `crates/liquidfun-wasm/src/scene/impulse.rs` | Box + group shove | ✓ VERIFIED | Force/impulse ranges + live push-mode (~465 lines) |
| `crates/liquidfun-wasm/src/scene/wave_machine.rs` | Motorized tank | ✓ VERIFIED | Sim-time motor + tests (~392 lines) |
| `crates/liquidfun-wasm/src/scene/theo_jansen.rs` | Soft walker + reverse | ✓ VERIFIED | Soft legs + live motor (~512 lines) + `theo_jansen/tests.rs` |
| `crates/liquidfun-wasm/src/scene.rs` | Five SceneId arms | ✓ VERIFIED | parse tokens + `=> *_::build` for all five |
| `web/src/catalog/scenes.ts` | Sixteen-scene catalog | ✓ VERIFIED | Order, ready, controls, pinned credits |
| `web/src/catalog/previews.tsx` | Five Static preview cases | ✓ VERIFIED | Exhaustive switch cases |
| `web/src/player/runtime.ts` | Sixteen-demo PAGE_SUMMARY | ✓ VERIFIED | "All sixteen demos run…" |
| `web/tests/scenes.test.ts` | Sixteen-scene Vitest contract | ✓ VERIFIED | Order, hints, controls, credits |
| `web/scripts/demo-media/model.ts` | Capture plan sync | ✓ VERIFIED | Five interaction plans + coverage assert |
| `web/e2e/player.spec.ts` | Watch-first + interactive | ✓ VERIFIED | `POINTER_CONTROL` for three interactive scenes |
| `web/e2e/player-helpers.ts` | Sixteen hash paths | ✓ VERIFIED | Five new `#/scene/…` paths |
| `web/e2e/shell.spec.ts` | Sixteen previews + shell | ✓ VERIFIED | 16 Static preview; Theo Jansen last; `.catalog-card` 0 |

gsd-tools `verify artifacts` returned `all_passed` for plans 28-01 through 28-08.

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | ---- | ------ | ------- |
| `scene.rs` | `soup.rs` | `SceneId::Soup => soup::build` | ✓ WIRED | Pattern in source |
| `soup_family.rs` | `World::destroy_particles_in_shape` | Carve under solids | ✓ WIRED | Pattern in source |
| `soup_stirrer.rs` | `soup_family` | Compose builder | ✓ WIRED | `soup_family::build_soup_family` |
| `soup_stirrer.rs` | Prismatic create/destroy | `Option<JointId>` toggle | ✓ WIRED | `PrismaticJointDef` + `destroy_joint` |
| `impulse.rs` | Group force/impulse ranges | Member shove | ✓ WIRED | Both apply_*_range APIs |
| `impulse.rs` | `ControlEffect::Live` | `push-mode` | ✓ WIRED | Pattern + tests |
| `wave_machine.rs` | `set_revolute_motor_speed` | `on_advance` | ✓ WIRED | Pattern in source |
| `wave_machine.rs` | `with_motor` | enable_motor at create | ✓ WIRED | Pattern in source |
| `theo_jansen.rs` | Soft distance joints | freq 10 / damp 0.5 | ✓ WIRED | Manual verify (`with_frequency(10.0)`); gsd-tools regex false-negative on escaped pattern |
| `theo_jansen.rs` | `set_revolute_motor_speed` | `motor-direction` | ✓ WIRED | Pattern in source |
| `scenes.ts` | WASM SceneId tokens | Five new ids | ✓ WIRED | Matches `parse_scene_id` (gsd-tools false-negative: cross-crate target) |
| `previews.tsx` | DemoNavigation | ScenePreview switch | ✓ WIRED | Five `case` arms (gsd-tools false-negative: DemoNavigation string) |
| `scenes.test.ts` | `scenes.ts` | UI-SPEC / credit asserts | ✓ WIRED | Pattern found |
| `demo-media/model.ts` | `SCENE_IDS` | `SCENE_CAPTURE_PLANS` | ✓ WIRED | Pattern found |
| `player.spec.ts` | catalog controls | watch-first / POINTER_CONTROL | ✓ WIRED | Pattern found |
| `justfile` `web-player-smoke` | e2e suite | `bun scripts/web-build.ts player-smoke` | ✓ WIRED | Manual verify (gsd-tools looks for file path, not just recipe) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| -------- | ------------- | ------ | ------------------ | ------ |
| Soup / Soup Stirrer frames | particle + rigid segments | Native `World` step after soup_family carve | Yes — session advance copies typed arrays | ✓ FLOWING |
| Impulse shove | group `member_ids` + force/impulse | Pointer → native range APIs | Yes — unit test asserts momentum change | ✓ FLOWING |
| Wave Machine motor | `self.time` + revolute speed | `on_advance` per session step | Yes — unit test locks cos(t) formula | ✓ FLOWING |
| Theo Jansen reverse | motor speed sign | Live `motor-direction` control | Yes — Live effect + `set_revolute_motor_speed` | ✓ FLOWING |
| Catalog / player | `SCENE_IDS` / scene modules | WASM `parse_scene_id` + catalog records | Yes — sixteen ready ids wired end-to-end | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command / check | Result | Status |
| -------- | --------------- | ------ | ------ |
| Sixteen SCENE_IDS ending with five interaction ids | Read `web/src/catalog/scenes.ts` | Length 16; last five soup…theo-jansen | ✓ PASS |
| All five build_scene arms | `rg` on `scene.rs` | Five `SceneId::* => *_::build` | ✓ PASS |
| Native destroy-in-shape + carve call sites | `rg destroy_particles_in_shape` | Public API + soup_family + stirrer paddle | ✓ PASS |
| Chromium player-smoke (recorded) | Plan 08 evidence + log files | `complete player-smoke`; `.last-run.json` `"status":"passed"`; SUMMARY 38/38 | ✓ PASS |
| Full workspace / float-or-sink cork | Known pre-existing at 7d4092c | Not re-run; not a phase 28 must_have | ? SKIP (out of scope) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ---------- | ----------- | ------ | -------- |
| ACT-01 | 28-01 (+06/07/08) | Watch Soup basin with floating solids | ✓ SATISFIED | Truth 1; soup_family + Soup scene + smoke |
| ACT-02 | 28-02 (+06/07/08) | Soup Stirrer stir + free/restore rail | ✓ SATISFIED | Truth 2; prismatic toggle + e2e gesture |
| ACT-03 | 28-03 (+06/07/08) | Impulse click/tap whole-blob shove | ✓ SATISFIED | Truth 3; group force/impulse + e2e |
| ACT-04 | 28-04 (+06/07/08) | Wave Machine rocks and sloshes | ✓ SATISFIED | Truth 4; sim-time motor + smoke |
| ACT-05 | 28-05 (+06/07/08) | Theo Jansen walks + reverse motor | ✓ SATISFIED | Truth 5; soft legs + live reverse + e2e |

REQUIREMENTS.md maps ACT-01…ACT-05 exclusively to Phase 28 (Complete). No orphaned Phase 28 requirement IDs. All five IDs appear in PLAN frontmatter across 28-01…28-08.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TODO/FIXME/placeholder stubs in five scene modules | — | None blocking |
| `soup.rs` / `soup_stirrer.rs` | (WR-01) | Duplicated circle render center vs `CIRCLE_LOCAL_CENTER` | ℹ️ Info (from 28-REVIEW) | Draw drift risk only; does not block goal |
| `impulse.rs` | (WR-02) | Force errors mapped to `SceneConstruction` | ℹ️ Info (from 28-REVIEW) | Advisory player fail-hard path; shove still native |

No blocker stubs. No JS physics fakes for phase 28 scenes.

### Human Verification Required

None. Watch/interact criteria are objectively covered by native unit tests + Chromium `just web-player-smoke` (same gate as Phases 26–27).

### Gaps Summary

No actionable gaps. Phase 28 roadmap success criteria and ACT-01…ACT-05 are met in the codebase. Advisory review warnings and the pre-existing Float or Sink cork test do not fail this phase.

### Lifecycle provenance

| Artifact | `lifecycle_mode` | `phase_lifecycle_id` |
| -------- | ---------------- | -------------------- |
| `28-CONTEXT.md` | yolo | `28-2026-09-22T12-56-07` |
| `28-01`…`28-08-PLAN.md` | yolo | `28-2026-09-22T12-56-07` |
| `28-01`…`28-08-SUMMARY.md` | yolo | `28-2026-09-22T12-56-07` |
| `28-VERIFICATION.md` | yolo | `28-2026-09-22T12-56-07` |

`lifecycle_validated: true` — CONTEXT, PLAN, SUMMARY, and VERIFICATION share compliant provenance (not `direct-fallback`).

---

_Verified: 2026-09-22T14:58:00Z_
_Verifier: Claude (gsd-verifier)_
