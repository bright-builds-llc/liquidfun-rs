import { SCENE_IDS, type SceneId } from "../../src/catalog/scenes";

export type PointRatio = {
  readonly x: number;
  readonly y: number;
};

export type SceneAction =
  | { readonly kind: "click"; readonly point: PointRatio }
  | {
      readonly kind: "drag";
      readonly start: PointRatio;
      readonly end: PointRatio;
    };

export type SceneCapturePlan = {
  readonly id: SceneId;
  readonly title: string;
  readonly route: `/liquidfun-rs/#/scene/${SceneId}`;
  readonly interactionStep: 180;
  readonly action: SceneAction;
};

export type CaptureProfile = {
  readonly viewport: { readonly width: 1280; readonly height: 960 };
  readonly deviceScaleFactor: 1;
  readonly simulationHz: 60;
  readonly outputFps: 30;
  readonly durationSeconds: 8;
  readonly stepsPerFrame: 2;
  readonly frameCount: 240;
};

export type CaptureFontFingerprint = {
  readonly corpusVersion: 1;
  readonly corpus: string;
  readonly canvas: {
    readonly width: number;
    readonly height: number;
  };
  readonly panelComputedStyle: {
    readonly fontFamily: string;
    readonly fontStyle: string;
    readonly fontWeight: string;
  };
  readonly samples: readonly {
    readonly label: "regular" | "semibold" | "bold";
    readonly fontStyle: string;
    readonly fontWeight: string;
    readonly fontSizePx: number;
    readonly canvasFont: string;
  }[];
  readonly rasterSha256: string;
};

export type ManifestCaptureProfile = CaptureProfile & {
  readonly inputSha256: string;
  readonly platform: string;
  readonly playwrightPackageVersion: string;
  readonly chromiumRevision: string;
  readonly expectedChromiumVersion: string;
  readonly runtimeChromiumVersion: string;
  readonly ffmpegVersionLine: string;
  readonly captureFontFingerprint: CaptureFontFingerprint;
};

export type MediaFileRecord = {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
};

export type SceneMediaRecord = {
  readonly id: SceneId;
  readonly route: string;
  readonly interactionStep: number;
  readonly files: readonly MediaFileRecord[];
};

export type DemoMediaManifest = {
  readonly schemaVersion: 1;
  readonly captureProfile: ManifestCaptureProfile;
  readonly scenes: readonly SceneMediaRecord[];
};

export const CAPTURE_PROFILE: CaptureProfile = {
  viewport: { width: 1280, height: 960 },
  deviceScaleFactor: 1,
  simulationHz: 60,
  outputFps: 30,
  durationSeconds: 8,
  stepsPerFrame: 2,
  frameCount: 240,
};

