import { expect, test, type Page } from "@playwright/test";
import { resolve } from "node:path";

import {
  FIRST_LEVEL_HINT,
  PUZZLE_CAPTIONS,
  PUZZLE_TITLES,
  maybeNextPuzzleId,
  puzzleHash,
} from "../src/catalog/puzzle-levels";
import { PUZZLE_IDS, type PuzzleId } from "../src/catalog/puzzles";
import { PUZZLE_TEST_PLACEMENTS_GLOBAL } from "../src/puzzle/placements";
import { installControlledRefreshRate } from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  installSolutionPlacements,
  overlappingLeaves,
  overlappingObstacles,
  puzzlePath,
  readHudGeometry,
  stablePuzzleCamera,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import { SCREENS_DIR_40, SCREENS_DIR_44, readPlacements } from "./puzzle-editor-helpers";

// Each test gets a fresh browser context, so storage starts empty and
// first-pour shows its placement hint.
test.describe.configure({ mode: "parallel" });

type Viewport = { readonly width: number; readonly height: number };

const RUN_TEST_TIMEOUT_MS = 90_000;
const ENDED_TIMEOUT_MS = 60_000;
// The last level's Next slot links back to the grid (Phase 41 D-04).
const ALL_LEVELS = "All levels";
const PHONE_SMALL = { width: 360, height: 780 } as const;
// Portrait tray leaf widths at each phone width (39-UI-SPEC, Phase 44 D-02).
const PORTRAIT_TRAY_CASES: readonly { readonly viewport: Viewport; readonly trayWidth: number }[] = [
  { viewport: PHONE_PORTRAIT, trayWidth: 218 },
  { viewport: PHONE_SMALL, trayWidth: 188 },
];
const PX_TOLERANCE = 0.5;
const TRAY_HEIGHT_PX = 44;
const GRAND_FINALE_TRAY_NAMES = ["Plank, 2 left", "Valve, 1 left"] as const;

type TrayItemFit = {
  readonly height: number;
  readonly width: number;
  readonly nameVisible: boolean;
  readonly nameScroll: number;
  readonly nameClient: number;
  readonly nameFontPx: number;
  readonly countFontPx: number;
  readonly countRight: number;
  readonly innerRight: number;
};

type TrayFit = {
  readonly kindCount: number;
  readonly trayWidth: number;
  readonly trayHeight: number;
  readonly items: readonly TrayItemFit[];
};

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

/** The caption leaf text on a fresh visit: first-pour shows its hint (D-19). */
function expectedCaption(id: PuzzleId): string {
  return id === "first-pour" ? FIRST_LEVEL_HINT : PUZZLE_CAPTIONS[id];
}

async function openLevelBuilding(page: Page, id: PuzzleId, viewport: Viewport): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(puzzlePath(id));
  await waitForPuzzlePhase(page, "building");
}

/** Tray box, item boxes and name/count fit, read in one evaluate (Phase 44 D-01, D-02). */
async function readTrayFit(page: Page): Promise<TrayFit> {
  return page.locator(".puzzle-tray").evaluate((tray): TrayFit => {
    const trayRect = tray.getBoundingClientRect();
    const items = Array.from(tray.querySelectorAll(".puzzle-tray-item"), (item) => {
      const name = item.querySelector(".puzzle-tray-name");
      const count = item.querySelector(".puzzle-tray-count");
      if (!(name instanceof HTMLElement) || !(count instanceof HTMLElement)) {
        throw new Error("Tray item is missing its name or count");
      }
      const itemRect = item.getBoundingClientRect();
      const itemStyle = getComputedStyle(item);
      return {
        height: itemRect.height,
        width: itemRect.width,
        nameVisible: getComputedStyle(name).display !== "none",
        nameScroll: name.scrollWidth,
        nameClient: name.clientWidth,
        nameFontPx: parseFloat(getComputedStyle(name).fontSize),
        countFontPx: parseFloat(getComputedStyle(count).fontSize),
        countRight: count.getBoundingClientRect().right,
        // The item's content edge: inside its right border and padding.
        innerRight:
          itemRect.right -
          parseFloat(itemStyle.borderRightWidth) -
          parseFloat(itemStyle.paddingRight),
      };
    });
    return {
      kindCount: Number(tray.getAttribute("data-kind-count")),
      trayWidth: trayRect.width,
      trayHeight: trayRect.height,
      items,
    };
  });
}

