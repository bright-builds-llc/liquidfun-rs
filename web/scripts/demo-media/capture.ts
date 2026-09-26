import { expect, type Locator, type Page } from "@playwright/test";
import { mkdir, readdir } from "node:fs/promises";
import { join } from "node:path";

import { accumulateStepTime } from "../../src/physics/clock";
import {
  CAPTURE_PROFILE,
  frameFileName,
  type PointRatio,
  type SceneCapturePlan,
} from "./model";

export type CaptureSceneFramesArgs = {
  readonly page: Page;
  readonly plan: SceneCapturePlan;
  readonly framesDirectory: string;
};

const SYNTHETIC_CLOCK_KEY = "__liquidfunSyntheticAnimationClock";
const DEMO_MEDIA_CAPTURE_ATTRIBUTE = "data-demo-media-capture";
const anchoredClockStates = new WeakMap<Page, SyntheticClockState>();

type SyntheticAnimationController = {
  readonly advance: (timestampMilliseconds: number) => number;
  readonly pendingCount: () => number;
};

type SceneSnapshot = {
  readonly sceneId: string | null;
  readonly playback: string | null;
  readonly stepIndex: number;
  readonly pointerAccepted: number;
};

type SyntheticClockState = {
  readonly callbackCount: number;
};

// A fixed one-ULP-safe 60 Hz interval derived once from 1_000 / simulationHz.
// Each callback timestamp is computed as callbackCount * interval so the runtime
// never searches for additional ULPs or changes cadence after initialization.
const FIXED_TIMESTAMP_DELTA_MILLISECONDS = nextUp(
  1_000 / CAPTURE_PROFILE.simulationHz,
);
assertFixedTimestampProgression(
  FIXED_TIMESTAMP_DELTA_MILLISECONDS,
  CAPTURE_PROFILE.frameCount * CAPTURE_PROFILE.stepsPerFrame,
);

export async function installSyntheticAnimationClock(page: Page): Promise<void> {
  anchoredClockStates.delete(page);
  await page.addInitScript((clockKey) => {
    type SyntheticAnimationController = {
      readonly advance: (timestampMilliseconds: number) => number;
      readonly pendingCount: () => number;
    };

    const callbackQueue = new Map<number, FrameRequestCallback>();
    let nextCallbackId = 1;

    const syntheticAnimationController: SyntheticAnimationController = {
      advance(timestampMilliseconds) {
        const callbacks = [...callbackQueue.entries()];
        callbackQueue.clear();

        for (const [, callback] of callbacks) {
          callback(timestampMilliseconds);
        }

        return callbacks.length;
      },
      pendingCount() {
        return callbackQueue.size;
      },
    };

    const globalWindow = window as typeof window &
      Record<string, SyntheticAnimationController>;

    window.requestAnimationFrame = (callback) => {
      const callbackId = nextCallbackId;
      nextCallbackId += 1;
      callbackQueue.set(callbackId, callback);
      return callbackId;
    };
    window.cancelAnimationFrame = (callbackId) => {
      callbackQueue.delete(callbackId);
    };
    globalWindow[clockKey] = syntheticAnimationController;
  }, SYNTHETIC_CLOCK_KEY);
}

export async function waitForReadyScene(
  page: Page,
  plan: SceneCapturePlan,
): Promise<number> {
  const main = page.locator(
    `main[data-scene="${plan.id}"][data-playback="playing"]`,
  );
  await expect(main).toBeVisible();
  await expect.poll(() => numericAttribute(main, "data-step-index")).toBeGreaterThan(0);
  await expect.poll(() => pendingAnimationCallbackCount(page)).toBe(1);

  const initialStep = await numericAttribute(main, "data-step-index");
  const advancedCallbacks = await advanceSyntheticClock(page, 0);
  if (advancedCallbacks !== 1) {
    throw new Error(
      `Synthetic anchor expected 1 callback, got ${advancedCallbacks}`,
    );
  }

  anchoredClockStates.set(page, {
    callbackCount: 0,
  });
  expect(await numericAttribute(main, "data-step-index")).toBe(initialStep);
  await expect.poll(() => pendingAnimationCallbackCount(page)).toBe(1);
  return initialStep;
}