export const SCENE_CAPTURE_PLANS: readonly SceneCapturePlan[] = [
  {
    id: "dam-break",
    title: "Dam Break",
    route: "/liquidfun-rs/#/scene/dam-break",
    interactionStep: 180,
    action: {
      kind: "drag",
      start: { x: 0.5, y: 0.45 },
      end: { x: 0.7, y: 0.35 },
    },
  },
  {
    id: "fountain",
    title: "Fountain",
    route: "/liquidfun-rs/#/scene/fountain",
    interactionStep: 180,
    action: {
      kind: "drag",
      start: { x: 0.35, y: 0.7 },
      end: { x: 0.7, y: 0.3 },
    },
  },
  {
    id: "float-or-sink",
    title: "Float or Sink",
    route: "/liquidfun-rs/#/scene/float-or-sink",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.35 } },
  },
  {
    id: "color-mixer",
    title: "Color Mixer",
    route: "/liquidfun-rs/#/scene/color-mixer",
    interactionStep: 180,
    action: {
      kind: "drag",
      start: { x: 0.35, y: 0.55 },
      end: { x: 0.65, y: 0.45 },
    },
  },
  {
    id: "jelly-drop",
    title: "Jelly Drop",
    route: "/liquidfun-rs/#/scene/jelly-drop",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.45 } },
  },
  {
    id: "water-wheel",
    title: "Water Wheel",
    route: "/liquidfun-rs/#/scene/water-wheel",
    interactionStep: 180,
    action: {
      kind: "drag",
      start: { x: 0.25, y: 0.65 },
      end: { x: 0.7, y: 0.4 },
    },
  },
  {
    id: "particles",
    title: "Particles",
    route: "/liquidfun-rs/#/scene/particles",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "liquid-timer",
    title: "Liquid Timer",
    route: "/liquidfun-rs/#/scene/liquid-timer",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "surface-tension",
    title: "Surface Tension",
    route: "/liquidfun-rs/#/scene/surface-tension",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "elastic-particles",
    title: "Elastic Particles",
    route: "/liquidfun-rs/#/scene/elastic-particles",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "rigid-particles",
    title: "Rigid Particles",
    route: "/liquidfun-rs/#/scene/rigid-particles",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "soup",
    title: "Soup",
    route: "/liquidfun-rs/#/scene/soup",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "soup-stirrer",
    title: "Soup Stirrer",
    route: "/liquidfun-rs/#/scene/soup-stirrer",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.55, y: 0.45 } },
  },
  {
    id: "impulse",
    title: "Impulse",
    route: "/liquidfun-rs/#/scene/impulse",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.45 } },
  },
  {
    id: "wave-machine",
    title: "Wave Machine",
    route: "/liquidfun-rs/#/scene/wave-machine",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "theo-jansen",
    title: "Theo Jansen",
    route: "/liquidfun-rs/#/scene/theo-jansen",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
  {
    id: "liquid-tumbler",
    title: "Liquid Tumbler",
    route: "/liquidfun-rs/#/scene/liquid-tumbler",
    interactionStep: 180,
    action: { kind: "click", point: { x: 0.5, y: 0.5 } },
  },
];

function assertSceneCapturePlanCoverage(): void {
  const planIds = SCENE_CAPTURE_PLANS.map((plan) => plan.id);
  if (planIds.length !== SCENE_IDS.length) {
    throw new Error(
      `SCENE_CAPTURE_PLANS length ${planIds.length} does not match SCENE_IDS length ${SCENE_IDS.length}`,
    );
  }
  for (let index = 0; index < SCENE_IDS.length; index += 1) {
    const expectedId = SCENE_IDS[index];
    const actualId = planIds[index];
    if (actualId !== expectedId) {
      throw new Error(
        `SCENE_CAPTURE_PLANS order mismatch at index ${index}: expected ${expectedId}, got ${actualId}`,
      );
    }
  }
  const uniquePlanIds = new Set(planIds);
  if (uniquePlanIds.size !== planIds.length) {
    throw new Error("SCENE_CAPTURE_PLANS contains duplicate scene ids");
  }
}

assertSceneCapturePlanCoverage();

export function frameFileName(frameIndex: number): string {
  return `frame-${String(frameIndex).padStart(4, "0")}.png`;
}

function sceneOrderIndex(id: SceneId): number {
  const index = SCENE_IDS.indexOf(id);
  if (index === -1) {
    throw new Error(`Unknown scene id in manifest: ${id}`);
  }
  return index;
}

export function canonicalManifestJson(manifest: DemoMediaManifest): string {
  const sortedScenes = [...manifest.scenes]
    .sort((left, right) => sceneOrderIndex(left.id) - sceneOrderIndex(right.id))
    .map((scene) => ({
      ...scene,
      files: [...scene.files].sort((left, right) =>
        left.path.localeCompare(right.path),
      ),
    }));

  const canonical: DemoMediaManifest = {
    schemaVersion: manifest.schemaVersion,
    captureProfile: manifest.captureProfile,
    scenes: sortedScenes,
  };

  return `${JSON.stringify(canonical, null, 2)}\n`;
}
