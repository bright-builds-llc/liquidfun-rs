import { expect, test, type Browser, type Locator, type Page } from "@playwright/test";
import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  DAM_BREAK_PATH,
  installControlledRefreshRate,
  numericAttribute,
  setDocumentHidden,
  withFormFactorShell,
} from "./player-helpers";
import {
  DESKTOP,
  PHONE_PORTRAIT,
  PUZZLE_PATH,
  installSolutionPlacements,
  overlappingLeaves,
  overlappingObstacles,
  readHudGeometry,
  waitForHeld,
  waitForPuzzlePhase,
  type Viewport,
} from "./puzzle-helpers";
import { readRunFingerprint, type RunFingerprint } from "./puzzle-editor-helpers";
import {
  installFakeAudio,
  installThrowingAudio,
  installThrowingResumeAudio,
  installUnavailableAudio,
  readAudioLog,
  readStoredSound,
  storeSoundPreference,
  type AudioLog,
} from "./sound-helpers";

// Phase 42 D-13: opt-in sound in a real browser with a recording fake
// AudioContext, plus the UI-SPEC naming and fixed-box rules. Each test gets a
// fresh browser context, so storage starts empty.
test.describe.configure({ mode: "parallel" });

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const SCREENS_DIR_42 = resolve(REPO_ROOT, "target/phase42/screens");
const PHONE_LANDSCAPE = { width: 844, height: 390 } as const;
const GRID_HASH = "#/puzzles";
const SCENE_HASH = new URL(DAM_BREAK_PATH, "http://localhost").hash;
const SOUND_LABEL = "Sound";
const SOUND_EFFECTS_LABEL = "Sound effects";
const SOUND_HINT = "No sound on iPhone? Check the silent switch.";
const SOUND_UNAVAILABLE_LABEL = "Sound unavailable in this browser";
const SOUND_UNAVAILABLE_LINE =
  "Sound is unavailable in this browser. The puzzle still works without it.";
const SOUND_FAILED_NOTE =
  "Sound stopped after an audio error. Turn on Sound effects to try again.";
const OPTIONS_LABEL = "Puzzle options";
const RUN_TEST_TIMEOUT_MS = 90_000;
const TWO_RUN_TEST_TIMEOUT_MS = 150_000;
const ENDED_TIMEOUT_MS = 60_000;
// A synthetic 15 Hz clock with 4× speed keeps each run to a few seconds.
const RUN_REFRESH_HZ = 15;
const IDLE_STEP_COUNT = 60;

function viewportName(viewport: Viewport): string {
  return `${viewport.width}x${viewport.height}`;
}

async function screenshot(page: Page, name: string): Promise<void> {
  mkdirSync(SCREENS_DIR_42, { recursive: true });
  await page.screenshot({ path: resolve(SCREENS_DIR_42, `${name}.png`) });
}

async function openPuzzle(page: Page, viewport: Viewport = DESKTOP): Promise<void> {
  await page.setViewportSize(viewport);
  await page.goto(PUZZLE_PATH);
  await waitForPuzzlePhase(page, "building");
}

function soundButton(page: Page): Locator {
  return page.getByRole("button", { name: SOUND_LABEL, exact: true });
}

function optionsDialog(page: Page): Locator {
  return page.getByRole("dialog", { name: OPTIONS_LABEL });
}

async function openOptions(page: Page): Promise<Locator> {
  await page.getByRole("button", { name: OPTIONS_LABEL }).click();
  const dialog = optionsDialog(page);
  await expect(dialog).toBeVisible();
  return dialog;
}

async function closeOptions(page: Page): Promise<void> {
  await page.keyboard.press("Escape");
  await expect(optionsDialog(page)).toHaveCount(0);
}

function lastState(log: AudioLog): string | undefined {
  return log.states[log.states.length - 1];
}

async function expectLastState(page: Page, state: string): Promise<void> {
  await expect.poll(async () => lastState(await readAudioLog(page))).toBe(state);
}

// Copied from puzzle-progress.spec.ts.
async function speedToFour(page: Page): Promise<void> {
  for (const nextSpeed of ["Speed 2×", "Speed 4×"]) {
    await page.getByRole("button", { name: /^Speed \d×$/ }).click();
    await expect(page.getByRole("button", { name: nextSpeed, exact: true })).toBeVisible();
  }
  await expect(page.locator("main")).toHaveAttribute("data-puzzle-speed", "4");
}

