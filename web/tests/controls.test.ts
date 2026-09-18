import { describe, expect, it } from "vitest";

import {
  APPLY_SETTING_LABEL,
  CONSTRUCTION_RESET_HINT,
  constructionHintVisible,
} from "../src/components/SceneControls";
import type { SceneControl } from "../src/catalog/scenes";

describe("constructionHintVisible", () => {
  it("is true only when the control recreates the scene", () => {
    // Arrange
    const recreatingPreset: SceneControl = {
      id: "water-amount",
      label: "Water amount",
      kind: "preset",
      recreates: true,
      values: [{ id: "medium", label: "Medium" }],
    };
    const runtimePreset: SceneControl = {
      id: "emission-rate",
      label: "Emission rate",
      kind: "preset",
      recreates: false,
      values: [{ id: "medium", label: "Medium" }],
    };
    const action: SceneControl = {
      id: "drop-body",
      label: "Drop body",
      kind: "action",
      recreates: false,
    };

    // Act
    const recreatingVisible = constructionHintVisible(recreatingPreset);
    const runtimeVisible = constructionHintVisible(runtimePreset);
    const actionVisible = constructionHintVisible(action);

    // Assert
    expect(recreatingVisible).toBe(true);
    expect(runtimeVisible).toBe(false);
    expect(actionVisible).toBe(false);
  });
});

describe("SceneControls copy", () => {
  it("locks the construction apply CTA and reset sentence", () => {
    // Arrange
    const applyLabel = APPLY_SETTING_LABEL;
    const resetHint = CONSTRUCTION_RESET_HINT;

    // Act
    const isNounBearing = applyLabel !== "Apply";

    // Assert
    expect(applyLabel).toBe("Apply setting");
    expect(isNounBearing).toBe(true);
    expect(resetHint).toBe(
      "Changing this setting recreates the scene from its documented initial state.",
    );
  });
});
