import { expect, test, type Locator } from "@playwright/test";

import {
  DAM_BREAK_PATH,
  expectReadySceneChrome,
  FOUNTAIN_PATH,
  PAUSED_STATUS,
  PLAYGROUND_ROOT_PATH,
  SCENE_HASH_PATHS,
  sessionStatus,
} from "./player-helpers";

async function expectCurrentDemoFullyVisible(scope: Locator): Promise<void> {
  await expect
    .poll(() =>
      scope.locator("[data-slot='sidebar-content']").evaluate((node) => {
        const current = node.querySelector("[aria-current='page']");
        if (!(current instanceof HTMLElement)) {
          return false;
        }
        const contentRect = node.getBoundingClientRect();
        const itemRect = current.getBoundingClientRect();
        return (
          itemRect.top >= contentRect.top - 1 &&
          itemRect.bottom <= contentRect.bottom + 1 &&
          node.scrollTop > 0
        );
      }),
    )
    .toBe(true);
}

test("renders the desktop shell and navigates from the sidebar", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".site-header")).toBeVisible();
  await expect(page.locator(".site-header")).not.toContainText(
    "All seventeen demos",
  );
  await expect(page.locator(".site-footer-summary")).toContainText(
    "All seventeen demos",
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

test("keeps scrolled mobile player controls painted", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(DAM_BREAK_PATH);
  await expectReadySceneChrome(page, "Dam Break");

  // Act
  await page.locator(".svg-export-pane").scrollIntoViewIfNeeded();

  // Assert
  await expect(page.locator(".site-header")).toHaveCSS("backdrop-filter", "none");
  await expect(page.locator(".site-header")).toHaveCSS(
    "background-color",
    "rgb(21, 29, 40)",
  );
  const sheet = await page.evaluate(() => {
    const tilt = document.querySelector(".tilt-pane");
    const pane = document.querySelector(".svg-export-pane");
    const title = document.querySelector("#svg-export-title");
    const button = document.querySelector(".svg-export-pane button");
    if (
      !(tilt instanceof HTMLElement) ||
      !(pane instanceof HTMLElement) ||
      !(title instanceof HTMLElement) ||
      !(button instanceof HTMLElement)
    ) {
      throw new Error("player sheet controls are missing");
    }
    const tiltBox = tilt.getBoundingClientRect();
    const paneBox = pane.getBoundingClientRect();
    const titleBox = title.getBoundingClientRect();
    const buttonBox = button.getBoundingClientRect();
    return {
      gap: paneBox.top - tiltBox.bottom,
      titleHeight: titleBox.height,
      buttonHeight: buttonBox.height,
      titleInsidePane: titleBox.top >= paneBox.top - 1 && titleBox.bottom <= paneBox.bottom + 1,
      buttonInsidePane:
        buttonBox.top >= paneBox.top - 1 && buttonBox.bottom <= paneBox.bottom + 1,
    };
  });
  expect(sheet.gap).toBeLessThan(32);
  expect(sheet.titleHeight).toBeGreaterThan(0);
  expect(sheet.buttonHeight).toBeGreaterThan(0);
  expect(sheet.titleInsidePane).toBe(true);
  expect(sheet.buttonInsidePane).toBe(true);
  await expect(page.getByRole("heading", { name: "Animated SVG" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Generate animated SVG" })).toBeVisible();
});

test("shows the build timestamp beside the other provenance", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Act
  const builtAt = page
    .locator(".site-footer-provenance div")
    .filter({ hasText: "Built at" });
  const timestamp = builtAt.locator("time");

  // Assert
  await expect(builtAt.getByText("Built at", { exact: true })).toBeVisible();
  await expect(timestamp).toHaveAttribute(
    "datetime",
    /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,3})?Z$/,
  );
  await expect(timestamp).toHaveText(
    /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2} UTC$/,
  );
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
  await expect(page).toHaveURL(/#\/scene\/wave-machine$/);
  await expectReadySceneChrome(page, "Wave Machine");

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
  // Dismiss + seventeen scene links = 18 focusables; exercise wrap past one full cycle.
  for (let tabIndex = 0; tabIndex < 19; tabIndex += 1) {
    await page.keyboard.press("Tab");
    await expect
      .poll(() =>
        dialog.evaluate((node) => node.contains(document.activeElement)),
      )
      .toBe(true);
  }

  // Arrange
  const closeButton = dialog.getByRole("button", { name: "Dismiss" });
  const lastLink = dialog.getByRole("link", { name: /Liquid Tumbler/ });
  await closeButton.focus();
  await expect(closeButton).toBeFocused();

  // Act / Assert
  // Dismiss + seventeen scene links = 18 focusables; 19 reverse tabs ≡ 1 (mod 18) lands on last.
  for (let reverseTabIndex = 0; reverseTabIndex < 19; reverseTabIndex += 1) {
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

test("shows seventeen static previews in the desktop sidebar", async ({ page }) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(DAM_BREAK_PATH);

  // Assert
  await expect(page.locator(".catalog-card")).toHaveCount(0);
  await expect(
    page.locator(".demo-sidebar").getByText("Static preview", { exact: true }),
  ).toHaveCount(17);
  await expect(
    page.locator(".demo-sidebar").locator("svg[aria-hidden='true']"),
  ).toHaveCount(17);
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

test("shows the current demo when the desktop sidebar is open", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(SCENE_HASH_PATHS["theo-jansen"]);
  const sidebar = page.locator(".demo-sidebar");

  // Assert
  await expect(
    sidebar.getByRole("link", { name: /Theo Jansen/ }),
  ).toHaveAttribute("aria-current", "page");
  await expectCurrentDemoFullyVisible(sidebar);
});

test("scrolls back to the current demo when the desktop sidebar reopens", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(SCENE_HASH_PATHS["theo-jansen"]);
  const sidebar = page.locator(".demo-sidebar");
  const content = sidebar.locator("[data-slot='sidebar-content']");
  await expectCurrentDemoFullyVisible(sidebar);
  await content.evaluate((node) => {
    node.scrollTop = 0;
  });
  const sidebarState = page.locator("[data-slot='sidebar']");

  // Act
  await page.getByRole("button", { name: "Demos" }).click();
  await expect(sidebarState).toHaveAttribute("data-state", "collapsed");
  await page.getByRole("button", { name: "Demos" }).click();

  // Assert
  await expect(sidebarState).toHaveAttribute("data-state", "expanded");
  await expectCurrentDemoFullyVisible(sidebar);
});

test("shows the current demo when the mobile demos drawer opens", async ({
  page,
}) => {
  // Arrange
  await page.setViewportSize({ width: 390, height: 812 });
  await page.goto(SCENE_HASH_PATHS["theo-jansen"]);
  await expectReadySceneChrome(page, "Theo Jansen");

  // Act
  await page.getByRole("button", { name: "Demos" }).click();
  const dialog = page.getByRole("dialog", { name: "Demos" });

  // Assert
  await expect(dialog).toBeVisible();
  await expect(
    dialog.getByRole("link", { name: /Theo Jansen/ }),
  ).toHaveAttribute("aria-current", "page");
  await expectCurrentDemoFullyVisible(dialog);
});
