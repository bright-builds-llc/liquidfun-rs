import { expect, test, type Page } from "@playwright/test";
import { resolve } from "node:path";

import { FIRST_LEVEL_HINT, PUZZLE_CAPTIONS } from "../src/catalog/puzzle-levels";
import { FIRST_HINT_STORAGE_KEY } from "../src/player/first-hint-preference";
import { installControlledRefreshRate } from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  PUZZLE_PATH,
  overlappingLeaves,
  overlappingObstacles,
  readHudGeometry,
  stablePuzzleCamera,
  waitForPuzzlePhase,
  type Viewport,
} from "./puzzle-helpers";
import { SCREENS_DIR_40, gridToPage } from "./puzzle-editor-helpers";

// Each test gets a fresh browser context, so storage starts empty.
test.describe.configure({ mode: "parallel" });

const PHONE_LANDSCAPE = { width: 844, height: 390 } as const;
const MIN_TARGET_PX = 44;
const FIRST_POUR_CAPTION = PUZZLE_CAPTIONS["first-pour"];
const REFERENCE_CELL = { x: 8, y: 64 } as const;

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

function caption(page: Page) {
  return page.locator(".puzzle-caption");
}

function captionText(page: Page) {
  return page.locator(".puzzle-caption-text");
}

function dismissButton(page: Page) {
  return page.getByRole("button", { name: "Dismiss hint", exact: true });
}

async function openBuilding(page: Page, viewport: Viewport): Promise<void> {
  await installControlledRefreshRate(page, 60);
  await page.setViewportSize(viewport);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
}

async function expectHintShown(page: Page): Promise<void> {
  await expect(caption(page)).toHaveAttribute("data-hint", "true");
  await expect(captionText(page)).toHaveText(FIRST_LEVEL_HINT);
  await expect(dismissButton(page)).toBeVisible();
}

async function expectCaptionShown(page: Page): Promise<void> {
  await expect(caption(page)).toHaveAttribute("data-hint", "false");
  await expect(captionText(page)).toHaveText(FIRST_POUR_CAPTION);
  await expect(dismissButton(page)).toHaveCount(0);
}

async function storedHintToken(page: Page): Promise<string | null> {
  return page.evaluate((key) => window.localStorage.getItem(key), FIRST_HINT_STORAGE_KEY);
}

for (const viewport of [DESKTOP, PHONE_PORTRAIT, PHONE_LANDSCAPE]) {
  test(`dismisses the hint without moving the camera at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    // Arrange
    await openBuilding(page, viewport);
    const before = await stablePuzzleCamera(page);
    await expectHintShown(page);
    const maybeDismissBox = await dismissButton(page).boundingBox();
    const beforeGeometry = await readHudGeometry(page);
    await page.screenshot({
      path: resolve(SCREENS_DIR_40, `hint-${viewportName(viewport)}-before.png`),
    });

    // Act
    await dismissButton(page).click();

    // Assert (D-19, Phase 38 D-18)
    await expectCaptionShown(page);
    const after = await stablePuzzleCamera(page);
    const afterGeometry = await readHudGeometry(page);
    await page.screenshot({
      path: resolve(SCREENS_DIR_40, `hint-${viewportName(viewport)}-after.png`),
    });
    expect(maybeDismissBox?.height ?? 0).toBeGreaterThanOrEqual(MIN_TARGET_PX);
    expect(before.usedFallback).toBe(false);
    expect(after.worldFrame).toBe(before.worldFrame);
    expect(after.pixelsPerMeter).toBe(before.pixelsPerMeter);
    expect(overlappingLeaves(beforeGeometry)).toEqual([]);
    expect(overlappingObstacles(beforeGeometry)).toEqual([]);
    expect(overlappingLeaves(afterGeometry)).toEqual([]);
    expect(overlappingObstacles(afterGeometry)).toEqual([]);
  });
}

test("keeps the hint dismissed after a reload", async ({ page }) => {
  // Arrange
  await openBuilding(page, DESKTOP);
  await expectHintShown(page);
  await dismissButton(page).click();
  await expectCaptionShown(page);

  // Act
  await page.reload();
  await waitForPuzzlePhase(page, "building");

  // Assert (LEVEL-03, D-20)
  await expectCaptionShown(page);
  expect(await storedHintToken(page)).toBe("dismissed");
});

test("placing the first tool dismisses the hint for good", async ({ page }) => {
  // Arrange
  await openBuilding(page, DESKTOP);
  await stablePuzzleCamera(page);
  await expectHintShown(page);

  // Act
  await page.getByRole("button", { name: "Plank, 2 left", exact: true }).click();
  const point = await gridToPage(page, REFERENCE_CELL.x, REFERENCE_CELL.y);
  await page.mouse.click(point.x, point.y);

  // Assert (D-20)
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-placements", "plank:8,64,0");
  await expectCaptionShown(page);

  // Act
  await page.reload();
  await waitForPuzzlePhase(page, "building");

  // Assert
  await expectCaptionShown(page);
  expect(await storedHintToken(page)).toBe("dismissed");
});

test("holds the dismissal in memory when storage is blocked", async ({ page }) => {
  // Arrange: every localStorage read throws, as in a locked-down browser.
  await page.addInitScript(() => {
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      get() {
        throw new DOMException("storage blocked", "SecurityError");
      },
    });
  });
  await openBuilding(page, DESKTOP);
  await expectHintShown(page);

  // Act
  await dismissButton(page).click();

  // Assert
  await expectCaptionShown(page);

  // Act: leave the route and come back in the same page session.
  await page.evaluate(() => {
    window.location.hash = "#/scene/dam-break";
  });
  await expect(page.locator("main")).not.toHaveAttribute("data-puzzle");
  await page.evaluate(() => {
    window.location.hash = "#/puzzle/first-pour";
  });
  await waitForPuzzlePhase(page, "building");

  // Assert (D-20)
  await expectCaptionShown(page);
});
