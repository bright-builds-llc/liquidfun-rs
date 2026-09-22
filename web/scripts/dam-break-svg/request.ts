import { maybeSceneById } from "../../src/catalog/scenes";
import {
  DEFAULT_SVG_EXPORT_SECONDS,
  svgExportFileName,
} from "../../src/export/duration";
import type { SvgExportRequest } from "../../src/export/messages";
import { IDENTITY_CAMERA_VIEW } from "../../src/render/camera";
import { DEFAULT_RENDER_MODE } from "../../src/render/mode";
import { DEFAULT_RENDERED_PARTICLE_LIMIT } from "../../src/render/particle-limit";
import { DEFAULT_WIREFRAME_STROKE_WIDTH } from "../../src/render/stroke-width";
import { CAPTURE_PROFILE } from "../demo-media/model";

/** README clip length. This is the playground export's default duration. */
export const DAM_BREAK_README_SECONDS = 10;

/** Repo path of the committed Dam Break loop. */
export const DAM_BREAK_SVG_REPO_PATH = `docs/assets/demos/${svgExportFileName("dam-break", DAM_BREAK_README_SECONDS)}`;

/**
 * Canonical Dam Break export for the README.
 *
 * Scene controls stay empty so the clip uses the scene's medium water and
 * normal gravity. The frame matches the demo gallery capture viewport, and
 * the camera, wireframe, and particle cap match a fresh playground session.
 */
export function damBreakSvgRequest(): SvgExportRequest {
  if (DAM_BREAK_README_SECONDS !== DEFAULT_SVG_EXPORT_SECONDS) {
    throw new Error(
      "The README Dam Break clip must stay on the playground's default export duration.",
    );
  }

  const maybeScene = maybeSceneById("dam-break");
  if (maybeScene === undefined) {
    throw new Error("Dam Break is missing from the scene catalog.");
  }

  return {
    sceneId: maybeScene.id,
    title: maybeScene.title,
    durationSeconds: DAM_BREAK_README_SECONDS,
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
