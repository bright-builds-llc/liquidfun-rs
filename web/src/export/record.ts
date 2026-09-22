import type { RenderFrame } from "../physics/frame";
import { createCamera, type CameraView } from "../render/camera";
import { STEPS_PER_SVG_SAMPLE, sampleCountForDuration } from "./duration";
import { projectRenderSample, type ProjectedSample } from "./project";

export type ExportControl = {
  readonly name: string;
  readonly value: string;
};

/** Narrow session surface the export worker can step without touching the live canvas. */
export type ExportDriver = {
  applyControl(name: string, value: string): void;
  advance(stepCount: number): void;
  captureFrame(): RenderFrame;
};

export type SampleRecordingRequest = {
  readonly durationSeconds: number;
  readonly controls: readonly ExportControl[];
  readonly viewportWidth: number;
  readonly viewportHeight: number;
  readonly zoom: number;
  readonly panX: number;
  readonly panY: number;
  readonly maxRenderedParticles: number;
};

/**
 * Samples a fresh scene from its initial frame through the requested duration.
 *
 * The first sample is the constructed scene. Later samples are evenly spaced
 * at 20 Hz, which is every third 60 Hz engine step.
 */
export function recordProjectedSamples(
  driver: ExportDriver,
  request: SampleRecordingRequest,
  onSample: (completed: number, total: number) => void,
): readonly ProjectedSample[] {
  for (const control of request.controls) {
    driver.applyControl(control.name, control.value);
  }

  const camera = createCamera(request.viewportWidth, request.viewportHeight, cameraView(request));
  const total = sampleCountForDuration(request.durationSeconds);
  const samples: ProjectedSample[] = [];
  for (let index = 0; index < total; index += 1) {
    if (index > 0) {
      driver.advance(STEPS_PER_SVG_SAMPLE);
    }

    samples.push(
      projectRenderSample(driver.captureFrame(), camera, request.maxRenderedParticles),
    );
    onSample(index + 1, total);
  }

  return samples;
}

function cameraView(request: SampleRecordingRequest): CameraView {
  return {
    zoom: request.zoom,
    panX: request.panX,
    panY: request.panY,
  };
}
