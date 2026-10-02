import { describe, expect, it, vi } from "vitest";

import { createTiltBinding } from "../src/input/tilt-binding";
import { createFrameClock } from "../src/player/frame-loop";
import type { SceneRuntime } from "../src/player/scene-runtime";
import { bindSurface } from "../src/player/scene-surface";

vi.mock("../src/player/frame-loop", async (importOriginal) => {
  const original = await importOriginal<typeof import("../src/player/frame-loop")>();
  return { ...original, presentOwnedFrame: vi.fn() };
});

function waveTankRuntime() {
  const live = { speed: "0.4", amplitude: "0.168" };
  const applyControl = vi.fn((name: string, value: string) => {
    if (["platform-width", "platform-slant", "gravity"].includes(name)) {
      live.speed = "0.4";
      live.amplitude = "0.168";
      return true;
    }
    if (name === "platform-speed") live.speed = value;
    if (name === "platform-amplitude") live.amplitude = value;
    return false;
  });
  const session = {
    constructionValues: {},
    maybeContext: {},
    maybeSession: { applyControl },
    clock: createFrameClock(),
    tiltBinding: createTiltBinding(),
    route: () => ({ kind: "scene", id: "wave-tank" }),
    gravitySliderMagnitude: () => 10,
    setView: vi.fn(),
    setTiltDebug: vi.fn(),
    setMaybeDebugFrame: vi.fn(),
    setStepsThisFrame: vi.fn(),
    setFpsTicks: vi.fn(),
    frameDeps: vi.fn(),
    failScene: vi.fn(),
  } as unknown as SceneRuntime;
  bindSurface(session);
  return { session, live, applyControl };
}

describe("wave tank live control persistence", () => {
  it.each([
    ["platform-width", "0.64"],
    ["platform-slant", "20"],
    ["gravity", "5"],
  ])("restores speed and amplitude after changing %s", (name, value) => {
    // Arrange
    const { session, live, applyControl } = waveTankRuntime();
    session.applySceneControl("platform-speed", "2.0");
    session.applySceneControl("platform-amplitude", "0.080");
    applyControl.mockClear();

    // Act
    session.applySceneControl(name, value);

    // Assert
    expect(live).toEqual({ speed: "2.0", amplitude: "0.080" });
    expect(applyControl.mock.calls.filter(([control]) => control === name)).toHaveLength(1);
    expect(session.failScene).not.toHaveBeenCalled();
  });
});
