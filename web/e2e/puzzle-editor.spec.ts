import { expect, test, type Browser, type Page } from "@playwright/test";
import { resolve } from "node:path";

import { installControlledRefreshRate, numericAttribute } from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  PUZZLE_PATH,
  installSolutionPlacements,
  overlappingObstacles,
  readHudGeometry,
  stablePuzzleCamera,
  waitForHeld,
  waitForPuzzlePhase,
  type PuzzleCamera,
} from "./puzzle-helpers";
import {
  SCREENS_DIR_39,
  expectPlayEnabled,
  gridToPage,
  readRunFingerprint,
  solutionExpectation,
  type RunFingerprint,
} from "./puzzle-editor-helpers";

test.describe.configure({ mode: "parallel" });

type Viewport = { readonly width: number; readonly height: number };

const PHONE_LANDSCAPE = { width: 844, height: 390 } as const;
const RUN_TEST_TIMEOUT_MS = 120_000;
const HELD_TIMEOUT_MS = 60_000;
const SOLUTION = solutionExpectation();

// The reference solution: one plank at grid (8, 64), one clockwise turn (index 23).
const REFERENCE_CELL = { x: 8, y: 64 } as const;
const CENTRE_CELL = { x: 10, y: 64 } as const;
const CUP_CELL = { x: 40, y: 44 } as const;
const REFERENCE_UNROTATED = "plank:8,64,0";
const REFERENCE_PLACEMENT = "plank:8,64,23";
const CENTRE_PLACEMENT = "plank:10,64,0";
const HELD_HASH_PATTERN = /^[0-9a-f]{16}$/;

const CUP_STATUS = "Overlaps the cup. Move or remove that tool to play.";
const TWO_TOOLS_STATUS =
  "Overlaps another tool. 2 tools need moving or removing before you can play.";

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

function main(page: Page) {
  return page.locator("main");
}

function button(page: Page, name: string) {
  return page.getByRole("button", { name, exact: true });
}

function trayPlank(page: Page) {
  return page.locator(".puzzle-tray-item").first();
}

function statusLine(page: Page) {
  return page.locator("#puzzle-status-line");
}

function starRows(page: Page, listName: "Stars" | "Stars this run", glyph: string) {
  return page
    .getByRole("list", { name: listName, exact: true })
    .locator(`li[data-glyph="${glyph}"]`);
}

async function openEditor(page: Page, viewport: Viewport): Promise<void> {
  await installControlledRefreshRate(page, 60);
  await page.setViewportSize(viewport);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
  await stablePuzzleCamera(page);
}

async function clickCell(page: Page, cell: { readonly x: number; readonly y: number }) {
  const point = await gridToPage(page, cell.x, cell.y);
  await page.mouse.click(point.x, point.y);
}

async function expectPlacements(page: Page, value: string): Promise<void> {
  await expect(main(page)).toHaveAttribute("data-puzzle-placements", value);
}

/** Arms the plank and clicks the reference cell, then turns it clockwise once. */
async function placeReferenceWithMouse(page: Page): Promise<void> {
  await button(page, "Plank, 2 left").click();
  await clickCell(page, REFERENCE_CELL);
  await expectPlacements(page, REFERENCE_UNROTATED);
  await button(page, "Rotate clockwise 15°").click();
  await expectPlacements(page, REFERENCE_PLACEMENT);
}

async function waitForSteps(page: Page, minimum: number): Promise<void> {
  await expect
    .poll(() => numericAttribute(main(page), "data-step-index"))
    .toBeGreaterThan(minimum);
}

/** Presses Play and waits until the decided run has finished its tail. */
async function playToHeld(page: Page): Promise<RunFingerprint> {
  await button(page, "Play").click();
  await waitForHeld(page, HELD_TIMEOUT_MS);
  return readRunFingerprint(page);
}

