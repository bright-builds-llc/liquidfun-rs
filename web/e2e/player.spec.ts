import { expect, test } from "@playwright/test";

import { SCENES, SCENE_IDS, type SceneId } from "../src/catalog/scenes";
import {
  activateLabeledControl,
  assertChromiumOnlyPlaywrightConfig,
  canvasPixelSha256,
  CONSTRUCTION_RESET_HINT,
  DAM_BREAK_HINT,
  DAM_BREAK_PATH,
  DESKTOP_VIEWPORT,
  dragCanvas,
  pressCanvas,
  expectAcceptedPointerGesture,
  expectReadySceneChrome,
  FOUNTAIN_PATH,
  installControlledRefreshRate,
  numericAttribute,
  openDesktopDemo,
  openDamBreakPlaying,
  PAUSE_HOLD_MS,
  PAUSED_STATUS,
  performSceneGesture,
  PLAYING_STATUS,
  proveHiddenTabMaxFour,
  PLAYGROUND_ROOT_PATH,
  RESET_STEP_CEILING,
  resetNearZero,
  SCENE_HASH_PATHS,
  SIX_SCENE_TIMEOUT_MS,
  tabUntilFirstSceneSelectFocused,
  UNKNOWN_SCENE_PATH,
} from "./player-helpers";

const POINTER_CONTROL: Readonly<
  Record<SceneId, { gesture: "drag" | "click"; control: string }>
> = {
  "dam-break": { gesture: "drag", control: "Drop obstacle" },
  fountain: { gesture: "drag", control: "Aim angle" },
  "float-or-sink": { gesture: "click", control: "Drop body" },
  "color-mixer": { gesture: "drag", control: "Stir speed" },
  "jelly-drop": { gesture: "click", control: "Poke jelly" },
  "water-wheel": { gesture: "drag", control: "Jet strength" },
};

test("advances steps and canvas pixels at controlled 120 Hz", async ({
  page,
}) => {
  await installControlledRefreshRate(page, 120);
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  const initialStep = await numericAttribute(main, "data-step-index");
  const initialPixelHash = await canvasPixelSha256(page);

  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(initialStep);
  await expect
    .poll(() => canvasPixelSha256(page))
    .not.toBe(initialPixelHash);
});

test("loads Dam Break under the production base and exercises pause, play, and reset", async ({
  page,
}) => {
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  const status = page.getByRole("status");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);

  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(status).toHaveText(PAUSED_STATUS);
  const pausedStep = await numericAttribute(main, "data-step-index");
  await page.waitForTimeout(PAUSE_HOLD_MS);
  expect(await numericAttribute(main, "data-step-index")).toBe(pausedStep);

  await page.getByRole("button", { name: "Play scene" }).click();
  await expect(status).toHaveText(PLAYING_STATUS);
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(pausedStep);

  const seriesStep = await numericAttribute(main, "data-step-index");
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(status).toHaveText(PLAYING_STATUS);
  const resetStep = await numericAttribute(main, "data-step-index");
  expect(resetStep).toBeLessThan(seriesStep);
  expect(resetStep).toBeLessThan(RESET_STEP_CEILING);
});

test("switches rendering without stepping and persists across scenes and reload", async ({
  page,
}) => {
  // Arrange
  await openDamBreakPlaying(page);
  const main = page.locator("main");
  const status = page.getByRole("status");
  await expect(main).toHaveAttribute("data-render-mode", "wireframe");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(4);
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(status).toHaveText(PAUSED_STATUS);
  const pausedStep = await numericAttribute(main, "data-step-index");
  const wireframePixels = await canvasPixelSha256(page);

  // Act
  await page.getByLabel("Rendering").selectOption("solid");

  // Assert
  await expect(main).toHaveAttribute("data-render-mode", "solid");
  await expect(status).toHaveText(PAUSED_STATUS);
  await expect(page.getByLabel("Rendering")).toHaveValue("solid");
  expect(await numericAttribute(main, "data-step-index")).toBe(pausedStep);
  expect(await canvasPixelSha256(page)).not.toBe(wireframePixels);

  // Act
  await page.goto(FOUNTAIN_PATH);
  await expectReadySceneChrome(page, "Fountain");

  // Assert
  await expect(main).toHaveAttribute("data-render-mode", "solid");
  await expect(page.getByLabel("Rendering")).toHaveValue("solid");

  // Act
  await page.reload();
  await expectReadySceneChrome(page, "Fountain");

  // Assert
  await expect(main).toHaveAttribute("data-render-mode", "solid");
  await expect(page.getByLabel("Rendering")).toHaveValue("solid");

  // Arrange
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(status).toHaveText(PAUSED_STATUS);
  const solidStep = await numericAttribute(main, "data-step-index");
  const solidPixels = await canvasPixelSha256(page);

  // Act
  await page.getByLabel("Rendering").selectOption("wireframe");

  // Assert
  await expect(main).toHaveAttribute("data-render-mode", "wireframe");
  await expect(status).toHaveText(PAUSED_STATUS);
  await expect(page.getByLabel("Rendering")).toHaveValue("wireframe");
  expect(await numericAttribute(main, "data-step-index")).toBe(solidStep);
  expect(await canvasPixelSha256(page)).not.toBe(solidPixels);
});

