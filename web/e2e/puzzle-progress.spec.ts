import { expect, test, type Locator, type Page } from "@playwright/test";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { PROGRESS_NOTE, PUZZLE_CAPTIONS, PUZZLE_TITLES } from "../src/catalog/puzzle-levels";
import { PUZZLE_IDS, type PuzzleId } from "../src/catalog/puzzles";
import { PROGRESS_STORAGE_KEY } from "../src/puzzle/progress";
import { DAM_BREAK_PATH, installControlledRefreshRate } from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  installSolutionPlacements,
  puzzlePath,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import { readPlacements } from "./puzzle-editor-helpers";

// Phase 41 D-20: level grid, soft locks, persisted progress and degraded
// storage in a real browser. Each test gets a fresh browser context, so
// storage starts empty; `localStorage` is shared by every page of one
// context, while init scripts are per page and re-run on every reload.
test.describe.configure({ mode: "parallel" });

type Viewport = { readonly width: number; readonly height: number };

const GRID_PATH = "/liquidfun-rs/#/puzzles";
const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const SCREENS_DIR_41 = resolve(REPO_ROOT, "target/phase41/screens");
const RUN_TEST_TIMEOUT_MS = 90_000;
const ENDED_TIMEOUT_MS = 60_000;
// A synthetic 15 Hz clock with 4× speed keeps each run to a few seconds.
const RUN_REFRESH_HZ = 15;
const DESKTOP_COLUMNS = 3;
const FIRST_POUR = "first-pour" satisfies PuzzleId;

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

function gridHeading(page: Page): Locator {
  return page.getByRole("heading", { name: "Puzzles", level: 2 });
}

function levelCard(page: Page, id: PuzzleId): Locator {
  return page.locator(`[data-level-id="${id}"]`);
}

function progressNote(page: Page): Locator {
  return page.locator(".puzzle-select [data-progress-note]");
}

async function openGrid(page: Page, viewport: Viewport): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(GRID_PATH);
  await expect(gridHeading(page)).toBeVisible();
}

async function openLevel(page: Page, id: PuzzleId, viewport: Viewport): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(puzzlePath(id));
  await waitForPuzzlePhase(page, "building");
}

/** Leaves the level through the hash lifecycle, as the in-app links do. */
async function showGrid(page: Page): Promise<void> {
  await page.evaluate(() => {
    window.location.hash = "#/puzzles";
  });
  await expect(gridHeading(page)).toBeVisible();
}

// Copied from puzzle-levels.spec.ts.
async function speedToFour(page: Page): Promise<void> {
  for (const nextSpeed of ["Speed 2×", "Speed 4×"]) {
    await page.getByRole("button", { name: /^Speed \d×$/ }).click();
    await expect(page.getByRole("button", { name: nextSpeed, exact: true })).toBeVisible();
  }
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-speed", "4");
}

/** Plays the current placements at 4× until the run ends. */
async function endRun(page: Page): Promise<void> {
  await speedToFour(page);
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
}

async function blockStorage(page: Page): Promise<void> {
  // Every localStorage read throws, as in a locked-down browser (puzzle-hint.spec.ts).
  await page.addInitScript(() => {
    Object.defineProperty(window, "localStorage", {
      configurable: true,
      get() {
        throw new DOMException("storage blocked", "SecurityError");
      },
    });
  });
}

async function storedProgress(page: Page): Promise<string | null> {
  return page.evaluate((key) => window.localStorage.getItem(key), PROGRESS_STORAGE_KEY);
}

/** Writes a raw progress value, then reloads so the app opens storage again. */
async function seedProgress(page: Page, raw: string): Promise<void> {
  await page.evaluate(
    ([key, value]) => {
      window.localStorage.setItem(key, value);
    },
    [PROGRESS_STORAGE_KEY, raw] as const,
  );
  await page.reload();
}

async function expectEveryLevelUnlocked(page: Page): Promise<void> {
  await expect(page.locator('[data-level-id][data-locked="false"]')).toHaveCount(PUZZLE_IDS.length);
}

