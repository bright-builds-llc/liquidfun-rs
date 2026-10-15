---
phase: 48-fake-ferrofluid
fixed_at: 2026-10-15T08:20:00Z
review_path: .planning/phases/48-fake-ferrofluid/48-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 48: Code Review Fix Report

**Fixed at:** 2026-10-15T08:20:00Z
**Source review:** .planning/phases/48-fake-ferrofluid/48-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8 (fix scope: all)
- Fixed: 8
- Skipped: 0

The default (no-input) run is unchanged. `just scene-fingerprint-gate` (full capture) reports PASS with 28/28 protected ids unchanged after every physics fix and again at HEAD `4aecb7d3b`. Fake-ferrofluid is still `e73ef71f0615b4eb`.

## Fixed Issues

### WR-01: Sideways pull does not fade with height, so a far-away magnet still clumps the puddle

**Files modified:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid/tests.rs`
**Commit:** c99401124
**Status:** fixed: requires human verification (field-model logic change)
**Applied fix:** Added `far_fade(h, reach)`. It is exactly 1 inside the reach and falls through `bump` to 0 at twice the reach. The sideways term is now `scale * |E(h)| * far_fade * weight_slope`. Inside the reach this is bit-identical to the old term, because multiplying by 1.0 changes nothing. That is why the default-run fingerprint did not change. The module doc now says the field is conservative only above the magnet and inside the reach. New test `pull_vanishes_beyond_twice_the_reach` checks that the pull is `Vec2::ZERO` at column-flank offsets 2 to 3 reaches above and below the magnet. All 80 release `fake_ferrofluid` tests passed after the change, including the spike, flow and curl-free tests.

### WR-02: Containment is not tested along drag paths or at clamp extremes at full strength

**Files modified:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/flow_tests.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid/control_tests.rs`
**Commit:** 94104fdff
**Status:** fixed: requires human verification (it changes how far a user can drag the magnet)
**Applied fix:** I wrote the drag-path test first. `dragged_at_full_strength` sets the strength to 100 and presses at rest. It lets the crown form for 120 steps, then drags the held magnet in 300 fixed steps per leg (about 0.01 m per step). After a 120-step dwell it releases and watches for 300 steps (5 s). Every step it records out-of-dish, escaped and evicted particles. Before the fix, with WR-01 already applied, the test showed escape:
- On path `(0,-0.35) -> (0,2.5) -> (3.2,2.5)`: 26 particles out of the dish, 17 escaped and 19 evicted.
- On path `(0,-0.35) -> (0,3.5) -> (3.2,3.5)`: 141 out, 117 escaped and 117 evicted.

The smallest fix that leaves the default run alone is the clamp, so that is the one I applied. A new constant `MAGNET_MAX_Y = WALL_TOP_Y - FIELD.reach` (1.4 m) caps the magnet height in `clamp_magnet`. The x clamp and the frame inset (UI-D82) are unchanged. I did not add lift fading above the magnet: the resting puddle sits above the magnet, so that would have changed the default run.

The committed tests ask for the clamp extremes:
- `dragging_up_then_past_the_wall_at_full_strength_keeps_the_liquid_in`: rest, then `(0,3.5)`, then `(3.2,3.5)`.
- `dragging_past_the_wall_then_up_at_full_strength_keeps_the_liquid_in`: rest, then `(3.2,-0.35)`, then `(3.2,3.5)`.

Both now keep every particle in the dish, with no escape and no eviction. I also tried legs of 60 and 1000 steps and dwells of 120 and 1200 steps; all four combinations passed. I reverted those temporary edits.

`HIGH_AT_THE_EDGE` now tracks the clamp corner `(3.2, MAGNET_MAX_Y)` and still passes its 60 s hold and the column-top margin. The new debug-safe test `pointer_down_above_the_frame_clamps_one_reach_below_the_wall_tops` covers the new limit. The release `fake_ferrofluid` tests (83) and the `particle_force` tests (9) pass.

### WR-03: The demo-media capture's center click switches the magnet off for the rest of the capture

**Files modified:** `web/scripts/demo-media/model.ts`, `.planning/phases/48-fake-ferrofluid/48-CONTEXT.md`
**Commit:** 744547ae9
**Applied fix:** I accepted the collapse on purpose. The capture runner's only gestures are `click` and `drag`, and both end in `mouse.up()`. A release always parks the magnet and switches it off (D-07), so no capture action can keep the spikes. A new action kind would add scope beyond this review.

