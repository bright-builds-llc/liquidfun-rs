import { describe, expect, it } from "vitest";

import { SCENE_IDS, SCENES } from "../src/catalog/scenes";
import { sceneNavigationItems } from "../src/player/navigation";

describe("sceneNavigationItems", () => {
  it("returns all canonical routes in catalog order", () => {
    // Act
    const items = sceneNavigationItems(undefined);

    // Assert
    expect(items.map((item) => item.id)).toEqual([...SCENE_IDS]);
    expect(items.map((item) => item.href)).toEqual(
      SCENE_IDS.map((id) => `#/scene/${id}`),
    );
  });

  it("maps catalog titles for each navigation item", () => {
    // Act
    const items = sceneNavigationItems(undefined);

    // Assert
    expect(items.map((item) => item.title)).toEqual(
      SCENES.map((scene) => scene.title),
    );
  });

  it("maps catalog descriptions for each navigation item", () => {
    // Act
    const items = sceneNavigationItems(undefined);

    // Assert
    expect(items.map((item) => item.description)).toEqual(
      SCENES.map((scene) => scene.description),
    );
  });

  it("marks only the selected scene current", () => {
    // Act
    const items = sceneNavigationItems("color-mixer");

    // Assert
    expect(items.filter((item) => item.isCurrent).map((item) => item.id)).toEqual([
      "color-mixer",
    ]);
  });
});