/** Plays the seam solution on a fresh page; sound on only when asked. */
async function playSolution(
  browser: Browser,
  soundOn: boolean,
): Promise<{ readonly fingerprint: RunFingerprint; readonly log: AudioLog }> {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    await installFakeAudio(page);
    await installControlledRefreshRate(page, RUN_REFRESH_HZ);
    await installSolutionPlacements(page);
    await openPuzzle(page);
    if (soundOn) {
      await soundButton(page).click();
      await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
    }
    await speedToFour(page);
    await page.getByRole("button", { name: "Play", exact: true }).click();
    await waitForPuzzlePhase(page, "ended", ENDED_TIMEOUT_MS);
    await waitForHeld(page);
    await expect(page.locator("main")).toHaveAttribute("data-puzzle-outcome", "won");
    return { fingerprint: await readRunFingerprint(page), log: await readAudioLog(page) };
  } finally {
    await context.close();
  }
}

test("keeps sound off and builds no audio context by default", async ({ page }) => {
  test.setTimeout(RUN_TEST_TIMEOUT_MS);
  // Arrange
  await installFakeAudio(page);
  await installControlledRefreshRate(page, 60);
  await openPuzzle(page);
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");
  await expect(soundButton(page)).toHaveAttribute("data-sound-state", "off");

  // Act
  await page.getByRole("button", { name: "Play", exact: true }).click();
  await waitForPuzzlePhase(page, "running");
  await expect
    .poll(() => numericAttribute(page.locator("main"), "data-step-index"))
    .toBeGreaterThan(IDLE_STEP_COUNT);

  // Assert (D-04)
  const log = await readAudioLog(page);
  expect(log.constructed).toBe(0);
  expect(log.started).toBe(0);
  expect(await readStoredSound(page)).toBeNull();
});

test("waits for a trusted gesture before building the context for a stored on", async ({
  page,
}) => {
  // Arrange
  await installFakeAudio(page);
  await storeSoundPreference(page, "on");
  await openPuzzle(page);

  // Assert (D-03, D-04, D-17): pressed, but no context before a gesture.
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
  expect((await readAudioLog(page)).constructed).toBe(0);

  // Act: a mouse press on the level title is an activation gesture.
  const titleBox = await page.locator(".puzzle-title").boundingBox();
  if (titleBox === null) {
    throw new Error("the level title has no box");
  }
  await page.mouse.click(titleBox.x + titleBox.width / 2, titleBox.y + titleBox.height / 2);

  // Assert
  await expectLastState(page, "running");
  expect((await readAudioLog(page)).constructed).toBe(1);
});

test("creates one context from the toggle and suspends it when turned off", async ({
  page,
}) => {
  // Arrange
  await installFakeAudio(page);
  await openPuzzle(page);

  // Act
  await soundButton(page).click();

  // Assert (D-01, D-04)
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
  await expectLastState(page, "running");
  expect((await readAudioLog(page)).constructed).toBe(1);
  expect(await readStoredSound(page)).toBe("on");

  // Act
  await soundButton(page).click();

  // Assert (D-04): the context is suspended for reuse, never rebuilt or closed.
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");
  await expectLastState(page, "suspended");
  const log = await readAudioLog(page);
  expect(log.constructed).toBe(1);
  expect(log.closed).toBe(0);
  expect(await readStoredSound(page)).toBe("off");
});

test("keeps sound on across a reload", async ({ page }) => {
  // Arrange
  await installFakeAudio(page);
  await openPuzzle(page);
  await soundButton(page).click();
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");

  // Act
  await page.reload();
  await waitForPuzzlePhase(page, "building");

  // Assert (D-03)
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
  expect(await readStoredSound(page)).toBe("on");
  expect((await readAudioLog(page)).constructed).toBe(0);
});

test("keeps sound off across a reload after turning it off", async ({ page }) => {
  // Arrange
  await installFakeAudio(page);
  await storeSoundPreference(page, "on");
  await openPuzzle(page);
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
  await soundButton(page).click();
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");

  // Act
  await page.reload();
  await waitForPuzzlePhase(page, "building");

  // Assert (D-03)
  await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");
  expect(await readStoredSound(page)).toBe("off");
});

test("suspends while the tab is hidden and resumes when it is visible", async ({ page }) => {
  // Arrange
  await installFakeAudio(page);
  await openPuzzle(page);
  await soundButton(page).click();
  await expectLastState(page, "running");

  // Act
  await setDocumentHidden(page, true);

  // Assert (D-10)
  await expectLastState(page, "suspended");

  // Act
  await setDocumentHidden(page, false);

  // Assert
  await expectLastState(page, "running");
  expect((await readAudioLog(page)).constructed).toBe(1);
});

