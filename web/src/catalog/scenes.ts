import { WORLD_BOUNDS, type WorldBounds } from "../render/camera";
import { WAVE_MACHINE_SPEED_CONTROL } from "./wave-machine-speed";

export const SCENE_IDS = [
  "wave-machine",
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
  "particles",
  "liquid-timer",
  "surface-tension",
  "elastic-particles",
  "rigid-particles",
  "soup",
  "soup-stirrer",
  "impulse",
  "theo-jansen",
  "liquid-tumbler",
] as const;

export type SceneId = (typeof SCENE_IDS)[number];

export type SceneControl =
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "preset";
      readonly recreates: boolean;
      readonly values: readonly { readonly id: string; readonly label: string }[];
    }
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "range";
      readonly recreates: boolean;
      readonly min: number;
      readonly max: number;
      readonly step: number;
      readonly defaultValue: number;
      readonly unit: string;
      readonly scale: "linear" | "logarithmic";
      readonly ticks: readonly number[];
      /** `hud` draws the slider above the play row. Other controls stay in the panel. */
      readonly surface?: "hud";
    }
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "action";
      readonly recreates: false;
    };

/** Former Dam Break Low gravity, in m/s² downward. */
export const DAM_BREAK_GRAVITY_MIN = 6;
/** Former Dam Break High gravity, in m/s² downward. */
export const DAM_BREAK_GRAVITY_FORMER_HIGH = 16;
/** Five times the former High gravity. Keep in sync with the wasm slider cap. */
export const DAM_BREAK_GRAVITY_MAX = DAM_BREAK_GRAVITY_FORMER_HIGH * 5;
export const DAM_BREAK_GRAVITY_STEP = 1;
/** Documented Dam Break normal gravity, in m/s² downward. */
export const DAM_BREAK_GRAVITY_DEFAULT = 10;
/** Labeled marks on the logarithmic Dam Break gravity slider. */
export const DAM_BREAK_GRAVITY_TICKS = [
  DAM_BREAK_GRAVITY_MIN,
  DAM_BREAK_GRAVITY_DEFAULT,
  DAM_BREAK_GRAVITY_FORMER_HIGH,
  DAM_BREAK_GRAVITY_MAX,
] as const;

export type SceneCredits = {
  readonly implementationPath: string;
  readonly inspiration: readonly { readonly label: string; readonly href: string }[];
};

export type SceneRecord = {
  readonly id: SceneId;
  readonly title: string;
  readonly ready: boolean;
  readonly description: string;
  readonly interactionHint: string;
  readonly controls: readonly SceneControl[];
  readonly credits: SceneCredits;
  /**
   * World rectangle fitted to the canvas.
   *
   * Omitted scenes use the shared 12 m by 9 m frame.
   */
  readonly viewBounds?: WorldBounds;
};

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
 * The tank is about 4 m wide around y = 1. The shared 12 m by 9 m frame leaves
 * it small on a phone, so this rectangle is about twice as tight and still
 * leaves room for the original tilt and splash.
 */
export const WAVE_MACHINE_VIEW_BOUNDS = {
  minX: -3,
  minY: -0.6,
  maxX: 3,
  maxY: 3.6,
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
      "Use Wave speed to rock the tank. It starts at the original speed and tilt, and higher speeds keep that tilt. Labeled controls also work from the keyboard.",
    controls: [WAVE_MACHINE_SPEED_CONTROL],
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
      {
        id: "gravity",
        label: "Gravity",
        kind: "range",
        recreates: true,
        min: DAM_BREAK_GRAVITY_MIN,
        max: DAM_BREAK_GRAVITY_MAX,
        step: DAM_BREAK_GRAVITY_STEP,
        defaultValue: DAM_BREAK_GRAVITY_DEFAULT,
        unit: "m/s²",
        scale: "logarithmic",
        ticks: DAM_BREAK_GRAVITY_TICKS,
      },
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
    controls: [
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
    ],
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
    controls: [
      runtimePreset("body", "Body", [
        option("cork", "Cork"),
        option("wood", "Wood"),
        option("stone", "Stone"),
      ]),
      action("drop-body", "Drop body"),
    ],
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
    controls: [
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
    ],
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
    controls: [
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
    ],
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
    controls: [
      runtimePreset("jet-strength", "Jet strength", [
        option("weak", "Weak"),
        option("medium", "Medium"),
        option("strong", "Strong"),
      ]),
      runtimePreset("emission", "Emission", [
        option("on", "On"),
        option("off", "Off"),
      ]),
    ],
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
    controls: [],
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
    controls: [],
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
    controls: [],
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
    controls: [],
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
    controls: [],
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
    controls: [],
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
    controls: [action("toggle-paddle-rail", "Toggle paddle rail")],
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
    controls: [
      runtimePreset("push-mode", "Push", [
        option("force", "Force"),
        option("impulse", "Impulse"),
      ]),
    ],
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
    controls: [
      runtimePreset("motor-direction", "Motor direction", [
        option("forward", "Forward"),
        option("reverse", "Reverse"),
      ]),
    ],
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
      "A drinking glass at real size: 74 mm wide, 0.8 mm particles, and Earth gravity. Phone tilt drives this water on a glass clock.",
    interactionHint: WATCH_FIRST_HINT,
    controls: [],
    viewBounds: LIQUID_TUMBLER_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("liquid_tumbler.rs"),
      inspiration: [SHOWCASE],
    },
  },
];

export function maybeSceneById(id: string): SceneRecord | undefined {
  return SCENES.find((scene) => scene.id === id);
}

/** World rectangle the canvas fits for one catalog scene. */
export function worldBoundsForScene(id: SceneId): WorldBounds {
  return maybeSceneById(id)?.viewBounds ?? WORLD_BOUNDS;
}

export function isReadySceneId(id: SceneId): boolean {
  const maybeScene = maybeSceneById(id);
  return maybeScene?.ready === true;
}