/** A clean reference run in a fresh context, placed through the test seam. */
async function cleanSeamRun(browser: Browser, viewport: Viewport): Promise<RunFingerprint> {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await installSolutionPlacements(page);
    await openEditor(page, viewport);
    await expectPlacements(page, REFERENCE_PLACEMENT);
    return await playToHeld(page);
  } finally {
    await context.close();
  }
}

/** The fixture's decision values (D-10, D-28). */
function expectSolutionDecision(run: RunFingerprint): void {
  expect(run.outcomeStep).toBe(SOLUTION.outcomeStep);
  expect(run.decidedStep).toBe(SOLUTION.decidedStep);
  expect(run.peakHeld).toBe(SOLUTION.peakHeld);
  expect(run.heldHash).toMatch(HELD_HASH_PATTERN);
}

test("wins by placing, rotating and playing with the mouse", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await openEditor(page, DESKTOP);

  // Act: arm the plank.
  await button(page, "Plank, 2 left").click();

  // Assert
  await expect(trayPlank(page)).toHaveAttribute("aria-pressed", "true");
  await expect(main(page)).toHaveAttribute("data-puzzle-armed", "plank");

  // Act: place at the reference cell from the published camera (D-28).
  await clickCell(page, REFERENCE_CELL);

  // Assert
  await expectPlacements(page, REFERENCE_UNROTATED);
  await expect(button(page, "Plank, 1 left")).toBeVisible();

  // Act
  await button(page, "Rotate clockwise 15°").click();

  // Assert
  await expectPlacements(page, REFERENCE_PLACEMENT);
  await expectPlayEnabled(page, true);

  // Act
  await button(page, "Play").click();
  await waitForPuzzlePhase(page, "ended", HELD_TIMEOUT_MS);

  // Assert: the D-10 win card and all three stars (STAR-01, STAR-02).
  await expect(main(page)).toHaveAttribute("data-puzzle-outcome", "won");
  await expect(page.locator(".puzzle-result-message")).toHaveText(SOLUTION.cardText);
  await expect(starRows(page, "Stars this run", "earned")).toHaveCount(3);
  await waitForHeld(page, HELD_TIMEOUT_MS);
  await expect(main(page)).toHaveAttribute("data-step-index", SOLUTION.heldStep);
  await page.screenshot({ path: resolve(SCREENS_DIR_39, "place-win-desktop.png") });
});

test("drags a tool with the grab offset", async ({ page }) => {
  // Arrange
  await openEditor(page, DESKTOP);
  await trayPlank(page).focus();
  await page.keyboard.press("Enter");
  await expectPlacements(page, CENTRE_PLACEMENT);
  const from = await gridToPage(page, CENTRE_CELL.x, CENTRE_CELL.y);
  const to = await gridToPage(page, REFERENCE_CELL.x, REFERENCE_CELL.y);

  // Act
  await page.mouse.move(from.x, from.y);
  await page.mouse.down();
  await page.mouse.move(to.x, to.y, { steps: 5 });
  await page.mouse.up();

  // Assert: the editor moved the tool; nothing reached the physics pointer.
  await expectPlacements(page, REFERENCE_UNROTATED);
  await expect(main(page)).toHaveAttribute("data-pointer-accepted", "0");
});

test("plays from the keyboard", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await openEditor(page, DESKTOP);
  await trayPlank(page).focus();

  // Act / Assert: Enter places at the build-area centre (TOOL-06).
  await page.keyboard.press("Enter");
  await expectPlacements(page, CENTRE_PLACEMENT);

  // Act / Assert: nudge two cells left, then turn clockwise.
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");
  await expectPlacements(page, REFERENCE_UNROTATED);
  await page.keyboard.press("Shift+R");
  await expectPlacements(page, REFERENCE_PLACEMENT);

  // Act: Shift+Tab reaches the canvas, and Space plays from there.
  await page.keyboard.press("Shift+Tab");
  await expect(page.getByRole("application")).toBeFocused();
  await page.keyboard.press("Space");

  // Assert
  await expect(main(page)).not.toHaveAttribute("data-puzzle-phase", "building");
  await waitForHeld(page, HELD_TIMEOUT_MS);
  await expect(main(page)).toHaveAttribute("data-puzzle-outcome", "won");
});

