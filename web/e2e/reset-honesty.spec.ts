import { expect, test, type Page } from "@playwright/test";

import {
  closeSceneControls,
  CONSTRUCTION_RESET_HINT,
  expectReadySceneChrome,
  expectSceneOption,
  PAUSED_STATUS,
  PLAYING_STATUS,
  revealLocator,
  SCENE_HASH_PATHS,
  selectSceneOption,
  ALL_SCENE_TIMEOUT_MS,
  sessionStatus,
} from "./player-helpers";

async function resetPlayingScene(page: Page, title: string): Promise<void> {
  await closeSceneControls(page);
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await expectReadySceneChrome(page, title);
}

test("resets Fountain Emission rate from high to medium", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS.fountain);
  await expectReadySceneChrome(page, "Fountain");

  // Act
  await selectSceneOption(page, "Emission rate", "high");
  await expectSceneOption(page, "Emission rate", "high");
  await resetPlayingScene(page, "Fountain");

  // Assert
  await expectSceneOption(page, "Emission rate", "medium");
});

test("resets Float or Sink Body from cork to wood", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["float-or-sink"]);
  await expectReadySceneChrome(page, "Float or Sink");

  // Act
  await selectSceneOption(page, "Body", "cork");
  await expectSceneOption(page, "Body", "cork");
  await resetPlayingScene(page, "Float or Sink");

  // Assert
  await expectSceneOption(page, "Body", "wood");
});

test("resets Color Mixer Stir speed from fast to slow", async ({ page }) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["color-mixer"]);
  await expectReadySceneChrome(page, "Color Mixer");

  // Act
  await selectSceneOption(page, "Stir speed", "fast");
  await expectSceneOption(page, "Stir speed", "fast");
  await resetPlayingScene(page, "Color Mixer");

  // Assert
  await expectSceneOption(page, "Stir speed", "slow");
});

test("resets Water Wheel Jet strength from strong to medium", async ({
  page,
}) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS["water-wheel"]);
  await expectReadySceneChrome(page, "Water Wheel");

  // Act
  await selectSceneOption(page, "Jet strength", "strong");
  await expectSceneOption(page, "Jet strength", "strong");
  await resetPlayingScene(page, "Water Wheel");

  // Assert
  await expectSceneOption(page, "Jet strength", "medium");
});

test("resets Dam Break Water amount from large to medium", async ({ page }) => {
  test.setTimeout(ALL_SCENE_TIMEOUT_MS);

  // Arrange
  await page.goto(SCENE_HASH_PATHS["dam-break"]);
  await expectReadySceneChrome(page, "Dam Break");
  await revealLocator(page, page.getByLabel("Water amount"));
  const waterAmount = page.locator(".scene-control").filter({
    has: page.getByLabel("Water amount"),
  });

  // Act
  await waterAmount.getByLabel("Water amount").selectOption("large");
  await expect(waterAmount.getByText(CONSTRUCTION_RESET_HINT)).toBeVisible();
  await expectReadySceneChrome(page, "Dam Break");
  await expectSceneOption(page, "Water amount", "large");
  await resetPlayingScene(page, "Dam Break");

  // Assert
  await expectSceneOption(page, "Water amount", "medium");
});

test("resets Color Mixer Mix strength from gentle to strong", async ({
  page,
}) => {
  test.setTimeout(ALL_SCENE_TIMEOUT_MS);

  // Arrange
  await page.goto(SCENE_HASH_PATHS["color-mixer"]);
  await expectReadySceneChrome(page, "Color Mixer");
  await revealLocator(page, page.getByLabel("Mix strength"));
  const mixStrength = page.locator(".scene-control").filter({
    has: page.getByLabel("Mix strength"),
  });

  // Act
  await mixStrength.getByLabel("Mix strength").selectOption("gentle");
  await expect(mixStrength.getByText(CONSTRUCTION_RESET_HINT)).toBeVisible();
  await expectReadySceneChrome(page, "Color Mixer");
  await expectSceneOption(page, "Mix strength", "gentle");
  await resetPlayingScene(page, "Color Mixer");

  // Assert
  await expectSceneOption(page, "Mix strength", "strong");
});

test("keeps Fountain Emission rate high across pause and play", async ({
  page,
}) => {
  // Arrange
  await page.goto(SCENE_HASH_PATHS.fountain);
  await expectReadySceneChrome(page, "Fountain");
  await selectSceneOption(page, "Emission rate", "high");
  await expectSceneOption(page, "Emission rate", "high");

  // Act
  await closeSceneControls(page);
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
  await expectSceneOption(page, "Emission rate", "high");
  await closeSceneControls(page);
  await page.getByRole("button", { name: "Play scene" }).click();

  // Assert
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await expectSceneOption(page, "Emission rate", "high");
});
