import { expect, test, type Browser, type Locator, type Page } from "@playwright/test";
import { createHash } from "node:crypto";
import { mkdirSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  canvasPixelSha256,
  closeSceneControls,
  DAM_BREAK_PATH,
  expectReadySceneChrome,
  FOUNTAIN_PATH,
  installControlledRefreshRate,
  numericAttribute,
  openSceneControls,
  PAUSED_STATUS,
  SCENE_HASH_PATHS,
  sessionStatus,
  withFormFactorShell,
} from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  PUZZLE_PATH,
  installSolutionPlacements,
  waitForHeld,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import { readRunFingerprint, type RunFingerprint } from "./puzzle-editor-helpers";
import {
  chooseRenderMode,
  drawnSurface,
  forceNoFloatTargets,
  forceWaterUnavailable,
  loseParticleContext,
  openPuzzleOptions,
  particlesSelect,
  restoreParticleContext,
  storeRenderMode,
  type StoredRenderMode,
} from "./water-helpers";

// Phase 43 D-19: the Water render mode in a real browser (Chromium with
// SwiftShader WebGL2). Every claim reads DOM attributes, computed styles,
// 2D overlay hashes or file hashes; shader output is never pixel-diffed.

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const SCREENS_DIR_43 = resolve(REPO_ROOT, "target/phase43/screens");
const WATER = "Water";
const SHADED_BLOB = "Shaded blob";
const WATER_UNAVAILABLE = "Water (unavailable)";
const TRANSPARENT = "rgba(0, 0, 0, 0)";
const LOOK_PLAY_MS = 3_000;
const SCENE_TEST_TIMEOUT_MS = 90_000;
const RUN_TEST_TIMEOUT_MS = 150_000;
const EXPORT_TEST_TIMEOUT_MS = 240_000;
const ENDED_TIMEOUT_MS = 60_000;
const WORKER_TIMEOUT_MS = 60_000;
// A synthetic 15 Hz clock with 4x speed keeps each puzzle run to a few seconds.
const RUN_REFRESH_HZ = 15;
const MID_RUN_STEP = 150;

async function screenshot(page: Page, name: string): Promise<void> {
  mkdirSync(SCREENS_DIR_43, { recursive: true });
  await page.screenshot({ path: resolve(SCREENS_DIR_43, `${name}.png`) });
}

async function openScene(page: Page, path: string, title: string): Promise<void> {
  await page.setViewportSize(DESKTOP);
  await page.goto(path);
  await expectReadySceneChrome(page, title);
}

type Viewport = { readonly width: number; readonly height: number };

async function openPuzzle(page: Page, viewport: Viewport = DESKTOP): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
}

async function expectDrawn(page: Page, surface: string): Promise<void> {
  await expect(drawnSurface(page)).toHaveAttribute("data-drawn-surface", surface);
}

async function pauseScene(page: Page): Promise<void> {
  await closeSceneControls(page);
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
}

/**
 * Checks the option's DOM `disabled` property. Playwright's `toBeDisabled`
 * reports a natively disabled `<option>` inside a closed select as enabled.
 */
async function expectOptionDisabled(
  select: Locator,
  name: string,
  disabled: boolean,
): Promise<void> {
  const option = select.getByRole("option", { name, exact: true });
  await expect(option).toHaveCount(1);
  await expect(option).toHaveJSProperty("disabled", disabled);
}

/** Alpha of one 2D scene-canvas pixel, read in the page. */
async function sceneCanvasAlpha(page: Page, x: number, y: number): Promise<number> {
  return drawnSurface(page).evaluate(
    (canvas, point) => {
      const maybeContext = (canvas as HTMLCanvasElement).getContext("2d");
      if (maybeContext === null) {
        throw new Error("Canvas 2D is unavailable");
      }
      return maybeContext.getImageData(point.x, point.y, 1, 1).data[3] ?? -1;
    },
    { x, y },
  );
}

/** Number of 2D scene-canvas pixels with alpha above zero. */
async function opaqueOverlayPixels(page: Page): Promise<number> {
  return drawnSurface(page).evaluate((canvas) => {
    const element = canvas as HTMLCanvasElement;
    const maybeContext = element.getContext("2d");
    if (maybeContext === null) {
      throw new Error("Canvas 2D is unavailable");
    }
    const pixels = maybeContext.getImageData(0, 0, element.width, element.height).data;
    let count = 0;
    for (let index = 3; index < pixels.length; index += 4) {
      if ((pixels[index] ?? 0) > 0) {
        count += 1;
      }
    }
    return count;
  });
}

// Copied from puzzle-sound.spec.ts.
async function speedToFour(page: Page): Promise<void> {
  for (const nextSpeed of ["Speed 2×", "Speed 4×"]) {
    await page.getByRole("button", { name: /^Speed \d×$/ }).click();
    await expect(page.getByRole("button", { name: nextSpeed, exact: true })).toBeVisible();
  }
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-speed", "4");
}