/** Top edges of every card in registry order, read in one layout pass. */
async function cardTops(page: Page): Promise<number[]> {
  return page
    .locator("[data-level-id]")
    .evaluateAll((cards) => cards.map((card) => card.getBoundingClientRect().top));
}

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  const name = viewportName(viewport);

  test(`shows six cards with level 1 up next on fresh storage at ${name}`, async ({ page }) => {
    // Arrange
    const [firstId, ...lockedIds] = PUZZLE_IDS;

    // Act
    await openGrid(page, viewport);

    // Assert (PUZ-01, D-05..D-07)
    const ids = await page
      .locator("[data-level-id]")
      .evaluateAll((cards) => cards.map((card) => card.getAttribute("data-level-id")));
    expect(ids).toEqual([...PUZZLE_IDS]);
    for (const id of PUZZLE_IDS) {
      await expect(levelCard(page, id).locator(".puzzle-level-title")).toHaveText(
        PUZZLE_TITLES[id],
      );
      await expect(levelCard(page, id).locator(".puzzle-level-caption")).toHaveText(
        PUZZLE_CAPTIONS[id],
      );
    }
    const first = levelCard(page, firstId);
    await expect(first).toHaveAttribute("data-locked", "false");
    await expect(first).toHaveAttribute("data-next-up", "true");
    await expect(first).toHaveAttribute("href", `#/puzzle/${firstId}`);
    expect(await first.evaluate((card) => card.tagName)).toBe("A");
    for (const [index, id] of lockedIds.entries()) {
      const previousId = PUZZLE_IDS[index] ?? firstId;
      const card = levelCard(page, id);
      await expect(card).toHaveAttribute("data-locked", "true");
      await expect(card).toHaveAttribute("data-next-up", "false");
      await expect(card).toHaveAttribute("role", "group");
      await expect(card.locator(".puzzle-level-footer")).toHaveText(
        `Solve ${PUZZLE_TITLES[previousId]} to unlock`,
      );
    }
    const labels = await page
      .locator("[data-level-id] .puzzle-level-star")
      .evaluateAll((slots) => slots.map((slot) => slot.getAttribute("aria-label") ?? ""));
    expect(labels).toHaveLength(PUZZLE_IDS.length * 3);
    expect(labels.filter((label) => !label.endsWith(": not yet"))).toEqual([]);
    await expect(page.locator(".puzzle-select-progress")).toHaveText("0 of 6 solved");
    await expect(page.locator("[data-progress-note]")).toHaveCount(0);
    expect(
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth),
    ).toBe(true);
    expect(
      await page
        .locator("section.puzzle-select")
        .evaluate((section) => section.scrollWidth <= section.clientWidth),
    ).toBe(true);
    const tops = await cardTops(page);
    if (viewport === DESKTOP) {
      // Three columns (D-07): one row of three, then the next row lower.
      expect(new Set(tops.slice(0, DESKTOP_COLUMNS)).size).toBe(1);
      expect(tops[DESKTOP_COLUMNS] ?? 0).toBeGreaterThan(tops[0] ?? 0);
    } else {
      // One column: every card starts below the one before it.
      for (const [index, top] of tops.slice(1).entries()) {
        expect(top).toBeGreaterThan(tops[index] ?? top);
      }
    }
    await page.screenshot({ path: resolve(SCREENS_DIR_41, `grid-fresh-${name}.png`) });
    const last = levelCard(page, PUZZLE_IDS[PUZZLE_IDS.length - 1] ?? firstId);
    await last.scrollIntoViewIfNeeded();
    await expect(last).toBeInViewport();
    const source = page.getByRole("link", { name: "View source on GitHub" });
    await source.scrollIntoViewIfNeeded();
    await expect(source).toBeVisible();
  });

  test(`opens the grid from the sidebar Puzzles link at ${name}`, async ({ page }) => {
    // Arrange
    await page.setViewportSize(viewport);
    await page.goto(DAM_BREAK_PATH);
    await page.getByRole("button", { name: "Demos" }).click();

    // Act
    await page
      .getByRole("dialog", { name: "Demos" })
      .getByRole("link", { name: "Puzzles", exact: true })
      .click();

    // Assert (D-03)
    await expect(gridHeading(page)).toBeVisible();
    await expect(page).toHaveURL(/#\/puzzles$/);
    await page.getByRole("button", { name: "Demos" }).click();
    const gridDialog = page.getByRole("dialog", { name: "Demos" });
    await expect(gridDialog.getByRole("link", { name: "Puzzles", exact: true })).toHaveAttribute(
      "aria-current",
      "page",
    );
    await page.keyboard.press("Escape");
    await expect(gridDialog).toHaveCount(0);

    // Act
    await levelCard(page, FIRST_POUR).click();

    // Assert: a level route marks the section, not the page (UI-D24).
    await waitForPuzzlePhase(page, "building");
    await page.getByRole("button", { name: "Demos" }).click();
    await expect(
      page
        .getByRole("dialog", { name: "Demos" })
        .getByRole("link", { name: "Puzzles", exact: true }),
    ).toHaveAttribute("aria-current", "true");
  });

  test(`opens a locked level from a deep link at ${name}`, async ({ page }) => {
    // Arrange
    const lockedId: PuzzleId = "timed-gate";

    // Act
    await openLevel(page, lockedId, viewport);

    // Assert (PROG-02): soft locks never redirect or block a deep link.
    await expect(page).toHaveURL(/#\/puzzle\/timed-gate$/);
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-phase", "building");
    await expect(page.getByText(/\blocked\b/i)).toHaveCount(0);

    // Act
    await showGrid(page);

    // Assert
    await expect(levelCard(page, lockedId)).toHaveAttribute("data-locked", "true");
  });

  test(`leaves the drawer with no overlay when All levels is used at ${name}`, async ({
    page,
  }) => {
    // Arrange
    await openLevel(page, FIRST_POUR, viewport);
    await page.getByRole("button", { name: "Puzzle options" }).click();

    // Act
    await page
      .getByRole("dialog", { name: "Puzzle options" })
      .getByRole("link", { name: "All levels", exact: true })
      .click();

    // Assert (D-04): no leftover dialog, inert layer or scroll lock.
    await expect(gridHeading(page)).toBeVisible();
    await expect(page).toHaveURL(/#\/puzzles$/);
    await expect(page.getByRole("dialog")).toHaveCount(0);
    const last = levelCard(page, PUZZLE_IDS[PUZZLE_IDS.length - 1] ?? FIRST_POUR);
    await last.scrollIntoViewIfNeeded();
    await expect(last).toBeInViewport();

    // Act
    await levelCard(page, FIRST_POUR).click();

    // Assert
    await waitForPuzzlePhase(page, "building");
  });

  test(`unlocks every level and stays playable with blocked storage at ${name}`, async ({
    page,
  }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    await blockStorage(page);
    await installControlledRefreshRate(page, RUN_REFRESH_HZ);
    await installSolutionPlacements(page);

    // Act
    await openGrid(page, viewport);

    // Assert (PROG-03, D-14, D-15)
    await expect(page.locator("section.puzzle-select")).toHaveAttribute(
      "data-progress-degraded",
      "true",
    );
    await expectEveryLevelUnlocked(page);
    await expect(progressNote(page)).toHaveText(PROGRESS_NOTE);

    // Act
    await levelCard(page, FIRST_POUR).click();
    await waitForPuzzlePhase(page, "building");
    await page.getByRole("button", { name: "Puzzle options" }).click();

    // Assert
    const options = page.getByRole("dialog", { name: "Puzzle options" });
    await expect(options.getByText(PROGRESS_NOTE)).toBeVisible();
    await expect(options.getByRole("link", { name: "All levels", exact: true })).toBeVisible();

    // Act
    await page.keyboard.press("Escape");
    await expect(options).toHaveCount(0);
    await endRun(page);

    // Assert
    await expect(page.locator(".puzzle-result")).toBeVisible();
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "won");

    // Act
    await showGrid(page);

    // Assert: this session's stars live in memory only.
    await expect(levelCard(page, FIRST_POUR)).toHaveAttribute("data-solved", "true");
    await expect(progressNote(page)).toHaveText(PROGRESS_NOTE);
  });

  test(`keeps a future-version record byte-for-byte at ${name}`, async ({ page }) => {
    test.setTimeout(RUN_TEST_TIMEOUT_MS);
    // Arrange
    const FUTURE_RAW = '{"version":2,"levels":{"first-pour":{"win":true}}}';
    await installControlledRefreshRate(page, RUN_REFRESH_HZ);
    await openGrid(page, viewport);
    await installSolutionPlacements(page);
    await seedProgress(page, FUTURE_RAW);

    // Assert (PROG-03, D-14)
    await expect(progressNote(page)).toHaveText(PROGRESS_NOTE);
    await expectEveryLevelUnlocked(page);

    // Act
    await levelCard(page, FIRST_POUR).click();
    await waitForPuzzlePhase(page, "building");
    await endRun(page);
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "won");
    await showGrid(page);

    // Assert: degraded mode never writes (T-41-25).
    await expect(levelCard(page, FIRST_POUR)).toHaveAttribute("data-solved", "true");
    expect(await storedProgress(page)).toBe(FUTURE_RAW);
  });

  test(`keeps a corrupt record byte-for-byte at ${name}`, async ({ page }) => {
    // Arrange
    const CORRUPT_RAW = "{not json";
    await openGrid(page, viewport);
    await seedProgress(page, CORRUPT_RAW);
    await expect(progressNote(page)).toHaveText(PROGRESS_NOTE);

    // Act
    await levelCard(page, FIRST_POUR).click();
    await waitForPuzzlePhase(page, "building");
    expect(await readPlacements(page)).toBe("");
    await showGrid(page);

    // Assert (PROG-03, T-41-25)
    await expect(progressNote(page)).toHaveText(PROGRESS_NOTE);
    expect(await storedProgress(page)).toBe(CORRUPT_RAW);
  });
}
