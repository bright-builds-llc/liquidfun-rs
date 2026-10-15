import { describe, expect, it } from "vitest";

import { maybeSceneById } from "../src/catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  maybeStepRangeValue,
} from "../src/components/range-control";
import { sceneControlsForSurface } from "../src/components/scene-controls";

describe("Fake Ferrofluid controls", () => {
  it("speaks magnet strength in percent", () => {
    // Arrange
    const value = "65";

    // Act
    const spoken = formatRangeValueText(value, "%");
    const readout = formatRangeReadout(value, "%");

    // Assert
    expect(spoken).toBe("65 percent");
    expect(readout).toBe("65 %");
  });

  it("keeps Magnet strength on the HUD and steps it by one percent", () => {
    // Arrange
    const maybeScene = maybeSceneById("fake-ferrofluid");
    expect(maybeScene).toBeDefined();
    if (maybeScene === undefined) {
      return;
    }
    const maybeStrength = maybeScene.controls.find(
      (control) => control.id === "magnet-strength",
    );
    expect(maybeStrength?.kind).toBe("range");
    if (maybeStrength === undefined || maybeStrength.kind !== "range") {
      return;
    }

    // Act
    const hud = sceneControlsForSurface(maybeScene.controls, "hud");
    const panel = sceneControlsForSurface(maybeScene.controls, "panel");
    const stronger = maybeStepRangeValue(maybeStrength, "65", 1);
    const belowMinimum = maybeStepRangeValue(maybeStrength, "0", -1);
    const aboveMaximum = maybeStepRangeValue(maybeStrength, "100", 1);

    // Assert
    expect(hud.map((control) => control.id)).toEqual(["magnet-strength"]);
    const panelIds = panel.map((control) => control.id);
    expect(panelIds).toEqual(["magnet", "gravity"]);
    expect(maybeStrength.widget).toBe("spinner");
    expect(stronger).toBe("66");
    expect(belowMinimum).toBe("0");
    expect(aboveMaximum).toBe("100");
  });
});