test("opens each native scene from desktop navigation, shows credits, and resets", async ({
  page,
}) => {
  test.setTimeout(SIX_SCENE_TIMEOUT_MS);
  await page.goto(PLAYGROUND_ROOT_PATH, { waitUntil: "domcontentloaded" });

  for (const [index, scene] of SCENES.entries()) {
    if (index % 2 === 0) {
      await openDesktopDemo(page, scene.title, scene.id);
    } else {
      await page.goto(SCENE_HASH_PATHS[scene.id]);
      await expect(page).toHaveURL(new RegExp(`#/scene/${scene.id}$`));
    }
    await expectReadySceneChrome(page, scene.title);
    await resetNearZero(page);
  }
});

test("returns from an unknown hash through Open Dam Break", async ({ page }) => {
  await page.goto(UNKNOWN_SCENE_PATH, { waitUntil: "domcontentloaded" });
  await expect(
    page.getByRole("heading", { name: "Scene not found" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Open Dam Break" }).click();
  await expect(page).toHaveURL(/#\/scene\/dam-break$/);
  await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
  await expect(page.getByRole("heading", { name: "Dam Break" })).toBeVisible();
});

test("leaves Dam Break for Fountain and restarts the step series", async ({
  page,
}) => {
  const { wasmUrls } = await openDamBreakPlaying(page);
  const wasmCountAfterDamBreak = wasmUrls.length;
  const main = page.locator("main");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);
  const damStep = await numericAttribute(main, "data-step-index");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(Math.min(damStep + 4, RESET_STEP_CEILING));

  await page.goto(FOUNTAIN_PATH);
  await expectReadySceneChrome(page, "Fountain");
  const fountainStep = await numericAttribute(main, "data-step-index");
  expect(fountainStep).toBeLessThan(RESET_STEP_CEILING);
  expect(wasmUrls.length).toBe(wasmCountAfterDamBreak);
});

test("applies a Dam Break Gravity construction setting and returns to Playing", async ({
  page,
}) => {
  await openDamBreakPlaying(page);

  const gravity = page.locator(".scene-control").filter({
    has: page.getByLabel("Gravity"),
  });
  await gravity.getByLabel("Gravity").selectOption("high");
  await expect(gravity.getByText(CONSTRUCTION_RESET_HINT)).toBeVisible();
  await gravity.getByRole("button", { name: "Apply setting" }).click();
  await expectReadySceneChrome(page, "Dam Break");
});

test("clears hidden-tab catch-up so the next frame advances at most four steps", async ({
  page,
}) => {
  await openDamBreakPlaying(page);
  await proveHiddenTabMaxFour(page);
});

test("plays each scene, accepts one pointer gesture, and activates a labeled control", async ({
  page,
}) => {
  test.setTimeout(SIX_SCENE_TIMEOUT_MS);

  for (const sceneId of SCENE_IDS) {
    const scene = SCENES.find((entry) => entry.id === sceneId);
    if (scene === undefined) {
      throw new Error(`missing catalog scene ${sceneId}`);
    }

    const mapping = POINTER_CONTROL[sceneId];
    await page.goto(SCENE_HASH_PATHS[sceneId]);
    await expectReadySceneChrome(page, scene.title);

    const main = page.locator("main");
    await expect
      .poll(() => numericAttribute(main, "data-step-index"))
      .toBeGreaterThan(0);

    await performSceneGesture(page, mapping.gesture);
    await expectAcceptedPointerGesture(page);
    await activateLabeledControl(page, mapping.control);
    await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
  }
});

test("sets data-last-pointer-kind to cancel after Dam Break pointercancel", async ({
  page,
}) => {
  await openDamBreakPlaying(page);
  await pressCanvas(page);
  await page.locator("canvas").dispatchEvent("pointercancel", { pointerId: 1 });
  await expect(page.locator("main")).toHaveAttribute(
    "data-last-pointer-kind",
    "cancel",
  );
});

test("accepts a Dam Break drag after resize without rebuilding the world", async ({
  page,
}) => {
  await page.setViewportSize(DESKTOP_VIEWPORT);
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);
  const acceptedBefore = await numericAttribute(main, "data-pointer-accepted");

  await page.setViewportSize({ width: 375, height: 812 });
  await page.waitForTimeout(200);
  await dragCanvas(page);
  await expect
    .poll(() => numericAttribute(main, "data-pointer-accepted"))
    .toBeGreaterThan(acceptedBefore);
});

test("caps hidden-tab recovery after one Dam Break pointer gesture", async ({
  page,
}) => {
  await openDamBreakPlaying(page);
  const main = page.locator("main");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);
  await dragCanvas(page);
  await expectAcceptedPointerGesture(page);
  await proveHiddenTabMaxFour(page);
});

test("tabs to a scene control and scrolls the page at 375px", async ({
  page,
}) => {
  assertChromiumOnlyPlaywrightConfig();
  expect(test.info().project.name).toBe("chromium");

  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(DAM_BREAK_PATH, { waitUntil: "domcontentloaded" });
  await expectReadySceneChrome(page, "Dam Break");
  await expect(page.locator("figcaption")).toHaveText(DAM_BREAK_HINT);
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(page.getByRole("status")).toHaveText(PAUSED_STATUS);

  await tabUntilFirstSceneSelectFocused(page);

  const canScrollPage = await page.evaluate(() => {
    const maybeScrolling = document.scrollingElement;
    if (maybeScrolling === null) {
      throw new Error("scrolling element is missing");
    }
    return maybeScrolling.scrollHeight > window.innerHeight;
  });
  expect(canScrollPage).toBe(true);

  await page.locator(".site-footer").scrollIntoViewIfNeeded();
  await expect
    .poll(() => page.evaluate(() => window.scrollY))
    .toBeGreaterThan(0);
});
