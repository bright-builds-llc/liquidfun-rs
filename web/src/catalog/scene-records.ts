import { GRAVITY_CONTROL, withGravitySlider } from "./gravity-slider";
import {
  PARTICLE_GUIDE,
  PINNED_DRAWING_PARTICLES_H,
  PINNED_DRAWING_PARTICLES_JS,
  PINNED_ELASTIC_PARTICLES_H,
  PINNED_ELASTIC_PARTICLES_JS,
  PINNED_FAUCET,
  PINNED_IMPULSE_H,
  PINNED_IMPULSE_JS,
  PINNED_LIQUID_TIMER_H,
  PINNED_LIQUID_TIMER_JS,
  PINNED_PARTICLES_H,
  PINNED_PARTICLES_JS,
  PINNED_RIGID_PARTICLES_H,
  PINNED_RIGID_PARTICLES_JS,
  PINNED_SOUP_H,
  PINNED_SOUP_JS,
  PINNED_SOUP_STIRRER_H,
  PINNED_SOUP_STIRRER_JS,
  PINNED_SPARKY_H,
  PINNED_SPARKY_JS,
  PINNED_SURFACE_TENSION_H,
  PINNED_SURFACE_TENSION_JS,
  PINNED_THEO_JANSEN_H,
  PINNED_THEO_JANSEN_JS,
  PINNED_WAVE_MACHINE_H,
  PINNED_WAVE_MACHINE_JS,
  SHOWCASE,
  WATCH_FIRST_HINT,
  action,
  option,
  runtimePreset,
  sceneSource,
} from "./scene-record-shared";
import { GAP_CONTROL } from "./hydraulic-fountain-gap";
import { DRUM_SPEED_CONTROL } from "./washing-machine-speed";
import {
  FLOW_DIRECTION_CONTROL,
  FLOW_RATE_CONTROL,
} from "./tesla-valve-controls";
import {
  WAVE_MACHINE_SPEED_CONTROL,
  WAVE_MACHINE_TILT_CONTROL,
} from "./wave-machine-speed";
import {
  WAVE_TANK_AMPLITUDE_CONTROL,
  WAVE_TANK_SLANT_CONTROL,
  WAVE_TANK_SPEED_CONTROL,
  WAVE_TANK_WIDTH_CONTROL,
} from "./wave-tank-controls";
import type { SceneRecord } from "./scenes";

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
 * Camera frame for Theo Jansen, in meters.
 *
 * The walker starts near the origin, with legs out to about x = ±7.2 and a
 * particle slab at y = 15. Its motor walks it at about a meter per second, so
 * a 12 m frame lets it leave immediately. This rectangle keeps the machine,
 * the slab, and a stretch of ground balls in view at the start.
 */
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
  maxY: 4.15,
} as const;

export const THEO_JANSEN_VIEW_BOUNDS = {
  minX: -24,
  minY: -0.5,
  maxX: 24,
  maxY: 16.4,
} as const;