export async function advanceEngineSteps(
  page: Page,
  count: number,
): Promise<void> {
  if (!Number.isInteger(count) || count < 0) {
    throw new Error(`Engine step count must be a non-negative integer, got ${count}`);
  }

  const maybeClockState = anchoredClockStates.get(page);
  if (maybeClockState === undefined) {
    throw new Error("Synthetic clock anchor is missing");
  }

  const main = page.locator("main");
  let currentStepIndex = await numericAttribute(main, "data-step-index");
  let currentClockState = maybeClockState;

  for (let completedSteps = 0; completedSteps < count; completedSteps += 1) {
    currentClockState = {
      callbackCount: currentClockState.callbackCount + 1,
    };
    const advancedCallbacks = await advanceSyntheticClock(
      page,
      currentClockState.callbackCount * FIXED_TIMESTAMP_DELTA_MILLISECONDS,
    );
    if (advancedCallbacks !== 1) {
      throw new Error(
        `Expected exactly 1 animation callback, got ${advancedCallbacks}`,
      );
    }

    await expect
      .poll(() => numericAttribute(main, "data-step-index"))
      .toBe(currentStepIndex + 1);
    const nextStepIndex = await numericAttribute(main, "data-step-index");

    const nextPendingCount = await pendingAnimationCallbackCount(page);
    if (nextPendingCount !== 1) {
      throw new Error(
        `Expected exactly 1 successor callback, got ${nextPendingCount}`,
      );
    }

    currentStepIndex = nextStepIndex;
  }

  anchoredClockStates.set(page, currentClockState);
}

export async function captureSceneFrames({
  page,
  plan,
  framesDirectory,
}: CaptureSceneFramesArgs): Promise<void> {
  assertCaptureViewport(page);
  await assertCaptureDeviceScaleFactor(page);
  await mkdir(framesDirectory, { recursive: true });
  await page.goto(plan.route);
  await waitForReadyScene(page, plan);
  await requireWireframeRenderMode(page);
  await withDemoMediaCaptureMode(page, async () => {
    let completedSteps = 0;
    let wroteFrameCount = 0;
    let interactionPerformed = false;

    for (
      let frameIndex = 0;
      frameIndex < CAPTURE_PROFILE.frameCount;
      frameIndex += 1
    ) {
      await assertPlayingScene(page, plan.id);

      if (!interactionPerformed && completedSteps === plan.interactionStep) {
        await performSceneAction(page, plan);
        interactionPerformed = true;
      }

      await advanceEngineSteps(page, CAPTURE_PROFILE.stepsPerFrame);
      completedSteps += CAPTURE_PROFILE.stepsPerFrame;

      const panel = page.locator(".player-panel");
      await expect(panel).toBeVisible();
      await panel.screenshot({
        path: join(framesDirectory, frameFileName(frameIndex)),
      });
      wroteFrameCount += 1;
    }

    if (!interactionPerformed) {
      throw new Error(
        `Scene action did not run at step ${plan.interactionStep}`,
      );
    }
    if (wroteFrameCount !== CAPTURE_PROFILE.frameCount) {
      throw new Error(
        `Expected ${CAPTURE_PROFILE.frameCount} screenshots, wrote ${wroteFrameCount}`,
      );
    }

    const outputFrameNames = (await readdir(framesDirectory))
      .filter((entry) => entry.endsWith(".png"))
      .sort();
    const expectedFrameNames = Array.from(
      { length: CAPTURE_PROFILE.frameCount },
      (_, frameIndex) => frameFileName(frameIndex),
    );
    if (outputFrameNames.length !== CAPTURE_PROFILE.frameCount) {
      throw new Error(
        `Expected ${CAPTURE_PROFILE.frameCount} PNG frames, found ${outputFrameNames.length}`,
      );
    }
    expect(outputFrameNames).toEqual(expectedFrameNames);
  });
}

