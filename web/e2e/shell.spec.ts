import { expect, test } from "@playwright/test";

import {
  DAM_BREAK_PATH,
  expectReadySceneChrome,
  FOUNTAIN_PATH,
  PAUSED_STATUS,
  PLAYGROUND_ROOT_PATH,
  sessionStatus,
} from "./player-helpers";

test("renders the desktop shell and navigates from the sidebar", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".site-header")).toBeVisible();
  await expect(page.locator(".site-header")).not.toContainText(
    "All eleven demos",
  );
  await expect(page.locator(".site-footer-summary")).toContainText(
    "All eleven demos",
  );
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

test("traps repeated forward and reverse Tab navigation inside the mobile drawer", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  await expectReadySceneChrome(page, "Dam Break");
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
  const trigger = page.getByRole("button", { name: "Demos" });
  await trigger.click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();

  // Act / Assert
  for (let tabIndex = 0; tabIndex < 13; tabIndex += 1) {
    await page.keyboard.press("Tab");
    await expect
      .poll(() =>
        dialog.evaluate((node) => node.contains(document.activeElement)),
      )
      .toBe(true);
  }

  // Arrange
  const closeButton = dialog.getByRole("button", { name: "Dismiss" });
  const lastLink = dialog.getByRole("link", { name: /Rigid Particles/ });
  await closeButton.focus();
  await expect(closeButton).toBeFocused();

  // Act / Assert
  // Dismiss + eleven scene links = 12 focusables; 13 reverse tabs ≡ 1 (mod 12) lands on last.
  for (let reverseTabIndex = 0; reverseTabIndex < 13; reverseTabIndex += 1) {
    await page.keyboard.press("Shift+Tab");
    await expect
      .poll(() =>
        dialog.evaluate((node) => node.contains(document.activeElement)),
      )
      .toBe(true);

    if (reverseTabIndex === 0) {
      await expect(lastLink).toBeFocused();
    }
  }

  await expect(lastLink).toBeFocused();
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

test("shows eleven static previews in the desktop sidebar", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".catalog-card")).toHaveCount(0);
  await expect(
    page.locator(".demo-sidebar").getByText("Static preview", { exact: true }),
  ).toHaveCount(11);
  await expect(
    page.locator(".demo-sidebar").locator("svg[aria-hidden='true']"),
  ).toHaveCount(11);
});

test("shows a static preview inside the mobile demos dialog", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(DAM_BREAK_PATH);

  // Act
  await page.getByRole("button", { name: "Demos" }).click();
  const dialog = page.getByRole("dialog", { name: "Demos" });

  // Assert
  await expect(dialog).toBeVisible();
  await expect(
    dialog.getByText("Static preview", { exact: true }).first(),
  ).toBeVisible();
  await expect(dialog.locator("svg[aria-hidden='true']").first()).toBeVisible();
  await expect(page.locator(".catalog-card")).toHaveCount(0);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
