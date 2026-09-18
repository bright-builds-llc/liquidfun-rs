export const SCENE_IDS = [
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
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
      readonly kind: "action";
      readonly recreates: false;
    };

export type SceneCredits = {
  readonly implementationPath: string;
  readonly inspiration: readonly { readonly label: string; readonly href: string }[];
};

export type SceneRecord = {
  readonly id: SceneId;
  readonly title: string;
  readonly ready: boolean;
  readonly description: string;
  readonly previewId: SceneId;
  readonly controls: readonly SceneControl[];
  readonly credits: SceneCredits;
};

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
    id: "dam-break",
    title: "Dam Break",
    ready: true,
    description: "Release a block of water into a basin and drop one obstacle.",
    previewId: "dam-break",
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
        kind: "preset",
        recreates: true,
        values: [
          option("low", "Low"),
          option("normal", "Normal"),
          option("high", "High"),
        ],
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
    previewId: "fountain",
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
    previewId: "float-or-sink",
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
    previewId: "color-mixer",
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
    previewId: "jelly-drop",
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
    previewId: "water-wheel",
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
];

export function maybeSceneById(id: string): SceneRecord | undefined {
  return SCENES.find((scene) => scene.id === id);
}

export function isReadySceneId(id: SceneId): boolean {
  const maybeScene = maybeSceneById(id);
  return maybeScene?.ready === true;
}
