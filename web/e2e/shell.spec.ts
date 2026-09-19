import { expect, test } from "@playwright/test";

import {
  DAM_BREAK_PATH,
  expectReadySceneChrome,
  FOUNTAIN_PATH,
  PLAYGROUND_ROOT_PATH,
} from "./player-helpers";

test("renders the desktop shell and navigates from the sidebar", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".site-header")).toBeVisible();
  await expect(page.locator(".demo-sidebar")).toBeVisible();
  await expect(page.locator(".player-panel")).toBeVisible();
  await expect(page.locator(".catalog-card")).toHaveCount(0);
  await expect(
    page.locator(".demo-sidebar").getByRole("link", { name: /Dam Break/ }),
  ).toHaveAttribute("aria-current", "page");

  // Act
  await page
    .locator(".demo-sidebar")
    .getByRole("link", { name: /Fountain/ })
    .click();

  // Assert
  await expectReadySceneChrome(page, "Fountain");
  await expect(
    page.locator(".demo-sidebar").getByRole("link", { name: /Fountain/ }),
  ).toHaveAttribute("aria-current", "page");
});

test("keeps the source link at least 44 pixels tall", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Act
  const sourceLink = page.getByRole("link", { name: "GitHub source" });
  const maybeBox = await sourceLink.boundingBox();

  // Assert
  expect(maybeBox).not.toBeNull();
  expect(maybeBox?.height).toBeGreaterThanOrEqual(44);
});

test("normalizes an empty hash without adding an extra history entry", async ({
  page,
}) => {
  // Arrange
  await page.goto(FOUNTAIN_PATH);
  await page.goto(PLAYGROUND_ROOT_PATH);

  // Assert
  await expect(page).toHaveURL(/#\/scene\/dam-break$/);
  await expectReadySceneChrome(page, "Dam Break");

  // Act
  await page.goBack();

  // Assert
  await expect(page).toHaveURL(/#\/scene\/fountain$/);
});

test("dismisses the mobile drawer through its overlay and restores focus", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Demos" });
  await expect(dialog).toBeVisible();

  // Act
  await page.locator(".demo-drawer-overlay").click({
    position: { x: 382, y: 400 },
  });

  // Assert
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("traps repeated Tab navigation inside the mobile drawer", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Demos" });
  await expect(dialog).toBeVisible();

  // Act / Assert
  for (let tabIndex = 0; tabIndex < 10; tabIndex += 1) {
    await page.keyboard.press("Tab");
    expect(
      await dialog.evaluate((node) => node.contains(document.activeElement)),
    ).toBe(true);
  }

  // Act / Assert
  await page.keyboard.press("Shift+Tab");
  expect(
    await dialog.evaluate((node) => node.contains(document.activeElement)),
  ).toBe(true);
});

test("locks background scrolling and hides outside content while open", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });
  const outsideRoot = page.locator("#root");

  // Act
  await trigger.click();

  // Assert
  await expect(page.getByRole("dialog", { name: "Demos" })).toBeVisible();
  await expect(page.locator("html")).toHaveCSS("overflow", "hidden");
  await expect(outsideRoot).toHaveAttribute("aria-hidden", "true");

  // Act
  await page.keyboard.press("Escape");

  // Assert
  await expect(page.getByRole("dialog", { name: "Demos" })).toBeHidden();
  await expect(page.locator("html")).toHaveCSS("overflow", "visible");
  await expect(outsideRoot).not.toHaveAttribute("aria-hidden", "true");
  await expect(trigger).toBeFocused();
});

test("closes the drawer for an authoritative external hash route", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Demos" });
  await expect(dialog).toBeVisible();

  // Act
  await page.evaluate(() => {
    window.location.hash = "#/scene/jelly-drop";
  });

  // Assert
  await expect(dialog).toBeHidden();
  await expectReadySceneChrome(page, "Jelly Drop");
  await expect(trigger).toBeFocused();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
