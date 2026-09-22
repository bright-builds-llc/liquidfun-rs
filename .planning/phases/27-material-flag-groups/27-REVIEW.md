---
phase: 27-material-flag-groups
reviewed: 2026-09-22T06:11:00Z
depth: standard
files_reviewed: 18
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/basin_family.rs
  - crates/liquidfun-wasm/src/scene/surface_tension.rs
  - crates/liquidfun-wasm/src/scene/elastic_particles.rs
  - crates/liquidfun-wasm/src/scene/rigid_particles.rs
  - crates/liquidfun-wasm/src/scene/particles.rs
  - crates/liquidfun-wasm/src/scene/liquid_timer.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - web/src/catalog/scenes.ts
  - web/src/catalog/previews.tsx
  - web/src/player/runtime.ts
  - web/tests/scenes.test.ts
  - web/scripts/demo-media/model.ts
  - web/e2e/player-helpers.ts
  - web/e2e/player.spec.ts
  - web/e2e/shell.spec.ts
  - web/e2e/reset-honesty.spec.ts
  - scripts/web-build.ts
findings:
  critical: 0
  warning: 0
  info: 3
  total: 3
status: issues_found
---

# Phase 27: Code Review Report

**Reviewed:** 2026-09-22T06:11:00Z
**Depth:** standard
**Files Reviewed:** 18
**Status:** issues_found

## Summary

Reviewed the Phase 27 material-flag-group WASM scenes (Surface Tension, Elastic Particles, Rigid Particles), shared basin helpers, catalog/preview wiring for all eleven demos, player runtime copy, demo-media capture plans, Chromium e2e helpers/specs, and `scripts/web-build.ts` player-smoke `CI=true` gate.

Rust scene factories match the pinned LiquidFun JS layouts (geometry, flags, damping 0.2 on Surface Tension only, spinning blue boxes on Elastic/Rigid). Catalog IDs, allowlist parsing, watch-first empty controls, and e2e coverage stay aligned at eleven scenes. No critical or warning defects found; three low-severity maintainability notes below.

## Info

### IN-01: Deprecated timeout alias has no remaining call sites

**File:** `web/e2e/player-helpers.ts:23-24`
**Issue:** `SIX_SCENE_TIMEOUT_MS` is still exported as a deprecated alias of `ALL_SCENE_TIMEOUT_MS`, but no `web/e2e` call site imports it anymore. Dead export adds naming drift risk (readers may think the budget is still six-scene scoped).
**Fix:** Remove the alias once no external scripts depend on it, or re-export only from a shared constants module with a short comment that the name is historical:

```ts
export const ALL_SCENE_TIMEOUT_MS = 220_000;
// Drop SIX_SCENE_TIMEOUT_MS — callers already use ALL_SCENE_TIMEOUT_MS.
```

### IN-02: Material create tests omit basin segment regression checks

**File:** `crates/liquidfun-wasm/src/scene/surface_tension.rs:201-208` (same pattern in `elastic_particles.rs:220-223`, `rigid_particles.rs:205-208`)
**Issue:** `create_builds_basin_groups_and_ball` asserts particle count and `rigid_circles().len() >= 3` (one packed circle), but unlike `particles.rs` it never asserts basin segments / `rigid_shape_count`. A broken `collect_segments` that returned `[]` would still pass these tests while the canvas lost the basin outline.
**Fix:** Mirror the Particles assertion after capture, e.g.:

```rust
assert!(
    session.rigid_shape_count() >= 4,
    "three basin segments plus one dynamic ball"
);
assert_eq!(frame.rigid_circles().len(), 3);
```

### IN-03: Watch-first `SceneHooks` boilerplate is duplicated five times

**File:** `crates/liquidfun-wasm/src/scene/surface_tension.rs:133-183` (also `elastic_particles.rs:148-198`, `rigid_particles.rs:133-183`, `particles.rs:159-209`, `liquid_timer.rs:148-194`)
**Issue:** Identical watch-first hook bodies (`on_advance` Ok, reject controls/actions, no-op pointer, static segments) are copied per scene. Future behavioral tweaks (e.g. pointer policy) risk drifting across the five implementations.
**Fix:** Extract a small helper trait or struct (e.g. `WatchFirstHooks { segments, maybe_ball }`) shared by basin-family and Liquid Timer scenes, keeping only scene-specific construction in each module.

---

_Reviewed: 2026-09-22T06:11:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