function expectTrayFits(fit: TrayFit, trayWidth: number): void {
  // A missing attribute reads as 0 and would silently skip the two-kind checks.
  expect([1, 2]).toContain(fit.kindCount);
  const maybeFirst = fit.items[0];
  expect(maybeFirst).toBeDefined();
  for (const item of fit.items) {
    if (item.nameVisible) {
      expect(item.nameScroll).toBeLessThanOrEqual(item.nameClient);
    }
    expect(Math.abs(item.height - TRAY_HEIGHT_PX)).toBeLessThanOrEqual(PX_TOLERANCE);
    expect(item.countRight).toBeLessThanOrEqual(item.innerRight + PX_TOLERANCE);
    expect(Math.abs(item.width - (maybeFirst?.width ?? 0))).toBeLessThanOrEqual(PX_TOLERANCE);
  }
  expect(Math.abs(fit.trayWidth - trayWidth)).toBeLessThanOrEqual(PX_TOLERANCE);
  expect(Math.abs(fit.trayHeight - TRAY_HEIGHT_PX)).toBeLessThanOrEqual(PX_TOLERANCE);
  if (fit.kindCount !== 2) {
    return;
  }
  for (const item of fit.items) {
    expect(item.nameVisible).toBe(true);
    expect(item.nameFontPx).toBe(14);
    expect(item.countFontPx).toBe(12);
  }
}

async function expectGrandFinaleTrayNames(page: Page): Promise<void> {
  const buttons = page.getByRole("group", { name: "Tools" }).getByRole("button");
  await expect(buttons).toHaveCount(GRAND_FINALE_TRAY_NAMES.length);
  for (const [index, name] of GRAND_FINALE_TRAY_NAMES.entries()) {
    await expect(buttons.nth(index)).toHaveAccessibleName(name);
  }
}

async function speedToFour(page: Page): Promise<void> {
  for (const nextSpeed of ["Speed 2×", "Speed 4×"]) {
    await page.getByRole("button", { name: /^Speed \d×$/ }).click();
    await expect(page.getByRole("button", { name: nextSpeed, exact: true })).toBeVisible();
  }
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-speed", "4");
}

/** Plays the level with no tools at 4× until the run ends. */
async function endEmptyRun(page: Page): Promise<void> {
  await speedToFour(page);
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
}

for (const id of PUZZLE_IDS) {
  for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
    test(`opens ${id} in building with its title and caption at ${viewportName(viewport)}`, async ({
      page,
    }) => {
      // Arrange
      await installControlledRefreshRate(page, 60);

      // Act
      await openLevelBuilding(page, id, viewport);
      await stablePuzzleCamera(page);
      const geometry = await readHudGeometry(page);

      // Assert
      await expect(page.locator("#player-title")).toHaveText(PUZZLE_TITLES[id]);
      await expect(page.locator(".puzzle-caption-text")).toHaveText(expectedCaption(id));
      await expect(page.locator(".puzzle-failure")).toHaveAttribute("data-failed", "false");
      await expect(page.locator(".puzzle-failure-heading")).toHaveCount(0);
      expect(geometry.usedFallback).toBe(false);
      expect(overlappingLeaves(geometry)).toEqual([]);
      expect(overlappingObstacles(geometry)).toEqual([]);
      await page.screenshot({
        path: resolve(SCREENS_DIR_40, `level-${id}-${viewportName(viewport)}.png`),
      });
    });
  }

  const maybeNextId = maybeNextPuzzleId(id);
  const nextTitle =
    maybeNextId === undefined
      ? "links All levels after the last level"
      : `links Next to ${maybeNextId}`;
  test(`${id} ${nextTitle} at ${viewportName(DESKTOP)}`, async ({ page }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    await installControlledRefreshRate(page, 15);
    await openLevelBuilding(page, id, DESKTOP);

    // Act
    await endEmptyRun(page);

    // Assert
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "lost");
    if (maybeNextId === undefined) {
      const allLevels = page
        .locator(".puzzle-result-actions")
        .getByRole("link", { name: ALL_LEVELS, exact: true });
      await expect(allLevels).toHaveAttribute("href", "#/puzzles");
      await expect(allLevels).toHaveClass(/product-control--accent/);
      await expect(page.getByRole("link", { name: "Next level" })).toHaveCount(0);
      await page.screenshot({ path: resolve(SCREENS_DIR_40, `level-${id}-last-next.png`) });

      // Act
      await allLevels.click();

      // Assert
      await expect(page.getByRole("heading", { name: "Puzzles", level: 2 })).toBeVisible();
      return;
    }
    await expect(page.getByRole("link", { name: "Next level" })).toHaveAttribute(
      "href",
      puzzleHash(maybeNextId),
    );
  });
}

