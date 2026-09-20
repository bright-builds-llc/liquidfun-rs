import { expect, test, type Page } from "@playwright/test";

import {
  expectReadySceneChrome,
  PLAYING_STATUS,
  SCENE_HASH_PATHS,
} from "./player-helpers";

async function resetPlayingScene(page: Page, title: string): Promise<void> {
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
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