I added a comment beside the `fake-ferrofluid` entry in `SCENE_CAPTURE_PLANS`. It explains that the 8 s capture shows the spikes rise for 3 s and then collapse at step 180. I also replaced the "idle" wording in D-21 with the same description and a reference to this finding. The README plan (watch-first at default settings, with `magnet` cues at samples 120 and 160) is unchanged. Web typecheck passes, and `tests/demo-media-model.test.ts` passes (4 tests).

### IN-01: The strength control is parsed with an RPM-named helper

**Files modified:** `crates/liquidfun-wasm/src/scene.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw.rs`, `crates/liquidfun-wasm/src/scene/archimedes_screw/control_tests.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs`, `crates/liquidfun-wasm/src/scene/washing_machine.rs`
**Commit:** 711cce7bf
**Applied fix:** Renamed `parse_whole_rpm` to `parse_whole_number` in all callers. The doc now says it parses whole-number control tokens such as rpm or percent, and the local variable is now `number`. Behavior is unchanged. The screw, washing-machine and ferrofluid control tests pass (45).

### IN-02: The "allocates nothing after the first step" doc claims are inaccurate

**Files modified:** `crates/liquidfun-wasm/src/scene/particle_force.rs`, `crates/liquidfun-wasm/src/scene/fake_ferrofluid.rs`
**Commit:** 84c31112e
**Applied fix:** Both comments now say only that the lane copies are reused. The `ParticleScratch` doc also says that each force application still allocates in the engine, once per affected particle.

### IN-03: `apply_group_acceleration` fails above 65,535 members

**Files modified:** `crates/liquidfun-wasm/src/scene/particle_force.rs`
**Commit:** 96c4e5c02
**Applied fix:** Documented in `# Errors` that the function returns `StepFailed` for groups with more than `u16::MAX` (65,535) members. Behavior is unchanged.

### IN-04: `apply_accelerations` can leave forces half-applied when it fails

**Files modified:** `crates/liquidfun-wasm/src/scene/particle_force.rs`
**Commit:** 7d9c69495
**Applied fix:** Added to `# Errors`: "Forces already applied to earlier particles in this call are not rolled back."

### IN-05: The spike and flow suites repeat the same release-mode holds

**Files modified:** `crates/liquidfun-wasm/src/scene/fake_ferrofluid/spike_tests.rs`
**Commit:** 4aecb7d3b
**Applied fix:** `full_strength_hold` now caches one report per `HOLD_POSITIONS` entry in a static `[OnceLock<HoldReport>; 7]`. The five P3 hold tests and the P4 column-top test share those reports. `default_appearance` caches the single default-strength appearance run that the four P1/P2/P2b tests use. Each test still asserts one concern.

The release `fake_ferrofluid` suite went from 8.4 s to 3.9 s. I kept the session-level holds in `flow_tests`, because they run through the pointer seam and the clamp rather than setting the magnet directly.

## Verification

All of these were run at HEAD `4aecb7d3b`:
- `cargo fmt --all --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: clean.
- `cargo test -p liquidfun-wasm --release`: 818 passed and 12 ignored. This includes every fake-ferrofluid release test and the `particle_force` tests.
- `cargo test -p liquidfun-wasm` (debug): 746 passed and 83 ignored. The ignored ones are release-only tests.
- `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`: 83 passed.
- `cargo test -p liquidfun-wasm --release --lib particle_force`: 9 passed.
- `just scene-fingerprint-gate` (full capture, observed `target/dam-break-perf/2026-10-15T07-54-25Z/scene-spot.json`): PASS. 28/28 protected ids are unchanged, with none changed, missing or added. Fake-ferrofluid is `e73ef71f0615b4eb`, matching the baseline.
- `cd web && bun run typecheck`: clean.
- `cd web && bun run test:unit`: 121 files and 1856 tests passed.
- `bun scripts/bright-builds-check.ts all`: 0 findings. The largest touched file, `flow_tests.rs`, is 561 lines, under the 628-line limit.
- I did not run `just markdown-check`. The only Markdown I touched is under `.planning/**`, which is parser-owned and excluded from mdformat.

`.planning/config.json` was already modified before this run. I never staged or committed it.

---

_Fixed: 2026-10-15T08:20:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
