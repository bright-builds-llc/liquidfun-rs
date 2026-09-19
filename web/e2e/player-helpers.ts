import { expect, type Locator, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { type SceneId } from "../src/catalog/scenes";

export const DAM_BREAK_PATH = "/liquidfun-rs/#/scene/dam-break";
export const UNKNOWN_SCENE_PATH = "/liquidfun-rs/#/scene/not-a-scene";
export const FOUNTAIN_PATH = "/liquidfun-rs/#/scene/fountain";
export const CATALOG_PATH = "/liquidfun-rs/";
export const LOADING_STATUS = "Loading Dam Break…";
export const PLAYING_STATUS = "Playing";
export const PAUSED_STATUS = "Paused";
export const PAUSE_HOLD_MS = 500;
export const HIDDEN_TAB_MS = 2_000;
export const MAX_STEPS_PER_FRAME = 4;
export const RESET_STEP_CEILING = 8;
export const CONSTRUCTION_RESET_HINT =
  "Changing this setting recreates the scene from its documented initial state.";
export const SIX_SCENE_TIMEOUT_MS = 120_000;
export const DAM_BREAK_HINT =
  "Drag the obstacle to a new place in the basin. Labeled controls also work from the keyboard.";
export const DESKTOP_VIEWPORT = { width: 1280, height: 720 } as const;

export const SCENE_HASH_PATHS: Readonly<Record<SceneId, string>> = {
  "dam-break": "/liquidfun-rs/#/scene/dam-break",
  fountain: "/liquidfun-rs/#/scene/fountain",
  "float-or-sink": "/liquidfun-rs/#/scene/float-or-sink",
  "color-mixer": "/liquidfun-rs/#/scene/color-mixer",
  "jelly-drop": "/liquidfun-rs/#/scene/jelly-drop",
  "water-wheel": "/liquidfun-rs/#/scene/water-wheel",
};

const SELECT_NEXT_VALUE: Readonly<Record<string, string>> = {
  "Aim angle": "left",
  "Stir speed": "fast",
  "Jet strength": "strong",
};

export async function numericAttribute(
  locator: Locator,
  name: string,
): Promise<number> {
  const maybeValue = await locator.getAttribute(name);
  if (maybeValue === null || !/^(0|[1-9]\d*)$/.test(maybeValue)) {
    throw new Error(`invalid numeric ${name}: ${String(maybeValue)}`);
  }
  return Number(maybeValue);
}

export function collectWasmUrls(page: Page): string[] {
  const wasmUrls: string[] = [];
  page.on("request", (request) => {
    const url = request.url();
    if (url.includes(".wasm")) {
      wasmUrls.push(url);
    }
  });
  return wasmUrls;
}

export async function gateWasmUntilLoadingObserved(
  page: Page,
): Promise<() => void> {
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

export async function expectReadySceneChrome(
  page: Page,
  title: string,
): Promise<void> {
  await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
  await expect(
    page.getByRole("heading", { name: title, exact: true }),
  ).toBeVisible();
  await expect(page.locator("#scene-credits-title")).toHaveText("Scene source");
  await expect(
    page.getByRole("link", { name: "View scene source" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Third-party notices" }),
  ).toBeVisible();
}

export async function openCatalogCard(
  page: Page,
  title: string,
  id: SceneId,
): Promise<void> {
  const card = page.locator(".catalog-card").filter({
    has: page.locator(".catalog-card-title", { hasText: title }),
  });
  await card.getByRole("link", { name: "Open" }).click();
  await expect(page).toHaveURL(new RegExp(`#/scene/${id}$`));
}

export async function resetNearZero(page: Page): Promise<void> {
  const main = page.locator("main");
  await expect
    .poll(() => numericAttribute(main, "data-step-index"))
    .toBeGreaterThan(4);

  const seriesStep = await numericAttribute(main, "data-step-index");
  await page.getByRole("button", { name: "Reset scene" }).click();
  await expect(page.getByRole("status")).toHaveText(PLAYING_STATUS);
  const resetStep = await numericAttribute(main, "data-step-index");
  expect(resetStep).toBeLessThan(seriesStep);
  expect(resetStep).toBeLessThan(RESET_STEP_CEILING);
}

export async function openDamBreakPlaying(page: Page): Promise<{
  readonly wasmUrls: string[];
}> {
  const wasmUrls = collectWasmUrls(page);
  const releaseWasmRequest = await gateWasmUntilLoadingObserved(page);

  await page.goto(DAM_BREAK_PATH, { waitUntil: "domcontentloaded" });
  const status = page.getByRole("status");
  await expect(status).toHaveText(LOADING_STATUS);
  releaseWasmRequest();
  await expectReadySceneChrome(page, "Dam Break");

  expect(
    wasmUrls.some((url) => url.includes("/liquidfun-rs/") && url.includes(".wasm")),
  ).toBe(true);

  return { wasmUrls };
}

export async function setDocumentHidden(
  page: Page,
  hidden: boolean,
): Promise<void> {
  await page.evaluate((nextHidden) => {
    Object.defineProperty(document, "hidden", {
      configurable: true,
      get: () => nextHidden,
    });
    document.dispatchEvent(new Event("visibilitychange"));
  }, hidden);
}

export async function restoreAndReadNextStep(
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

export async function proveHiddenTabMaxFour(page: Page): Promise<void> {
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
}

async function requireCanvasBox(page: Page): Promise<{
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}> {
  const maybeBox = await page.locator("canvas").boundingBox();
  if (maybeBox === null) {
    throw new Error("canvas box missing");
  }
  return maybeBox;
}

export async function dragCanvas(
  page: Page,
  startRatio = { x: 0.5, y: 0.45 },
  endRatio = { x: 0.7, y: 0.35 },
): Promise<void> {
  const box = await requireCanvasBox(page);
  await page.mouse.move(box.x + box.width * startRatio.x, box.y + box.height * startRatio.y);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * endRatio.x, box.y + box.height * endRatio.y);
  await page.mouse.up();
}

export async function clickCanvas(
  page: Page,
  ratio = { x: 0.5, y: 0.45 },
): Promise<void> {
  const box = await requireCanvasBox(page);
  await page.mouse.click(box.x + box.width * ratio.x, box.y + box.height * ratio.y);
}

export async function performSceneGesture(
  page: Page,
  gesture: "drag" | "click",
): Promise<void> {
  if (gesture === "drag") {
    await dragCanvas(page);
    return;
  }

  await clickCanvas(page);
}

export async function activateLabeledControl(
  page: Page,
  control: string,
): Promise<void> {
  const maybeSelectValue = SELECT_NEXT_VALUE[control];
  if (maybeSelectValue !== undefined) {
    await page.getByLabel(control).selectOption(maybeSelectValue);
    return;
  }

  await page.getByRole("button", { name: control, exact: true }).click();
}

export async function expectAcceptedPointerGesture(page: Page): Promise<void> {
  const main = page.locator("main");
  const accepted = await numericAttribute(main, "data-pointer-accepted");
  expect(accepted).toBeGreaterThan(0);
  const maybeKind = await main.getAttribute("data-last-pointer-kind");
  expect(maybeKind === "up" || maybeKind === "down").toBe(true);
}

export async function tabUntilFirstSelectFocused(page: Page): Promise<void> {
  const firstSelect = page.locator("select").first();
  for (let attempt = 0; attempt < 40; attempt += 1) {
    const isFocused = await firstSelect.evaluate(
      (node) => node === document.activeElement,
    );
    if (isFocused) {
      break;
    }
    await page.keyboard.press("Tab");
  }
  await expect(firstSelect).toBeFocused();
}

export function assertChromiumOnlyPlaywrightConfig(): void {
  const configPath = resolve(
    dirname(fileURLToPath(import.meta.url)),
    "../playwright.config.ts",
  );
  const source = readFileSync(configPath, "utf8");
  expect(source).not.toMatch(/\bfirefox\b|\bwebkit\b/);
}
