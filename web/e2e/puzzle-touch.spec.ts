import { expect, test, type Page } from "@playwright/test";
import { resolve } from "node:path";

import { installControlledRefreshRate } from "./player-helpers";
import {
  PUZZLE_PATH,
  stablePuzzleCamera,
  waitForHeld,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import {
  SCREENS_DIR_39,
  gridToPage,
  readPlacements,
  solutionExpectation,
} from "./puzzle-editor-helpers";

// Runs only in the `touch` project (hasTouch, 390x844 portrait).

const RUN_TEST_TIMEOUT_MS = 120_000;
const HELD_TIMEOUT_MS = 60_000;
const MIN_TARGET_PX = 44;
const EDITOR_BUTTONS =
  ".puzzle-tray button, .puzzle-selection-bar button, .puzzle-transport button";

function button(page: Page, name: string) {
  return page.getByRole("button", { name, exact: true });
}

/** Every visible editor button box that is smaller than a 44x44 touch target. */
async function smallTouchTargets(page: Page): Promise<string[]> {
  const buttons = page.locator(EDITOR_BUTTONS);
  const count = await buttons.count();
  expect(count).toBeGreaterThan(0);
  const small: string[] = [];
  for (let index = 0; index < count; index += 1) {
    const target = buttons.nth(index);
    const maybeBox = await target.boundingBox();
    if (maybeBox === null) {
      continue;
    }
    if (maybeBox.width < MIN_TARGET_PX || maybeBox.height < MIN_TARGET_PX) {
      const name = await target.getAttribute("aria-label");
      small.push(`${String(name)}: ${maybeBox.width}x${maybeBox.height}`);
    }
  }
  return small;
}

test("wins with taps on a phone", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installControlledRefreshRate(page, 60);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
  await stablePuzzleCamera(page);
  const main = page.locator("main");
  const solution = solutionExpectation();

  // Act: arm, then tap the reference cell from the published camera (D-28).
  await button(page, "Plank, 2 left").tap();
  await expect(main).toHaveAttribute("data-puzzle-armed", "plank");
  const point = await gridToPage(page, 8, 64);
  await page.touchscreen.tap(point.x, point.y);

  // Assert
  await expect(main).toHaveAttribute("data-puzzle-placements", "plank:8,64,0");

  // Act
  await button(page, "Rotate clockwise 15°").tap();

  // Assert: a selected tool shows every editor control at touch size (TOOL-02).
  await expect(main).toHaveAttribute("data-puzzle-placements", "plank:8,64,23");
  expect(await smallTouchTargets(page)).toEqual([]);

  // Act
  await button(page, "Play").tap();
  await waitForPuzzlePhase(page, "ended", HELD_TIMEOUT_MS);

  // Assert
  await expect(main).toHaveAttribute("data-puzzle-outcome", "won");
  await expect(page.locator(".puzzle-result-message")).toHaveText(solution.cardText);
  await expect(
    page
      .getByRole("list", { name: "Stars this run", exact: true })
      .locator('li[data-glyph="earned"]'),
  ).toHaveCount(3);
  await waitForHeld(page, HELD_TIMEOUT_MS);
  expect(await readPlacements(page)).toBe("plank:8,64,23");
  expect(await smallTouchTargets(page)).toEqual([]);
  await page.screenshot({ path: resolve(SCREENS_DIR_39, "place-win-touch.png") });
});