export async function withDemoMediaCaptureMode<T>(
  page: Page,
  callback: () => Promise<T>,
): Promise<T> {
  await setDemoMediaCaptureAttribute(page, true);
  try {
    return await callback();
  } finally {
    await setDemoMediaCaptureAttribute(page, false);
  }
}

async function requireWireframeRenderMode(page: Page): Promise<void> {
  const maybeRenderMode = await page
    .locator("main")
    .getAttribute("data-render-mode");
  if (maybeRenderMode !== "wireframe") {
    throw new Error(
      `Demo media capture requires wireframe rendering, got ${String(maybeRenderMode)}`,
    );
  }
}

async function setDemoMediaCaptureAttribute(
  page: Page,
  enabled: boolean,
): Promise<void> {
  await page.evaluate(
    ({ attributeName, enabledCaptureMode }) => {
      if (enabledCaptureMode) {
        document.documentElement.setAttribute(attributeName, "true");
        return;
      }
      document.documentElement.removeAttribute(attributeName);
    },
    {
      attributeName: DEMO_MEDIA_CAPTURE_ATTRIBUTE,
      enabledCaptureMode: enabled,
    },
  );
}

async function advanceSyntheticClock(
  page: Page,
  timestampMilliseconds: number,
): Promise<number> {
  return page.evaluate(
    ({ clockKey, nextTimestampMilliseconds }) => {
      const globalWindow = window as typeof window &
        Record<string, SyntheticAnimationController | undefined>;
      const maybeController = globalWindow[clockKey];
      if (maybeController === undefined) {
        throw new Error("Synthetic animation clock is unavailable");
      }
      return maybeController.advance(nextTimestampMilliseconds);
    },
    {
      clockKey: SYNTHETIC_CLOCK_KEY,
      nextTimestampMilliseconds: timestampMilliseconds,
    },
  );
}

async function pendingAnimationCallbackCount(page: Page): Promise<number> {
  return page.evaluate((clockKey) => {
    const globalWindow = window as typeof window &
      Record<string, SyntheticAnimationController | undefined>;
    const maybeController = globalWindow[clockKey];
    if (maybeController === undefined) {
      throw new Error("Synthetic animation clock is unavailable");
    }
    return maybeController.pendingCount();
  }, SYNTHETIC_CLOCK_KEY);
}

async function numericAttribute(
  locator: Locator,
  name: string,
): Promise<number> {
  const maybeValue = await locator.getAttribute(name);
  if (maybeValue === null || !/^(0|[1-9]\d*)$/.test(maybeValue)) {
    throw new Error(`Invalid numeric ${name}: ${String(maybeValue)}`);
  }
  return Number(maybeValue);
}

async function assertPlayingScene(
  page: Page,
  expectedSceneId: SceneCapturePlan["id"],
): Promise<SceneSnapshot> {
  const sceneSnapshot = await readSceneSnapshot(page);
  if (sceneSnapshot.sceneId !== expectedSceneId) {
    throw new Error(
      `Expected scene ${expectedSceneId}, got ${String(sceneSnapshot.sceneId)}`,
    );
  }
  if (sceneSnapshot.playback !== "playing") {
    throw new Error(
      `Expected playback playing, got ${String(sceneSnapshot.playback)}`,
    );
  }
  return sceneSnapshot;
}

async function readSceneSnapshot(page: Page): Promise<SceneSnapshot> {
  const main = page.locator("main");
  const sceneId = await main.getAttribute("data-scene");
  const playback = await main.getAttribute("data-playback");

  return {
    sceneId,
    playback,
    stepIndex: await numericAttribute(main, "data-step-index"),
    pointerAccepted: await numericAttribute(main, "data-pointer-accepted"),
  };
}

async function performSceneAction(
  page: Page,
  plan: SceneCapturePlan,
): Promise<void> {
  const beforeAction = await assertPlayingScene(page, plan.id);
  if (plan.action.kind === "click") {
    await clickCanvas(page, plan.action.point);
  } else {
    await dragCanvas(page, plan.action.start, plan.action.end);
  }

  await expect
    .poll(async () => (await readSceneSnapshot(page)).pointerAccepted)
    .toBeGreaterThan(beforeAction.pointerAccepted);
}

