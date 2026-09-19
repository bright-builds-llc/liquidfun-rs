import { expect, test } from "@playwright/test";

import { SCENE_CAPTURE_PLANS } from "../scripts/demo-media/model";
import {
  advanceEngineSteps,
  installSyntheticAnimationClock,
  waitForReadyScene,
} from "../scripts/demo-media/capture";

test("advances an exact number of steps under synthetic time", async ({
  page,
}) => {
  // Arrange
  const plan = SCENE_CAPTURE_PLANS[0];
  if (plan === undefined) {
    throw new Error("Dam Break capture plan is missing");
  }
  await installSyntheticAnimationClock(page);
  await page.goto(plan.route);
  const initialStep = await waitForReadyScene(page, plan);

  // Act
  await advanceEngineSteps(page, 12);

  // Assert
  await expect(page.locator("main")).toHaveAttribute(
    "data-step-index",
    String(initialStep + 12),
  );
});
