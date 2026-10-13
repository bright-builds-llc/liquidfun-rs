import { expect, test, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { parseFixture, type FixturePlacement } from "../scripts/puzzle-replay/fixture";
import { defaultOpenTick } from "../src/puzzle/placements";
import { valveSecondsText } from "../src/puzzle/valve-format";
import { installControlledRefreshRate, numericAttribute } from "./player-helpers";
import {
  DESKTOP,
  fixturePath,
  puzzlePath,
  stablePuzzleCamera,
  waitForHeld,
  waitForPuzzlePhase,
} from "./puzzle-helpers";
import {
  SCREENS_DIR_40,
  expectPlayEnabled,
  gridToPage,
  readPlacements,
} from "./puzzle-editor-helpers";

const LEVEL_ID = "timed-gate" as const;
const SOLUTION = parseFixture(readFileSync(fixturePath("timed-gate-solution.txt"), "utf8"));
const RUN_TEST_TIMEOUT_MS = 120_000;
const HELD_TIMEOUT_MS = 60_000;
const ANGLE_STEPS = 24;
// The first few steps already show the jet on its way to the closed valve.
const RUNNING_SCREENSHOT_STEP = 90;

const EARLIER = "Open the valve 0.5 s earlier";
const LATER = "Open the valve 0.5 s later";

function button(page: Page, name: string) {
  return page.getByRole("button", { name, exact: true });
}

function main(page: Page) {
  return page.locator("main");
}

function valveReadout(page: Page) {
  return page.locator(".puzzle-valve-time");
}

/** `kind:x,y,angle` or `valve:x,y,angle,openTick`, as `data-puzzle-placements` writes it. */
function placementToken(placement: FixturePlacement): string {
  const pose = `${placement.kind}:${placement.x},${placement.y},${placement.angle}`;
  return placement.kind === "valve" ? `${pose},${placement.openTick}` : pose;
}

function tokens(raw: string): readonly string[] {
  return raw === "" ? [] : raw.split(";");
}

/** Pose of a `kind:x,y,angle[,openTick]` token. */
type Pose = { readonly x: number; readonly y: number; readonly angle: number };

function parseToken(token: string): Pose {
  const [, pose = ""] = token.split(":");
  const [x, y, angle] = pose.split(",").map(Number);
  if (x === undefined || y === undefined || angle === undefined) {
    throw new Error(`invalid placement token: ${token}`);
  }
  return { x, y, angle };
}

/** The token that `after` has and `before` lacks: the tool just placed or moved. */
async function newToken(page: Page, before: readonly string[]): Promise<string> {
  const added = tokens(await readPlacements(page)).filter((token) => !before.includes(token));
  const [maybeToken] = added;
  if (added.length !== 1 || maybeToken === undefined) {
    throw new Error(`expected one new placement, found ${added.length}`);
  }
  return maybeToken;
}

async function pressTimes(page: Page, key: string, times: number): Promise<void> {
  for (let press = 0; press < times; press += 1) {
    await page.keyboard.press(key);
  }
}

/** Arrow-key nudges from the tapped cell to the fixture cell (one cell per press). */
async function nudgeTo(page: Page, from: Pose, to: FixturePlacement): Promise<void> {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  await pressTimes(page, dx > 0 ? "ArrowRight" : "ArrowLeft", Math.abs(dx));
  await pressTimes(page, dy > 0 ? "ArrowUp" : "ArrowDown", Math.abs(dy));
}

/** `r` turns counter-clockwise one step and `Shift+R` clockwise; take the shorter way. */
async function rotateTo(page: Page, from: number, to: number): Promise<void> {
  const counterClockwise = (((to - from) % ANGLE_STEPS) + ANGLE_STEPS) % ANGLE_STEPS;
  if (counterClockwise <= ANGLE_STEPS / 2) {
    await pressTimes(page, "r", counterClockwise);
  } else {
    await pressTimes(page, "Shift+R", ANGLE_STEPS - counterClockwise);
  }
}

/** Steps the selected valve from the default tick to the fixture tick with the stepper. */
async function stepValveTo(page: Page, openTick: number): Promise<void> {
  const delta = openTick - defaultOpenTick("valve");
  const name = delta > 0 ? LATER : EARLIER;
  for (let click = 0; click < Math.abs(delta); click += 1) {
    await button(page, name).click();
  }
  await expect(valveReadout(page)).toHaveText(valveSecondsText(openTick));
}

/** Tray, canvas tap, arrow nudges, rotation keys and (valves) the stepper (D-17). */
async function placeFixtureTool(page: Page, placement: FixturePlacement): Promise<void> {
  const before = tokens(await readPlacements(page));
  const trayName = new RegExp(`^${placement.kind}, \\d+ left$`, "i");
  await page.getByRole("button", { name: trayName }).click();
  const point = await gridToPage(page, placement.x, placement.y, LEVEL_ID);
  await page.mouse.click(point.x, point.y);
  const tapped = parseToken(await newToken(page, before));
  await nudgeTo(page, tapped, placement);
  await rotateTo(page, tapped.angle, placement.angle);
  if (placement.kind === "valve") {
    await stepValveTo(page, placement.openTick);
  }
  expect(await newToken(page, before)).toBe(placementToken(placement));
}

async function openTimedGate(page: Page): Promise<void> {
  await installControlledRefreshRate(page, 60);
  await page.setViewportSize(DESKTOP);
  await page.goto(puzzlePath(LEVEL_ID));
  await waitForPuzzlePhase(page, "building");
  await stablePuzzleCamera(page);
}

test("replays the timed-gate reference through the tray, canvas and valve stepper", async ({
  page,
}) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  const maybeExpectation = SOLUTION.maybeExpectation;
  if (maybeExpectation === undefined || maybeExpectation.outcome !== "won") {
    throw new Error("timed-gate-solution.txt must record a won run");
  }
  const expected = SOLUTION.placements.map(placementToken).join(";");
  const [valve] = SOLUTION.placements.filter((placement) => placement.kind === "valve");
  if (valve === undefined) {
    throw new Error("timed-gate-solution.txt must place a valve");
  }

  // Arrange
  await openTimedGate(page);

  // Act: build the fixture's placements through the player controls.
  for (const placement of SOLUTION.placements) {
    await placeFixtureTool(page, placement);
  }

  // Assert
  expect(await readPlacements(page)).toBe(expected);
  await expectPlayEnabled(page, true);

  // Act: deselect, then reselect the valve and step its time with the keys.
  await page.keyboard.press("Escape");

  // Assert: the stepper stays mounted but disabled without a selected valve (D-05).
  await expect(main(page)).toHaveAttribute("data-puzzle-selected", "");
  for (const name of [EARLIER, LATER]) {
    await expect(button(page, name)).toHaveAttribute("aria-disabled", "true");
  }

  // Act
  const valvePoint = await gridToPage(page, valve.x, valve.y, LEVEL_ID);
  await page.mouse.click(valvePoint.x, valvePoint.y);
  await expect(main(page)).toHaveAttribute("data-puzzle-selected", "0");
  await page.keyboard.press("]");

  // Assert
  await expect(valveReadout(page)).toHaveText(valveSecondsText(valve.openTick + 1));

  // Act
  await page.keyboard.press("[");

  // Assert
  await expect(valveReadout(page)).toHaveText(valveSecondsText(valve.openTick));
  expect(await readPlacements(page)).toBe(expected);

  // Act: play, with a screenshot while the valve is still closed.
  await button(page, "Play").click();
  await expect
    .poll(() => numericAttribute(main(page), "data-step-index"))
    .toBeGreaterThan(RUNNING_SCREENSHOT_STEP);
  await page.screenshot({ path: resolve(SCREENS_DIR_40, "valve-running.png") });
  await waitForPuzzlePhase(page, "ended", HELD_TIMEOUT_MS);
  await waitForHeld(page, HELD_TIMEOUT_MS);

  // Assert: the browser reaches the fixture's recorded outcome (D-08, D-17).
  await expect(main(page)).toHaveAttribute("data-puzzle-outcome", "won");
  await expect(main(page)).toHaveAttribute(
    "data-puzzle-outcome-step",
    String(maybeExpectation.outcomeStep),
  );
  await expect(main(page)).toHaveAttribute(
    "data-puzzle-peak-held",
    String(maybeExpectation.peakHeld),
  );
  await expect(page.locator(".puzzle-result-message")).toContainText("Filled!");
  await expect(
    page
      .getByRole("list", { name: "Stars this run", exact: true })
      .locator('li[data-criterion="brim"][data-glyph="earned"]'),
  ).toHaveCount(1);
  await page.screenshot({ path: resolve(SCREENS_DIR_40, "valve-won.png") });
});