async function clickCanvas(page: Page, point: PointRatio): Promise<void> {
  const resolvedPoint = await resolveCanvasPoint(page, point);
  await page.mouse.click(resolvedPoint.x, resolvedPoint.y);
}

async function dragCanvas(
  page: Page,
  start: PointRatio,
  end: PointRatio,
): Promise<void> {
  const startPoint = await resolveCanvasPoint(page, start);
  const endPoint = await resolveCanvasPoint(page, end);

  await page.mouse.move(startPoint.x, startPoint.y);
  await page.mouse.down();
  await page.mouse.move(endPoint.x, endPoint.y, { steps: 8 });
  await page.mouse.up();
}

async function resolveCanvasPoint(
  page: Page,
  point: PointRatio,
): Promise<{ readonly x: number; readonly y: number }> {
  const canvas = page.locator("canvas.scene-canvas");
  await canvas.scrollIntoViewIfNeeded();
  await expect(canvas).toBeVisible();

  const maybeBox = await canvas.boundingBox();
  if (
    maybeBox === null ||
    maybeBox.width <= 0 ||
    maybeBox.height <= 0
  ) {
    throw new Error("Canvas bounding box is missing or empty");
  }

  return {
    x: maybeBox.x + maybeBox.width * point.x,
    y: maybeBox.y + maybeBox.height * point.y,
  };
}

function assertCaptureViewport(page: Page): void {
  const viewport = page.viewportSize();
  if (
    viewport?.width === CAPTURE_PROFILE.viewport.width &&
    viewport.height === CAPTURE_PROFILE.viewport.height
  ) {
    return;
  }

  throw new Error(
    `Expected Playwright viewport 1280x960 before capture; configure test.use({ viewport: CAPTURE_PROFILE.viewport })`,
  );
}

async function assertCaptureDeviceScaleFactor(page: Page): Promise<void> {
  const deviceScaleFactor = await page.evaluate(() => window.devicePixelRatio);
  if (deviceScaleFactor === CAPTURE_PROFILE.deviceScaleFactor) {
    return;
  }

  throw new Error(
    `Expected Playwright deviceScaleFactor 1 before capture; configure test.use({ deviceScaleFactor: CAPTURE_PROFILE.deviceScaleFactor })`,
  );
}

function assertFixedTimestampProgression(
  fixedDeltaMilliseconds: number,
  requiredStepCount: number,
) {
  if (
    supportsFixedTimestampProgression(
      fixedDeltaMilliseconds,
      requiredStepCount,
    )
  ) {
    return;
  }

  throw new Error(
    `Fixed timestamp delta ${fixedDeltaMilliseconds} does not advance exactly one step across ${requiredStepCount} callbacks`,
  );
}

function supportsFixedTimestampProgression(
  fixedDeltaMilliseconds: number,
  stepCount: number,
): boolean {
  let previousTimestampMilliseconds = 0;
  let remainderSeconds = 0;

  for (let callbackCount = 1; callbackCount <= stepCount; callbackCount += 1) {
    const currentTimestampMilliseconds =
      callbackCount * fixedDeltaMilliseconds;
    const stepTime = accumulateStepTime(
      remainderSeconds,
      (currentTimestampMilliseconds - previousTimestampMilliseconds) / 1_000,
    );
    if (stepTime.stepCount !== 1) {
      return false;
    }
    remainderSeconds = stepTime.remainderSeconds;
    previousTimestampMilliseconds = currentTimestampMilliseconds;
  }

  return true;
}

function nextUp(value: number): number {
  if (Number.isNaN(value) || value === Number.POSITIVE_INFINITY) {
    return value;
  }
  if (value === 0) {
    return Number.MIN_VALUE;
  }

  const buffer = new ArrayBuffer(8);
  const view = new DataView(buffer);
  view.setFloat64(0, value, false);
  const bits = view.getBigUint64(0, false);
  view.setBigUint64(0, value > 0 ? bits + 1n : bits - 1n, false);
  return view.getFloat64(0, false);
}
