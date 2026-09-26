import { expect, test, type Locator, type Page } from "@playwright/test";

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

  const box = await page.locator("canvas.scene-canvas").boundingBox();
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
  await expect(page.getByRole("button", { name: "Phone accelerometer" })).toHaveCount(
    0,
  );
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
  await expect(sheet.getByLabel("Particles", { exact: true })).toBeVisible();
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

test("scrolls scene controls inside the sheet instead of the transformed drawer", async ({
  page,
}) => {
  // Arrange
  await installUnavailableFullscreen(page);
  await page.setViewportSize(PORTRAIT);
  await page.goto(DAM_BREAK_PATH);
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await page.getByRole("button", { name: "Scene controls" }).click();
  const sheet = page.getByRole("dialog", { name: "Scene controls" });
  await expect(sheet).toBeVisible();
  const scroller = sheet.locator(".canvas-controls-scroll");

  // Act
  const layout = await scroller.evaluate((node) => {
    const tilt = node.querySelector(".tilt-pane");
    const pane = node.querySelector(".svg-export-pane");
    if (!(tilt instanceof HTMLElement) || !(pane instanceof HTMLElement)) {
      throw new Error("sheet sections are missing");
    }
    const tiltBox = tilt.getBoundingClientRect();
    const paneBox = pane.getBoundingClientRect();
    return {
      gap: paneBox.top - tiltBox.bottom,
      canScroll: node.scrollHeight > node.clientHeight + 1,
    };
  });
  const button = sheet.getByRole("button", { name: "Generate animated SVG" });
  await button.scrollIntoViewIfNeeded();

  // Assert
  await expect(sheet).toHaveCSS("transform", "none");
  await expect(sheet).toHaveCSS("overflow-y", "visible");
  await expect(scroller).toHaveCSS("overflow-y", "auto");
  expect(layout.gap).toBeLessThan(32);
  expect(layout.canScroll).toBe(true);
  const buttonInScroller = await button.evaluate((node) => {
    const scrollport = node.closest(".canvas-controls-scroll");
    if (!(scrollport instanceof HTMLElement)) {
      return false;
    }
    const buttonBox = node.getBoundingClientRect();
    const scrollerBox = scrollport.getBoundingClientRect();
    return (
      buttonBox.bottom > scrollerBox.top && buttonBox.top < scrollerBox.bottom
    );
  });
  expect(buttonInScroller).toBe(true);
});

test("toggles phone accelerometer from the upper-right canvas HUD", async ({
  page,
}) => {
  // Arrange
  await installUnavailableFullscreen(page);
  // Headless Chromium exposes requestPermission and emits empty motion events.
  // Grant permission and ignore samples with no acceleration so a live arrow can appear.
  await page.addInitScript(() => {
    const motion = DeviceMotionEvent as unknown as {
      requestPermission?: () => Promise<PermissionState>;
    };
    motion.requestPermission = () => Promise.resolve("granted");
    const nativeAdd = window.addEventListener.bind(window);
    window.addEventListener = (type, listener, options) => {
      if (type !== "devicemotion" || typeof listener !== "function") {
        nativeAdd(type, listener, options);
        return;
      }
      const wrapped: EventListener = (event) => {
        if (!(event instanceof DeviceMotionEvent)) {
          return;
        }
        const sample = event.accelerationIncludingGravity;
        if (sample?.x == null || sample.y == null) {
          return;
        }
        listener.call(window, event);
      };
      nativeAdd(type, wrapped, options);
    };
  });
  await page.setViewportSize(PORTRAIT);
  await page.goto(DAM_BREAK_PATH);
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  const controls = page.getByRole("button", { name: "Scene controls" });
  const accelerometer = page.getByRole("button", { name: "Phone accelerometer" });

  // Assert
  await expect(accelerometer).toBeVisible();
  await expect(accelerometer).toHaveAttribute("aria-pressed", "false");
  const stacked = await stackedRightEdges(controls, accelerometer);
  expect(stacked.lowerIsBelow).toBe(true);
  expect(stacked.rightEdgesAligned).toBe(true);

  // Act
  await accelerometer.click();

  // Assert
  await expect(accelerometer).toHaveAttribute("aria-pressed", "true");
  await expect(async () => {
    await dispatchTiltSample(page, { x: 0, y: 9.81, z: 0 });
    await expect(page.getByRole("img", { name: /Gravity direction/ })).toBeVisible({
      timeout: 500,
    });
  }).toPass({ timeout: 5_000 });
  const arrow = page.getByRole("img", { name: /Gravity direction/ });
  const aboveArrow = await stackedRightEdges(accelerometer, arrow);
  expect(aboveArrow.lowerIsBelow).toBe(true);
  expect(aboveArrow.rightEdgesAligned).toBe(true);

  // Act
  await controls.click();
  const sheet = page.getByRole("dialog", { name: "Scene controls" });
  const checkbox = sheet.getByRole("checkbox", { name: "Use phone accelerometer" });

  // Assert
  await expect(checkbox).toBeChecked();

  // Act
  await checkbox.uncheck();

  // Assert
  await expect(accelerometer).toHaveAttribute("aria-pressed", "false");
  await expect(arrow).toHaveCount(0);
});

