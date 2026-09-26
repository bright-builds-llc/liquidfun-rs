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
  maybeMagnitudeForSliderPosition,
  maybeParseRangeControlValue,
  rangeTickMarks,
  sliderPositionForMagnitude,
  LOG_SLIDER_POSITION_MAX,
  type RangeControl,
} from "../src/components/range-control";
import {
  CONSTRUCTION_RESET_HINT,
  constructionHintVisible,
  initialPresetValue,
  sceneControlsForSurface,
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
      scale: "logarithmic",
      ticks: [DAM_BREAK_GRAVITY_MIN, DAM_BREAK_GRAVITY_MAX],
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

  it("maps the thumb logarithmically between the ends", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }
    const linearNormal =
      ((DAM_BREAK_GRAVITY_DEFAULT - DAM_BREAK_GRAVITY_MIN) /
        (DAM_BREAK_GRAVITY_MAX - DAM_BREAK_GRAVITY_MIN)) *
      LOG_SLIDER_POSITION_MAX;

    // Act
    const normalPosition = sliderPositionForMagnitude(
      maybeControl,
      DAM_BREAK_GRAVITY_DEFAULT,
    );
    const low = maybeMagnitudeForSliderPosition(maybeControl, "0");
    const cap = maybeMagnitudeForSliderPosition(
      maybeControl,
      String(LOG_SLIDER_POSITION_MAX),
    );
    const roundTrip = maybeMagnitudeForSliderPosition(
      maybeControl,
      String(normalPosition),
    );

    // Assert
    expect(low).toBe("6");
    expect(cap).toBe("80");
    expect(roundTrip).toBe("10");
    expect(normalPosition).toBeGreaterThan(linearNormal);
    expect(normalPosition).toBeLessThan(LOG_SLIDER_POSITION_MAX / 2);
  });

  it("places tick marks by magnitude ratio", () => {
    // Arrange
    const maybeControl = maybeGravityControl();
    expect(maybeControl).toBeDefined();
    if (maybeControl === undefined) {
      return;
    }

    // Act
    const ticks = rangeTickMarks(maybeControl);
    const normal = ticks.find((tick) => tick.label === "10");
    const formerHigh = ticks.find((tick) => tick.label === "16");
    const cap = ticks.find((tick) => tick.label === "80");

    // Assert
    expect(ticks.map((tick) => tick.label)).toEqual(["6", "10", "16", "80"]);
    expect(normal?.ratio).toBeGreaterThan(0);
    expect(formerHigh?.ratio).toBeGreaterThan(normal?.ratio ?? 1);
    expect(cap?.ratio).toBe(1);
    expect(formerHigh?.ratio).toBeLessThan(0.5);
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

  it("reads a wave-speed multiplier as times the original speed", () => {
    // Arrange
    const magnitude = "1.0";

    // Act
    const readout = formatRangeReadout(magnitude, "×");
    const spoken = formatRangeValueText(magnitude, "×");

    // Assert
    expect(readout).toBe("1.0 ×");
    expect(spoken).toBe("1.0 times the original wave speed");
  });

  it("reads a wave-tilt magnitude as degrees", () => {
    // Arrange
    const magnitude = "9";

    // Act
    const readout = formatRangeReadout(magnitude, "°");
    const spoken = formatRangeValueText(magnitude, "°");

    // Assert
    expect(readout).toBe("9 °");
    expect(spoken).toBe("9 degrees");
  });
});

describe("sceneControlsForSurface", () => {
  it("keeps Wave speed on the HUD and out of the scene panel", () => {
    // Arrange
    const maybeScene = maybeSceneById("wave-machine");
    expect(maybeScene).toBeDefined();
    if (maybeScene === undefined) {
      return;
    }

    // Act
    const hud = sceneControlsForSurface(maybeScene.controls, "hud");
    const panel = sceneControlsForSurface(maybeScene.controls, "panel");

    // Assert
    expect(hud.map((control) => control.id)).toEqual([
      "wave-speed",
      "wave-tilt",
    ]);
    expect(panel).toEqual([]);
  });
});
