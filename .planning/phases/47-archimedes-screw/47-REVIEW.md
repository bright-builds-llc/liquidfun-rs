---
phase: 47-archimedes-screw
reviewed: 2026-10-14T22:51:41Z
depth: standard
files_reviewed: 32
files_reviewed_list:
  - crates/liquidfun-wasm/src/scene/archimedes_screw.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/flow_tests.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/hooks_tests.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/spike_tests.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/test_support.rs
  - crates/liquidfun-wasm/src/scene/archimedes_screw/tests.rs
  - crates/liquidfun-wasm/src/scene.rs
  - crates/liquidfun-wasm/src/scene/gravity_slider.rs
  - crates/liquidfun-wasm/src/scene_spot.rs
  - crates/liquidfun-wasm/src/session/tests.rs
  - web/src/catalog/archimedes-screw-speed.ts
  - web/src/catalog/scene-records-curiosity.ts
  - web/src/catalog/previews-curiosity.tsx
  - web/src/catalog/portrait-bounds.ts
  - web/src/catalog/scene-view-bounds.ts
  - web/src/catalog/scenes.ts
  - web/src/catalog/scene-records.ts
  - web/src/catalog/previews.tsx
  - web/scripts/readme-svg/plans.ts
  - web/scripts/demo-media/model.ts
  - web/e2e/player-helpers.ts
  - web/e2e/player.spec.ts
  - web/tests/controls.test.ts
  - web/tests/portrait-bounds.test.ts
  - web/tests/scene-catalog-controls.test.ts
  - web/tests/scene-catalog-credits.test.ts
  - web/tests/scenes.test.ts
  - docs/adding-a-scene.md
  - README.md
  - docs/benchmarks/scene-fingerprints.json
findings:
  critical: 0
  warning: 0
  info: 7
  total: 7
status: issues_found
---

# Phase 47: Code Review Report

**Reviewed:** 2026-10-14T22:51:41Z
**Depth:** standard
**Files Reviewed:** 32
**Status:** issues_found

## Summary

I reviewed the new Archimedes Screw scene: the builder, hooks, geometry, and the
six test modules in full. For the shared files (`scene.rs`, the web catalog,
tests, scripts, README, docs, and fingerprints) I reviewed only the hunks changed
since `360b1e62b`.

The production code is sound. I found no Critical or Warning issues.

- **Control parsing** rejects empty input, signs, whitespace, leading zeros,
  decimals, and values above `MAX_SCREW_RPM`, as `parse_drum_rpm` does.
- **Intent application:** `apply_control` stores intent only, and `on_advance`
  turns it into vane motion. The wasm session order is `on_advance`, then
  eviction, then `world.step`.
- **Live controls:** both controls are `recreates: false`, so the web never sends
  them as construction presets. This is why `build` rejecting any non-gravity
  preset is safe. After a gravity rebuild, `worker-controls.ts` replays both
  live values.
- **Snap and wrap:** after a snap teleport, the computed velocity equals
  `step / SIM_DT`, so the body follows the ideal phase. Each snap resets any
  drift.
- **Snap invariant (D-03):** the snapped-in or snapped-out vane stays inside a
  housing for every step size the `const` assert allows. The bottom housing
  margin is about 0.054 m along the axis. The largest step the assert allows is
  under 0.05 m, and the step at 60 rpm is 0.0167 m.
- **Geometry checks:**
  - The computed wall corners lie 0.20 to 0.21 m inside the `FRAME_*` rectangle.
  - The weir lip (2.05) lies between the outlet floor (1.936) and the lowest
    top-housing corner (2.397).
  - The floor slab's undercut inlet end meets the vane coordinate `q = INLET_LENGTH`
    at `t = 0`.
- **Fingerprint:** the baseline grew in its own commit (`4be8ffdab`). No source
  changed after its provenance commit `23541975c`.
- **Tests:** `cargo test -p liquidfun-wasm --lib archimedes_screw` passes in
  debug: 46 passed, 16 release-only tests ignored.

The findings are test-coverage gaps, one documentation error, a duplicated
helper, and a theoretical edge in the wrap function.

## Info

### IN-01: Snap-housing tests check one phase each, not the full range of snap poses

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw/tests.rs:71-107`

**Issue:** D-03 says a vane added or removed by a snap is always inside a
housing. The two tests check one pose each:

- vane 0 at phase `0.0`
- vane 6 at phase `PITCH - 1e-4`

Snaps actually happen across these ranges:

- **Vane 0:** phase in `[-max_step, max_step]`. A forward snap places it below
  zero, and a reverse snap removes it while the phase is in `[0, step)`.
- **Vane 6:** phase in `[PITCH - max_step, PITCH + max_step]`. A reverse snap
  adds it at `[PITCH, PITCH + step)`, which is not tested at all.

The vane-6 test checks the easiest end of its range. The bottom housing is the
tight case: vane 0 is fully inside it only while its phase is at most about
0.054. Retuning `MAX_SCREW_RPM`, `VANE_THICKNESS`, or `BOTTOM_FACE_TAN` could
break D-03 without failing these tests.

**Fix:** Check both ends of each range, using the largest step:

```rust
let max_step = MAX_AXIAL_SPEED * SIM_DT;
for phase in [-max_step, 0.0, max_step] {
    assert_inside(&bottom_housing(), vane_at_phase(0, phase));
}
for phase in [PITCH - max_step, PITCH, PITCH + max_step] {
    assert_inside(&top_housing(), vane_at_phase(VANE_COUNT - 1, phase));
}
```

### IN-02: Vanes are drawn from the stored phase, so frame checks cannot see body drift

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw.rs:297-305`;
`crates/liquidfun-wasm/src/scene/archimedes_screw/flow_tests.rs:222-225`

