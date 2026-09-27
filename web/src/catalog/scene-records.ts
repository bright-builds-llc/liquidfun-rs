import { GRAVITY_CONTROL, withGravitySlider } from "./gravity-slider";
import {
  WAVE_MACHINE_SPEED_CONTROL,
  WAVE_MACHINE_TILT_CONTROL,
} from "./wave-machine-speed";
import type { SceneControl, SceneRecord } from "./scenes";

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
export const THEO_JANSEN_VIEW_BOUNDS = {
  minX: -24,
  minY: -0.5,
  maxX: 24,
  maxY: 16.4,
} as const;

const SHOWCASE = {
  label: "LiquidFun showcase",
  href: "https://google.github.io/liquidfun/",
} as const;

const PARTICLE_GUIDE = {
  label: "Particle guide",
  href: "https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html",
} as const;

const PINNED_FAUCET = {
  label: "Pinned Faucet",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Faucet.h",
} as const;

const PINNED_PARTICLES_JS = {
  label: "Pinned Particles.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testParticles.js",
} as const;

const PINNED_PARTICLES_H = {
  label: "Pinned Particles.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Particles.h",
} as const;

const PINNED_LIQUID_TIMER_JS = {
  label: "Pinned LiquidTimer.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testLiquidTimer.js",
} as const;

const PINNED_LIQUID_TIMER_H = {
  label: "Pinned LiquidTimer.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/LiquidTimer.h",
} as const;

const PINNED_SURFACE_TENSION_JS = {
  label: "Pinned SurfaceTension.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSurfaceTension.js",
} as const;

const PINNED_SURFACE_TENSION_H = {
  label: "Pinned ParticlesSurfaceTension.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/ParticlesSurfaceTension.h",
} as const;

const PINNED_ELASTIC_PARTICLES_JS = {
  label: "Pinned ElasticParticles.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testElasticParticles.js",
} as const;

const PINNED_ELASTIC_PARTICLES_H = {
  label: "Pinned ElasticParticles.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/ElasticParticles.h",
} as const;

const PINNED_RIGID_PARTICLES_JS = {
  label: "Pinned RigidParticles.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testRigidParticles.js",
} as const;

const PINNED_RIGID_PARTICLES_H = {
  label: "Pinned RigidParticles.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/RigidParticles.h",
} as const;

const PINNED_SOUP_JS = {
  label: "Pinned Soup.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSoup.js",
} as const;

const PINNED_SOUP_H = {
  label: "Pinned Soup.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Soup.h",
} as const;

const PINNED_SOUP_STIRRER_JS = {
  label: "Pinned SoupStirrer.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSoupStirrer.js",
} as const;

const PINNED_SOUP_STIRRER_H = {
  label: "Pinned SoupStirrer.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/SoupStirrer.h",
} as const;

const PINNED_IMPULSE_JS = {
  label: "Pinned Impulse.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testImpulse.js",
} as const;

const PINNED_IMPULSE_H = {
  label: "Pinned Impulse.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Impulse.h",
} as const;

const PINNED_WAVE_MACHINE_JS = {
  label: "Pinned WaveMachine.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testWaveMachine.js",
} as const;

const PINNED_WAVE_MACHINE_H = {
  label: "Pinned WaveMachine.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/WaveMachine.h",
} as const;

const PINNED_THEO_JANSEN_JS = {
  label: "Pinned TheoJansen.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testTheoJansen.js",
} as const;

const PINNED_THEO_JANSEN_H = {
  label: "Pinned TheoJansen.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/TheoJansen.h",
} as const;

const PINNED_DRAWING_PARTICLES_JS = {
  label: "Pinned DrawingParticles.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testDrawingParticles.js",
} as const;

const PINNED_DRAWING_PARTICLES_H = {
  label: "Pinned DrawingParticles.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/DrawingParticles.h",
} as const;

const PINNED_SPARKY_JS = {
  label: "Pinned Sparky.js",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/lfjs/testbed/tests/testSparky.js",
} as const;

const PINNED_SPARKY_H = {
  label: "Pinned Sparky.h",
  href: "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Sparky.h",
} as const;

const WATCH_FIRST_HINT =
  "This scene is watch-first. Use Play scene, Pause scene, and Reset scene.";

function option(
  id: string,
  label: string,
): { readonly id: string; readonly label: string } {
  return { id, label };
}

function runtimePreset(
  id: string,
  label: string,
  values: readonly { readonly id: string; readonly label: string }[],
): SceneControl {
  return { id, label, kind: "preset", recreates: false, values };
}

function action(id: string, label: string): SceneControl {
  return { id, label, kind: "action", recreates: false };
}

function sceneSource(fileName: string): string {
  return `crates/liquidfun-wasm/src/scene/${fileName}`;
}

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
      "Watch a timed piston squeeze one water reservoir so that liquid travels through a throat into the other chamber. This is an original experimental scene.",
    interactionHint: WATCH_FIRST_HINT,
    controls: withGravitySlider([]),
    viewBounds: {
      minX: -1.3,
      minY: -0.15,
      maxX: 1.3,
      maxY: 1.55,
    },
    credits: {
      implementationPath:
        "crates/liquidfun-wasm/src/scene/hydraulic_fountain.rs",
      inspiration: [SHOWCASE],
    },
  },
];
