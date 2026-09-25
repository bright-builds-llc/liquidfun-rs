import { describe, expect, it } from "vitest";

import {
  DAM_BREAK_GRAVITY_DEFAULT,
  DAM_BREAK_GRAVITY_MAX,
  DAM_BREAK_GRAVITY_MIN,
  maybeSceneById,
  type SceneControl,
} from "../src/catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  initialRangeValue,
  maybeParseRangeControlValue,
  type RangeControl,
} from "../src/components/range-control";
import {
  CONSTRUCTION_RESET_HINT,
  constructionHintVisible,
  initialPresetValue,
} from "../src/components/scene-controls";

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
    const recreatingRange: SceneControl = {
      id: "gravity",
      label: "Gravity",
      kind: "range",
      recreates: true,
      min: DAM_BREAK_GRAVITY_MIN,
      max: DAM_BREAK_GRAVITY_MAX,
      step: 1,
      defaultValue: DAM_BREAK_GRAVITY_DEFAULT,
      unit: "m/s²",
    };

    // Act
    const recreatingVisible = constructionHintVisible(recreatingPreset);
    const runtimeVisible = constructionHintVisible(runtimePreset);
    const actionVisible = constructionHintVisible(action);
    const rangeVisible = constructionHintVisible(recreatingRange);

    // Assert
    expect(recreatingVisible).toBe(true);
    expect(runtimeVisible).toBe(false);
    expect(actionVisible).toBe(false);
    expect(rangeVisible).toBe(true);
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

function maybeGravityControl(): RangeControl | undefined {
  const maybeControl = maybeSceneById("dam-break")?.controls.find(
    (control) => control.id === "gravity",
  );
  if (maybeControl === undefined || maybeControl.kind !== "range") {
    return undefined;
  }

  return maybeControl;
}

describe("Dam Break gravity slider", () => {
  it("starts at normal gravity when the bag is empty", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialRangeValue(maybeControl, {});

    // Assert
    expect(value).toBe(String(DAM_BREAK_GRAVITY_DEFAULT));
  });

  it("keeps an applied magnitude inside the slider", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialRangeValue(maybeControl, { gravity: "80" });

    // Assert
    expect(value).toBe("80");
    expect(value).toBe(String(DAM_BREAK_GRAVITY_MAX));
  });

  it("falls back to normal gravity for a retired preset name", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const value = initialRangeValue(maybeControl, { gravity: "high" });

    // Assert
    expect(value).toBe("10");
  });

  it("accepts the former low and five-times-high ends", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const low = maybeParseRangeControlValue(
      maybeControl,
      String(DAM_BREAK_GRAVITY_MIN),
    );
    const cap = maybeParseRangeControlValue(maybeControl, "80");
    const pastCap = maybeParseRangeControlValue(maybeControl, "81");
    const named = maybeParseRangeControlValue(maybeControl, "high");

    // Assert
    expect(low).toBe("6");
    expect(cap).toBe("80");
    expect(pastCap).toBeUndefined();
    expect(named).toBeUndefined();
  });

  it("reads the magnitude in meters per second squared", () => {
    // Arrange
    const magnitude = "16";

    // Act
    const readout = formatRangeReadout(magnitude, "m/s²");
    const spoken = formatRangeValueText(magnitude, "m/s²");

    // Assert
    expect(readout).toBe("16 m/s²");
    expect(spoken).toBe("16 meters per second squared");
  });
});
