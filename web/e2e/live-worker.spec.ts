import { expect, test } from "./worker-quality-fixture";

const TESLA = "/liquidfun-rs/#/scene/tesla-valve";
const DAM = "/liquidfun-rs/#/scene/dam-break";

test("uses an owned Tesla worker, settles pause, and retains the direct other-scene backend", async ({
  page,
}) => {
  await page.goto(TESLA);
  const main = page.locator("main"),
    canvas = page.locator("canvas.scene-canvas");
  await expect(canvas).toHaveAttribute("data-simulation-backend", "worker");
  await expect(main).toHaveAttribute("data-playback", "playing");
  await expect
    .poll(async () => Number(await main.getAttribute("data-step-index")))
    .toBeGreaterThan(4);
  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(main).toHaveAttribute("data-playback", "paused");
  await page.waitForTimeout(250);
  const settled = await main.getAttribute("data-step-index");
  await page.waitForTimeout(150);
  await expect(main).toHaveAttribute("data-step-index", settled!);
  await expect(main).toHaveAttribute("data-playback", "paused");
  await page.goto(DAM);
  await expect(canvas).toHaveAttribute("data-simulation-backend", "direct");
  await expect(main).toHaveAttribute("data-scene", "dam-break");
  await expect(main).toHaveAttribute("data-playback", "playing");
});

for (const rate of [1440, 2880])
  for (const direction of ["1", "-1"]) {
    test(`actual worker/direct state and strict containment agree at ${rate}/${direction}`, async ({
      page,
      workerQualityUrl,
    }) => {
      test.setTimeout(120_000);
      await page.goto(workerQualityUrl);
      await page.waitForFunction(() => window.teslaBenchmark !== undefined);
      const result = await page.evaluate(
        ({ rate, direction }) =>
          window.teslaBenchmark.compareWorkerDirect(rate, direction, 240),
        { rate, direction },
      );
      expect(result.worker).toEqual(result.direct);
      expect(result.worker.stepIndex).toBe(240);
      expect(result.worker.particleCount).toBeLessThan(16384);
    });
  }

test("an overflowing gravity sample stays recoverable in the actual worker like direct playback", async ({
  page,
  workerQualityUrl,
}) => {
  await page.goto(workerQualityUrl);
  await page.waitForFunction(() => window.teslaBenchmark !== undefined);
  const result = await page.evaluate(() =>
    window.teslaBenchmark.compareWorkerDirect(1440, "1", 8, true),
  );
  expect(result.worker).toEqual(result.direct);
  expect(result.worker.stepIndex).toBe(8);
});

for (const [direction, steps] of [
  ["1", 400],
  ["-1", 424],
] as const) {
  test(`actual default worker warmup and forty-step checkpoint agree for ${direction}`, async ({
    page,
    workerQualityUrl,
  }) => {
    test.setTimeout(120_000);
    await page.goto(workerQualityUrl);
    await page.waitForFunction(() => window.teslaBenchmark !== undefined);
    const result = await page.evaluate(
      ({ direction, steps }) =>
        window.teslaBenchmark.compareWorkerDirect(1440, direction, steps),
      { direction, steps },
    );
    expect(result.worker).toEqual(result.direct);
    expect(result.worker.stepIndex).toBe(steps);
  });
}
