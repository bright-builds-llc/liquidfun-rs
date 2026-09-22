import { expect, test } from "@playwright/test";
import { readFile } from "node:fs/promises";

import {
  numericAttribute,
  openDamBreakPlaying,
  PLAYING_STATUS,
  sessionStatus,
} from "./player-helpers";

test("exports a one-second animated SVG while the scene keeps playing", async ({
  page,
}) => {
  test.setTimeout(120_000);
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  const stepBefore = await numericAttribute(main, "data-step-index");
  await page.getByLabel("Seconds").fill("1");
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Generate animated SVG" }).click();

  await expect(page.getByRole("progressbar", { name: "Animated SVG progress" })).toBeVisible();
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
  await expect.poll(() => numericAttribute(main, "data-step-index")).toBeGreaterThan(stepBefore);

  const download = await downloadPromise;
  expect(download.suggestedFilename()).toBe("dam-break-1s.svg");
  const filePath = await download.path();
  expect(filePath).toBeTruthy();
  if (filePath === null) {
    return;
  }
  const svg = await readFile(filePath, "utf8");
  expect(svg.startsWith("<svg")).toBe(true);
  expect(svg).toContain("<animate");
  await expect(page.getByRole("status").filter({ hasText: "Generated in" })).toBeVisible();
});
