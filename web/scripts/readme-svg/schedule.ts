import { STEPS_PER_SVG_SAMPLE, sampleCountForDuration } from "../../src/export/duration";
import { README_WEBP_FPS, frameCountForDuration } from "./webp";

/**
 * One step of a README preview recording.
 *
 * SVG samples stay on the existing 20 Hz clock, including the sample at the
 * loop point. WebP frames are the engine states at 60 Hz and stop one step
 * before that loop point, so a 10 second clip is 600 frames.
 */
export type SceneClipEvent =
  | { readonly kind: "cue"; readonly sampleIndex: number }
  | { readonly kind: "advance" }
  | { readonly kind: "webp-frame"; readonly index: number }
  | { readonly kind: "svg-sample"; readonly index: number };

/**
 * Orders cues, engine steps, WebP frames, and SVG samples for one clip.
 *
 * A cue at sample 0 runs before any step. A later cue runs immediately before
 * the three steps that produce that 20 Hz sample, matching the SVG recorder.
 */
export function sceneClipSchedule(durationSeconds: number): readonly SceneClipEvent[] {
  const sampleCount = sampleCountForDuration(durationSeconds);
  const webpFrameCount = frameCountForDuration(durationSeconds, README_WEBP_FPS);
  const events: SceneClipEvent[] = [];
  let webpIndex = 0;

  for (let sampleIndex = 0; sampleIndex < sampleCount; sampleIndex += 1) {
    events.push({ kind: "cue", sampleIndex });
    if (sampleIndex === 0) {
      events.push({ kind: "webp-frame", index: webpIndex });
      webpIndex += 1;
    } else {
      for (let step = 0; step < STEPS_PER_SVG_SAMPLE; step += 1) {
        events.push({ kind: "advance" });
        if (webpIndex < webpFrameCount) {
          events.push({ kind: "webp-frame", index: webpIndex });
          webpIndex += 1;
        }
      }
    }
    events.push({ kind: "svg-sample", index: sampleIndex });
  }

  if (webpIndex !== webpFrameCount) {
    throw new Error(
      `README clip scheduled ${webpIndex} WebP frames, expected ${webpFrameCount}.`,
    );
  }

  return events;
}
