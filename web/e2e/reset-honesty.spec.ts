import { expect, test, type Page } from "@playwright/test";

import {
  CONSTRUCTION_RESET_HINT,
  expectReadySceneChrome,
  PAUSED_STATUS,
  PLAYING_STATUS,
  SCENE_HASH_PATHS,
  ALL_SCENE_TIMEOUT_MS,
  sessionStatus,
} from "./player-helpers";

async function resetPlayingScene(page: Page, title: string): Promise<void> {
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await expectReadySceneChrome(page, title);
}

test("resets Fountain Emission rate from high to medium", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS.fountain);
  await expectReadySceneChrome(page, "Fountain");

  // Act
  await page.getByLabel("Emission rate").selectOption("high");
  await expect(page.getByLabel("Emission rate")).toHaveValue("high");
  await resetPlayingScene(page, "Fountain");

  // Assert
  await expect(page.getByLabel("Emission rate")).toHaveValue("medium");
});

test("resets Float or Sink Body from cork to wood", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["float-or-sink"]);
  await expectReadySceneChrome(page, "Float or Sink");

  // Act
  await page.getByLabel("Body").selectOption("cork");
  await expect(page.getByLabel("Body")).toHaveValue("cork");
  await resetPlayingScene(page, "Float or Sink");

  // Assert
  await expect(page.getByLabel("Body")).toHaveValue("wood");
});

test("resets Color Mixer Stir speed from fast to slow", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["color-mixer"]);
  await expectReadySceneChrome(page, "Color Mixer");

  // Act
  await page.getByLabel("Stir speed").selectOption("fast");
  await expect(page.getByLabel("Stir speed")).toHaveValue("fast");
  await resetPlayingScene(page, "Color Mixer");

  // Assert
  await expect(page.getByLabel("Stir speed")).toHaveValue("slow");
});

test("resets Water Wheel Jet strength from strong to medium", async ({
  page,
}) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["water-wheel"]);
  await expectReadySceneChrome(page, "Water Wheel");

  // Act
  await page.getByLabel("Jet strength").selectOption("strong");
  await expect(page.getByLabel("Jet strength")).toHaveValue("strong");
  await resetPlayingScene(page, "Water Wheel");

  // Assert
  await expect(page.getByLabel("Jet strength")).toHaveValue("medium");
});

test("resets Dam Break Water amount from large to medium", async ({ page }) => {
  test.setTimeout(ALL_SCENE_TIMEOUT_MS);

  // Arrange
  await page.goto(SCENE_HASH_PATHS["dam-break"]);
  await expectReadySceneChrome(page, "Dam Break");
  const waterAmount = page.locator(".scene-control").filter({
    has: page.getByLabel("Water amount"),
  });

  // Act
  await waterAmount.getByLabel("Water amount").selectOption("large");
  await expect(waterAmount.getByText(CONSTRUCTION_RESET_HINT)).toBeVisible();
  await expectReadySceneChrome(page, "Dam Break");
  await expect(page.getByLabel("Water amount")).toHaveValue("large");
  await resetPlayingScene(page, "Dam Break");

  // Assert
  await expect(page.getByLabel("Water amount")).toHaveValue("medium");
});

test("resets Color Mixer Mix strength from gentle to strong", async ({
  page,
}) => {
  test.setTimeout(ALL_SCENE_TIMEOUT_MS);

  // Arrange
  await page.goto(SCENE_HASH_PATHS["color-mixer"]);
  await expectReadySceneChrome(page, "Color Mixer");
  const mixStrength = page.locator(".scene-control").filter({
    has: page.getByLabel("Mix strength"),
  });

  // Act
  await mixStrength.getByLabel("Mix strength").selectOption("gentle");
  await expect(mixStrength.getByText(CONSTRUCTION_RESET_HINT)).toBeVisible();
  await expectReadySceneChrome(page, "Color Mixer");
  await expect(page.getByLabel("Mix strength")).toHaveValue("gentle");
  await resetPlayingScene(page, "Color Mixer");

  // Assert
  await expect(page.getByLabel("Mix strength")).toHaveValue("strong");
});

test("keeps Fountain Emission rate high across pause and play", async ({
  page,
}) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS.fountain);
  await expectReadySceneChrome(page, "Fountain");
  await page.getByLabel("Emission rate").selectOption("high");
  await expect(page.getByLabel("Emission rate")).toHaveValue("high");

  // Act
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
  await expect(page.getByLabel("Emission rate")).toHaveValue("high");
  await page.getByRole("button", { name: "Play scene" }).click();

  // Assert
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await expect(page.getByLabel("Emission rate")).toHaveValue("high");
});