test("suspends when the player leaves the puzzle and shows no toggle on scenes", async ({
  page,
}) => {
  // Arrange
  await installFakeAudio(page);
  await openPuzzle(page);
  await soundButton(page).click();
  await expectLastState(page, "running");

  // Act
  await page.evaluate((hash) => {
    window.location.hash = hash;
  }, GRID_HASH);
  await expect(page.getByRole("heading", { name: "Puzzles", level: 2 })).toBeVisible();

  // Assert (D-10): suspended, kept for reuse.
  await expectLastState(page, "suspended");
  expect((await readAudioLog(page)).closed).toBe(0);

  // Act
  await page.evaluate((hash) => {
    window.location.hash = hash;
  }, SCENE_HASH);
  await expect(page.locator("canvas.scene-canvas")).toBeVisible();

  // Assert (D-01): sound is puzzle-only.
  await expect(page.locator("main[data-puzzle]")).toHaveCount(0);
  await expect(soundButton(page)).toHaveCount(0);
});

test("reaches the same run with sound on and off", async ({ browser }) => {
  test.setTimeout(TWO_RUN_TEST_TIMEOUT_MS);
  // Act
  const [withSound, withoutSound] = await Promise.all([
    playSolution(browser, true),
    playSolution(browser, false),
  ]);

  // Assert (D-08, D-13, criterion 4)
  console.log(
    `sound-on fingerprint ${JSON.stringify(withSound.fingerprint)} started=${withSound.log.started}`,
  );
  console.log(`sound-off fingerprint ${JSON.stringify(withoutSound.fingerprint)}`);
  expect(withSound.fingerprint).toEqual(withoutSound.fingerprint);
  expect(withSound.log.started).toBeGreaterThan(0);
  expect(withoutSound.log.constructed).toBe(0);
  expect(withoutSound.log.started).toBe(0);
});

test("disables both controls when Web Audio is unavailable and still plays", async ({
  page,
}) => {
  // Arrange
  await installUnavailableAudio(page);
  await openPuzzle(page);

  // Assert (D-05)
  const unavailable = page.getByRole("button", { name: SOUND_UNAVAILABLE_LABEL, exact: true });
  await expect(unavailable).toBeDisabled();
  await expect(unavailable).toHaveAttribute("data-sound-state", "unavailable");
  await expect(soundButton(page)).toHaveCount(0);

  // Act
  const dialog = await openOptions(page);

  // Assert
  const checkbox = dialog.getByRole("checkbox", { name: SOUND_EFFECTS_LABEL });
  await expect(checkbox).toBeDisabled();
  await expect(checkbox).not.toBeChecked();
  await expect(dialog.locator(".puzzle-sound-section")).toContainText(SOUND_UNAVAILABLE_LINE);

  // Act
  await closeOptions(page);
  await page.getByRole("button", { name: "Play", exact: true }).click();

  // Assert
  await waitForPuzzlePhase(page, "running");
});

test("keeps focus on the HUD toggle when context creation fails", async ({ page }) => {
  // Arrange
  await installThrowingAudio(page);
  await openPuzzle(page);
  await soundButton(page).focus();

  // Act
  await page.keyboard.press("Enter");

  // Assert (D-05, IN-03): the same button turns unavailable and keeps focus.
  const unavailable = page.getByRole("button", { name: SOUND_UNAVAILABLE_LABEL, exact: true });
  await expect(unavailable).toBeDisabled();
  await expect(unavailable).toBeFocused();
  await expect(unavailable).toHaveAttribute("aria-pressed", "false");
});

test("unchecks the drawer checkbox when context creation fails", async ({ page }) => {
  // Arrange
  await installThrowingAudio(page);
  await openPuzzle(page);
  const dialog = await openOptions(page);
  const checkbox = dialog.getByRole("checkbox", { name: SOUND_EFFECTS_LABEL });

  // Act
  await checkbox.click();

  // Assert (D-05, IN-03)
  await expect(checkbox).toBeDisabled();
  await expect(checkbox).not.toBeChecked();
  expect(await readStoredSound(page)).toBeNull();
});

test("unchecks the drawer checkbox when resume throws", async ({ page }) => {
  // Arrange
  await installThrowingResumeAudio(page);
  await openPuzzle(page);
  const dialog = await openOptions(page);
  const checkbox = dialog.getByRole("checkbox", { name: SOUND_EFFECTS_LABEL });

  // Act
  await checkbox.click();

  // Assert (D-08, D-16, IN-03): sound stays available but off after the failure.
  await expect(dialog.locator(".puzzle-sound-note")).toContainText(SOUND_FAILED_NOTE);
  await expect(checkbox).toBeEnabled();
  await expect(checkbox).not.toBeChecked();
  expect(await readStoredSound(page)).toBeNull();
});