**Issue:** `collect_segments` draws vanes from `self.phase`, not from the train
body's position. If the body ever falls behind the phase (for example, when
`clamp_speed` limits a correction at 60 rpm), the drawn vanes and the vanes the
water collides with no longer match. No test would notice:

- The `rigid_segments` bit comparison in the release `speed_zero_holds_the_vanes_still`
  test depends only on `hooks.phase`, so it cannot fail independently of the
  body check beside it.
- `the_train_stays_within_one_pitch_across_snaps` checks that the body stays
  within `[0, PITCH]`. It does not check that the body matches the phase.

**Fix:** In the `drive` loop of `hooks_tests.rs`, assert after each step that
the body matches the phase:

```rust
(train_origin(world, train) - train_position(phase)).length() < LATTICE_TOLERANCE
```

The alternative is to compute the drawn phase from
`body_snapshot(train).position()`.

### IN-03: The reverse-motion test compares positions only modulo a pitch

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs:239-242`

**Issue:** The test reduces the error modulo `PITCH`, so any whole-pitch
teleport counts as a pass. It does catch "reverse ignored", because that gives
an error of `2 * step`. But it cannot tell a correct reverse step from a wrong
snap, such as a snap of `2 * PITCH` or a snap in the wrong direction. The test
name promises "down the axis at the current speed", and the test does not
directly check that. `hooks_tests.rs:181-184` uses the same modulo reduction,
although that 30-step run never snaps.

**Fix:** Check the velocity directly. Also check that the phase wrapped to
`PITCH - step`:

```rust
let along = train_snapshot(&parts).linear_velocity().dot(AXIS);
assert!((along - axial_speed(DEFAULT_SCREW_RPM, true)).abs() < 1e-5);
```

### IN-04: The web-to-Rust sync test matches substrings and misses the minimum, step, and direction tokens

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs:246-258`;
`web/src/catalog/scene-records-curiosity.ts:42-46`

**Issue:** `web_speed_constants_match_rust` looks for the exact substrings
`SCREW_SPEED_DEFAULT = 30;` and `SCREW_SPEED_MAX = 60;`. The R14 row that this
phase added to `docs/adding-a-scene.md` says to compare parsed values, not
printed substrings.

The test also leaves three values unchecked against Rust:

- `SCREW_SPEED_MIN = 0`
- `SCREW_SPEED_STEP = 1`
- the `forward` and `reverse` tokens of `screw-direction`

If the web step became fractional, or a direction token were renamed, the live
control would fail at runtime with `UnknownControl`, and the Rust tests would
still pass. The web test `scene-catalog-controls.test.ts` only pins the web side.

**Fix:**

- Parse each constant with a helper like `ts_field`, and check that `MIN == 0`
  and `STEP == 1`.
- Use `include_str!` on `scene-records-curiosity.ts`. Check that each
  `option("<token>"` in the `screw-direction` block parses through
  `apply_control`.

### IN-05: The `VANE_OVERHANG` doc comment describes half the value

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs:76-78`

**Issue:** The comment says "Axial reach of a vane tip beyond its mid-height
point". The value `(TUBE_GAP + VANE_EMBED) * VANE_TAN` (about 0.291 m) is the
full axial lean from one tip to the other. The reach of one tip beyond mid-height
is `(TUBE_GAP / 2 + VANE_EMBED) * VANE_TAN` (about 0.164 m). The value is used
as a conservative margin in the tests, so nothing breaks, but the comment is
wrong.

**Fix:** Change the comment to "Axial lean of a vane from its bottom tip to its
top tip".

### IN-06: Duplicated rpm parser and tube-membership test

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw.rs:60-72`;
`crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs:459-460, 509-513`

**Issue:**

- `parse_screw_rpm` is a copy of `washing_machine::parse_drum_rpm`
  (`washing_machine.rs:258-271`), with only the maximum changed.
- `wanted_water` repeats the body of the test-only `in_tube`.

**Fix:**

- Move a shared `parse_whole_rpm(value: &str, max: u16) -> Option<u16>` into
  `scene.rs`, and use it in both scenes.
- Make `in_tube` available outside tests, and call it from `wanted_water`.

### IN-07: `advance_phase` can return exactly `PITCH` because of f32 rounding

**File:** `crates/liquidfun-wasm/src/scene/archimedes_screw.rs:88-89`

**Issue:** If `next` lands in about `(-3e-8, 0)`, then `next + PITCH` rounds to
`1.0`. That breaks the documented `[0, PITCH)` range for `phase`. I simulated
every rpm from 1 to 60 in both directions for 60,000 steps from phase 0, and it
never happened. After an rpm change, though, the phase can take any value, so
the case is possible. The result is harmless, because phase `PITCH` gives the
same vane lattice, and the next step brings the phase back into range.

**Fix:** Clamp the wrapped value, or relax the documented range to
`[0, PITCH]`:

```rust
if next < 0.0 {
    let wrapped = next + PITCH;
    return if wrapped < PITCH { (wrapped, PITCH) } else { (0.0, 0.0) };
}
```

---

_Reviewed: 2026-10-14T22:51:41Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
