import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { MAX_RAW_DISPLAY_LENGTH, truncateRaw } from "../src/puzzle/fallback-text";

const WEB_SRC = join(dirname(fileURLToPath(import.meta.url)), "../src");

const PUZZLE_COMPONENTS = [
  "components/PuzzleHud.tsx",
  "components/PuzzleResultCard.tsx",
  "components/PuzzleOptionsDrawer.tsx",
  "components/PuzzlePanel.tsx",
  "components/PuzzleFallbackPanel.tsx",
] as const;

const SCENE_ONLY_CHROME =
  /ViewportTools|CanvasHud|PlaybackButtons|SceneControls|HudControlSlider|DebugReadout|GravityArrow|AnimatedSvgPane|PlayerSceneChrome|SceneCredits|tilt/;

const WALL_CLOCK_OR_HTML = /innerHTML|Date\.now|performance\.now|setTimeout|setInterval/;

function source(path: string): string {
  return readFileSync(join(WEB_SRC, path), "utf8");
}

/** Opening tag text of the element whose class list contains `className`. */
function openingTagWithClass(text: string, className: string): string {
  const pattern = new RegExp(`<[a-z][a-z0-9]*\\b[^>]*\\bclass="(?:[^"]* )?${className}(?: [^"]*)?"[^>]*>`);
  return pattern.exec(text)?.[0] ?? "";
}

/** Declaration block of the first CSS rule whose selector list ends with `selector`. */
function cssBlock(css: string, selector: string): string {
  const start = css.indexOf(`${selector} {`);
  if (start < 0) {
    return "";
  }
  return css.slice(start, css.indexOf("}", start) + 1);
}

describe("puzzle HUD chrome", () => {
  it("marks every HUD leaf in PuzzleHud with data-hud-obstacle", () => {
    // Arrange
    const hud = source("components/PuzzleHud.tsx");
    const leaves = [
      "puzzle-title",
      "puzzle-meter",
      "puzzle-timer",
      "puzzle-transport",
      "puzzle-options-slot",
      "puzzle-legend-slot",
    ];

    // Act
    const unmarked = leaves.filter(
      (leaf) => !openingTagWithClass(hud, leaf).includes("data-hud-obstacle"),
    );

    // Assert
    expect(unmarked).toEqual([]);
  });

  it("marks the result card slot with data-hud-obstacle", () => {
    // Arrange
    const resultCard = source("components/PuzzleResultCard.tsx");

    // Act
    const tag = openingTagWithClass(resultCard, "puzzle-result");

    // Assert
    expect(tag).toContain("data-hud-obstacle");
  });

  it("renders the scale legend in both the canvas stage and form-factor layouts", () => {
    // Arrange
    const hud = source("components/PuzzleHud.tsx");
    const panel = source("components/PuzzlePanel.tsx");

    // Act
    const canvasStageHasLegend = hud.includes("<ScaleLegend");
    const formFactorHasLegend = panel.includes("<ScaleLegend");

    // Assert
    expect(canvasStageHasLegend).toBe(true);
    expect(formFactorHasLegend).toBe(true);
  });

  it("references no scene-only chrome in any puzzle component", () => {
    // Arrange
    const sources = PUZZLE_COMPONENTS.map((path) => ({ path, text: source(path) }));

    // Act
    const offenders = sources
      .filter(({ text }) => SCENE_ONLY_CHROME.test(text))
      .map(({ path }) => path);

    // Assert
    expect(offenders).toEqual([]);
  });

  it("exposes the timer as a non-live role=timer named Simulation time left", () => {
    // Arrange
    const hud = source("components/PuzzleHud.tsx");

    // Act
    const timerTag = openingTagWithClass(hud, "puzzle-timer");

    // Assert
    expect(timerTag).toContain('role="timer"');
    expect(timerTag).toContain('aria-label="Simulation time left"');
    expect(hud).not.toContain("<output");
  });

  it("uses no wall-clock APIs or innerHTML in any puzzle component", () => {
    // Arrange
    const sources = PUZZLE_COMPONENTS.map((path) => ({ path, text: source(path) }));

    // Act
    const offenders = sources
      .filter(({ text }) => WALL_CLOCK_OR_HTML.test(text))
      .map(({ path }) => path);

    // Assert
    expect(offenders).toEqual([]);
  });
});

describe("puzzle stylesheet", () => {
  it("has no transitions or animations", () => {
    // Arrange
    const css = source("styles/puzzle.css");

    // Act
    const hasMotion = /transition|animation/.test(css);

    // Assert
    expect(hasMotion).toBe(false);
  });

  it("gives the meter, timer, result, play and legend slots a fixed width and height", () => {
    // Arrange
    const css = source("styles/puzzle.css");
    const leaves = [".puzzle-meter", ".puzzle-timer", ".puzzle-result", ".puzzle-play", ".puzzle-legend-slot"];

    // Act
    const unsized = leaves.filter((leaf) => {
      const sizedBlocks = css
        .split("}")
        .filter((block) => {
          const selector = block.slice(0, block.indexOf("{"));
          return new RegExp(`${leaf.replace(".", "\\.")}(?![-\\w])`).test(selector);
        })
        .filter((block) => /\bwidth:/.test(block) && /\bheight:/.test(block));
      return sizedBlocks.length === 0;
    });

    // Assert
    expect(unsized).toEqual([]);
  });

  it("scopes every data-puzzle-layout selector to the canvas stage", () => {
    // Arrange
    const css = source("styles/puzzle.css");
    const prefix = ".puzzle-panel--canvas-stage";

    // Act
    const occurrences = [...css.matchAll(/\[data-puzzle-layout/g)].map((match) => match.index);
    const unscoped = occurrences.filter(
      (index) => css.slice(index - prefix.length, index) !== prefix,
    );

    // Assert
    expect(occurrences.length).toBeGreaterThan(0);
    expect(unscoped).toEqual([]);
  });

  it("keeps the legend slot box at the shared 100 by 28 legend size", () => {
    // Arrange
    const css = source("styles/puzzle.css");

    // Act
    const block = cssBlock(css, ".puzzle-legend-slot");

    // Assert
    expect(block).toContain("width: 100px;");
    expect(block).toContain("height: 28px;");
  });
});

describe("truncateRaw", () => {
  it("returns an 80-character id unchanged", () => {
    // Arrange
    const raw = "a".repeat(MAX_RAW_DISPLAY_LENGTH);

    // Act
    const shown = truncateRaw(raw);

    // Assert
    expect(shown).toBe(raw);
  });

  it("cuts an 81-character id to 80 characters plus an ellipsis", () => {
    // Arrange
    const raw = `${"b".repeat(MAX_RAW_DISPLAY_LENGTH)}c`;

    // Act
    const shown = truncateRaw(raw);

    // Assert
    expect(shown).toBe(`${"b".repeat(MAX_RAW_DISPLAY_LENGTH)}…`);
  });

  it("returns markup-looking text verbatim for text binding", () => {
    // Arrange
    const raw = "<img src=x>";

    // Act
    const shown = truncateRaw(raw);

    // Assert
    expect(shown).toBe("<img src=x>");
  });
});

describe("unknown-level fallback", () => {
  it("renders the raw id through truncateRaw and never through innerHTML", () => {
    // Arrange
    const fallback = source("components/PuzzleFallbackPanel.tsx");

    // Act
    const usesTruncate = fallback.includes("truncateRaw(props.maybeRaw)");
    const usesInnerHtml = fallback.includes("innerHTML");

    // Assert
    expect(usesTruncate).toBe(true);
    expect(usesInnerHtml).toBe(false);
  });
});