/** Plays the seam solution on a fresh page in the stored mode. */
async function playSolution(
  browser: Browser,
  mode: StoredRenderMode,
): Promise<{ readonly fingerprint: RunFingerprint; readonly speedLane: string | null }> {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await storeRenderMode(page, mode);
    await installControlledRefreshRate(page, RUN_REFRESH_HZ);
    await installSolutionPlacements(page);
    await openPuzzle(page);
    await speedToFour(page);
    await page.getByRole("button", { name: "Play", exact: true }).click();
    await waitForPuzzlePhase(page, "running");
    const speedLane = await drawnSurface(page).getAttribute("data-speed-lane");
    await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
    await waitForHeld(page);
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "won");
    return { fingerprint: await readRunFingerprint(page), speedLane };
  } finally {
    await context.close();
  }
}

/** Generates the 1 s Dam Break SVG on a fresh page and hashes the file. */
async function exportDamBreakSha256(
  browser: Browser,
  mode: StoredRenderMode,
): Promise<string> {
  const context = await browser.newContext({ viewport: DESKTOP });
  try {
    const page = await context.newPage();
    await storeRenderMode(page, mode);
    await page.goto(DAM_BREAK_PATH);
    await expectReadySceneChrome(page, "Dam Break");
    await expect(page.locator("main")).toHaveAttribute("data-render-mode", mode);
    await openSceneControls(page);
    await page.getByLabel("Seconds").fill("1");
    const downloadPromise = page.waitForEvent("download");
    await page.getByRole("button", { name: "Generate animated SVG" }).click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe("dam-break-1s.svg");
    const maybePath = await download.path();
    if (maybePath === null) {
      throw new Error("the SVG download has no file");
    }
    const svg = await readFile(maybePath);
    expect(svg.subarray(0, 4).toString("utf8")).toBe("<svg");
    return createHash("sha256").update(svg).digest("hex");
  } finally {
    await context.close();
  }
}

test("draws Water in a scene with the speed lane under a transparent overlay", async ({
  page,
}) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await openScene(page, DAM_BREAK_PATH, "Dam Break");
  const select = await particlesSelect(page);
  await expectOptionDisabled(select, WATER, false);

  // Act
  await chooseRenderMode(page, WATER);

  // Assert (WATER-01, D-15, D-16)
  await expect(page.locator("main")).toHaveAttribute("data-render-mode", "water");
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "present");
  await expect(page.locator("canvas.particle-surface")).toHaveAttribute("data-active", "true");
  await expect(drawnSurface(page)).toHaveCSS("background-color", TRANSPARENT);
  await closeSceneControls(page);
  await page.waitForTimeout(LOOK_PLAY_MS);
  await expectDrawn(page, "webgl-water");
  await screenshot(page, "dam-break-water");
});

test("switches from Water back to Shaded blob on the next frames", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");
  await openScene(page, DAM_BREAK_PATH, "Dam Break");
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "present");

  // Act
  await chooseRenderMode(page, SHADED_BLOB);

  // Assert
  await expect(page.locator("main")).toHaveAttribute("data-render-mode", "shaded-blob");
  await expectDrawn(page, "webgl");
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "absent");
});

test("draws Water with the speed lane on the Tesla Valve worker", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange: no synthetic clock, so the live worker path runs.
  await openScene(page, SCENE_HASH_PATHS["tesla-valve"], "Tesla Valve");
  await expect(drawnSurface(page)).toHaveAttribute("data-simulation-backend", "worker", {
    timeout: WORKER_TIMEOUT_MS,
  });

  // Act
  await chooseRenderMode(page, WATER);

  // Assert (D-11 worker owner)
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "present", {
    timeout: WORKER_TIMEOUT_MS,
  });
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveAttribute("data-simulation-backend", "worker");
});

test("draws Water in a puzzle with overlays identical to Shaded blob", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");
  await openPuzzle(page);
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveCSS("background-color", TRANSPARENT);
  const waterOverlay = await canvasPixelSha256(page);
  const waterOpaque = await opaqueOverlayPixels(page);

  // Act
  await chooseRenderMode(page, SHADED_BLOB);
  await page.keyboard.press("Escape");
  await expectDrawn(page, "webgl");

  // Assert (D-16): walls, cup, target line, tools and labels draw the same
  // on the 2D layer above either GPU surface.
  expect(await canvasPixelSha256(page)).toBe(waterOverlay);
  expect(waterOpaque).toBeGreaterThan(0);
});

test("gives the same first-pour run in Water and Shaded blob", async ({ browser }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS * 2);
  // Act
  const water = await playSolution(browser, "water");
  const shaded = await playSolution(browser, "shaded-blob");

  // Assert (D-11)
  expect(water.speedLane).toBe("present");
  expect(shaded.speedLane).toBe("absent");
  expect(water.fingerprint).toEqual(shaded.fingerprint);
  test.info().annotations.push({
    type: "run-fingerprint",
    description: JSON.stringify({ water: water.fingerprint, shaded: shaded.fingerprint }),
  });
});