async function dispatchTiltSample(
  page: Page,
  acceleration: { readonly x: number; readonly y: number; readonly z: number },
): Promise<void> {
  await page.evaluate((sample) => {
    const event = new DeviceMotionEvent("devicemotion");
    Object.defineProperty(event, "accelerationIncludingGravity", {
      configurable: true,
      value: sample,
    });
    window.dispatchEvent(event);
  }, acceleration);
}

async function stackedLeftEdges(
  upper: Locator,
  lower: Locator,
): Promise<{
  readonly lowerIsBelow: boolean;
  readonly leftEdgesAligned: boolean;
}> {
  const upperBox = await upper.boundingBox();
  const lowerBox = await lower.boundingBox();
  if (upperBox === null || lowerBox === null) {
    throw new Error("HUD control bounds are unavailable");
  }
  return {
    lowerIsBelow: lowerBox.y >= upperBox.y + upperBox.height - 1,
    leftEdgesAligned: Math.abs(upperBox.x - lowerBox.x) < 2,
  };
}

async function stackedRightEdges(
  upper: Locator,
  lower: Locator,
): Promise<{
  readonly lowerIsBelow: boolean;
  readonly rightEdgesAligned: boolean;
}> {
  const upperBox = await upper.boundingBox();
  const lowerBox = await lower.boundingBox();
  if (upperBox === null || lowerBox === null) {
    throw new Error("HUD control bounds are unavailable");
  }
  return {
    lowerIsBelow: lowerBox.y >= upperBox.y + upperBox.height - 1,
    rightEdgesAligned:
      Math.abs(upperBox.x + upperBox.width - (lowerBox.x + lowerBox.width)) < 2,
  };
}

test("toggles debug info from under the canvas fps counter", async ({ page }) => {
  // Arrange
  await installUnavailableFullscreen(page);
  await page.setViewportSize(PORTRAIT);
  await page.goto(DAM_BREAK_PATH);
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  const fps = page.locator(".canvas-fps");
  const debug = page.getByRole("button", { name: "Debug info", exact: true });

  // Assert
  await expect(fps).toBeVisible();
  await expect(debug).toBeVisible();
  await expect(debug).toHaveAttribute("aria-pressed", "false");
  await expect(page.locator(".debug-readout")).toHaveCount(0);
  const underFps = await stackedLeftEdges(fps, debug);
  expect(underFps.lowerIsBelow).toBe(true);
  expect(underFps.leftEdgesAligned).toBe(true);

  // Act
  await debug.click();

  // Assert
  await expect(debug).toHaveAttribute("aria-pressed", "true");
  const readout = page.locator(".debug-readout");
  await expect(readout).toBeVisible();
  const underToggle = await stackedLeftEdges(debug, readout);
  expect(underToggle.lowerIsBelow).toBe(true);
  expect(underToggle.leftEdgesAligned).toBe(true);

  // Act
  await page.getByRole("button", { name: "Scene controls" }).click();
  const sheet = page.getByRole("dialog", { name: "Scene controls" });
  const checkbox = sheet.getByRole("checkbox", { name: "Debug info" });

  // Assert
  await expect(checkbox).toBeChecked();

  // Act
  await checkbox.uncheck();

  // Assert
  await expect(debug).toHaveAttribute("aria-pressed", "false");
  await expect(readout).toHaveCount(0);
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
