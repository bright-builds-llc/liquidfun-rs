/** Catalog records for the classic scenes, in catalog order. */
import { GRAVITY_CONTROL, withGravitySlider } from "./gravity-slider";
import {
  PARTICLE_GUIDE,
  PINNED_FAUCET,
  PINNED_WAVE_MACHINE_H,
  PINNED_WAVE_MACHINE_JS,
  SHOWCASE,
  action,
  option,
  runtimePreset,
  sceneSource,
} from "./scene-record-shared";
import { WAVE_MACHINE_VIEW_BOUNDS } from "./scene-view-bounds";
import {
  WAVE_MACHINE_SPEED_CONTROL,
  WAVE_MACHINE_TILT_CONTROL,
} from "./wave-machine-speed";
import type { SceneRecord } from "./scenes";

export const CLASSIC_SCENE_RECORDS: readonly SceneRecord[] = [
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
];
