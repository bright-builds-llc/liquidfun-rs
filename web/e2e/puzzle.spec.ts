import { expect, test, type Page } from "@playwright/test";
import { resolve } from "node:path";

import {
  PLAYING_STATUS,
  SCENE_HASH_PATHS,
  dragCanvas,
  expectAcceptedPointerGesture,
  installControlledRefreshRate,
  numericAttribute,
  sessionStatus,
  withFormFactorShell,
} from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  PUZZLE_PATH,
  SCREENS_DIR,
  UNKNOWN_PUZZLE_PATH,
  firstBuildingCamera,
  installSolutionPlacements,
  overlappingObstacles,
  readHudGeometry,
  readPuzzleCamera,
  recordFirstBuildingCamera,
  stablePuzzleCamera,
  waitForHeld,
  waitForPuzzlePhase,
  type PuzzleCamera,
} from "./puzzle-helpers";

test.describe.configure({ mode: "parallel" });

type Viewport = { readonly width: number; readonly height: number };

const RUN_TEST_TIMEOUT_MS = 90_000;
const ENDED_TIMEOUT_MS = 60_000;
// Zero placements show the goal plus the tray hint (Phase 39 UI-SPEC empty state).
const EMPTY_BUILDING_TEXT =
  "Fill the cup to 60% before time runs out. Pick a tool from the tray to start.";
const NO_NEXT_LEVEL = "Next: No next level yet";
const EMPTY_HELD_STEP = "840";
const SOLUTION_HELD_STEP = "321";

const SCENE_CHROME_BUTTONS = [
  "Scene controls",
  "Pause scene",
  "Reset scene",
  "Zoom in",
  "Zoom out",
  "Phone accelerometer",
  "Show sliders",
  "Hide sliders",
  "Debug info",
] as const;

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

async function screenshot(page: Page, name: string): Promise<void> {
  await page.screenshot({ path: resolve(SCREENS_DIR, `${name}.png`) });
}

async function openPuzzleBuilding(page: Page, viewport: Viewport): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
}

function playButton(page: Page) {
  return page.getByRole("button", { name: "Play", exact: true });
}

function resultMessage(page: Page) {
  return page.locator(".puzzle-result-message");
}

function timer(page: Page) {
  return page.getByRole("timer", { name: "Simulation time left" });
}

function fillMeter(page: Page) {
  return page.getByRole("meter", { name: "Cup fill" });
}

async function expectResultActions(page: Page): Promise<void> {
  await expect(page.getByRole("button", { name: NO_NEXT_LEVEL })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Retry" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Edit" })).toBeEnabled();
}

async function expectSolutionWin(page: Page): Promise<void> {
  const main = page.locator("main");
  await expect(main).toHaveAttribute("data-puzzle-outcome", "won");
  await expect(resultMessage(page)).toHaveText("Filled! 76% of 60%");
  await expect(timer(page)).toHaveText("0:08.6");
  await waitForHeld(page);
  await expect(main).toHaveAttribute("data-step-index", SOLUTION_HELD_STEP);
  // The meter shows the latched count, not the last tail frame (D-11, UI-D5).
  await expect(fillMeter(page)).toHaveAttribute("aria-valuenow", "76");
  await expect(fillMeter(page)).toContainText("76%");
  await expect(fillMeter(page)).toContainText(" / 60%");
}

test("opens First Pour in building", async ({ page }) => {
  // Arrange
  await installControlledRefreshRate(page, 15);

  // Act
  await openPuzzleBuilding(page, DESKTOP);

  // Assert
  await expect(page.locator('main[data-step-index="0"]')).toBeVisible();
  await expect(page.locator("#player-title")).toHaveText("First Pour");
  await expect(page).toHaveTitle("First Pour · puzzle · liquidfun-rs playground");
  await expect(fillMeter(page)).toContainText("0%");
  await expect(fillMeter(page)).toContainText(" / 60%");
  await expect(timer(page)).toHaveText("0:12.0");
  await expect(playButton(page)).toBeVisible();
  await expect(playButton(page)).not.toHaveAttribute("aria-disabled");
  await expect(resultMessage(page)).toHaveText(EMPTY_BUILDING_TEXT);
  await expect(page.locator("canvas.scene-canvas")).toHaveAttribute(
    "data-simulation-backend",
    "direct",
  );
  await screenshot(page, "desktop-building");
});

test("loses a natural run with the out-of-time card", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installControlledRefreshRate(page, 15);
  await openPuzzleBuilding(page, PHONE_PORTRAIT);
  const main = page.locator("main");

  // Act
  await playButton(page).click();
  await waitForPuzzlePhase(page, "running");
  await expect.poll(() => numericAttribute(main, "data-step-index")).toBeGreaterThan(0);
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);

  // Assert
  await expect(main).toHaveAttribute("data-puzzle-outcome", "lost");
  await expect(resultMessage(page)).toHaveText(/^Out of time: \d+% of \d+%$/);
  await expect(resultMessage(page)).toHaveText("Out of time: 0% of 60%");
  await expect(timer(page)).toHaveText("0:00.0");
  await expectResultActions(page);
  await waitForHeld(page);
  await expect(main).toHaveAttribute("data-step-index", EMPTY_HELD_STEP);
  await screenshot(page, "phone-lost");
});

