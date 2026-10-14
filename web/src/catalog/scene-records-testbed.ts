/** Catalog records for the testbed scenes, in catalog order. */
import { withGravitySlider } from "./gravity-slider";
import {
  PINNED_DRAWING_PARTICLES_H,
  PINNED_DRAWING_PARTICLES_JS,
  PINNED_ELASTIC_PARTICLES_H,
  PINNED_ELASTIC_PARTICLES_JS,
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
  SHOWCASE,
  WATCH_FIRST_HINT,
  action,
  option,
  runtimePreset,
  sceneSource,
} from "./scene-record-shared";
import {
  LIQUID_TUMBLER_VIEW_BOUNDS,
  THEO_JANSEN_VIEW_BOUNDS,
} from "./scene-view-bounds";
import type { SceneRecord } from "./scenes";

export const TESTBED_SCENE_RECORDS: readonly SceneRecord[] = [
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
];
