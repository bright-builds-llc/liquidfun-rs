import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const WEB_SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");

function source(path: string): string {
  return readFileSync(join(WEB_SRC, path), "utf8");
}

/** Opening tag text of the first element whose attributes contain `marker`. */
function openingTagWith(text: string, marker: string): string {
  const markerIndex = text.indexOf(marker);
  if (markerIndex < 0) {
    return "";
  }
  const start = text.lastIndexOf("<", markerIndex);
  const end = text.indexOf(">", markerIndex);
  return text.slice(start, end + 1);
}

describe("HUD sliders toggle chrome", () => {
  it("renders the toggle only for scenes with HUD controls", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const gated = /<Show when=\{props\.hasHudSlider\}>\s*<HudSlidersToggle/.test(
      canvasStage,
    );

    // Assert
    expect(gated).toBe(true);
  });

  it("points the toggle at the slider card with aria-controls and aria-expanded", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const toggleTag = openingTagWith(canvasStage, "canvas-sliders-toggle");

    // Assert
    expect(toggleTag).toContain("aria-controls={HUD_SLIDERS_ID}");
    expect(toggleTag).toContain("aria-expanded=");
  });

  it("places the toggle inside the toolbar before the transport", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const toolbar = canvasStage.indexOf('class="canvas-hud-toolbar"');
    const toggle = canvasStage.indexOf("<HudSlidersToggle", toolbar);
    const transport = canvasStage.indexOf('class="canvas-transport"', toolbar);

    // Assert
    expect(toolbar).toBeGreaterThan(-1);
    expect(toggle).toBeGreaterThan(toolbar);
    expect(transport).toBeGreaterThan(toggle);
  });

  it("removes the slider card from the DOM while the sliders are hidden", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const keyedOnVisibility =
      /<Show when=\{sliders\(\) === "shown"\}>\s*\{props\.children\}\s*<\/Show>/.test(
        canvasStage,
      );

    // Assert
    expect(keyedOnVisibility).toBe(true);
  });

  it("persists the toggle choice", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const persists = canvasStage.includes("persistHudSlidersVisibility(");

    // Assert
    expect(persists).toBe(true);
  });

  it.each([
    ['<h2 id="player-title"'],
    ['class={`session-status'],
    ['class="canvas-hud-stats"'],
    ['class="canvas-hud-corner"'],
    ['class="canvas-transport"'],
    ['class="canvas-hud-trailing"'],
    ['class="hud-button canvas-sliders-toggle"'],
  ])("marks the %s chrome leaf as a HUD obstacle", (marker) => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const tag = openingTagWith(canvasStage, marker);

    // Assert
    expect(tag).toContain("data-hud-obstacle");
  });

  it("marks the slider card as a HUD obstacle with the shared id", () => {
    // Arrange
    const sceneControls = source("components/SceneControls.tsx");

    // Act
    const tag = openingTagWith(sceneControls, 'class="canvas-hud-slider"');

    // Assert
    expect(tag).toContain("id={HUD_SLIDERS_ID}");
    expect(tag).toContain("data-hud-obstacle");
  });

  it.each([['class="canvas-hud-title"'], ['class="canvas-hud-bottom"']])(
    "does not mark the full-width %s container as an obstacle",
    (marker) => {
      // Arrange
      const canvasStage = source("components/CanvasStage.tsx");

      // Act
      const tag = openingTagWith(canvasStage, marker);

      // Assert
      expect(tag).not.toContain("data-hud-obstacle");
    },
  );

  it("measures HUD obstacles only in the canvas-stage layout", () => {
    // Arrange
    const playerPanel = source("components/PlayerPanel.tsx");

    // Act
    const guardIndex = playerPanel.indexOf("!layoutProps.canvasStage ||");
    const measureIndex = playerPanel.indexOf("createHudObstacles(");

    // Assert
    expect(guardIndex).toBeGreaterThan(-1);
    expect(measureIndex).toBeGreaterThan(guardIndex);
  });
});

/** Source of `function name(` up to the next top-level function, or end of file. */
function functionBody(text: string, name: string): string {
  const start = text.indexOf(`function ${name}(`);
  if (start < 0) {
    return "";
  }
  const next = text.indexOf("\nfunction ", start + 1);
  return text.slice(start, next < 0 ? undefined : next);
}

function pathData(body: string): Set<string> {
  return new Set([...body.matchAll(/d="([^"]+)"/g)].map((match) => match[1]));
}

describe("HUD sliders toggle icon", () => {
  it("renders the panel chevron icon bound to visibility, not the drawer glyph", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const toggle = functionBody(canvasStage, "HudSlidersToggle");

    // Assert
    expect(toggle).toContain(
      '<SliderPanelIcon expanded={props.visibility === "shown"} />',
    );
    expect(toggle).not.toContain("<ControlsIcon");
    expect(toggle).not.toContain("<SlidersIcon");
  });

  it("draws a panel whose paths share nothing with the Controls drawer glyph", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");
    const panelIcon = functionBody(canvasStage, "SliderPanelIcon");
    const controlsIcon = functionBody(canvasStage, "ControlsIcon");

    // Act
    const panelPaths = pathData(panelIcon);
    const shared = [...pathData(controlsIcon)].filter((d) => panelPaths.has(d));

    // Assert
    expect(panelIcon).toContain("<rect");
    expect(panelPaths.size).toBeGreaterThan(0);
    expect(shared).toEqual([]);
  });

  it("points the chevron down while expanded and up while collapsed", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const panelIcon = functionBody(canvasStage, "SliderPanelIcon");

    // Assert
    expect(panelIcon).toMatch(
      /<Show\s+when=\{props\.expanded\}\s+fallback=\{<path d="m9 12 3-3 3 3" \/>\}\s*>\s*<path d="m9 9 3 3 3-3" \/>/,
    );
  });

  it("removes the old slider glyph", () => {
    // Arrange
    const canvasStage = source("components/CanvasStage.tsx");

    // Act
    const hasSlidersIcon = canvasStage.includes("function SlidersIcon(");

    // Assert
    expect(hasSlidersIcon).toBe(false);
  });
});
