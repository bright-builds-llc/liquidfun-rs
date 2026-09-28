import { access } from "node:fs/promises";

import { createCanvas, GlobalFonts } from "@napi-rs/canvas";

import type { RenderFrame } from "../../src/physics/frame";
import type { Camera } from "../../src/render/camera";
import { drawRenderFrame } from "../../src/render/canvas";
import { canvasRenderMode, type SvgRenderMode } from "../../src/render/mode";

/** Pinned face so label text does not depend on the host font set. */
export const README_PREVIEW_FONT_FILE = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

let registeredFontFile: string | undefined;

export type FramePainter = {
  paint(
    frame: RenderFrame,
    camera: Camera,
    renderMode: SvgRenderMode,
    wireframeStrokeWidth: number,
    maxRenderedParticles: number,
  ): Buffer;
};

/**
 * Draws README frames with the playground canvas painter.
 *
 * The Skia context implements the 2D calls `drawRenderFrame` uses. Pixel
 * readback goes through `getImageData`, which returns straight RGBA in
 * canvas order, matching ffmpeg's rawvideo input.
 */
export async function createFramePainter(
  width: number,
  height: number,
  fontFile: string,
): Promise<FramePainter> {
  await registerPreviewFont(fontFile);

  const canvas = createCanvas(width, height);
  const context = canvas.getContext("2d");
  const drawing = context as unknown as CanvasRenderingContext2D;

  return {
    paint(frame, camera, renderMode, wireframeStrokeWidth, maxRenderedParticles) {
      drawRenderFrame(
        drawing,
        frame,
        camera,
        canvasRenderMode(renderMode),
        wireframeStrokeWidth,
        maxRenderedParticles,
      );
      const image = context.getImageData(0, 0, width, height);
      if (image.data.byteLength !== width * height * 4) {
        throw new Error(
          `Frame raster is ${image.data.byteLength} bytes, expected ${width * height * 4}.`,
        );
      }
      return Buffer.from(image.data);
    },
  };
}

async function registerPreviewFont(fontFile: string): Promise<void> {
  if (registeredFontFile === fontFile) {
    return;
  }
  await assertFontFile(fontFile);
  if (!GlobalFonts.registerFromPath(fontFile, "sans-serif")) {
    throw new Error(`Could not register ${fontFile} as the sans-serif face.`);
  }
  registeredFontFile = fontFile;
}

async function assertFontFile(fontFile: string): Promise<void> {
  try {
    await access(fontFile);
  } catch (error) {
    throw new Error(
      `README WebP font is missing at ${fontFile}. Install DejaVu Sans (fonts-dejavu-core).`,
      { cause: error },
    );
  }
}
