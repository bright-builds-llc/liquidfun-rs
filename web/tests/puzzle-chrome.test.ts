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
  "components/PuzzleToolTray.tsx",
  "components/PuzzleSelectionBar.tsx",
  "components/PuzzleTransport.tsx",
  "components/PuzzleStarList.tsx",
  "components/PuzzleIcons.tsx",
  "components/PuzzleCaption.tsx",
  "components/PuzzleSelectPanel.tsx",
  "components/PuzzleLevelLinks.tsx",
  "components/PuzzleSoundToggle.tsx",
] as const;

/** Stylesheets of the canvas puzzle HUD, which branch on `data-puzzle-layout`. */
const HUD_STYLESHEETS = ["styles/puzzle.css", "styles/puzzle-editor.css"] as const;

const PUZZLE_STYLESHEETS = [...HUD_STYLESHEETS, "styles/puzzle-select.css"] as const;

/** Phase 39 editor leaves and the component that renders each one (D-26). */
const EDITOR_LEAVES = [
  { leaf: "puzzle-tray", path: "components/PuzzleToolTray.tsx" },
  { leaf: "puzzle-selection-bar", path: "components/PuzzleSelectionBar.tsx" },
  { leaf: "puzzle-transport", path: "components/PuzzleTransport.tsx" },
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

  it("marks the tray, selection bar and transport leaves with data-hud-obstacle", () => {
    // Arrange
    const leaves = EDITOR_LEAVES.map(({ leaf, path }) => ({ leaf, text: source(path) }));

    // Act
    const unmarked = leaves
      .filter(({ leaf, text }) => !openingTagWithClass(text, leaf).includes("data-hud-obstacle"))
      .map(({ leaf }) => leaf);

    // Assert
    expect(unmarked).toEqual([]);
  });

  it("marks the caption leaf with data-hud-obstacle", () => {
    // Arrange
    const caption = source("components/PuzzleCaption.tsx");

    // Act
    const tag = openingTagWithClass(caption, "puzzle-caption");

    // Assert
    expect(tag).toContain("data-hud-obstacle");
  });

  it("mounts the caption in the canvas-stage HUD and the form-factor status row", () => {
    // Arrange
    const panel = source("components/PuzzlePanel.tsx");
    const statusRow = panel.slice(panel.indexOf('class="status-row"'), panel.indexOf("<figure>"));

    // Act
    const inHud = panel.includes("caption={caption()}");
    const inStatusRow = statusRow.includes("{caption()}");

    // Assert
    expect([inHud, inStatusRow]).toEqual([true, true]);
  });

  it("gives the caption a fixed height in both canvas layouts", () => {
    // Arrange
    const css = source("styles/puzzle.css");
    const cases = [
      { layout: "landscape", height: 48 },
      { layout: "portrait", height: 52 },
    ];

    // Act
    const unsized = cases
      .filter(({ layout, height }) => {
        const block = cssBlock(
          css,
          `.puzzle-panel--canvas-stage[data-puzzle-layout="${layout}"] .puzzle-caption`,
        );
        const spansWidth = /\bleft:/.test(block) && /\bright:/.test(block);
        return !spansWidth || !block.includes(`height: ${height}px;`);
      })
      .map(({ layout }) => layout);

    // Assert
    expect(unsized).toEqual([]);
  });

  it("mounts the tray, selection bar and transport in the canvas-stage HUD", () => {
    // Arrange
    const panel = source("components/PuzzlePanel.tsx");

    // Act
    const props = [
      "toolTray={toolTray()}",
      "selectionBar={selectionBar()}",
      "transport={transport()}",
    ];
    const mounted = props.map((prop) => panel.includes(prop));

    // Assert
    expect(mounted).toEqual([true, true, true]);
  });

  it("makes the puzzle canvas a focusable application described by the keyboard help", () => {
    // Arrange
    const panel = source("components/PuzzlePanel.tsx");

    // Act: the ref callback contains "=>", so slice up to the fallback text.
    const start = panel.indexOf('class="scene-canvas"');
    const canvasTag = panel.slice(start, panel.indexOf("Canvas is required", start));

    // Assert
    expect(canvasTag).toContain('role="application"');
    expect(canvasTag).toContain('tabindex="0"');
    expect(canvasTag).toContain("aria-describedby={KEYBOARD_HELP_ID}");
    expect(canvasTag).not.toContain('role="img"');
  });

  it("uses aria-disabled and never native disabled on the editor controls", () => {
    // Arrange
    const sources = EDITOR_LEAVES.map(({ path }) => ({ path, text: source(path) }));

    // Act
    const offenders = sources
      .filter(({ text }) => /\sdisabled[\s=>]/.test(text))
      .map(({ path }) => path);

    // Assert
    expect(offenders).toEqual([]);
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

  it("keeps data-hud-obstacle off the level grid", () => {
    // Arrange
    const panel = source("components/PuzzleSelectPanel.tsx");

    // Act
    const marksObstacle = panel.includes("data-hud-obstacle");

    // Assert
    expect(marksObstacle).toBe(false);
  });

  it("keeps data-hud-obstacle off the level links and the options sheet", () => {
    // Arrange
    const paths = ["components/PuzzleLevelLinks.tsx", "components/PuzzleOptionsDrawer.tsx"];

    // Act
    const offenders = paths.filter((path) => source(path).includes("data-hud-obstacle"));

    // Assert
    expect(offenders).toEqual([]);
  });
});

describe("puzzle stylesheet", () => {
  it("has no transitions or animations in any puzzle stylesheet", () => {
    // Arrange
    const sheets = PUZZLE_STYLESHEETS.map((path) => ({ path, css: source(path) }));

    // Act
    const withMotion = sheets
      .filter(({ css }) => /transition|animation/.test(css))
      .map(({ path }) => path);

    // Assert
    expect(withMotion).toEqual([]);
  });

  it("gives the tray and selection bar a fixed width and height in both canvas layouts", () => {
    // Arrange
    const css = source("styles/puzzle-editor.css");
    // The portrait selection bar stacks the valve stepper over the tool buttons (Phase 40 D-05).
    const cases = [
      { layout: "landscape", leaf: ".puzzle-tray", height: 44 },
      { layout: "landscape", leaf: ".puzzle-selection-bar", height: 44 },
      { layout: "portrait", leaf: ".puzzle-tray", height: 44 },
      { layout: "portrait", leaf: ".puzzle-selection-bar", height: 96 },
    ];

    // Act
    const unsized = cases
      .filter(({ layout, leaf, height }) => {
        const block = cssBlock(
          css,
          `.puzzle-panel--canvas-stage[data-puzzle-layout="${layout}"] ${leaf}`,
        );
        return !/\bwidth:/.test(block) || !block.includes(`height: ${height}px;`);
      })
      .map(({ layout, leaf }) => `${layout} ${leaf}`);

    // Assert
    expect(unsized).toEqual([]);
  });

  it("gives the meter, timer, result, transport and legend slots a fixed width and height", () => {
    // Arrange
    const css = `${source("styles/puzzle.css")}\n${source("styles/puzzle-editor.css")}`;
    const leaves = [
      ".puzzle-meter",
      ".puzzle-timer",
      ".puzzle-caption",
      ".puzzle-result",
      ".puzzle-transport",
      ".puzzle-legend-slot",
    ];

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

  it.each(HUD_STYLESHEETS)("scopes data-puzzle-layout selectors to the canvas stage in %s", (path) => {
    // Arrange
    const css = source(path);
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

describe("puzzle sound controls", () => {
  const SOUND_COPY = [
    '"Sound"',
    '"Sound effects"',
    '"No sound on iPhone? Check the silent switch."',
    '"Sound unavailable in this browser"',
    '"Sound is unavailable in this browser. The puzzle still works without it."',
    '"Sound stopped after an audio error. Turn on Sound effects to try again."',
  ];

  it("keeps data-hud-obstacle off the sound toggle and section", () => {
    // Arrange
    const toggle = source("components/PuzzleSoundToggle.tsx");

    // Act
    const marksObstacle = toggle.includes("data-hud-obstacle");

    // Assert
    expect(marksObstacle).toBe(false);
  });

  it("contains every locked sound copy string verbatim", () => {
    // Arrange
    const toggle = source("components/PuzzleSoundToggle.tsx");

    // Act
    const missing = SOUND_COPY.filter((copy) => !toggle.includes(copy));

    // Assert
    expect(missing).toEqual([]);
  });

  it("never touches the audio context, storage or the audio session directly", () => {
    // Arrange
    const toggle = source("components/PuzzleSoundToggle.tsx");

    // Act
    const offenders = ["AudioContext", "localStorage", "audioSession"].filter((word) =>
      toggle.includes(word),
    );

    // Assert
    expect(offenders).toEqual([]);
  });

  it.each(["landscape", "portrait"])("sizes the %s options slot at 96 by 44 for two buttons", (layout) => {
    // Arrange
    const css = source("styles/puzzle.css");

    // Act
    const block = cssBlock(
      css,
      `.puzzle-panel--canvas-stage[data-puzzle-layout="${layout}"] .puzzle-options-slot`,
    );

    // Assert
    expect(block).toContain("width: 96px;");
    expect(block).toContain("height: 44px;");
  });

  it("ends the landscape caption 8 px left of the wider options slot", () => {
    // Arrange
    const css = source("styles/puzzle.css");

    // Act
    const block = cssBlock(
      css,
      '.puzzle-panel--canvas-stage[data-puzzle-layout="landscape"] .puzzle-caption',
    );

    // Assert
    expect(block).toContain("right: calc(var(--puzzle-inset-right) + 104px);");
  });

  it("ends the portrait title 8 px left of the wider options slot", () => {
    // Arrange
    const css = source("styles/puzzle.css");

    // Act
    const block = cssBlock(
      css,
      '.puzzle-panel--canvas-stage[data-puzzle-layout="portrait"] .puzzle-title',
    );

    // Assert
    expect(block).toContain(
      "width: calc(100% - var(--puzzle-inset-left) - var(--puzzle-inset-right) - 156px);",
    );
  });

  it("renders the sound toggle before the options trigger inside the options slot", () => {
    // Arrange
    const hud = source("components/PuzzleHud.tsx");
    const slotStart = hud.indexOf('class="puzzle-options-slot"');
    const slot = hud.slice(slotStart, hud.indexOf("</div>", slotStart));

    // Act
    const soundIndex = slot.indexOf("{props.soundToggle}");
    const optionsIndex = slot.indexOf("{props.optionsTrigger}");

    // Assert
    expect(soundIndex).toBeGreaterThan(0);
    expect(optionsIndex).toBeGreaterThan(soundIndex);
  });

  it("styles the sound controls without the accent color", () => {
    // Arrange
    const css = source("styles/puzzle-sound.css");

    // Act
    const usesAccent = /#4DA3FF/i.test(css);

    // Assert
    expect(usesAccent).toBe(false);
  });

  it("styles the sound controls without transitions or animations", () => {
    // Arrange
    const css = source("styles/puzzle-sound.css");

    // Act
    const hasMotion = /transition|animation/.test(css);

    // Assert
    expect(hasMotion).toBe(false);
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
