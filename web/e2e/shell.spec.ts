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

test("opens and closes the mobile demos dialog accessibly", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto(DAM_BREAK_PATH);
  const trigger = page.getByRole("button", { name: "Demos" });

  // Act
  await trigger.click();
  const dialog = page.getByRole("dialog", { name: "Demos" });

  // Assert
  await expect(dialog).toBeVisible();
  expect(
    await dialog.evaluate((node) => node.contains(document.activeElement)),
  ).toBe(true);

  // Act / Assert: Escape restores trigger focus
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();

  // Act / Assert: selection closes and navigates
  await trigger.click();
  await dialog.getByRole("link", { name: /Jelly Drop/ }).click();
  await expect(dialog).toBeHidden();
  await expectReadySceneChrome(page, "Jelly Drop");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
});
