import { worldBoundsForViewport } from "../../src/catalog/portrait-bounds";
import { buildAnimatedSvg } from "../../src/export/animated-svg";
import type { SvgExportRequest } from "../../src/export/messages";
import { projectRenderSample, type ProjectedSample } from "../../src/export/project";
import type { ExportDriver } from "../../src/export/record";
import { createCamera, type Camera } from "../../src/render/camera";
import { startAnimatedWebpEncoder, type AnimatedWebpEncoder } from "./encode";
import { createFramePainter, type FramePainter } from "./paint";
import { sceneClipSchedule, type SceneClipEvent } from "./schedule";
import {
  README_WEBP_FPS,
  README_WEBP_PRESET,
  README_WEBP_QUALITY,
} from "./webp";

export type RecordReadmePreviewArgs = {
  readonly driver: ExportDriver;
  readonly request: SvgExportRequest;
  readonly fontFile: string;
  readonly onProgress: (message: string) => void;
  readonly beforeSample: (sampleIndex: number) => void;
};

/**
 * Records one scene's SVG samples and its animated WebP from the same run.
 *
 * WebP frames are painted from the engine, not from the SVG. The SVG is still
 * built from the 20 Hz samples so the vector file stays in the repo.
 */
export async function recordReadmePreview(
  args: RecordReadmePreviewArgs,
): Promise<{ readonly svg: string; readonly webp: Buffer }> {
  const { request } = args;
  for (const control of request.controls) {
    args.driver.applyControl(control.name, control.value);
  }

  const camera = createCamera(
    request.viewportWidth,
    request.viewportHeight,
    { zoom: request.zoom, panX: request.panX, panY: request.panY },
    worldBoundsForViewport(request.sceneId, request.viewportWidth, request.viewportHeight),
  );
  const painter = await createFramePainter(
    request.viewportWidth,
    request.viewportHeight,
    args.fontFile,
  );
  const schedule = sceneClipSchedule(request.durationSeconds);
  const webpFrameCount = countWebpFrames(schedule);
  const encoder = await startAnimatedWebpEncoder({
    width: request.viewportWidth,
    height: request.viewportHeight,
    frameCount: webpFrameCount,
    framesPerSecond: README_WEBP_FPS,
    quality: README_WEBP_QUALITY,
    preset: README_WEBP_PRESET,
  });

  try {
    const samples = await walkSchedule(args, camera, painter, encoder, schedule, webpFrameCount);
    const webp = await encoder.finish();
    const svg = buildAnimatedSvg({
      title: request.title,
      samples,
      durationSeconds: request.durationSeconds,
      viewportWidth: request.viewportWidth,
      viewportHeight: request.viewportHeight,
      renderMode: request.renderMode,
      wireframeStrokeWidth: request.wireframeStrokeWidth,
    });
    if (!svg.startsWith("<svg")) {
      throw new Error(`${request.sceneId} export did not produce an SVG document.`);
    }
    return { svg, webp };
  } catch (error) {
    await encoder.abort();
    throw error;
  }
}

function countWebpFrames(schedule: readonly SceneClipEvent[]): number {
  let count = 0;
  for (const event of schedule) {
    if (event.kind === "webp-frame") {
      count += 1;
    }
  }
  return count;
}

async function walkSchedule(
  args: RecordReadmePreviewArgs,
  camera: Camera,
  painter: FramePainter,
  encoder: AnimatedWebpEncoder,
  schedule: readonly SceneClipEvent[],
  webpTotal: number,
): Promise<readonly ProjectedSample[]> {
  const samples: ProjectedSample[] = [];
  let webpCount = 0;

  for (const event of schedule) {
    if (event.kind === "cue") {
      args.beforeSample(event.sampleIndex);
      continue;
    }
    if (event.kind === "advance") {
      args.driver.advance(1);
      continue;
    }
    if (event.kind === "svg-sample") {
      samples.push(
        projectRenderSample(
          args.driver.captureFrame(),
          camera,
          args.request.maxRenderedParticles,
        ),
      );
      continue;
    }

    const pixels = painter.paint(
      args.driver.captureFrame(),
      camera,
      args.request.renderMode,
      args.request.wireframeStrokeWidth,
      args.request.maxRenderedParticles,
    );
    await encoder.writeFrame(pixels);
    webpCount += 1;
    if (webpCount === 1 || webpCount === webpTotal || webpCount % 60 === 0) {
      args.onProgress(`${args.request.sceneId} recorded ${webpCount} of ${webpTotal}`);
    }
  }

  return samples;
}