test(`wins first-pour and opens thick-syrup ready to build from Next at ${viewportName(DESKTOP)}`, async ({
  page,
}) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installControlledRefreshRate(page, 15);
  await installSolutionPlacements(page);
  await openLevelBuilding(page, "first-pour", DESKTOP);
  expect(maybeNextPuzzleId("first-pour")).toBe("thick-syrup");

  // Act: win first-pour with the seam's reference placements.
  await speedToFour(page);
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "won");
  // Next is a hash link, so the init script does not run again. A leftover seam
  // would name first-pour while the route enters thick-syrup (D-07).
  await page.evaluate((name) => {
    Reflect.deleteProperty(window, name);
  }, PUZZLE_TEST_PLACEMENTS_GLOBAL);
  expect(await page.evaluate((name) => name in window, PUZZLE_TEST_PLACEMENTS_GLOBAL)).toBe(false);
  await page.getByRole("link", { name: "Next level" }).click();

  // Assert
  await expect(page).toHaveURL(/#\/puzzle\/thick-syrup$/);
  await expect(page.locator("main")).toHaveAttribute("data-puzzle", "thick-syrup");
  await waitForPuzzlePhase(page, "building");
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-phase", "building");
  expect(await readPlacements(page)).toBe("");
  await expect(page.locator("#player-title")).toHaveText(PUZZLE_TITLES["thick-syrup"]);
  await expect(page.locator(".puzzle-failure")).toHaveAttribute("data-failed", "false");
  await stablePuzzleCamera(page);
  expect((await readHudGeometry(page)).usedFallback).toBe(false);
});

for (const { viewport, trayWidth } of PORTRAIT_TRAY_CASES) {
  test(`shows every tray name in full at ${viewportName(viewport)}`, async ({ page }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    await installControlledRefreshRate(page, 60);
    await page.setViewportSize(viewport);
    let twoKindLevels = 0;

    for (const id of PUZZLE_IDS) {
      // Act
      await page.goto(puzzlePath(id));
      // A hash-only goto keeps the page, so wait for this level before reading it.
      await expect(page.locator("main")).toHaveAttribute("data-puzzle", id);
      await waitForPuzzlePhase(page, "building");
      await stablePuzzleCamera(page);
      const fit = await readTrayFit(page);

      // Assert
      expectTrayFits(fit, trayWidth);
      if (fit.kindCount === 2) {
        twoKindLevels += 1;
      }
    }

    // Assert (the two-kind name and font checks ran at least once)
    expect(twoKindLevels).toBeGreaterThan(0);

    // Assert (grand-finale is the last level loaded)
    expect(PUZZLE_IDS.at(-1)).toBe("grand-finale");
    await expectGrandFinaleTrayNames(page);
    await expect(page.locator(".puzzle-tray-name")).toHaveText(["Plank", "Valve"]);
    await expect(page.locator(".puzzle-tray-count")).toHaveText(["×2", "×1"]);
    if (viewport === PHONE_PORTRAIT) {
      await page.screenshot({ path: resolve(SCREENS_DIR_44, "level-grand-finale-390x844.png") });
    }
  });
}

test(`keeps grand-finale's landscape tray as icon and count at ${viewportName(DESKTOP)}`, async ({
  page,
}) => {
  // Arrange
  await installControlledRefreshRate(page, 60);

  // Act
  await openLevelBuilding(page, "grand-finale", DESKTOP);
  const nameDisplays = await page
    .locator(".puzzle-tray-name")
    .evaluateAll((names) => names.map((name) => getComputedStyle(name).display));

  // Assert
  expect(nameDisplays).toEqual(["none", "none"]);
  await expectGrandFinaleTrayNames(page);
});