for (const viewport of [DESKTOP, PHONE_PORTRAIT]) {
  test(`mirrors the toggle in the options drawer at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    // Arrange
    await installFakeAudio(page);
    await openPuzzle(page, viewport);

    // Assert (D-11, D-18): one "Sound" button whose title is the hint.
    await expect(soundButton(page)).toHaveCount(1);
    await expect(soundButton(page)).toHaveAttribute("title", SOUND_HINT);
    await expect(soundButton(page)).toHaveAttribute("aria-describedby", "puzzle-sound-help");

    // Act
    let dialog = await openOptions(page);
    const checkbox = () => optionsDialog(page).getByRole("checkbox", { name: SOUND_EFFECTS_LABEL });

    // Assert
    await expect(dialog.getByRole("heading", { name: SOUND_LABEL, exact: true })).toBeVisible();
    await expect(dialog.locator(".puzzle-sound-section")).toContainText(SOUND_HINT);
    await expect(checkbox()).not.toBeChecked();
    if (viewport === PHONE_PORTRAIT) {
      await screenshot(page, "portrait-drawer");
    }

    // Act: the checkbox turns the HUD button on.
    await checkbox().check();
    await expect(checkbox()).toBeChecked();
    await closeOptions(page);

    // Assert
    await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
    await expectLastState(page, "running");

    // Act: the HUD button turns the checkbox off.
    await soundButton(page).click();
    await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");
    dialog = await openOptions(page);

    // Assert
    await expect(checkbox()).not.toBeChecked();
    await expect(dialog).toBeVisible();
  });
}

for (const [viewport, name] of [
  [PHONE_PORTRAIT, "portrait"],
  [PHONE_LANDSCAPE, "landscape"],
] as const) {
  test(`keeps every HUD box fixed when sound toggles at ${viewportName(viewport)}`, async ({
    page,
  }) => {
    // Arrange
    await installFakeAudio(page);
    await openPuzzle(page, viewport);
    await expect(soundButton(page)).toHaveAttribute("aria-pressed", "false");
    const before = await readHudGeometry(page);
    await screenshot(page, `${name}-off`);

    // Act
    await soundButton(page).click();
    await expect(soundButton(page)).toHaveAttribute("aria-pressed", "true");
    const after = await readHudGeometry(page);
    await screenshot(page, `${name}-on`);

    // Assert (D-01): the toggle never moves, resizes or overlaps a leaf.
    expect(after.obstacles).toEqual(before.obstacles);
    expect(overlappingLeaves(after)).toEqual([]);
    expect(overlappingObstacles(after)).toEqual([]);
  });
}

test("shows the checkbox and hint in the form-factor row without a HUD toggle", async ({
  page,
}) => {
  // Arrange
  await installFakeAudio(page);
  await page.setViewportSize(DESKTOP);

  // Act
  await page.goto(withFormFactorShell(PUZZLE_PATH));
  await waitForPuzzlePhase(page, "building");

  // Assert (D-18)
  await expect(page.getByRole("checkbox", { name: SOUND_EFFECTS_LABEL })).toBeVisible();
  await expect(page.locator(".control-row .puzzle-sound-section")).toContainText(SOUND_HINT);
  await expect(soundButton(page)).toHaveCount(0);
  await screenshot(page, "form-factor");
});

test("builds no context when the label text turns a stored on off", async ({ page }) => {
  // Arrange
  await installFakeAudio(page);
  await storeSoundPreference(page, "on");
  await page.setViewportSize(DESKTOP);
  await page.goto(withFormFactorShell(PUZZLE_PATH));
  await waitForPuzzlePhase(page, "building");
  const checkbox = page.getByRole("checkbox", { name: SOUND_EFFECTS_LABEL });
  await expect(checkbox).toBeChecked();

  const label = page.locator(".control-row .puzzle-sound-section label");
  const labelBox = await label.boundingBox();
  if (labelBox === null) {
    throw new Error("the sound label has no box");
  }

  // Act: press the label's far end, away from the box itself.
  await label.click({ position: { x: labelBox.width - 2, y: labelBox.height / 2 } });

  // Assert (D-04, UI-D35, IN-04)
  await expect(checkbox).not.toBeChecked();
  expect((await readAudioLog(page)).constructed).toBe(0);
  expect(await readStoredSound(page)).toBe("off");
});
