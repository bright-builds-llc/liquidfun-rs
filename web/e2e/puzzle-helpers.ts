import { expect, type Locator, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { parseFixture } from "../scripts/puzzle-replay/fixture";

export const PUZZLE_PATH = "/liquidfun-rs/#/puzzle/first-pour";
export const UNKNOWN_PUZZLE_PATH = "/liquidfun-rs/#/puzzle/nope";
export const DESKTOP = { width: 1280, height: 800 } as const;
export const PHONE_PORTRAIT = { width: 390, height: 844 } as const;

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const SOLUTION_FIXTURE = resolve(
  REPO_ROOT,
  "crates/liquidfun-wasm/src/puzzle/fixtures/first-pour-solution.txt",
);

/** Verifier screenshots for Phase 38. */
export const SCREENS_DIR = resolve(REPO_ROOT, "target/phase38/screens");

const OVERLAP_TOLERANCE_PX = 1;
const STABLE_CAMERA_GAP_MS = 250;
const DEFAULT_PHASE_TIMEOUT_MS = 20_000;

export type PuzzlePhase = "building" | "running" | "ended";

/** The playground `main` element while a puzzle route is mounted. */
export function puzzleMain(page: Page): Locator {
  return page.locator("main[data-puzzle]");
}

export async function waitForPuzzlePhase(
  page: Page,
  phase: PuzzlePhase,
  timeoutMs = DEFAULT_PHASE_TIMEOUT_MS,
): Promise<void> {
  await expect(page.locator(`main[data-puzzle-phase="${phase}"]`)).toBeVisible({
    timeout: timeoutMs,
  });
}

/** Waits until the ended run has stepped through its post-outcome tail. */
export async function waitForHeld(
  page: Page,
  timeoutMs = DEFAULT_PHASE_TIMEOUT_MS,
): Promise<void> {
  await expect(page.locator('main[data-puzzle-held="true"]')).toBeVisible({
    timeout: timeoutMs,
  });
}

/** Raw camera attributes, compared as strings so any drift is visible. */
export type PuzzleCamera = {
  readonly worldFrame: string;
  readonly pixelsPerMeter: string;
  readonly usedFallback: boolean;
};

export async function readPuzzleCamera(page: Page): Promise<PuzzleCamera> {
  const canvas = page.locator("canvas.scene-canvas");
  const maybeWorldFrame = await canvas.getAttribute("data-world-frame");
  const maybePixelsPerMeter = await page
    .locator(".puzzle-panel")
    .getAttribute("data-pixels-per-meter");
  if (maybeWorldFrame === null || maybePixelsPerMeter === null) {
    throw new Error(
      `puzzle camera attributes missing: frame=${String(maybeWorldFrame)} ppm=${String(maybePixelsPerMeter)}`,
    );
  }
  const maybeFallback = await canvas.getAttribute("data-world-frame-fallback");
  return {
    worldFrame: maybeWorldFrame,
    pixelsPerMeter: maybePixelsPerMeter,
    usedFallback: maybeFallback === "true",
  };
}

/** Camera after layout settles: two reads a moment apart agree. */
export async function stablePuzzleCamera(page: Page): Promise<PuzzleCamera> {
  let maybeStable: PuzzleCamera | undefined;
  await expect(async () => {
    const first = await readPuzzleCamera(page);
    await page.waitForTimeout(STABLE_CAMERA_GAP_MS);
    const second = await readPuzzleCamera(page);
    expect(second).toEqual(first);
    maybeStable = second;
  }).toPass({ timeout: 5_000 });
  if (maybeStable === undefined) {
    throw new Error("puzzle camera never settled");
  }
  return maybeStable;
}

/**
 * Injects the recorded first-pour solution through the D-22 test seam. The
 * placements cross into the page only as plain objects in an init script,
 * never through the URL.
 */
export async function installSolutionPlacements(page: Page): Promise<void> {
  const fixture = parseFixture(readFileSync(SOLUTION_FIXTURE, "utf8"));
  await page.addInitScript(
    (value) => {
      Object.defineProperty(window, "__liquidfunPuzzleTestPlacements", {
        value,
        configurable: true,
      });
    },
    {
      level: fixture.level,
      placements: fixture.placements.map((placement) => ({
        kind: placement.kind,
        x: placement.x,
        y: placement.y,
        angle: placement.angle,
      })),
    },
  );
}

type Box = {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
};

type LabeledBox = Box & { readonly label: string };

/** Projected world rectangle and HUD obstacle boxes, both in page pixels. */
export type HudGeometry = {
  readonly maybeWorldFrame: Box | undefined;
  readonly usedFallback: boolean;
  readonly obstacles: readonly LabeledBox[];
};

// Copied from hud-safe-frame.spec.ts (protected), not imported from it.
export async function readHudGeometry(page: Page): Promise<HudGeometry> {
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

/** Obstacles that cover the world frame by more than 1 px, as readable labels. */
export function overlappingObstacles(geometry: HudGeometry): string[] {
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
