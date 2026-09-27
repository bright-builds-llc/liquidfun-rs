---
phase: 30-periodic-hydraulic-fountain
reviewed: 2026-09-27T15:48:32Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs
  - crates/liquidfun-wasm/src/scene/hydraulic_fountain/tests.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - web/e2e/player-helpers.ts
  - web/e2e/shell.spec.ts
  - web/scripts/demo-media/model.ts
  - web/scripts/readme-svg/plans.ts
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/previews.tsx
  - web/src/catalog/scene-records.ts
  - web/src/catalog/scenes.ts
  - web/src/player/runtime.ts
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 30: Code Review Report

**Reviewed:** 2026-09-27T15:48:32Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Reviewed the hydraulic-fountain scene, its catalog wiring, and the player/e2e companions. The locked behavior holds: the piston is a dynamic body on a limited prismatic motor, one default water group is created at build, the step hook only rewrites motor speed, `fountain.rs` is untouched, and `MAX_ADVANCE_STEPS` remains 4. Catalog ids, gravity-slider coverage, the phone frame, and the static preview are consistent with a twenty-demo watch-first scene.

The live frame does not draw the piston. `collect_segments` returns four static wall lines, and `collect_circles` is empty, while the playground and README export render only those lanes. The body that the scene is named for is simulated and invisible.

## Warnings

### WR-01: The moving piston is omitted from the rendered frame

**File:** `crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs:256-258`
**Issue:** `HydraulicFountainHooks` keeps the joint id and elapsed time, then drops the piston `BodyId` after construction. `collect_segments` always returns the static `WALL_SEGMENTS` and ignores the world transform. Frame capture draws only those segments and the empty circle list, so the piston box never appears in the playground canvas or in a README SVG. Wave Machine and Soup Stirrer both transform local box corners into segments each capture so a moving body stays visible. Water can still cross the throat, but the timed piston the catalog describes is not on screen. The static preview draws a piston, so the sidebar thumbnail and the live scene disagree.
**Fix:** Keep the piston body on the hooks and append its four transformed edges after the wall segments.

```rust
const PISTON_LOCAL_CORNERS: [Vec2; 4] = [
    Vec2::new(-PISTON_HALF_WIDTH, -PISTON_HALF_HEIGHT),
    Vec2::new(PISTON_HALF_WIDTH, -PISTON_HALF_HEIGHT),
    Vec2::new(PISTON_HALF_WIDTH, PISTON_HALF_HEIGHT),
    Vec2::new(-PISTON_HALF_WIDTH, PISTON_HALF_HEIGHT),
];

struct HydraulicFountainHooks {
    elapsed: f32,
    joint: JointId,
    piston: BodyId,
}

fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
    let mut segments = WALL_SEGMENTS.to_vec();
    let transform = world
        .body_snapshot(self.piston)
        .map_err(|_error| SessionError::FrameCaptureFailed)?
        .transform();
    let world_corners = PISTON_LOCAL_CORNERS.map(|corner| transform.apply(corner));
    for index in 0..world_corners.len() {
        segments.push(RigidSegment {
            start: world_corners[index],
            end: world_corners[(index + 1) % world_corners.len()],
        });
    }
    Ok(segments)
}
```

---

_Reviewed: 2026-09-27T15:48:32Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
