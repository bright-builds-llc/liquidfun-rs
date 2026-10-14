/** Camera frames for catalog scenes whose action needs a custom world rectangle. */

/**
 * Camera frame for Archimedes Screw, in meters.
 *
 * Keep in sync with `FRAME_*` in
 * `crates/liquidfun-wasm/src/scene/archimedes_screw/geometry.rs`. This is the
 * bounding box of every static wall (reservoir, tube housings, basin, weir,
 * chute and hood) plus about 0.2 m, so every wall stays in view.
 */
export const ARCHIMEDES_SCREW_VIEW_BOUNDS = {
  minX: -6.1,
  minY: -2.0,
  maxX: 2.65,
  maxY: 4.45,
} as const;

/**
 * Camera frame for Cartesian Diver, in meters.
 *
 * Keep in sync with `crates/liquidfun-wasm/src/scene/cartesian_diver/geometry.rs`.
 * This is the physical outer wall rectangle, which is the drawn bottle outline
 * (x = ±1.45, y = -0.25..5.85) plus 0.15 m, so every wall stays in view.
 */
export const CARTESIAN_DIVER_VIEW_BOUNDS = {
  minX: -1.6,
  minY: -0.4,
  maxX: 1.6,
  maxY: 6.0,
} as const;

/**
 * Camera frame for Liquid Tumbler, in meters.
 *
 * Keep in sync with the glass comments in
 * `crates/liquidfun-wasm/src/scene/liquid_tumbler.rs`. The cup is
 * x = ±0.037 and y = 0..0.12; this rectangle leaves a small margin.
 */
export const LIQUID_TUMBLER_VIEW_BOUNDS = {
  minX: -0.055,
  minY: -0.02,
  maxX: 0.055,
  maxY: 0.15,
} as const;

/**
 * Camera frame for Wave Machine, in meters.
 *
 * Matches the tank in `wave_machine.rs` (about x = ±2.05, y = -0.05..2.05
 * around (0, 1)) plus the default 9° rock and a slim margin.
 */
export const WAVE_MACHINE_VIEW_BOUNDS = {
  minX: -2.3,
  minY: -0.47,
  maxX: 2.3,
  maxY: 2.47,
} as const;

/**
 * Camera frame for the washing machine, in meters.
 *
 * The drum wall runs out to about 1.48 m. This square leaves a slim margin
 * so the spinning wall stays on the canvas.
 */
export const WASHING_MACHINE_VIEW_BOUNDS = {
  minX: -1.68,
  minY: -1.68,
  maxX: 1.68,
  maxY: 1.68,
} as const;

/**
 * Camera frame for the Tesla valve, in meters.
 *
 * Keep in sync with `FRAME_*` in
 * `crates/liquidfun-wasm/src/scene/tesla_valve/geometry.rs`. The alternating loops and
 * splitter islands run from the inlet near y = 4 down through the drain. This
 * rectangle includes those walls with a small margin.
 */
export const TESLA_VALVE_VIEW_BOUNDS = {
  minX: -0.7,
  minY: -0.12,
  maxX: 0.58,
  maxY: 4.32,
} as const;

/**
 * Camera frame for Theo Jansen, in meters.
 *
 * The walker starts near the origin, with legs out to about x = ±7.2 and a
 * particle slab at y = 15. Its motor walks it at about a meter per second, so
 * a 12 m frame lets it leave immediately. This rectangle keeps the machine,
 * the slab, and a stretch of ground balls in view at the start.
 */
export const THEO_JANSEN_VIEW_BOUNDS = {
  minX: -24,
  minY: -0.5,
  maxX: 24,
  maxY: 16.4,
} as const;
