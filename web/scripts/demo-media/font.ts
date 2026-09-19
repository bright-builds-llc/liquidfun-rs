import type { Page } from "@playwright/test";

import type { CaptureFontFingerprint } from "./model";

const FONT_CORPUS =
  "LiquidFun 0123456789 ABCDEFG abcdefg !?@#$% []{} Ω→≈";
const FONT_CANVAS_WIDTH = 768;
const FONT_CANVAS_HEIGHT = 192;

export async function captureFontFingerprint(
  page: Page,
): Promise<CaptureFontFingerprint> {
  return await page.evaluate(
    async ({ corpus, width, height }) => {
      const panel = document.querySelector<HTMLElement>(".player-panel");
      if (panel === null) {
        throw new Error("Player panel is unavailable for font fingerprinting");
      }

      const panelStyle = getComputedStyle(panel);
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext("2d", {
        alpha: false,
        willReadFrequently: true,
      });
      if (context === null) {
        throw new Error("2D canvas context is unavailable for font fingerprinting");
      }

      context.fillStyle = "#0B0F14";
      context.fillRect(0, 0, width, height);
      context.textBaseline = "alphabetic";

      const sampleInputs = [
        { label: "regular", fontStyle: "normal", fontWeight: "400" },
        { label: "semibold", fontStyle: "normal", fontWeight: "600" },
        { label: "bold", fontStyle: "normal", fontWeight: "700" },
      ] as const;
      const samples = sampleInputs.map((sample, index) => {
        const fontSizePx = 24;
        context.font =
          `${sample.fontStyle} ${sample.fontWeight} ${fontSizePx}px ${panelStyle.fontFamily}`;
        context.fillStyle = ["#F4F7FA", "#9FD8FF", "#FFD580"][index] ?? "#F4F7FA";
        context.fillText(
          `${sample.label}: ${corpus}`,
          20,
          48 + index * 56,
        );
        return {
          ...sample,
          fontSizePx,
          canvasFont: context.font,
        };
      });

      const rgbaBytes = context.getImageData(0, 0, width, height).data;
      const digest = await crypto.subtle.digest("SHA-256", rgbaBytes);
      const rasterSha256 = [...new Uint8Array(digest)]
        .map((byte) => byte.toString(16).padStart(2, "0"))
        .join("");

      return {
        corpusVersion: 1,
        corpus,
        canvas: { width, height },
        panelComputedStyle: {
          fontFamily: panelStyle.fontFamily,
          fontStyle: panelStyle.fontStyle,
          fontWeight: panelStyle.fontWeight,
        },
        samples,
        rasterSha256,
      };
    },
    {
      corpus: FONT_CORPUS,
      width: FONT_CANVAS_WIDTH,
      height: FONT_CANVAS_HEIGHT,
    },
  );
}
