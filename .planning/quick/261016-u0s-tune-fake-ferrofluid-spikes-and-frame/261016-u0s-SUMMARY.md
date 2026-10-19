---
phase: quick-261016-u0s
plan: "01"
subsystem: playground-scene
tags: [fake-ferrofluid, tuning, view-bounds, readme-svg, scene-fingerprint]
generated_by: gsd-quick-orchestrator
lifecycle_mode: quick
phase_lifecycle_id: quick-261016-u0s
generated_at: "2026-10-17T03:20:00Z"
requires: []
provides:
  - Pointed fake-ferrofluid spikes about 0.6 m tall at the default strength
  - A 2.6 m dish and a 2.9 m frame top, down from 3.0 m and 3.6 m
affects:
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs
  - crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs
  - web/src/catalog/scene-view-bounds.ts
key-files:
  created: []
  modified:
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/field.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/geometry.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/test_support.rs
    - crates/liquidfun-wasm/src/scene/fake_ferrofluid/flow_tests.rs
    - web/src/catalog/scene-view-bounds.ts
    - web/tests/portrait-bounds.test.ts
    - docs/assets/readme/fake-ferrofluid-10s.svg
    - docs/assets/readme/fake-ferrofluid-10s.webp
    - docs/benchmarks/scene-fingerprints.json
decisions:
  - "The cap now exceeds gravity at the default strength, so a lift-equals-gravity contour that narrows with height shapes each column into a spike"
  - "Walls drop to 2.6 m and the existing one-reach rule caps the magnet at 0.6 m; liquid above the magnet is lifted, so the clamp height sets the frame"
  - "Default strength stays 65 percent and the column bump stays b^4; a b^8 bump made narrower but not taller columns"
metrics:
  completed: 2026-10-17
  tasks: 3
  files: 9
---

# Quick 261016-u0s: Tune fake-ferrofluid spikes and frame

## Outcome

At the default strength the crown is seven pointed spikes. They rise about
0.6 m above the troughs, up from about 0.28 m, and their tips sit near
0.93 m. The frame top drops from 3.6 m to 2.9 m (4.4 m to 3.7 m tall).

The brief assumed the 1.4 m magnet clamp would let the walls come down.
A probe showed the opposite. The lift also pushes liquid above the magnet
upward, so at full strength a column rises 1.2 to 1.5 m above a raised
magnet. With the magnet held at (1.0, 1.4), a column reached 2.87 m. That
broke the 0.3 m margin under the 3.0 m walls. The P4 test missed it
because its listed positions topped out at 2.68 m. A smaller frame
therefore needs a lower clamp. The walls are now 2.6 m, so
`MAGNET_MAX_Y = WALL_TOP_Y - FIELD.reach` caps the magnet at 0.6 m. That is
still above the settled puddle.

## Field

| Parameter | Before | After |
| --- | --- | --- |
| reach | 1.6 | 2.0 |
| strength_scale | 2.5 | 3.5 |
| cap | 1.5 | 2.5 |
| drag_per_second | 3.0 | 5.0 |
| wavelength, envelope, default strength | 0.65, 2.6, 65 % | unchanged |

With the old cap, the lift at 65 % was at most 0.975 g, below gravity.
Columns rose only because the surrounding puddle pressed them up, which gave
low rounded crests. With a cap of 2.5, the lift beats gravity inside a
region that narrows with height, which makes cones. The extra drag holds
the fastest full-strength speed near 3 m/s; the hold limit is 4 m/s.

## Probe evidence (release, temporary test, not committed)

| Measurement | Result |
| --- | --- |
| Old field, magnet clamp 1.4 m: tallest 60 s full-strength column | 2.866 m at (1.0, 1.4) |
| Old field, crest profile at step 600 | 7 peaks, tops 0.59 to 0.70, troughs about 0.42 |
| New field, crest profile at step 600 | 7 peaks, tallest tip 0.93 m, troughs about 0.32 |
| New field, 16 x 6 hold grid up to y 0.6: tallest column | 2.265 m at (1.5, 0.6), within 2.6 - 0.3 |
| New field, hold grid: fastest particle | 3.03 m/s |
| Rejected: cap 2.5 at drag 3 with a 1.2 m magnet | 4.6 m/s, failed the 4 m/s hold rule |
| Rejected: b^8 column bump | narrower, not taller (tops 0.80 at cap 2.5) |

## Tasks

| Task | Name | Commit | Files |
| --- | --- | --- | --- |
| 1 | Probe column tops and sweep the field | (probe removed) | scratch only |
| 2 | Apply field, walls, frames and hold positions | 6b0fcb34e | field.rs, geometry.rs, test_support.rs, flow_tests.rs, scene-view-bounds.ts, portrait-bounds.test.ts |
| 3a | Refresh the README clip | 43f84a605 | fake-ferrofluid-10s.svg, fake-ferrofluid-10s.webp |
| 3b | Re-pin the fingerprint baseline alone | 4d8bff99a | scene-fingerprints.json |

## Verification

| Check | Result |
| --- | --- |
| `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid` | 83 passed, 0 failed (4+ peaks by step 12, 7 at step 180, collapse, re-rise, 7 full-strength holds, drag paths, gravity 2 and 80, no evictions) |
| `cargo fmt --all --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo build --all-targets --all-features`; `cargo test --all-features` | pass; 1084 tests passed, 0 failed |
| `bun run typecheck`; `bunx vitest run` | pass; 1856 tests passed |
| `bun run test:player`; `playwright test e2e/hud-safe-frame.spec.ts` | 45 passed; 74 passed |
| `bun scripts/bright-builds-check.ts all` | findings=0 |
| `just readme-svg` (DejaVu font on macOS) | frames 61, 351, 471 and 591 reviewed: spikes, flat after off, spikes again after on |
| `just scene-fingerprint-gate --spot target/dam-break-perf/2026-10-19T00-30-27Z/scene-spot.json` before the re-pin | 27/28 unchanged; only fake-ferrofluid changed |
| Same gate after the re-pin | PASS, 28/28 unchanged |

## Residual risks and notes

- A phone portrait canvas is width-bound, so the shorter frame does not
  enlarge the scene there. The fitted frame now covers 0.218 of the phone
  height (the test floor drops from 0.25 to 0.21). On a phone the gain comes
  from the taller spikes: about 0.6 m of crest at about 54 px/m, roughly
  32 px instead of 15 px.
- A dragged magnet now stops at 0.6 m. Pressing higher on the canvas
  clamps it there, and the old tall-tower interaction above the puddle is
  gone.
- The column-top margin is measured, not proven: 2.265 m against a
  2.3 m limit on the hold grid.
