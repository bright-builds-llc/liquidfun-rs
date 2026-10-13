import { expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { worldBoundsForPuzzle } from "../src/catalog/puzzle-bounds";
import type { PuzzleId } from "../src/catalog/puzzles";
import { parseFixture } from "../scripts/puzzle-replay/fixture";
import { SOLUTION_FIXTURE } from "./puzzle-helpers";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

/** Verifier screenshots for Phase 39. */
export const SCREENS_DIR_39 = resolve(REPO_ROOT, "target/phase39/screens");
export const SCREENS_DIR_40 = resolve(REPO_ROOT, "target/phase40/screens");

/** Placement grid pitch in meters. */
const GRID_METERS = 0.05;
const SCALE_TOLERANCE = 0.01;

// first-pour level spec: capacity, 60% target and the 12 s step limit.
const FIRST_POUR_FULL = 900;
const FIRST_POUR_TARGET_PERCENT = 60;
const FIRST_POUR_STEP_LIMIT = 720;
// Steps the shell keeps running after the decision before holding (D-10).
const SETTLE_TAIL_STEPS = 120;
const STEPS_PER_TENTH = 6;

export type PagePoint = { readonly x: number; readonly y: number };

function parseWorldFrame(raw: string): readonly [number, number, number, number] {
  const edges = raw.split(",").map(Number);
  const [left, top, right, bottom] = edges;
  if (
    edges.length !== 4 ||
    left === undefined ||
    top === undefined ||
    right === undefined ||
    bottom === undefined ||
    edges.some((edge) => !Number.isFinite(edge))
  ) {
    throw new Error(`invalid data-world-frame: ${raw}`);
  }
  return [left, top, right, bottom];
}

/**
 * Page coordinates of a level's world point, from the published camera
 * (`data-world-frame`). Fails fast when the derived scale disagrees with the
 * panel's `data-pixels-per-meter`.
 */
export async function worldToPage(
  page: Page,
  world: { readonly x: number; readonly y: number },
  levelId: PuzzleId = "first-pour",
): Promise<PagePoint> {
  const canvas = page.locator("canvas.scene-canvas");
  const maybeBox = await canvas.boundingBox();
  const maybeFrame = await canvas.getAttribute("data-world-frame");
  const maybePixelsPerMeter = await page
    .locator(".puzzle-panel")
    .getAttribute("data-pixels-per-meter");
  if (maybeBox === null || maybeFrame === null || maybePixelsPerMeter === null) {
    throw new Error(
      `puzzle camera unavailable: box=${String(maybeBox)} frame=${String(maybeFrame)} ppm=${String(maybePixelsPerMeter)}`,
    );
  }
  const [left, top, right] = parseWorldFrame(maybeFrame);
  const bounds = worldBoundsForPuzzle(levelId, maybeBox.width, maybeBox.height);
  const scale = (right - left) / (bounds.maxX - bounds.minX);
  const publishedScale = Number(maybePixelsPerMeter);
  if (!(Math.abs(scale - publishedScale) <= SCALE_TOLERANCE)) {
    throw new Error(
      `camera scale ${scale} disagrees with data-pixels-per-meter ${maybePixelsPerMeter}`,
    );
  }
  return {
    x: maybeBox.x + left + (world.x - bounds.minX) * scale,
    y: maybeBox.y + top + (bounds.maxY - world.y) * scale,
  };
}

/** Page coordinates of a placement grid cell centre. */
export async function gridToPage(
  page: Page,
  cellX: number,
  cellY: number,
  levelId: PuzzleId = "first-pour",
): Promise<PagePoint> {
  return worldToPage(page, { x: cellX * GRID_METERS, y: cellY * GRID_METERS }, levelId);
}

/** Canonical `kind:x,y,angle` placements published on `main`. */
export async function readPlacements(page: Page): Promise<string> {
  const maybeValue = await page.locator("main").getAttribute("data-puzzle-placements");
  if (maybeValue === null) {
    throw new Error("data-puzzle-placements is missing");
  }
  return maybeValue;
}

/** Waits for Play to reach the given enabled state on both `main` and the button. */
export async function expectPlayEnabled(page: Page, enabled: boolean): Promise<void> {
  await expect(page.locator("main")).toHaveAttribute(
    "data-puzzle-play-enabled",
    String(enabled),
  );
  const play = page.getByRole("button", { name: "Play", exact: true });
  if (enabled) {
    await expect(play).not.toHaveAttribute("aria-disabled");
  } else {
    await expect(play).toHaveAttribute("aria-disabled", "true");
  }
}

/** Expected won-run values for the recorded first-pour solution (D-10). */
export type SolutionExpectation = {
  readonly cardText: string;
  readonly timerText: string;
  readonly heldStep: string;
  readonly meterValue: string;
  readonly outcomeStep: string;
  readonly decidedStep: string;
  readonly peakHeld: string;
};

/** `m:ss.t`, floored to tenths of a simulated second (Phase 38 timer). */
function timerText(remainingSteps: number): string {
  const tenths = Math.floor(Math.max(0, remainingSteps) / STEPS_PER_TENTH);
  const minutes = Math.floor(tenths / 600);
  const seconds = Math.floor((tenths % 600) / 10);
  return `${minutes}:${String(seconds).padStart(2, "0")}.${tenths % 10}`;
}

/**
 * Won-run literals derived from the recorded solution fixture: the decision
 * is the Brim step, the card and meter show the peak held percent, and the
 * timer stops at the decision.
 */
export function solutionExpectation(): SolutionExpectation {
  const fixture = parseFixture(readFileSync(SOLUTION_FIXTURE, "utf8"));
  const maybeExpectation = fixture.maybeExpectation;
  if (maybeExpectation === undefined || maybeExpectation.outcome !== "won") {
    throw new Error("the solution fixture must record a won run");
  }
  const { outcomeStep, peakHeld, maybeBrimStep } = maybeExpectation;
  if (maybeBrimStep === undefined) {
    throw new Error("the solution fixture must record a Brim step");
  }
  const percent = Math.min(100, Math.floor((peakHeld * 100) / FIRST_POUR_FULL));
  return {
    cardText: `Filled! ${percent}% of ${FIRST_POUR_TARGET_PERCENT}%`,
    timerText: timerText(FIRST_POUR_STEP_LIMIT - maybeBrimStep),
    heldStep: String(maybeBrimStep + SETTLE_TAIL_STEPS),
    meterValue: String(percent),
    outcomeStep: String(outcomeStep),
    decidedStep: String(maybeBrimStep),
    peakHeld: String(peakHeld),
  };
}

/** The four D-27/D-28 run attributes of an ended, held run. */
export type RunFingerprint = {
  readonly outcomeStep: string;
  readonly decidedStep: string;
  readonly peakHeld: string;
  readonly heldHash: string;
};

/** Reads the ended-run attributes; call after `waitForHeld`. */
export async function readRunFingerprint(page: Page): Promise<RunFingerprint> {
  const main = page.locator("main");
  const read = async (name: string): Promise<string> => {
    const maybeValue = await main.getAttribute(name);
    if (maybeValue === null) {
      throw new Error(`${name} is missing`);
    }
    return maybeValue;
  };
  return {
    outcomeStep: await read("data-puzzle-outcome-step"),
    decidedStep: await read("data-puzzle-decided-step"),
    peakHeld: await read("data-puzzle-peak-held"),
    heldHash: await read("data-puzzle-held-hash"),
  };
}
