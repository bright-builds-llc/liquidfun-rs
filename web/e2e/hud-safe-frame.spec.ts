import { expect, test, type Page } from "@playwright/test";

import { SCENES, SCENE_IDS, type SceneId } from "../src/catalog/scenes";
import { sceneControlsForSurface } from "../src/components/scene-controls";
import { HUD_SLIDERS_STORAGE_KEY } from "../src/player/hud-sliders-preference";
import {
  PAUSED_STATUS,
  PLAYING_STATUS,
  SCENE_HASH_PATHS,
  sessionStatus,
} from "./player-helpers";

test.describe.configure({ mode: "parallel" });

type Box = {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
};

type LabeledBox = Box & { readonly label: string };

type HudGeometry = {
  readonly maybeWorldFrame: Box | undefined;
  readonly usedFallback: boolean;
  readonly obstacles: readonly LabeledBox[];
};

const OVERLAP_TOLERANCE_PX = 1;
const STABLE_FRAME_GAP_MS = 250;
const HIDE_SLIDERS = "Hide sliders";
const SHOW_SLIDERS = "Show sliders";

const PHONE_PORTRAIT = { width: 390, height: 844 } as const;
const DESKTOP = { width: 1280, height: 800 } as const;
const PHONE_LANDSCAPE = { width: 844, height: 390 } as const;
const TABLET_PORTRAIT = { width: 768, height: 1024 } as const;

function hasHudControls(id: SceneId): boolean {
  const maybeScene = SCENES.find((scene) => scene.id === id);
  if (maybeScene === undefined) {
    throw new Error(`catalog scene ${id} is missing`);
  }
  return sceneControlsForSurface(maybeScene.controls, "hud").length > 0;
}

/** Projected world rectangle and HUD obstacle boxes, both in page pixels. */
async function readHudGeometry(page: Page): Promise<HudGeometry> {
  return page.evaluate(() => {
    const canvas = document.querySelector<HTMLCanvasElement>("canvas.scene-canvas");
    if (canvas === null) {
      throw new Error("scene canvas is missing");
    }
    const canvasRect = canvas.getBoundingClientRect();
    const maybeRaw = canvas.dataset.worldFrame;
    const maybeEdges = maybeRaw?.split(",").map(Number);
    const maybeWorldFrame =
      maybeEdges === undefined || maybeEdges.length !== 4
        ? undefined
        : {
            left: canvasRect.left + (maybeEdges[0] ?? 0),
            top: canvasRect.top + (maybeEdges[1] ?? 0),
            right: canvasRect.left + (maybeEdges[2] ?? 0),
            bottom: canvasRect.top + (maybeEdges[3] ?? 0),
          };
    const obstacles = Array.from(
      document.querySelectorAll<HTMLElement>("[data-hud-obstacle]"),
    )
      .map((element) => {
        const rect = element.getBoundingClientRect();
        return {
          left: rect.left,
          top: rect.top,
          right: rect.right,
          bottom: rect.bottom,
          label: `${element.tagName.toLowerCase()}${element.id === "" ? "" : `#${element.id}`}.${element.className}`,
        };
      })
      .filter((box) => box.right > box.left && box.bottom > box.top);
    return {
      maybeWorldFrame,
      usedFallback: canvas.dataset.worldFrameFallback === "true",
      obstacles,
    };
  });
}

/** Smaller of the two axis overlaps; positive only when the boxes intersect. */
function overlapPx(a: Box, b: Box): number {
  const x = Math.min(a.right, b.right) - Math.max(a.left, b.left);
  const y = Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top);
  return Math.min(x, y);
}

function overlappingObstacles(geometry: HudGeometry): string[] {
  const maybeFrame = geometry.maybeWorldFrame;
  if (maybeFrame === undefined) {
    return ["data-world-frame is missing"];
  }
  return geometry.obstacles
    .filter((obstacle) => overlapPx(maybeFrame, obstacle) > OVERLAP_TOLERANCE_PX)
    .map(
      (obstacle) =>
        `${obstacle.label} overlaps by ${overlapPx(maybeFrame, obstacle).toFixed(1)}px`,
    );
}

function area(box: Box): number {
  return (box.right - box.left) * (box.bottom - box.top);
}

function expectSameBox(actual: Box, expected: Box): void {
  expect(Math.abs(actual.left - expected.left)).toBeLessThanOrEqual(1);
  expect(Math.abs(actual.top - expected.top)).toBeLessThanOrEqual(1);
  expect(Math.abs(actual.right - expected.right)).toBeLessThanOrEqual(1);
  expect(Math.abs(actual.bottom - expected.bottom)).toBeLessThanOrEqual(1);
}

async function openPlayingScene(
  page: Page,
  id: SceneId,
  viewport: { readonly width: number; readonly height: number },
): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(SCENE_HASH_PATHS[id]);
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS, {
    timeout: 20_000,
  });
}

