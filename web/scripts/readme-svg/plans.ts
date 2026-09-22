import { maybeSceneById, SCENE_IDS, type SceneId } from "../../src/catalog/scenes";
import {
  DEFAULT_SVG_EXPORT_SECONDS,
  sampleCountForDuration,
  svgExportFileName,
} from "../../src/export/duration";
import type { SvgExportRequest } from "../../src/export/messages";
import { IDENTITY_CAMERA_VIEW } from "../../src/render/camera";
import { DEFAULT_RENDER_MODE } from "../../src/render/mode";
import { DEFAULT_RENDERED_PARTICLE_LIMIT } from "../../src/render/particle-limit";
import { DEFAULT_WIREFRAME_STROKE_WIDTH } from "../../src/render/stroke-width";
import { CAPTURE_PROFILE } from "../demo-media/model";

/** README clip length. This is the playground export's default duration. */
export const README_SVG_SECONDS = 10;

/**
 * One authored nudge applied while sampling a README clip.
 *
 * Watch-first scenes leave this empty. Scenes that stay still until a named
 * action or an in-bounds pointer use a cue so the gallery shows motion.
 */
export type ReadmeSvgCue =
  | {
      readonly atSample: number;
      readonly kind: "action";
      readonly name: string;
    }
  | {
      readonly atSample: number;
      readonly kind: "pointer-up";
      readonly worldX: number;
      readonly worldY: number;
    };

export type ReadmeSvgPlan = {
  readonly id: SceneId;
  readonly cues: readonly ReadmeSvgCue[];
};

/**
 * README scenes, in catalog order.
 *
 * Add a plan here when a scene joins `SCENE_IDS`. The coverage check rejects
 * a missing, extra, or reordered id. Add a cue when the default scene does
 * not move on its own.
 */
export const README_SVG_PLANS: readonly ReadmeSvgPlan[] = [
  { id: "dam-break", cues: [] },
  { id: "fountain", cues: [] },
  {
    id: "float-or-sink",
    cues: [{ atSample: 0, kind: "action", name: "drop-body" }],
  },
  { id: "color-mixer", cues: [] },
  { id: "jelly-drop", cues: [] },
  { id: "water-wheel", cues: [] },
  { id: "particles", cues: [] },
  { id: "liquid-timer", cues: [] },
  { id: "surface-tension", cues: [] },
  { id: "elastic-particles", cues: [] },
  { id: "rigid-particles", cues: [] },
  { id: "soup", cues: [] },
  { id: "soup-stirrer", cues: [] },
  {
    id: "impulse",
    cues: [{ atSample: 20, kind: "pointer-up", worldX: 1, worldY: 2 }],
  },
  { id: "wave-machine", cues: [] },
  { id: "theo-jansen", cues: [] },
];

/** Repo path of one committed README loop. */
export function readmeSvgRepoPath(sceneId: SceneId): string {
  return `docs/assets/readme/${svgExportFileName(sceneId, README_SVG_SECONDS)}`;
}

/**
 * Canonical export settings for one README scene.
 *
 * Controls stay empty so the clip uses the scene defaults. The frame matches
 * the demo gallery capture viewport, and the camera, wireframe, and particle
 * cap match a fresh playground session.
 */
export function readmeSvgRequest(plan: ReadmeSvgPlan): SvgExportRequest {
  if (README_SVG_SECONDS !== DEFAULT_SVG_EXPORT_SECONDS) {
    throw new Error(
      "README scene clips must stay on the playground's default export duration.",
    );
  }

  const maybeScene = maybeSceneById(plan.id);
  if (maybeScene === undefined || !maybeScene.ready) {
    throw new Error(`README SVG plan ${plan.id} is not a ready catalog scene.`);
  }

  return {
    sceneId: maybeScene.id,
    title: maybeScene.title,
    durationSeconds: README_SVG_SECONDS,
    controls: [],
    viewportWidth: CAPTURE_PROFILE.viewport.width,
    viewportHeight: CAPTURE_PROFILE.viewport.height,
    zoom: IDENTITY_CAMERA_VIEW.zoom,
    panX: IDENTITY_CAMERA_VIEW.panX,
    panY: IDENTITY_CAMERA_VIEW.panY,
    renderMode: DEFAULT_RENDER_MODE,
    wireframeStrokeWidth: DEFAULT_WIREFRAME_STROKE_WIDTH,
    maxRenderedParticles: DEFAULT_RENDERED_PARTICLE_LIMIT,
  };
}

/** Rejects a plan list that has drifted from the scene catalog. */
export function assertReadmeSvgPlanCoverage(
  plans: readonly ReadmeSvgPlan[] = README_SVG_PLANS,
): void {
  const planIds = plans.map((plan) => plan.id);
  if (planIds.length !== SCENE_IDS.length) {
    throw new Error(
      `README SVG plans length ${planIds.length} does not match SCENE_IDS length ${SCENE_IDS.length}`,
    );
  }

  for (let index = 0; index < SCENE_IDS.length; index += 1) {
    const expectedId = SCENE_IDS[index];
    const actualId = planIds[index];
    if (actualId !== expectedId) {
      throw new Error(
        `README SVG plan order mismatch at index ${index}: expected ${expectedId}, got ${actualId}`,
      );
    }
  }

  if (new Set(planIds).size !== planIds.length) {
    throw new Error("README SVG plans contain duplicate scene ids");
  }

  const sampleCount = sampleCountForDuration(README_SVG_SECONDS);
  for (const plan of plans) {
    const maybeScene = maybeSceneById(plan.id);
    if (maybeScene === undefined || !maybeScene.ready) {
      throw new Error(`README SVG plan ${plan.id} is not a ready catalog scene.`);
    }

    for (const cue of plan.cues) {
      if (!Number.isSafeInteger(cue.atSample) || cue.atSample < 0 || cue.atSample >= sampleCount) {
        throw new Error(
          `README SVG cue for ${plan.id} is outside the ${sampleCount} sample clip.`,
        );
      }
    }
  }
}
