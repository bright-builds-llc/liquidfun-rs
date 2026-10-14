import { expect, type Locator, type Page } from "@playwright/test";

import { openSceneControls } from "./player-helpers";

/** Stored render-mode tokens the Water proofs seed. */
export type StoredRenderMode = "water" | "shaded-blob";

const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";
const SEEDED_MARKER_KEY = "__liquidfunRenderModeSeeded";
const LOSE_CONTEXT_SLOT = "__liquidfunWaterLoseContext";
const MAX_DRAW_BUFFERS = 0x8824;
const PUZZLE_OPTIONS_LABEL = "Puzzle options";

/**
 * Seeds the stored render mode on the first navigation only, so a later
 * reload observes what the app itself wrote.
 */
export async function storeRenderMode(page: Page, mode: StoredRenderMode): Promise<void> {
  await page.addInitScript(
    ({ key, marker, stored }) => {
      if (window.sessionStorage.getItem(marker) !== null) {
        return;
      }
      window.sessionStorage.setItem(marker, "1");
      window.localStorage.setItem(key, stored);
    },
    { key: RENDER_MODE_STORAGE_KEY, marker: SEEDED_MARKER_KEY, stored: mode },
  );
}

/**
 * Reports a single draw buffer on every WebGL2 context before any app code
 * runs: the Water probe and the live limit check fail, shaded blob still
 * draws (Research Pattern 6).
 */
export async function forceWaterUnavailable(page: Page): Promise<void> {
  await page.addInitScript((maxDrawBuffers) => {
    const prototype = WebGL2RenderingContext.prototype;
    const nativeGetParameter = prototype.getParameter;
    prototype.getParameter = function getParameter(
      this: WebGL2RenderingContext,
      pname: number,
    ): unknown {
      return pname === maxDrawBuffers ? 1 : nativeGetParameter.call(this, pname);
    };
  }, MAX_DRAW_BUFFERS);
}

/**
 * Hides both color-buffer float extensions before any app code runs, so no
 * WebGL surface can draw and the 2D canvas paints the fallback.
 */
export async function forceNoFloatTargets(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const blocked = new Set(["EXT_color_buffer_float", "EXT_color_buffer_half_float"]);
    const prototype = WebGL2RenderingContext.prototype;
    const nativeGetExtension: (
      this: WebGL2RenderingContext,
      name: string,
    ) => unknown = prototype.getExtension;
    prototype.getExtension = function getExtension(
      this: WebGL2RenderingContext,
      name: string,
    ) {
      return blocked.has(name) ? null : nativeGetExtension.call(this, name);
    } as typeof prototype.getExtension;
  });
}

/**
 * Loses the particle surface context through `WEBGL_lose_context`. The
 * extension object is kept on `window` because a lost context may not hand
 * it out again.
 */
export async function loseParticleContext(page: Page): Promise<void> {
  await page.evaluate((slot) => {
    const maybeCanvas = document.querySelector<HTMLCanvasElement>("canvas.particle-surface");
    const maybeExtension = maybeCanvas?.getContext("webgl2")?.getExtension("WEBGL_lose_context");
    if (maybeExtension === undefined || maybeExtension === null) {
      throw new Error("the particle surface has no WEBGL_lose_context");
    }
    Object.defineProperty(window, slot, { value: maybeExtension, configurable: true });
    maybeExtension.loseContext();
  }, LOSE_CONTEXT_SLOT);
}

/** Restores the context lost by `loseParticleContext`. */
export async function restoreParticleContext(page: Page): Promise<void> {
  await page.evaluate((slot) => {
    const maybeExtension = Reflect.get(window, slot) as WEBGL_lose_context | undefined;
    if (maybeExtension === undefined) {
      throw new Error("loseParticleContext was not called");
    }
    maybeExtension.restoreContext();
  }, LOSE_CONTEXT_SLOT);
}

/** The 2D scene canvas, which carries `data-drawn-surface` and `data-speed-lane`. */
export function drawnSurface(page: Page): Locator {
  return page.locator("canvas.scene-canvas");
}

async function isPuzzleRoute(page: Page): Promise<boolean> {
  return (await page.locator("main[data-puzzle]").count()) > 0;
}

/** Opens the "Puzzle options" drawer unless it is already open. */
export async function openPuzzleOptions(page: Page): Promise<Locator> {
  const dialog = page.getByRole("dialog", { name: PUZZLE_OPTIONS_LABEL });
  if (await dialog.isVisible()) {
    return dialog;
  }
  await page.getByRole("button", { name: PUZZLE_OPTIONS_LABEL }).click();
  await expect(dialog).toBeVisible();
  return dialog;
}

/**
 * The visible "Particles" select, opening the scene controls or the puzzle
 * options drawer when no picker is on screen.
 */
export async function particlesSelect(page: Page): Promise<Locator> {
  const select = page.getByRole("combobox", { name: "Particles", exact: true });
  if ((await select.count()) > 0 && (await select.first().isVisible())) {
    return select.first();
  }
  if (await isPuzzleRoute(page)) {
    await openPuzzleOptions(page);
  } else {
    await openSceneControls(page);
  }
  await expect(select.first()).toBeVisible();
  return select.first();
}

/** Chooses a render mode by its option label in the visible Particles select. */
export async function chooseRenderMode(page: Page, label: string): Promise<void> {
  const select = await particlesSelect(page);
  await select.selectOption({ label });
}
