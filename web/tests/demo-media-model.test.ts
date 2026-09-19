import { describe, expect, it } from "vitest";

import { SCENE_IDS } from "../src/catalog/scenes";
import {
  CAPTURE_PROFILE,
  SCENE_CAPTURE_PLANS,
  canonicalManifestJson,
  frameFileName,
  type DemoMediaManifest,
} from "../scripts/demo-media/model";

describe("demo media capture model", () => {
  it("covers every scene once in canonical order", () => {
    // Arrange
    const expectedIds = [...SCENE_IDS];

    // Act
    const actualIds = SCENE_CAPTURE_PLANS.map((plan) => plan.id);

    // Assert
    expect(actualIds).toEqual(expectedIds);
    expect(new Set(actualIds).size).toBe(expectedIds.length);
  });

  it("defines an exact eight-second capture profile", () => {
    // Arrange
    const expectedFrameCount = 240;

    // Act
    const representedSteps =
      CAPTURE_PROFILE.frameCount * CAPTURE_PROFILE.stepsPerFrame;

    // Assert
    expect(CAPTURE_PROFILE.simulationHz).toBe(60);
    expect(CAPTURE_PROFILE.outputFps).toBe(30);
    expect(CAPTURE_PROFILE.durationSeconds).toBe(8);
    expect(CAPTURE_PROFILE.frameCount).toBe(expectedFrameCount);
    expect(representedSteps).toBe(480);
  });

  it("sorts manifest scenes and emits one trailing newline", () => {
    // Arrange
    const manifest = {
      schemaVersion: 1,
      captureProfile: { inputSha256: "a".repeat(64) },
      scenes: [
        { id: "water-wheel", files: [] },
        { id: "dam-break", files: [] },
      ],
    } as unknown as DemoMediaManifest;

    // Act
    const serialized = canonicalManifestJson(manifest);

    // Assert
    expect(serialized.endsWith("\n")).toBe(true);
    expect(serialized.indexOf("dam-break")).toBeLessThan(
      serialized.indexOf("water-wheel"),
    );
  });

  it("formats zero-padded frame names", () => {
    // Arrange
    const frameIndexes = [0, 9, 239];

    // Act
    const names = frameIndexes.map(frameFileName);

    // Assert
    expect(names).toEqual([
      "frame-0000.png",
      "frame-0009.png",
      "frame-0239.png",
    ]);
  });
});