test("wins with the recorded solution through the test seam", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installControlledRefreshRate(page, 15);
  await installSolutionPlacements(page);
  await openPuzzleBuilding(page, DESKTOP);
  const main = page.locator("main");
  const meter = fillMeter(page);
  await expect(meter).toHaveAttribute("aria-valuenow", "0");

  // Act: the fill rises from live per-step frame data while running (GOAL-01).
  await playButton(page).click();
  await expect
    .poll(() => numericAttribute(meter, "aria-valuenow"), { timeout: 20_000 })
    .toBeGreaterThan(0);
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);

  // Assert
  await expectSolutionWin(page);
  await expectResultActions(page);
  await screenshot(page, "desktop-won");

  // Act: Retry replays the same placements. Running lasts about 0.8 s at
  // 15 Hz, so wait on the held flag dropping instead of the running phase.
  await page.getByRole("button", { name: "Retry" }).click();
  await expect(page.locator('main[data-puzzle-held="false"]')).toBeVisible();
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);

  // Assert: deterministic replay.
  await expectSolutionWin(page);

  // Act
  await page.getByRole("button", { name: "Edit" }).click();

  // Assert
  await waitForPuzzlePhase(page, "building");
  await expect(main).toHaveAttribute("data-step-index", "0");
  await expect(playButton(page)).not.toHaveAttribute("aria-disabled");
  await expect(timer(page)).toHaveText("0:12.0");
});

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`keeps the camera fixed across building, running and ended at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange: one step per frame, so running lasts about 12 s.
    await installControlledRefreshRate(page, 60);
    await openPuzzleBuilding(page, viewport);
    const main = page.locator("main");
    const reads: PuzzleCamera[] = [await stablePuzzleCamera(page)];

    // Act
    await playButton(page).click();
    await waitForPuzzlePhase(page, "running");
    await expect.poll(() => numericAttribute(main, "data-step-index")).toBeGreaterThan(30);
    reads.push(await readPuzzleCamera(page));
    await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
    reads.push(await readPuzzleCamera(page));
    await waitForHeld(page);
    reads.push(await readPuzzleCamera(page));

    // Assert (D-19)
    const [building] = reads;
    for (const read of reads) {
      expect(read.usedFallback).toBe(false);
      expect(read).toEqual(building);
    }
    await expect(page.locator(".puzzle-panel")).toHaveAttribute(
      "data-puzzle-layout",
      viewport === DESKTOP ? "landscape" : "portrait",
    );
  });
}

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`keeps the camera fixed from the first building frame after in-app entry at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange: a running scene has already initialized wasm, so the build
    // resolves in microtasks, ahead of any animation-frame measurement.
    await installControlledRefreshRate(page, 60);
    await page.setViewportSize(viewport);
    await page.goto(SCENE_HASH_PATHS["dam-break"]);
    await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS, { timeout: 20_000 });
    await recordFirstBuildingCamera(page);
    const main = page.locator("main");

    // Act
    await page.evaluate(() => {
      window.location.hash = "#/puzzle/first-pour";
    });
    await waitForPuzzlePhase(page, "building");
    const reads: PuzzleCamera[] = [
      await firstBuildingCamera(page),
      await stablePuzzleCamera(page),
    ];
    await playButton(page).click();
    await waitForPuzzlePhase(page, "running");
    await expect.poll(() => numericAttribute(main, "data-step-index")).toBeGreaterThan(30);
    reads.push(await readPuzzleCamera(page));
    await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
    reads.push(await readPuzzleCamera(page));

    // Assert (D-19, WR-02)
    const [building] = reads;
    for (const read of reads) {
      expect(read.usedFallback).toBe(false);
      expect(read).toEqual(building);
    }
  });
}

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`keeps every HUD leaf clear of the world at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    await installControlledRefreshRate(page, 15);
    await openPuzzleBuilding(page, viewport);

    // Act / Assert: building
    let buildingObstacles = 0;
    await expect(async () => {
      const geometry = await readHudGeometry(page);
      expect(geometry.usedFallback).toBe(false);
      expect(overlappingObstacles(geometry)).toEqual([]);
      buildingObstacles = geometry.obstacles.length;
    }).toPass({ timeout: 5_000 });
    expect(buildingObstacles).toBeGreaterThan(0);
    await expect(page.locator(".puzzle-legend-slot .scale-legend")).toBeVisible();

    // Act
    await playButton(page).click();
    await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);

    // Assert: ended keeps the same leaves clear of the world (D-18).
    await expect(async () => {
      const geometry = await readHudGeometry(page);
      expect(geometry.usedFallback).toBe(false);
      expect(overlappingObstacles(geometry)).toEqual([]);
      expect(geometry.obstacles.length).toBe(buildingObstacles);
    }).toPass({ timeout: 5_000 });
    await expect(page.locator(".puzzle-legend-slot .scale-legend")).toBeVisible();
  });
}

for (const viewport of [PHONE_PORTRAIT, DESKTOP]) {
  test(`turns scene interactions and chrome off at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    // Arrange
    await openPuzzleBuilding(page, viewport);
    const main = page.locator("main");

    // Assert: no scene chrome.
    for (const name of SCENE_CHROME_BUTTONS) {
      await expect(page.getByRole("button", { name, exact: true })).toHaveCount(0);
    }
    await expect(main.locator("input[type=range]")).toHaveCount(0);

    // Act
    const optionsTrigger = page.getByRole("button", { name: "Puzzle options" });
    await expect(optionsTrigger).toBeVisible();
    await optionsTrigger.click();

    // Assert: display options and provenance only.
    const sheet = page.getByRole("dialog", { name: "Puzzle options" });
    await expect(sheet).toBeVisible();
    await expect(sheet.getByLabel("Particles")).toBeVisible();
    await expect(sheet.locator(".site-header-source")).toHaveCount(1);
    await expect(sheet.locator(".site-footer")).toHaveCount(1);
    await expect(sheet.getByText("Phone tilt gravity")).toHaveCount(0);

    // Act
    await page.keyboard.press("Escape");
    await expect(sheet).toBeHidden();
    await dragCanvas(page);

    // Assert: the canvas drag is not forwarded (D-16, D-17).
    await expect(main).toHaveAttribute("data-pointer-accepted", "0");
    await expect(main).toHaveAttribute("data-puzzle-phase", "building");
    await expect(page.locator("canvas.scene-canvas")).not.toHaveClass(
      /\bcanvas-interactive\b/,
    );
  });
}

