import { createRoot } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";

import { incrementGeneration } from "../src/player/puzzle-host";
import {
  createPuzzleController,
  leavePuzzle,
  releaseFailedPuzzle,
} from "../src/player/puzzle-lifecycle";
import { createPuzzleRuntime } from "../src/player/puzzle-runtime";
import type { PuzzleSound } from "../src/player/puzzle-sound";
import { createHost, stubPuzzleGlobals } from "./puzzle-host-fixture";

/** A recording PuzzleSound whose accessors return fixed values. */
function soundSpy(values = { enabled: true, available: true, failed: true }) {
  return {
    enabled: vi.fn(() => values.enabled),
    available: vi.fn(() => values.available),
    failed: vi.fn(() => values.failed),
    setEnabled: vi.fn(),
    present: vi.fn(),
    stopVoices: vi.fn(),
    pause: vi.fn(),
    attach: vi.fn(),
    leave: vi.fn(),
  } satisfies PuzzleSound;
}

function hostWithSound(sound: PuzzleSound) {
  stubPuzzleGlobals();
  return createHost({ maybeSound: sound });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("puzzle sound wiring", () => {
  it("stops voices on every generation bump", () => {
    // Arrange
    const sound = soundSpy();
    const { host, dispose } = hostWithSound(sound);

    // Act
    incrementGeneration(host);

    // Assert
    expect(sound.stopVoices).toHaveBeenCalledTimes(1);
    dispose();
  });

  it("leaves sound when the puzzle route is left", () => {
    // Arrange
    const sound = soundSpy();
    const { host, dispose } = hostWithSound(sound);

    // Act
    leavePuzzle(host);

    // Assert
    expect(sound.leave).toHaveBeenCalledTimes(1);
    dispose();
  });

  it("pauses sound when a failed puzzle is released", () => {
    // Arrange
    const sound = soundSpy();
    const { host, dispose } = hostWithSound(sound);

    // Act
    releaseFailedPuzzle(host);

    // Assert
    expect(sound.pause).toHaveBeenCalledTimes(1);
    dispose();
  });

  it("reads the sound accessors through the controller", () => {
    // Arrange
    const sound = soundSpy({ enabled: true, available: false, failed: true });
    const { host, dispose } = hostWithSound(sound);
    const controller = createPuzzleController(host);

    // Act
    const read = {
      enabled: controller.soundEnabled(),
      available: controller.soundAvailable(),
      failed: controller.soundFailed(),
    };

    // Assert
    expect(read).toEqual({ enabled: true, available: false, failed: true });
    dispose();
  });

  it("forwards setSoundEnabled to the sound", () => {
    // Arrange
    const sound = soundSpy();
    const { host, dispose } = hostWithSound(sound);

    // Act
    createPuzzleController(host).setSoundEnabled(true);

    // Assert
    expect(sound.setEnabled).toHaveBeenCalledWith(true);
    dispose();
  });

  it("defaults a runtime to unavailable sound", () => {
    // Act
    const runtime = createRoot((dispose) => {
      const created = createPuzzleRuntime();
      dispose();
      return created;
    });

    // Assert
    expect(runtime.sound.available()).toBe(false);
  });
});