/** World frame after layout settles: two reads a moment apart agree. */
async function stableWorldFrame(page: Page): Promise<Box> {
  let maybeStable: Box | undefined;
  await expect(async () => {
    const first = (await readHudGeometry(page)).maybeWorldFrame;
    await page.waitForTimeout(STABLE_FRAME_GAP_MS);
    const second = (await readHudGeometry(page)).maybeWorldFrame;
    expect(first).toBeDefined();
    expect(second).toEqual(first);
    maybeStable = second;
  }).toPass({ timeout: 5_000 });
  if (maybeStable === undefined) {
    throw new Error("world frame never settled");
  }
  return maybeStable;
}

const OVERLAP_CASES = SCENE_IDS.flatMap((id) => {
  const viewports = hasHudControls(id)
    ? [PHONE_PORTRAIT, DESKTOP, PHONE_LANDSCAPE, TABLET_PORTRAIT]
    : [PHONE_PORTRAIT, DESKTOP];
  return viewports.map((viewport) => ({ id, viewport }));
});

for (const { id, viewport } of OVERLAP_CASES) {
  test(`${id} at ${viewport.width}x${viewport.height} keeps the world clear of HUD chrome`, async ({
    page,
  }) => {
    await openPlayingScene(page, id, viewport);

    let usedFallback = false;
    await expect(async () => {
      const geometry = await readHudGeometry(page);
      usedFallback = geometry.usedFallback;
      if (usedFallback) {
        return;
      }
      expect(overlappingObstacles(geometry)).toEqual([]);
    }).toPass({ timeout: 5_000 });

    if (!usedFallback) {
      return;
    }

    test.info().annotations.push({
      type: "safe-frame-fallback",
      description: `${id} ${viewport.width}x${viewport.height}`,
    });
    expect(
      hasHudControls(id),
      `${id} fell back without HUD sliders; inspect its chrome`,
    ).toBe(true);
    await page.getByRole("button", { name: HIDE_SLIDERS }).click();
    await expect(async () => {
      const geometry = await readHudGeometry(page);
      expect(geometry.usedFallback).toBe(false);
      expect(overlappingObstacles(geometry)).toEqual([]);
    }).toPass({ timeout: 5_000 });
  });
}

// Tesla valve's tall world is height-limited by its slider card on a phone,
// so hiding the card grows the frame. Wave tank's wide world is
// width-limited there and only re-centres.
test("hiding the sliders grows the frame, survives a reload, and showing restores it", async ({
  page,
}) => {
  await openPlayingScene(page, "tesla-valve", PHONE_PORTRAIT);
  const toggle = page.locator(".canvas-sliders-toggle");
  await expect(toggle).toHaveAccessibleName(HIDE_SLIDERS);
  await expect(toggle).toHaveAttribute("aria-expanded", "true");
  const shownFrame = await stableWorldFrame(page);

  await toggle.click();

  await expect(page.locator("#canvas-hud-sliders")).toHaveCount(0);
  await expect(toggle).toHaveAccessibleName(SHOW_SLIDERS);
  await expect(toggle).toHaveAttribute("aria-expanded", "false");
  const stored = await page.evaluate(
    (key) => window.localStorage.getItem(key),
    HUD_SLIDERS_STORAGE_KEY,
  );
  expect(stored).toBe("hidden");
  const hiddenFrame = await stableWorldFrame(page);
  expect(area(hiddenFrame)).toBeGreaterThan(area(shownFrame));

  await page.reload();
  await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS, {
    timeout: 20_000,
  });
  await expect(toggle).toHaveAccessibleName(SHOW_SLIDERS);
  await expect(page.locator("#canvas-hud-sliders")).toHaveCount(0);
  expectSameBox(await stableWorldFrame(page), hiddenFrame);

  await toggle.click();

  await expect(page.locator("#canvas-hud-sliders")).toHaveCount(1);
  expectSameBox(await stableWorldFrame(page), shownFrame);
});

test("a scene without HUD controls shows no sliders toggle", async ({ page }) => {
  const maybeScene = SCENES.find(
    (scene) => sceneControlsForSurface(scene.controls, "hud").length === 0,
  );
  if (maybeScene === undefined) {
    throw new Error("every catalog scene has HUD controls");
  }

  await openPlayingScene(page, maybeScene.id, PHONE_PORTRAIT);

  await expect(page.getByRole("button", { name: HIDE_SLIDERS })).toHaveCount(0);
  await expect(page.getByRole("button", { name: SHOW_SLIDERS })).toHaveCount(0);
});

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`pausing and resuming wave-tank at ${viewport.width}x${viewport.height} keeps the world frame`, async ({
    page,
  }) => {
    await openPlayingScene(page, "wave-tank", viewport);
    const playingFrame = await stableWorldFrame(page);

    await page.getByRole("button", { name: "Pause scene" }).click();
    await expect(sessionStatus(page)).toHaveText(PAUSED_STATUS);
    const pausedFrame = await stableWorldFrame(page);
    await page.getByRole("button", { name: "Play scene" }).click();
    await expect(sessionStatus(page)).toHaveText(PLAYING_STATUS);
    const resumedFrame = await stableWorldFrame(page);

    expectSameBox(pausedFrame, playingFrame);
    expectSameBox(resumedFrame, playingFrame);
  });
}