test("falls back to canvas on a lost context and recovers Water while paused", async ({
  page,
}) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");
  await openScene(page, DAM_BREAK_PATH, "Dam Break");
  await expectDrawn(page, "webgl-water");
  await pauseScene(page);
  const pausedStep = await numericAttribute(page.locator("main"), "data-step-index");

  // Act
  await loseParticleContext(page);

  // Assert (WATER-02, D-14)
  await expectDrawn(page, "canvas");

  // Act
  await restoreParticleContext(page);

  // Assert
  await expectDrawn(page, "webgl-water");
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
  expect(await numericAttribute(page.locator("main"), "data-step-index")).toBe(pausedStep);
});

test("falls back to canvas on a lost context and recovers Water while building", async ({
  page,
}) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");
  await openPuzzle(page);
  await expectDrawn(page, "webgl-water");

  // Act
  await loseParticleContext(page);

  // Assert
  await expectDrawn(page, "canvas");

  // Act
  await restoreParticleContext(page);

  // Assert
  await expectDrawn(page, "webgl-water");
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-phase", "building");
});

test("shows Water (unavailable) in both pickers and draws Shaded blob", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await forceWaterUnavailable(page);
  await storeRenderMode(page, "water");

  // Act
  await openScene(page, DAM_BREAK_PATH, "Dam Break");
  const sceneSelect = await particlesSelect(page);

  // Assert (WATER-02, D-05, UI-D49)
  await expectOptionDisabled(sceneSelect, WATER_UNAVAILABLE, true);
  await expect(sceneSelect.getByRole("option", { name: WATER, exact: true })).toHaveCount(0);
  await expect(sceneSelect).toHaveValue("water");
  await expectDrawn(page, "webgl");

  // Act
  await closeSceneControls(page);
  await openPuzzle(page);
  const dialog = await openPuzzleOptions(page);
  const puzzleSelect = dialog.getByRole("combobox", { name: "Particles", exact: true });

  // Assert
  await expectOptionDisabled(puzzleSelect, WATER_UNAVAILABLE, true);
  await expect(puzzleSelect.getByRole("option", { name: WATER, exact: true })).toHaveCount(0);
  await expect(puzzleSelect).toHaveValue("water");
  await expectDrawn(page, "webgl");
  // The drawer slides in; capture it once the transition ends.
  await expect(dialog).not.toHaveAttribute("data-transitioning");
  await expect(puzzleSelect).toBeInViewport();
  await screenshot(page, "water-unavailable-drawer");
});

test("paints the opaque canvas fallback without float render targets", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await forceNoFloatTargets(page);
  await storeRenderMode(page, "water");

  // Act
  await openScene(page, DAM_BREAK_PATH, "Dam Break");

  // Assert (WATER-02, D-12): the 2D layer paints its own #071018 fill.
  await expectDrawn(page, "canvas");
  expect(await sceneCanvasAlpha(page, 0, 0)).toBe(255);

  // Act
  await chooseRenderMode(page, SHADED_BLOB);
  await closeSceneControls(page);

  // Assert
  await expect(page.locator("main")).toHaveAttribute("data-render-mode", "shaded-blob");
  await expectDrawn(page, "canvas");
  expect(await sceneCanvasAlpha(page, 0, 0)).toBe(255);
});

test("exports the same one-second SVG in Water and Shaded blob", async ({ browser }) => {
  test.setTimeout(EXPORT_TEST_TIMEOUT_MS);
  // Act
  const shaded = await exportDamBreakSha256(browser, "shaded-blob");
  const water = await exportDamBreakSha256(browser, "water");

  // Assert (WATER-03, D-17)
  expect(water).toBe(shaded);
  test.info().annotations.push({ type: "svg-sha256", description: water });
});

test("offers Water in the form-factor puzzle control row", async ({ page }) => {
  // Arrange
  await page.setViewportSize(DESKTOP);

  // Act
  await page.goto(withFormFactorShell(PUZZLE_PATH));
  await waitForPuzzlePhase(page, "building");

  // Assert (D-03)
  const select = page
    .locator(".control-row")
    .getByRole("combobox", { name: "Particles", exact: true });
  await expect(select).toHaveCount(1);
  await expectOptionDisabled(select, WATER, false);
});

test("draws Water on the Fountain jets", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");

  // Act
  await openScene(page, FOUNTAIN_PATH, "Fountain");
  await page.waitForTimeout(LOOK_PLAY_MS);

  // Assert
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "present");
  await screenshot(page, "fountain-water");
});

test("draws Water under the cup, target line and meter on a phone run", async ({ page }) => {
  test.setTimeout(SCENE_TEST_TIMEOUT_MS);
  // Arrange
  await storeRenderMode(page, "water");
  await installSolutionPlacements(page);
  await openPuzzle(page, PHONE_PORTRAIT);

  // Act
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await waitForPuzzlePhase(page, "running");
  await expect
    .poll(() => numericAttribute(page.locator("main"), "data-step-index"), {
      timeout: ENDED_TIMEOUT_MS,
    })
    .toBeGreaterThan(MID_RUN_STEP);

  // Assert
  await expectDrawn(page, "webgl-water");
  await expect(drawnSurface(page)).toHaveAttribute("data-speed-lane", "present");
  await screenshot(page, "puzzle-water-portrait");
});