export const SCENES: readonly SceneRecord[] = [
  {
    id: "wave-machine",
    title: "Wave Machine",
    ready: true,
    description: "Watch a motorized tank rock and slosh the water inside.",
    interactionHint:
      "Use Wave speed and Wave tilt to rock the tank. Speed starts at the original rate, and tilt starts at 9°, the original angle. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      WAVE_MACHINE_SPEED_CONTROL,
      WAVE_MACHINE_TILT_CONTROL,
    ]),
    viewBounds: WAVE_MACHINE_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("wave_machine.rs"),
      inspiration: [PINNED_WAVE_MACHINE_JS, PINNED_WAVE_MACHINE_H, SHOWCASE],
    },
  },
  {
    id: "dam-break",
    title: "Dam Break",
    ready: true,
    description: "Release a block of water into a basin and drop one obstacle.",
    interactionHint:
      "Drag the obstacle to a new place in the basin. Labeled controls also work from the keyboard.",
    controls: [
      {
        id: "water-amount",
        label: "Water amount",
        kind: "preset",
        recreates: true,
        values: [
          option("small", "Small"),
          option("medium", "Medium"),
          option("large", "Large"),
        ],
      },
      GRAVITY_CONTROL,
      action("drop-obstacle", "Drop obstacle"),
      action("reset-obstacle", "Reset obstacle"),
    ],
    credits: {
      implementationPath: sceneSource("dam_break.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "fountain",
    title: "Fountain",
    ready: true,
    description: "Aim a bounded stream into a bowl until particle count plateaus.",
    interactionHint:
      "Drag on the canvas to aim the stream. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("emission-rate", "Emission rate", [
        option("off", "Off"),
        option("low", "Low"),
        option("medium", "Medium"),
        option("high", "High"),
      ]),
      runtimePreset("launch-speed", "Launch speed", [
        option("slow", "Slow"),
        option("medium", "Medium"),
        option("fast", "Fast"),
      ]),
      runtimePreset("aim-angle", "Aim angle", [
        option("left", "Left"),
        option("up", "Up"),
        option("right", "Right"),
      ]),
    ]),
    credits: {
      implementationPath: sceneSource("fountain.rs"),
      inspiration: [PINNED_FAUCET, SHOWCASE],
    },
  },
  {
    id: "float-or-sink",
    title: "Float or Sink",
    ready: true,
    description:
      "Drop cork, wood, or stone into a pool and watch native body response.",
    interactionHint:
      "Click or tap the canvas to drop the selected body at that horizontal position. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("body", "Body", [
        option("cork", "Cork"),
        option("wood", "Wood"),
        option("stone", "Stone"),
      ]),
      action("drop-body", "Drop body"),
    ]),
    credits: {
      implementationPath: sceneSource("float_or_sink.rs"),
      inspiration: [PARTICLE_GUIDE, SHOWCASE],
    },
  },
  {
    id: "color-mixer",
    title: "Color Mixer",
    ready: true,
    description:
      "Stir two colored groups and watch contact-driven particle-color mixing.",
    interactionHint:
      "Drag on the canvas to stir the colored groups. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      {
        id: "mix-strength",
        label: "Mix strength",
        kind: "preset",
        recreates: true,
        values: [
          option("off", "Off"),
          option("gentle", "Gentle"),
          option("strong", "Strong"),
        ],
      },
      runtimePreset("stir-speed", "Stir speed", [
        option("off", "Off"),
        option("slow", "Slow"),
        option("fast", "Fast"),
      ]),
    ]),
    credits: {
      implementationPath: sceneSource("color_mixer.rs"),
      inspiration: [PARTICLE_GUIDE, SHOWCASE],
    },
  },
  {
    id: "jelly-drop",
    title: "Jelly Drop",
    ready: true,
    description: "Drop an elastic particle shape onto obstacles, then poke it.",
    interactionHint:
      "Click or tap the canvas to poke the jelly at that location. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      {
        id: "shape",
        label: "Shape",
        kind: "preset",
        recreates: true,
        values: [option("circle", "Circle"), option("square", "Square")],
      },
      {
        id: "softness",
        label: "Softness",
        kind: "preset",
        recreates: true,
        values: [
          option("soft", "Soft"),
          option("medium", "Medium"),
          option("firm", "Firm"),
        ],
      },
      action("poke-jelly", "Poke jelly"),
    ]),
    credits: {
      implementationPath: sceneSource("jelly_drop.rs"),
      inspiration: [PARTICLE_GUIDE, SHOWCASE],
    },
  },
  {
    id: "water-wheel",
    title: "Water Wheel",
    ready: true,
    description:
      "Vary a jet that turns a pinned paddle wheel through native coupling.",
    interactionHint:
      "Drag on the canvas to aim the jet. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("jet-strength", "Jet strength", [
        option("weak", "Weak"),
        option("medium", "Medium"),
        option("strong", "Strong"),
      ]),
      runtimePreset("emission", "Emission", [
        option("on", "On"),
        option("off", "Off"),
      ]),
    ]),
    credits: {
      implementationPath: sceneSource("water_wheel.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "particles",
    title: "Particles",
    ready: true,
    description:
      "Watch water fall in an open basin while a ball drops into it.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("particles.rs"),
      inspiration: [PINNED_PARTICLES_JS, PINNED_PARTICLES_H, SHOWCASE],
    },
  },
  {
    id: "liquid-timer",
    title: "Liquid Timer",
    ready: true,
    description:
      "Watch tensile, viscous liquid drain through shelves into bottom columns.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("liquid_timer.rs"),
      inspiration: [PINNED_LIQUID_TIMER_JS, PINNED_LIQUID_TIMER_H, SHOWCASE],
    },
  },
  {
    id: "surface-tension",
    title: "Surface Tension",
    ready: true,
    description:
      "Watch three colored tensile groups bead and bleed color when a ball hits them.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("surface_tension.rs"),
      inspiration: [
        PINNED_SURFACE_TENSION_JS,
        PINNED_SURFACE_TENSION_H,
        SHOWCASE,
      ],
    },
  },
  {
    id: "elastic-particles",
    title: "Elastic Particles",
    ready: true,
    description:
      "Watch three soft particle clumps deform when a ball falls on them.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("elastic_particles.rs"),
      inspiration: [
        PINNED_ELASTIC_PARTICLES_JS,
        PINNED_ELASTIC_PARTICLES_H,
        SHOWCASE,
      ],
    },
  },
  {
    id: "rigid-particles",
    title: "Rigid Particles",
    ready: true,
    description:
      "Watch three colored rigid clumps stay solid when a ball hits them.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("rigid_particles.rs"),
      inspiration: [
        PINNED_RIGID_PARTICLES_JS,
        PINNED_RIGID_PARTICLES_H,
        SHOWCASE,
      ],
    },
  },
  {
    id: "soup",
    title: "Soup",
    ready: true,
    description: "Watch a basin of liquid hold floating solid bits.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    credits: {
      implementationPath: sceneSource("soup.rs"),
      inspiration: [PINNED_SOUP_JS, PINNED_SOUP_H, SHOWCASE],
    },
  },
  {
    id: "soup-stirrer",
    title: "Soup Stirrer",
    ready: true,
    description: "Watch a paddle stir soup, and free or restore its rail.",
    interactionHint:
      "Click or tap the canvas, or use Toggle paddle rail, to free the paddle from its rail or put it back. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      action("toggle-paddle-rail", "Toggle paddle rail"),
    ]),
    credits: {
      implementationPath: sceneSource("soup_stirrer.rs"),
      inspiration: [PINNED_SOUP_STIRRER_JS, PINNED_SOUP_STIRRER_H, SHOWCASE],
    },
  },
  {
    id: "impulse",
    title: "Impulse",
    ready: true,
    description:
      "Click or tap inside the box to shove the whole particle blob.",
    interactionHint:
      "Click or tap inside the box to shove the particle blob. Use Push to choose force or impulse. Clicks outside the box do nothing. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("push-mode", "Push", [
        option("force", "Force"),
        option("impulse", "Impulse"),
      ]),
    ]),
    credits: {
      implementationPath: sceneSource("impulse.rs"),
      inspiration: [PINNED_IMPULSE_JS, PINNED_IMPULSE_H, SHOWCASE],
    },
  },
  {
    id: "theo-jansen",
    title: "Theo Jansen",
    ready: true,
    description:
      "Watch a walker move under a particle load and reverse its motor.",
    interactionHint:
      "Use Motor direction to walk forward or reverse under the particle load. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("motor-direction", "Motor direction", [
        option("forward", "Forward"),
        option("reverse", "Reverse"),
      ]),
    ]),
    viewBounds: THEO_JANSEN_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("theo_jansen.rs"),
      inspiration: [PINNED_THEO_JANSEN_JS, PINNED_THEO_JANSEN_H, SHOWCASE],
    },
  },
  {
    id: "liquid-tumbler",
    title: "Liquid Tumbler",
    ready: true,
    description:
      "A drinking glass at real size: 74 mm wide, 1.05 mm diameter particles, and Earth gravity. Phone tilt drives this water on a glass clock.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    viewBounds: LIQUID_TUMBLER_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("liquid_tumbler.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "drawing-particles",
    title: "Drawing Particles",
    ready: true,
    description:
      "Paint into an empty vessel, including elastic paint that clumps instead of flowing like water.",
    interactionHint:
      "Drag on the canvas to paint into the vessel. A click with no move leaves one stamp. Use Material to paint Water or Elastic. Elastic paint clumps instead of flowing like water. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      runtimePreset("material", "Material", [
        option("water", "Water"),
        option("elastic", "Elastic"),
      ]),
    ]),
    viewBounds: {
      minX: -4.2,
      minY: -2.2,
      maxX: 4.2,
      maxY: 6.2,
    },
    credits: {
      implementationPath: sceneSource("drawing_particles.rs"),
      inspiration: [
        PINNED_DRAWING_PARTICLES_JS,
        PINNED_DRAWING_PARTICLES_H,
        SHOWCASE,
      ],
    },
  },
  {
    id: "sparky",
    title: "Sparky",
    ready: true,
    description: "Watch colliding circles throw fading particle sparks.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    viewBounds: {
      minX: -22,
      minY: -1,
      maxX: 22,
      maxY: 42,
    },
    credits: {
      implementationPath: sceneSource("sparky.rs"),
      inspiration: [PINNED_SPARKY_JS, PINNED_SPARKY_H, SHOWCASE],
    },
  },
  {
    id: "hydraulic-fountain",
    title: "Hydraulic Fountain",
    ready: true,
    description:
      "Watch two raised platforms ease down into a pool, pause, and squeeze a jet of water up through the gap between them. This is an original experimental scene.",
    interactionHint:
      "Use Gap to set the opening between the platforms. It starts at 9 cm. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([GAP_CONTROL]),
    viewBounds: {
      minX: -2.6,
      minY: -0.28,
      maxX: 2.6,
      maxY: 3.25,
    },
    credits: {
      implementationPath:
        "crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs",
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "wave-tank",
    title: "Wave Tank",
    ready: true,
    description:
      "Watch a still pool whose end platform rises and falls and sends a wave toward the far wall. This is an original experimental scene.",
    interactionHint:
      "Use Platform width, Platform slant, Platform speed, and Platform amplitude to drive the wave. Width starts at 0.92 m. Slant starts at 5 degrees. Speed starts at 0.4× the original rate, and amplitude starts at 0.168 m. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      WAVE_TANK_WIDTH_CONTROL,
      WAVE_TANK_SLANT_CONTROL,
      WAVE_TANK_SPEED_CONTROL,
      WAVE_TANK_AMPLITUDE_CONTROL,
    ]),
    viewBounds: {
      minX: -0.12,
      minY: -0.16,
      maxX: 4.16,
      maxY: 1.04,
    },
    credits: {
      implementationPath: "crates/liquidfun-wasm/src/scene/wave_tank.rs",
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "liquid-bubbler",
    title: "Liquid Bubbler",
    ready: true,
    description:
      "Watch colored liquid drip through three shelves and turn a small wheel under each hole. This is an original experimental scene.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    viewBounds: { minX: -0.71, minY: -0.16, maxX: 1.12, maxY: 2.48 },
    credits: {
      implementationPath: "crates/liquidfun-wasm/src/scene/liquid_bubbler.rs",
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "stacked-drip",
    title: "Stacked Drip",
    ready: true,
    description:
      "Watch colored liquid drain through three tipping trays. This is an original experimental scene.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    viewBounds: { minX: -0.86, minY: -0.16, maxX: 1.68, maxY: 2.18 },
    credits: {
      implementationPath: "crates/liquidfun-wasm/src/scene/stacked_drip.rs",
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "washing-machine",
    title: "Washing Machine",
    ready: true,
    description:
      "Watch ribs on a spinning drum tumble water and a few elastic socks. This is an original experimental scene.",
    interactionHint:
      "Use Drum speed to change how fast the drum turns. It starts at 20 rpm. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([DRUM_SPEED_CONTROL]),
    viewBounds: WASHING_MACHINE_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("washing_machine.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "tesla-valve",
    title: "Tesla Valve",
    ready: true,
    description:
      "Watch water follow a winding pipe with semicircular bypasses around solid splitter islands. The straight runs and circular bends share a 6 cm channel width. Reverse flips the valve so the branches redirect water against the incoming stream. A source at the top keeps pouring, and a drain at the bottom removes what gets through. This is an original experimental scene.",
    interactionHint:
      "Use Flow rate to change how fast water pours in at the top, and Flow direction to flip the valve. It starts forward. Particles that reach the bottom drain are removed. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([FLOW_RATE_CONTROL, FLOW_DIRECTION_CONTROL]),
    viewBounds: TESLA_VALVE_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("tesla_valve.rs"),
      inspiration: [SHOWCASE],
    },
  },
];
