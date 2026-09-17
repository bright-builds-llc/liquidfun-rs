import { describe, expect, it } from "vitest";

import {
  SCENE_IDS,
  SCENES,
  isReadySceneId,
  maybeSceneById,
} from "../src/catalog/scenes";

describe("SCENES", () => {
  it("lists six locked scenes in the approved order", () => {
    // Arrange
    const expectedIds = [
      "dam-break",
      "fountain",
      "float-or-sink",
      "color-mixer",
      "jelly-drop",
      "water-wheel",
    ] as const;

    // Act
    const ids = SCENES.map((scene) => scene.id);

    // Assert
    expect(SCENES).toHaveLength(6);
    expect(ids).toEqual([...SCENE_IDS]);
    expect(ids).toEqual([...expectedIds]);
  });

  it("uses the locked titles for each scene", () => {
    // Arrange
    const expectedTitles = [
      "Dam Break",
      "Fountain",
      "Float or Sink",
      "Color Mixer",
      "Jelly Drop",
      "Water Wheel",
    ];

    // Act
    const titles = SCENES.map((scene) => scene.title);

    // Assert
    expect(titles).toEqual(expectedTitles);
  });

  it("marks only Dam Break as ready", () => {
    // Arrange
    const readyScenes = SCENES.filter((scene) => scene.ready);

    // Act
    const readyIds = readyScenes.map((scene) => scene.id);
    const notReadyIds = SCENES.filter((scene) => !scene.ready).map(
      (scene) => scene.id,
    );

    // Assert
    expect(readyIds).toEqual(["dam-break"]);
    expect(notReadyIds).toEqual([
      "fountain",
      "float-or-sink",
      "color-mixer",
      "jelly-drop",
      "water-wheel",
    ]);
  });
});

describe("maybeSceneById", () => {
  it("returns the Dam Break record for the locked id", () => {
    // Arrange
    const id = "dam-break";

    // Act
    const maybeScene = maybeSceneById(id);

    // Assert
    expect(maybeScene).toEqual({
      id: "dam-break",
      title: "Dam Break",
      ready: true,
    });
  });

  it("returns undefined for an unknown id", () => {
    // Arrange
    const id = "not-a-scene";

    // Act
    const maybeScene = maybeSceneById(id);

    // Assert
    expect(maybeScene).toBeUndefined();
  });
});

describe("isReadySceneId", () => {
  it("is true only for Dam Break", () => {
    // Arrange
    const readyId = "dam-break";

    // Act
    const damBreakReady = isReadySceneId(readyId);
    const laterSceneReady = SCENE_IDS.filter((id) => id !== readyId).map(
      (id) => isReadySceneId(id),
    );

    // Assert
    expect(damBreakReady).toBe(true);
    expect(laterSceneReady).toEqual([false, false, false, false, false]);
  });
});
