import { expect, test, type Page } from "@playwright/test";

import { SCENE_HASH_PATHS } from "./player-helpers";

// Chromium drops -webkit-touch-callout from computed style, so this spec checks
// what Chromium can compute and the built stylesheet text for the iOS callout.
// A real-iPhone long press remains an optional manual check.
test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true });

const SCENE_CANVAS = "canvas.scene-canvas";
const SCENE_READY_TIMEOUT_MS = 30_000;

async function openDiver(page: Page): Promise<void> {
  await page.goto(SCENE_HASH_PATHS["cartesian-diver"]);
  await expect(page.locator(SCENE_CANVAS)).toBeVisible({
    timeout: SCENE_READY_TIMEOUT_MS,
  });
}

test.describe("scene canvas long press", () => {
  test("blocks text selection and keeps touch-action off on the scene canvas", async ({
    page,
  }) => {
    // Arrange
    await openDiver(page);

    // Act
    const styles = await page.locator(SCENE_CANVAS).evaluate((canvas) => {
      const style = getComputedStyle(canvas);
      return {
        userSelect: style.userSelect,
        webkitUserSelect: style.getPropertyValue("-webkit-user-select"),
        touchAction: style.touchAction,
      };
    });

    // Assert
    expect(styles).toEqual({
      userSelect: "none",
      webkitUserSelect: "none",
      touchAction: "none",
    });
  });

  test("prevents the context menu on the scene canvas", async ({ page }) => {
    // Arrange
    await openDiver(page);

    // Act
    const notCanceled = await page
      .locator(SCENE_CANVAS)
      .evaluate((canvas) =>
        canvas.dispatchEvent(
          new MouseEvent("contextmenu", { bubbles: true, cancelable: true }),
        ),
      );

    // Assert
    expect(notCanceled).toBe(false);
  });

  test("ships the WebKit callout declaration in the built stylesheet", async ({
    page,
  }) => {
    // Arrange
    await openDiver(page);

    // Act
    const stylesheetTexts = await page.evaluate(async () => {
      const hrefs = Array.from(
        document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]'),
        (link) => link.href,
      );
      return Promise.all(
        hrefs.map(async (href) => (await fetch(href)).text()),
      );
    });

    // Assert
    expect(
      stylesheetTexts.some((text) => /-webkit-touch-callout:\s*none/.test(text)),
    ).toBe(true);
  });
});
