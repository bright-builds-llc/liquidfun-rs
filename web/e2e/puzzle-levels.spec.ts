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
import { installControlledRefreshRate } from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  overlappingLeaves,
  overlappingObstacles,
  puzzlePath,
  readHudGeometry,
  stablePuzzleCamera,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import { SCREENS_DIR_40 } from "./puzzle-editor-helpers";

// Each test gets a fresh browser context, so storage starts empty and
// first-pour shows its placement hint.
test.describe.configure({ mode: "parallel" });

type Viewport = { readonly width: number; readonly height: number };

const RUN_TEST_TIMEOUT_MS = 90_000;
const ENDED_TIMEOUT_MS = 60_000;
// The last level's Next slot links back to the grid (Phase 41 D-04).
const ALL_LEVELS = "All levels";

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
