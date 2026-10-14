import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const WEB_SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");

/** Body of the first `canvas.scene-canvas {` rule in the player stylesheet. */
function sceneCanvasRule(): string {
  const css = readFileSync(join(WEB_SRC, "styles/player-viewport.css"), "utf8");
  const start = css.indexOf("canvas.scene-canvas {");
  if (start < 0) {
    return "";
  }
  const end = css.indexOf("}", start);
  return css.slice(start, end + 1);
}

// Chromium drops -webkit-touch-callout from computed style, so the e2e check
// cannot see it; the source text is the guard for the iOS callout.
describe("scene canvas long-press suppression", () => {
  it("declares the WebKit callout and selection suppression on the scene canvas", () => {
    // Arrange
    const rule = sceneCanvasRule();

    // Act
    const declarations = [
      "-webkit-touch-callout: none;",
      "-webkit-user-select: none;",
      " user-select: none;",
    ].filter((declaration) => rule.includes(declaration));

    // Assert
    expect(declarations).toHaveLength(3);
  });

  it("keeps the existing scene canvas stacking declarations", () => {
    // Arrange
    const rule = sceneCanvasRule();

    // Act
    const keepsPosition = rule.includes("position: relative;");
    const keepsZIndex = rule.includes("z-index: 1;");

    // Assert
    expect([keepsPosition, keepsZIndex]).toEqual([true, true]);
  });
});
