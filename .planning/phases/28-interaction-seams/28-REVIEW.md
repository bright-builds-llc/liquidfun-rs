---
phase: 28-interaction-seams
reviewed: 2026-09-22T14:47:00Z
depth: standard
files_reviewed: 15
files_reviewed_list:
  - crates/liquidfun/src/world/particle_object/particle.rs
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/soup_family.rs
  - crates/liquidfun-wasm/src/scene/soup.rs
  - crates/liquidfun-wasm/src/scene/soup_stirrer.rs
  - crates/liquidfun-wasm/src/scene/impulse.rs
  - crates/liquidfun-wasm/src/scene/wave_machine.rs
  - crates/liquidfun-wasm/src/scene/theo_jansen.rs
  - crates/liquidfun-wasm/src/frame.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - web/src/catalog/scenes.ts
  - web/src/catalog/previews.tsx
  - web/src/player/runtime.ts
  - web/e2e/player.spec.ts
  - web/e2e/shell.spec.ts
findings:
  critical: 0
  warning: 2
  info: 3
  total: 5
status: issues_found
---

# Phase 28: Code Review Report

**Reviewed:** 2026-09-22T14:47:00Z
**Depth:** standard
**Files Reviewed:** 15
**Status:** issues_found

## Summary

Phase 28 interaction seams look solid overall: allowlisted scene IDs, typed pointer parsing, native `destroy_particles_in_shape`, Impulse whole-group shove via range APIs, Soup Stirrer’s prismatic rail toggle, Wave Machine / Theo Jansen live motors, raised frame caps for Theo Jansen, and matching web catalog/e2e coverage. No critical security or crash issues. Two warnings cover render-constant drift risk and Impulse shove error mapping that can fail the player session; three info items note maintainability nits.

## Warnings

### WR-01: Soup circle render center duplicates private geometry constant

**File:** `crates/liquidfun-wasm/src/scene/soup.rs:137`
**Also:** `crates/liquidfun-wasm/src/scene/soup_stirrer.rs:299`
**Issue:** Frame export applies a hardcoded `Vec2::new(0.0, 0.5)` while construction uses private `CIRCLE_LOCAL_CENTER` in `soup_family.rs`. Changing the fixture local center without updating both collectors mis-draws the floating circle.
**Fix:** Export or share the constant and use it in both collectors:

```rust
// soup_family.rs
pub(crate) const CIRCLE_LOCAL_CENTER: Vec2 = Vec2::new(0.0, 0.5);

// soup.rs / soup_stirrer.rs
use super::soup_family::CIRCLE_LOCAL_CENTER;
let center = transform.apply(CIRCLE_LOCAL_CENTER);
```

### WR-02: Impulse shove maps force failures to `SceneConstruction`

**File:** `crates/liquidfun-wasm/src/scene/impulse.rs:185-204`
**Issue:** `particle_group_view` / `apply_particle_*_range` errors become `SessionError::SceneConstruction`. The player treats pointer errors as fatal (`forwardScenePointer` → `fail`), so an empty member range or transient force error kills the session instead of no-opping or reporting a control failure.
**Fix:** Guard empty members and map force errors to a softer outcome (or `Ok(())` no-op):

```rust
if members.is_empty() {
    return Ok(());
}
world
    .apply_particle_force_range(system, &members, force)
    .map_err(|_error| SessionError::StepFailed)?; // or Ok(()) for fail-soft
```

## Info

### IN-01: Duplicated Soup solid segment collection

**File:** `crates/liquidfun-wasm/src/scene/soup.rs:99-130`
**Also:** `crates/liquidfun-wasm/src/scene/soup_stirrer.rs:260-290`
**Issue:** Basin / box / edge segment export is copy-pasted across Soup and Soup Stirrer.
**Fix:** Move a shared `collect_soup_solid_segments(...)` helper into `soup_family` (or a small private module) used by both hooks.

### IN-02: Soup Stirrer damping pin is debug-only

**File:** `crates/liquidfun-wasm/src/scene/soup_stirrer.rs:68-79`
**Issue:** The `SetDamping(1.0)` pin is only enforced via `debug_assert!` against the shared builder’s default. Release builds will not notice if `ParticleSystemDef::default().damping` drifts away from `1.0`.
**Fix:** Call `with_damping(PARTICLE_DAMPING)?` on the system after construction (or in `soup_family` when built for stirrer), or keep a release-time equality check that returns `SceneError::ParticleSystem`.

### IN-03: Theo Jansen sits close to raised frame caps

**File:** `crates/liquidfun-wasm/src/frame.rs:6-7`
**Also:** `crates/liquidfun-wasm/src/scene/theo_jansen.rs:444-507`
**Issue:** Walker export is about 43 segments and 41 circles against caps 64 / 48. Headroom is intentional for this scene, but a small visualization expansion would start failing `FrameData::new` / capture.
**Fix:** Document the Theo Jansen budget next to the constants, or assert segment/circle counts in `theo_jansen/tests.rs` against the caps so future growth is caught early.

---

_Reviewed: 2026-09-22T14:47:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
