import { describe, expect, it } from "vitest";

import { WEIGHT_RATIO_CONTROL } from "../src/catalog/rayleigh-taylor-ratio";
import { maybeSceneById } from "../src/catalog/scenes";
import {
  DEFAULT_PRESET_VALUES,
  sceneControlsForSurface,
} from "../src/components/scene-controls";

/** Claims the Rayleigh–Taylor copy must not make about its extra weight (UI-D90). */
const RT_FORBIDDEN_CLAIMS = [
  "density",
  "denser",
  "atwood",
  "growth rate",
  "measured",
  "simulated",
  "real fluid",
  "two fluids",
] as const;

describe("Rayleigh–Taylor Instability", () => {
  it("keeps the Rayleigh–Taylor caption honest about its extra weight", () => {
    // Arrange
    const maybeScene = maybeSceneById("rayleigh-taylor");
    const description = maybeScene?.description ?? "";
    const copy = `${description} ${maybeScene?.interactionHint ?? ""}`.toLowerCase();

    // Act
    const claims = RT_FORBIDDEN_CLAIMS.filter((claim) => copy.includes(claim));

    // Assert
    expect(claims).toEqual([]);
    expect(description).toContain("given extra weight");
    expect(description).toContain("same particle liquid");
    expect(maybeScene?.title).toBe("Rayleigh–Taylor Instability");
  });

  it("offers Weight ratio as a recreating panel preset", () => {
    // Arrange
    const controls = maybeSceneById("rayleigh-taylor")?.controls ?? [];
    const values =
      WEIGHT_RATIO_CONTROL.kind === "preset" ? WEIGHT_RATIO_CONTROL.values : [];

    // Act
    const panelIds = sceneControlsForSurface(controls, "panel").map((control) => control.id);
    const hudIds = sceneControlsForSurface(controls, "hud").map((control) => control.id);

    // Assert
    expect(WEIGHT_RATIO_CONTROL.id).toBe("weight-ratio");
    expect(WEIGHT_RATIO_CONTROL.label).toBe("Weight ratio");
    expect(WEIGHT_RATIO_CONTROL.kind).toBe("preset");
    expect(WEIGHT_RATIO_CONTROL.recreates).toBe(true);
    expect(values.map((value) => value.id)).toEqual(["1.2", "1.5", "2", "3"]);
    expect(values.map((value) => value.label)).toEqual(["1.2×", "1.5×", "2×", "3×"]);
    expect(DEFAULT_PRESET_VALUES["weight-ratio"]).toBe("2");
    expect(panelIds).toEqual(["weight-ratio", "gravity"]);
    expect(hudIds).toEqual([]);
  });
});
