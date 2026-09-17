import { expect, test, type Locator, type Page } from "@playwright/test";

const DAM_BREAK_PATH = "/liquidfun-rs/#/scene/dam-break";
const UNKNOWN_SCENE_PATH = "/liquidfun-rs/#/scene/not-a-scene";
const FOUNTAIN_PATH = "/liquidfun-rs/#/scene/fountain";
const LOADING_STATUS = "Loading Dam Break…";
const PLAYING_STATUS = "Playing";
const PAUSED_STATUS = "Paused";
const PAUSE_HOLD_MS = 500;
const HIDDEN_TAB_MS = 2_000;
const MAX_STEPS_PER_FRAME = 4;
const RESET_STEP_CEILING = 8;

async function numericAttribute(
  locator: Locator,
  name: string,
): Promise<number> {
  const maybeValue = await locator.getAttribute(name);
  if (maybeValue === null || !/^(0|[1-9]\d*)$/.test(maybeValue)) {
    throw new Error(`invalid numeric ${name}: ${String(maybeValue)}`);
  }
  return Number(maybeValue);
}

function collectWasmUrls(page: Page): string[] {
  const wasmUrls: string[] = [];
  page.on("request", (request) => {
    const url = request.url();
    if (url.includes(".wasm")) {
      wasmUrls.push(url);
    }
  });
  return wasmUrls;
}

async function gateWasmUntilLoadingObserved(page: Page): Promise<() => void> {
  let releaseWasmRequest = (): void => {
    throw new Error("WASM request was not observed");
  };
  const wasmRequestGate = new Promise<void>((resolveRequest) => {
    releaseWasmRequest = resolveRequest;
  });
  await page.route("**/*.wasm", async (route) => {
    await wasmRequestGate;
    await route.continue();
  });
  return releaseWasmRequest;
}

async function openDamBreakPlaying(page: Page): Promise<{
  readonly wasmUrls: string[];
}> {
  const wasmUrls = collectWasmUrls(page);
  const releaseWasmRequest = await gateWasmUntilLoadingObserved(page);

  await page.goto(DAM_BREAK_PATH, { waitUntil: "domcontentloaded" });
  const status = page.getByRole("status");
  await expect(status).toHaveText(LOADING_STATUS);
  releaseWasmRequest();
  await expect(status).toHaveText(PLAYING_STATUS);
  await expect(page.getByRole("heading", { name: "Dam Break" })).toBeVisible();

  expect(
    wasmUrls.some((url) => url.includes("/liquidfun-rs/") && url.includes(".wasm")),
  ).toBe(true);

  return { wasmUrls };
}

async function setDocumentHidden(page: Page, hidden: boolean): Promise<void> {
  await page.evaluate((nextHidden) => {
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => nextHidden,
    });
    document.dispatchEvent(new Event("visibilitychange"));
  }, hidden);
}

async function restoreAndReadNextStep(
  page: Page,
  previousStep: number,
): Promise<number> {
  return page.evaluate((stepBeforeResume) => {
    const maybeMain = document.querySelector("main");
    if (maybeMain === null) {
      throw new Error("main is missing");
    }

    const nextStep = new Promise<number>((resolve, reject) => {
      const observer = new MutationObserver(() => {
        const maybeNext = Number(maybeMain.getAttribute("data-step-index"));
        if (Number.isFinite(maybeNext) && maybeNext > stepBeforeResume) {
          observer.disconnect();
          resolve(maybeNext);
        }
      });
      observer.observe(maybeMain, {
        attributes: true,
        attributeFilter: ["data-step-index"],
      });
      window.setTimeout(() => {
        observer.disconnect();
        reject(new Error("hidden-tab resume did not observe a later step index"));
      }, 2_000);
    });

    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => false,
    });
    document.dispatchEvent(new Event("visibilitychange"));
    return nextStep;
  }, previousStep);
}

test("loads Dam Break under the production base and exercises pause, play, and reset", async ({
  page,
}) => {
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  const status = page.getByRole("status");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);

  await page.getByRole("button", { name: "Pause scene" }).click();
  await expect(status).toHaveText(PAUSED_STATUS);
  const pausedStep = await numericAttribute(main, "data-step-index");
  await page.waitForTimeout(PAUSE_HOLD_MS);
  expect(await numericAttribute(main, "data-step-index")).toBe(pausedStep);

  await page.getByRole("button", { name: "Play scene" }).click();
  await expect(status).toHaveText(PLAYING_STATUS);
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(pausedStep);

  const seriesStep = await numericAttribute(main, "data-step-index");
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(status).toHaveText(PLAYING_STATUS);
  const resetStep = await numericAttribute(main, "data-step-index");
  expect(resetStep).toBeLessThan(seriesStep);
  expect(resetStep).toBeLessThan(RESET_STEP_CEILING);
});

test("returns from an unknown hash through Open Dam Break", async ({ page }) => {
  await page.goto(UNKNOWN_SCENE_PATH, { waitUntil: "domcontentloaded" });
  await expect(
    page.getByRole("heading", { name: "Scene not found" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Open Dam Break" }).click();
  await expect(page).toHaveURL(/#\/scene\/dam-break$/);
  await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
  await expect(page.getByRole("heading", { name: "Dam Break" })).toBeVisible();
});

test("leaves Dam Break for Fountain without a second WASM session", async ({
  page,
}) => {
  const { wasmUrls } = await openDamBreakPlaying(page);
  const wasmCountAfterDamBreak = wasmUrls.length;

  await page.goto(FOUNTAIN_PATH);
  await expect(
    page.getByRole("heading", { name: "Fountain is not ready yet" }),
  ).toBeVisible();
  await expect(page.locator("main")).toHaveAttribute("data-playback", "fallback");
  await expect(page.locator("main")).toHaveAttribute("data-scene", "fountain");
  await expect(
    page.getByRole("img", { name: /Live Dam Break scene/ }),
  ).toHaveCount(0);
  expect(wasmUrls.length).toBe(wasmCountAfterDamBreak);
});

test("clears hidden-tab catch-up so the next frame advances at most four steps", async ({
  page,
}) => {
  await openDamBreakPlaying(page);

  const main = page.locator("main");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(0);

  await setDocumentHidden(page, true);
  const hiddenStep = await numericAttribute(main, "data-step-index");
  await page.waitForTimeout(HIDDEN_TAB_MS);
  expect(await numericAttribute(main, "data-step-index")).toBe(hiddenStep);

  const firstVisibleStep = await restoreAndReadNextStep(page, hiddenStep);
  expect(firstVisibleStep - hiddenStep).toBeLessThanOrEqual(MAX_STEPS_PER_FRAME);
});
