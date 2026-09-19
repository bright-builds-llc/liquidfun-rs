import { readdir } from "node:fs/promises";

import { expect, test } from "@playwright/test";

import {
  CAPTURE_PROFILE,
  SCENE_CAPTURE_PLANS,
  frameFileName,
} from "../scripts/demo-media/model";
import {
  advanceEngineSteps,
  captureSceneFrames,
  installSyntheticAnimationClock,
  waitForReadyScene,
} from "../scripts/demo-media/capture";
import { captureFontFingerprint } from "../scripts/demo-media/font";

test.use({
  viewport: CAPTURE_PROFILE.viewport,
  deviceScaleFactor: CAPTURE_PROFILE.deviceScaleFactor,
});

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

test("captures a stable real-page font fingerprint", async ({ page }) => {
  // Arrange
  const plan = SCENE_CAPTURE_PLANS[0];
  if (plan === undefined) {
    throw new Error("Dam Break capture plan is missing");
  }
  await installSyntheticAnimationClock(page);
  await page.goto(plan.route);
  await waitForReadyScene(page, plan);

  // Act
  const firstFingerprint = await captureFontFingerprint(page);
  const secondFingerprint = await captureFontFingerprint(page);

  // Assert
  expect(secondFingerprint).toEqual(firstFingerprint);
  expect(
    firstFingerprint.panelComputedStyle.fontFamily.length,
  ).toBeGreaterThan(0);
  expect(firstFingerprint.samples.map((sample) => sample.fontWeight)).toEqual([
    "400",
    "600",
    "700",
  ]);
  expect(firstFingerprint.rasterSha256).toMatch(/^[a-f0-9]{64}$/);
});

test("changes the font fingerprint when the capture font changes", async ({
  page,
}) => {
  // Arrange
  const plan = SCENE_CAPTURE_PLANS[0];
  if (plan === undefined) {
    throw new Error("Dam Break capture plan is missing");
  }
  await installSyntheticAnimationClock(page);
  await page.goto(plan.route);
  await waitForReadyScene(page, plan);
  const originalFingerprint = await captureFontFingerprint(page);

  // Act
  await page.addStyleTag({
    content: ".player-panel { font-family: monospace !important; }",
  });
  const changedFingerprint = await captureFontFingerprint(page);

  // Assert
  expect(changedFingerprint.panelComputedStyle.fontFamily).not.toBe(
    originalFingerprint.panelComputedStyle.fontFamily,
  );
  expect(changedFingerprint.rasterSha256).not.toBe(
    originalFingerprint.rasterSha256,
  );
});

test("captures 240 numbered frames and accepts the planned pointer action", async ({
  page,
}, testInfo) => {
  // Arrange
  const plan = SCENE_CAPTURE_PLANS[0];
  if (plan === undefined) {
    throw new Error("Dam Break capture plan is missing");
  }
  const framesDirectory = testInfo.outputPath("dam-break-capture");
  await installSyntheticAnimationClock(page);

  // Act
  await captureSceneFrames({ page, plan, framesDirectory });

  // Assert
  await expect
    .poll(async () => {
      const maybeAccepted = await page
        .locator("main")
        .getAttribute("data-pointer-accepted");
      return Number(maybeAccepted);
    })
    .toBeGreaterThan(0);
  const outputFrameNames = (await readdir(framesDirectory))
    .filter((entry) => entry.endsWith(".png"))
    .sort();
  expect(outputFrameNames).toEqual(
    Array.from({ length: CAPTURE_PROFILE.frameCount }, (_, frameIndex) =>
      frameFileName(frameIndex),
    ),
  );
});

test.describe("capture preconditions", () => {
  test.use({
    viewport: { width: 1280, height: 720 },
    deviceScaleFactor: CAPTURE_PROFILE.deviceScaleFactor,
  });

  test("rejects pages that do not match the capture viewport", async ({
    page,
  }, testInfo) => {
    // Arrange
    const plan = SCENE_CAPTURE_PLANS[0];
    if (plan === undefined) {
      throw new Error("Dam Break capture plan is missing");
    }
    await installSyntheticAnimationClock(page);

    // Act / Assert
    await expect(
      captureSceneFrames({
        page,
        plan,
        framesDirectory: testInfo.outputPath("wrong-viewport-capture"),
      }),
    ).rejects.toThrow(
      "Expected Playwright viewport 1280x960 before capture; configure test.use({ viewport: CAPTURE_PROFILE.viewport })",
    );
  });
});