test("shows the form-factor layout with the legend", async ({ page }) => {
  // Arrange
  await page.setViewportSize(DESKTOP);

  // Act
  await page.goto(withFormFactorShell(PUZZLE_PATH));
  await waitForPuzzlePhase(page, "building");

  // Assert
  await expect(page.locator("html")).not.toHaveAttribute("data-canvas-stage", "true");
  await expect(playButton(page)).toBeVisible();
  await expect(fillMeter(page)).toBeVisible();
  await expect(timer(page)).toBeVisible();
  await expect(page.locator(".puzzle-result")).toBeVisible();

  // Act
  await playButton(page).click();

  // Assert
  await expect(page.locator(".viewport-frame .scale-legend")).toBeVisible();
});

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`shows a helpful fallback for an unknown level at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    // Arrange
    await page.setViewportSize(viewport);

    // Act
    await page.goto(UNKNOWN_PUZZLE_PATH);

    // Assert
    await expect(page.getByRole("heading", { name: "Level not found" })).toBeVisible();
    await expect(page.locator("body")).toContainText("No level is called “nope”.");
    await expect(page.getByRole("link", { name: "Open First Pour" })).toHaveAttribute(
      "href",
      /#\/puzzle\/first-pour$/,
    );
    await expect(page.getByRole("link", { name: "Back to scenes" })).toHaveAttribute(
      "href",
      /#\/scene\/wave-machine$/,
    );
    await expect(page).toHaveTitle("liquidfun-rs playground");

    // Act: an HTML payload in the id renders as text only (T-38-24).
    await page.goto("/liquidfun-rs/#/puzzle/%3Cimg%20src=x%20onerror=window.__pwned=1%3E");

    // Assert
    await expect(page.locator("body")).toContainText("No level is called");
    await expect(page.locator(".fallback-panel img")).toHaveCount(0);
    expect(await page.evaluate(() => "__pwned" in window)).toBe(false);

    // Act
    await page.goto("/liquidfun-rs/#/puzzle");

    // Assert
    await expect(page.locator("body")).toContainText(
      "This puzzle link does not name a level.",
    );
  });
}

test("keeps levels out of the scene catalog routes", async ({ page }) => {
  // Arrange
  await page.setViewportSize(DESKTOP);

  // Act
  await page.goto("/liquidfun-rs/#/puzzles");

  // Assert
  await expect(page.getByRole("heading", { name: "Scene not found" })).toBeVisible();

  // Act: the page layout renders the catalog sidebar in the DOM.
  await page.goto(withFormFactorShell(SCENE_HASH_PATHS["dam-break"]));

  // Assert (D-03): the check is not vacuous, and no level is listed.
  await expect(
    page.locator(".demo-sidebar").getByRole("link", { name: /Dam Break/ }),
  ).toHaveCount(1);
  await expect(page.locator('a[href*="#/puzzle/"]')).toHaveCount(0);
  await expect(page.getByRole("link", { name: /First Pour/ })).toHaveCount(0);
});

test("returns to scenes cleanly", async ({ page }) => {
  // Arrange
  await openPuzzleBuilding(page, DESKTOP);

  // Act
  await page.evaluate(() => {
    window.location.hash = "#/scene/dam-break";
  });

  // Assert
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS, { timeout: 20_000 });
  await expect(page.locator("main")).not.toHaveAttribute("data-puzzle");

  // Act
  await dragCanvas(page);

  // Assert: pointer input is reattached.
  await expectAcceptedPointerGesture(page);
});