test("removes the selected tool with Delete", async ({ page }) => {
  // Arrange
  await openEditor(page, DESKTOP);
  await trayPlank(page).focus();
  await page.keyboard.press("Enter");
  await expectPlacements(page, CENTRE_PLACEMENT);

  // Act
  await page.keyboard.press("Delete");

  // Assert
  await expectPlacements(page, "");
  await expect(button(page, "Plank, 2 left")).toBeVisible();
});

test("highlights invalid placements and disables Play", async ({ page }) => {
  // Arrange
  await openEditor(page, DESKTOP);

  // Act: one plank across the cup.
  await button(page, "Plank, 2 left").click();
  await clickCell(page, CUP_CELL);

  // Assert (D-20, D-21)
  await expect(statusLine(page)).toHaveText(CUP_STATUS);
  await expect(button(page, "Play")).toHaveAttribute("aria-disabled", "true");
  await expect(button(page, "Play")).toHaveAttribute("aria-describedby", "puzzle-status-line");
  await expectPlayEnabled(page, false);

  // Act
  await button(page, "Clear all tools").click();

  // Assert
  await expectPlacements(page, "");
  await expectPlayEnabled(page, true);
  await expect(button(page, "Plank, 2 left")).toBeVisible();

  // Act: two planks at the centre overlap; the second is selected.
  await trayPlank(page).focus();
  await page.keyboard.press("Enter");
  await expectPlacements(page, CENTRE_PLACEMENT);
  await page.keyboard.press("Enter");

  // Assert
  await expectPlacements(page, `${CENTRE_PLACEMENT};${CENTRE_PLACEMENT}`);
  await expect(statusLine(page)).toHaveText(TWO_TOOLS_STATUS);
  await expect(trayPlank(page)).toHaveAccessibleName("Plank, none left");
  await expect(trayPlank(page)).toHaveAttribute("aria-disabled", "true");
  await expectPlayEnabled(page, false);

  // Act
  await button(page, "Clear all tools").click();

  // Assert
  await expectPlacements(page, "");
  await expectPlayEnabled(page, true);
});

test("Stop keeps placements and replays the same outcome", async ({ page, browser }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await openEditor(page, DESKTOP);
  await placeReferenceWithMouse(page);

  // Act: stop part-way through a run (TOOL-04).
  await button(page, "Play").click();
  await waitForSteps(page, 20);
  await button(page, "Stop").click();

  // Assert
  await waitForPuzzlePhase(page, "building");
  await expectPlacements(page, REFERENCE_PLACEMENT);
  await expect(main(page)).toHaveAttribute("data-step-index", "0");

  // Act
  const replay = await playToHeld(page);
  const clean = await cleanSeamRun(browser, DESKTOP);

  // Assert
  expect(replay).toEqual(clean);
  expectSolutionDecision(replay);
});

test("keeps the best stars across Edit, Stop and a losing run", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installSolutionPlacements(page);
  await openEditor(page, DESKTOP);
  await expect(starRows(page, "Stars", "none")).toHaveCount(3);

  // Act: win with all three stars.
  await playToHeld(page);
  await expect(starRows(page, "Stars this run", "earned")).toHaveCount(3);
  await button(page, "Edit").click();
  await waitForPuzzlePhase(page, "building");

  // Assert: building shows the best-of stars (STAR-02).
  await expect(starRows(page, "Stars", "earned")).toHaveCount(3);

  // Act: a stopped run and a losing run with no tools.
  await button(page, "Play").click();
  await waitForSteps(page, 5);
  await button(page, "Stop").click();
  await waitForPuzzlePhase(page, "building");
  await expect(starRows(page, "Stars", "earned")).toHaveCount(3);
  await button(page, "Clear all tools").click();
  await expectPlacements(page, "");
  await button(page, "Play").click();
  await waitForPuzzlePhase(page, "ended", HELD_TIMEOUT_MS);

  // Assert: nothing earned this run, but the best stars are kept.
  await expect(main(page)).toHaveAttribute("data-puzzle-outcome", "lost");
  await expect(starRows(page, "Stars this run", "best")).toHaveCount(3);
  await button(page, "Edit").click();
  await waitForPuzzlePhase(page, "building");
  await expect(starRows(page, "Stars", "earned")).toHaveCount(3);
});

