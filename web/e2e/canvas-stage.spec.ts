import { expect, test, type Page } from "@playwright/test";

import {
  DAM_BREAK_PATH,
  PAUSED_STATUS,
  PLAYING_STATUS,
  sessionStatus,
} from "./player-helpers";

const PORTRAIT = { width: 390, height: 812 } as const;
const LANDSCAPE = { width: 932, height: 430 } as const;

async function installAndroidPhone(page: Page): Promise<void> {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "userAgentData", {
      configurable: true,
      get() {
        return { mobile: true, platform: "Android" };
      },
    });
  });
}

async function installUnavailableFullscreen(page: Page): Promise<void> {
  await page.addInitScript(() => {
    Object.defineProperty(Document.prototype, "fullscreenEnabled", {
      configurable: true,
      get() {
        return false;
      },
    });
  });
}

async function expectCanvasFillsViewport(page: Page): Promise<void> {
  const viewport = page.viewportSize();
  if (viewport === null) {
    throw new Error("viewport is unavailable");
  }

  const box = await page.locator("canvas").boundingBox();
  expect(box).not.toBeNull();
  expect(box?.width).toBeGreaterThanOrEqual(viewport.width - 1);
  expect(box?.height).toBeGreaterThanOrEqual(viewport.height - 1);
}

test("keeps element fullscreen on a browser that supports it", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator("html")).not.toHaveAttribute(
    "data-canvas-stage",
    "true",
  );
  await expect(page.getByRole("button", { name: "Full screen" })).toBeVisible();
  await expect(page.locator(".site-footer")).toBeVisible();
});

test("uses the canvas stage on an Android phone that can enter fullscreen", async ({
  page,
}) => {
  // Arrange
  await installAndroidPhone(page);
  await page.setViewportSize(LANDSCAPE);
  await page.goto(DAM_BREAK_PATH);

  // Act
  const fullscreenEnabled = await page.evaluate(() => document.fullscreenEnabled);

  // Assert
  expect(fullscreenEnabled).toBe(true);
  await expect(page.locator("html")).toHaveAttribute("data-canvas-stage", "true");
  await expect(page.getByRole("button", { name: "Full screen" })).toHaveCount(0);
  await expectCanvasFillsViewport(page);
  await expect(page.locator("[data-slot='sidebar-container']")).toHaveCount(0);
});

test("fills the portrait viewport when element fullscreen is unavailable", async ({
  page,
}) => {
  // Arrange
  await installUnavailableFullscreen(page);
  await page.setViewportSize(PORTRAIT);
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator("html")).toHaveAttribute("data-canvas-stage", "true");
  await expect(page.locator("html")).toHaveCSS("overflow", "hidden");
  await expectCanvasFillsViewport(page);
  await expect(page.getByRole("button", { name: "Full screen" })).toHaveCount(0);
  await expect(page.locator(".debug-readout")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Dam Break" })).toBeVisible();
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);

  // Act
  await page.mouse.wheel(0, 800);

  // Assert
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});

test("opens scene details without leaving the portrait canvas", async ({
  page,
}) => {
  // Arrange
  await installUnavailableFullscreen(page);
  await page.setViewportSize(PORTRAIT);
  await page.goto(DAM_BREAK_PATH);
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);

  // Act
  await page.getByRole("button", { name: "Pause scene" }).click();

  // Assert
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);

  // Act
  await page.getByRole("button", { name: "Scene controls" }).click();
  const sheet = page.getByRole("dialog", { name: "Scene controls" });

  // Assert
  await expect(sheet).toBeVisible();
  await expect(sheet.getByLabel("Rendering")).toBeVisible();
  await sheet.getByRole("checkbox", { name: "Debug info" }).check();
  await expect(page.locator(".debug-readout")).toBeVisible();
  await expect(sheet.locator("#scene-credits-title")).toHaveText("Scene source");
  await expect(sheet.getByRole("link", { name: "View source on GitHub" })).toBeVisible();
  const sourceLink = sheet.getByRole("link", { name: "GitHub source" });
  await expect(sourceLink).toBeVisible();
  const maybeBox = await sourceLink.boundingBox();
  expect(maybeBox).not.toBeNull();
  expect(maybeBox?.height).toBeGreaterThanOrEqual(44);
  await expectCanvasFillsViewport(page);
});

test("uses a demos drawer in landscape when element fullscreen is unavailable", async ({
  page,
}) => {
  // Arrange
  await installUnavailableFullscreen(page);
  await page.setViewportSize(LANDSCAPE);
  await page.goto(DAM_BREAK_PATH);
  await expect(page.locator("html")).toHaveAttribute("data-canvas-stage", "true");

  // Assert
  await expectCanvasFillsViewport(page);
  await expect(page.locator("[data-slot='sidebar-container']")).toHaveCount(0);

  // Act
  await page.getByRole("button", { name: "Demos" }).click();

  // Assert
  const dialog = page.getByRole("dialog", { name: "Demos" });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("link", { name: /Dam Break/ })).toHaveAttribute(
    "aria-current",
    "page",
  );
});
