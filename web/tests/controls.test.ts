import { describe, expect, it } from "vitest";

import {
  CONSTRUCTION_RESET_HINT,
  constructionHintVisible,
  initialPresetValue,
} from "../src/components/scene-controls";
import { maybeSceneById, type SceneControl } from "../src/catalog/scenes";

function maybePresetControl(
  sceneId: string,
  controlId: string,
): Extract<SceneControl, { kind: "preset" }> | undefined {
  const maybeScene = maybeSceneById(sceneId);
  const maybeControl = maybeScene?.controls.find(
    (control) => control.kind === "preset" && control.id === controlId,
  );
  if (maybeControl === undefined || maybeControl.kind !== "preset") {
    return undefined;
  }

  return maybeControl;
}

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
  it("locks the construction reset sentence", () => {
    // Arrange / Act
    const resetHint = CONSTRUCTION_RESET_HINT;

    // Assert
    expect(resetHint).toBe(
      "Changing this setting recreates the scene from its documented initial state.",
    );
  });
});

describe("initialPresetValue", () => {
  it("returns medium for Fountain emission-rate with an empty bag", () => {
    // Arrange
    const maybeControl = maybePresetControl("fountain", "emission-rate");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, {});

    // Assert
    expect(value).toBe("medium");
  });

  it("returns wood for Float or Sink body when maybeValues is undefined", () => {
    // Arrange
    const maybeControl = maybePresetControl("float-or-sink", "body");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, undefined);

    // Assert
    expect(value).toBe("wood");
  });

  it("returns slow for Color Mixer stir-speed with an empty bag", () => {
    // Arrange
    const maybeControl = maybePresetControl("color-mixer", "stir-speed");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, {});

    // Assert
    expect(value).toBe("slow");
  });

  it("returns medium for Water Wheel jet-strength with an empty bag", () => {
    // Arrange
    const maybeControl = maybePresetControl("water-wheel", "jet-strength");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, {});

    // Assert
    expect(value).toBe("medium");
  });

  it("returns medium for Dam Break water-amount with an empty bag", () => {
    // Arrange
    const maybeControl = maybePresetControl("dam-break", "water-amount");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, {});

    // Assert
    expect(value).toBe("medium");
  });

  it("returns strong for Color Mixer mix-strength with an empty bag", () => {
    // Arrange
    const maybeControl = maybePresetControl("color-mixer", "mix-strength");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, {});

    // Assert
    expect(value).toBe("strong");
  });

  it("returns the applied Dam Break water-amount when the bag holds large", () => {
    // Arrange
    const maybeControl = maybePresetControl("dam-break", "water-amount");
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialPresetValue(maybeControl, { "water-amount": "large" });

    // Assert
    expect(value).toBe("large");
  });
});
