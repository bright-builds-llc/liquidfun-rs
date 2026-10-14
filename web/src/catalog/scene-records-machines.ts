/** Catalog records for the machine scenes, in catalog order. */
import { withGravitySlider } from "./gravity-slider";
import { GAP_CONTROL } from "./hydraulic-fountain-gap";
import { SHOWCASE, WATCH_FIRST_HINT, sceneSource } from "./scene-record-shared";
import {
  TESLA_VALVE_VIEW_BOUNDS,
  WASHING_MACHINE_VIEW_BOUNDS,
} from "./scene-view-bounds";
import {
  FLOW_DIRECTION_CONTROL,
  FLOW_RATE_CONTROL,
} from "./tesla-valve-controls";
import { DRUM_SPEED_CONTROL } from "./washing-machine-speed";
import {
  WAVE_TANK_AMPLITUDE_CONTROL,
  WAVE_TANK_SLANT_CONTROL,
  WAVE_TANK_SPEED_CONTROL,
  WAVE_TANK_WIDTH_CONTROL,
} from "./wave-tank-controls";
import type { SceneRecord } from "./scenes";

export const MACHINE_SCENE_RECORDS: readonly SceneRecord[] = [
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
      "Watch water pass through a Tesla Valve whose outer loop walls continue into the next pipe section around solid splitter islands. The straight runs and circular bends share a 6 cm channel width. Reverse flips the valve so the branches redirect water against the incoming stream. A source at the top keeps pouring, and a drain at the bottom removes what gets through. This is an original experimental scene.",
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