for (const viewport of [DESKTOP, PHONE_PORTRAIT, PHONE_LANDSCAPE]) {
  test(`keeps the camera fixed across place, Play and Stop at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    await openEditor(page, viewport);
    const reads: PuzzleCamera[] = [await stablePuzzleCamera(page)];
    expect(overlappingObstacles(await readHudGeometry(page))).toEqual([]);

    // Act
    await placeReferenceWithMouse(page);
    reads.push(await stablePuzzleCamera(page));
    await button(page, "Play").click();
    await waitForSteps(page, 20);
    reads.push(await stablePuzzleCamera(page));
    const runningOverlaps = overlappingObstacles(await readHudGeometry(page));
    await button(page, "Stop").click();
    await waitForPuzzlePhase(page, "building");
    reads.push(await stablePuzzleCamera(page));

    // Assert (D-26)
    expect(runningOverlaps).toEqual([]);
    const [building] = reads;
    for (const read of reads) {
      expect(read.usedFallback).toBe(false);
      expect(read).toEqual(building);
    }
  });
}

test("edit history gives the same fingerprint as a clean placement", async ({
  page,
  browser,
}) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS * 2);
  // Arrange
  await openEditor(page, DESKTOP);

  // Act: place, remove, re-place, turn, Play and Stop, three times (D-27).
  for (let round = 0; round < 3; round += 1) {
    if (round > 0) {
      await clickCell(page, REFERENCE_CELL);
      await expect(main(page)).toHaveAttribute("data-puzzle-selected", "0");
      await button(page, "Remove selected tool").click();
      await expectPlacements(page, "");
    }
    await button(page, "Plank, 2 left").click();
    await clickCell(page, REFERENCE_CELL);
    await expectPlacements(page, REFERENCE_UNROTATED);
    await button(page, "Remove selected tool").click();
    await expectPlacements(page, "");
    await placeReferenceWithMouse(page);
    await button(page, "Play").click();
    await waitForSteps(page, 5);
    await button(page, "Stop").click();
    await waitForPuzzlePhase(page, "building");
  }
  const edited = await playToHeld(page);
  const clean = await cleanSeamRun(browser, DESKTOP);

  // Assert
  expect(edited.heldHash).toMatch(HELD_HASH_PATTERN);
  expect(clean.heldHash).toMatch(HELD_HASH_PATTERN);
  expect(edited).toEqual(clean);
  await expect(main(page)).toHaveAttribute("data-puzzle-held-hash", clean.heldHash);
});

test("fast-forward keeps outcome, decision and peak", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS * 2);
  // Arrange
  await installSolutionPlacements(page);
  await openEditor(page, DESKTOP);
  const runs: RunFingerprint[] = [];

  // Act: 1x
  runs.push(await playToHeld(page));

  // Act: 2x and 4x replays of the same placements (TOOL-07).
  for (const nextSpeed of ["Speed 2×", "Speed 4×"]) {
    await page.getByRole("button", { name: /^Speed \d×$/ }).click();
    await expect(button(page, nextSpeed)).toBeVisible();
    await button(page, "Retry").click();
    await expect(page.locator('main[data-puzzle-held="false"]')).toBeVisible();
    await waitForHeld(page, HELD_TIMEOUT_MS);
    runs.push(await readRunFingerprint(page));
  }

  // Assert (D-28)
  await expect(main(page)).toHaveAttribute("data-puzzle-speed", "4");
  const [normal] = runs;
  for (const run of runs) {
    expect(run).toEqual(normal);
    expectSolutionDecision(run);
  }
});
