import { describe, expect, it } from "vitest";

import { sceneClipSchedule } from "../scripts/readme-svg/schedule";

describe("sceneClipSchedule", () => {
  it("keeps SVG samples on the 20 Hz steps and WebP frames on the 60 Hz steps", () => {
    // Arrange
    let step = 0;
    const svgSteps: number[] = [];
    const webpSteps: number[] = [];

    // Act
    for (const event of sceneClipSchedule(10)) {
      if (event.kind === "advance") {
        step += 1;
      }
      if (event.kind === "svg-sample") {
        svgSteps.push(step);
      }
      if (event.kind === "webp-frame") {
        webpSteps.push(step);
      }
    }

    // Assert
    expect(svgSteps).toHaveLength(201);
    expect(svgSteps[0]).toBe(0);
    expect(svgSteps[1]).toBe(3);
    expect(svgSteps[20]).toBe(60);
    expect(svgSteps[200]).toBe(600);
    expect(webpSteps).toHaveLength(600);
    expect(webpSteps[0]).toBe(0);
    expect(webpSteps[60]).toBe(60);
    expect(webpSteps[599]).toBe(599);
  });

  it("applies a later cue before the steps that produce that SVG sample", () => {
    // Arrange
    const events = sceneClipSchedule(10);
    const cueAt = events.findIndex(
      (event) => event.kind === "cue" && event.sampleIndex === 20,
    );

    // Act
    const following = events.slice(cueAt, cueAt + 4);

    // Assert
    expect(following.map((event) => event.kind)).toEqual([
      "cue",
      "advance",
      "webp-frame",
      "advance",
    ]);
  });
});
