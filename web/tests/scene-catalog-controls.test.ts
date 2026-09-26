import { describe, expect, it } from "vitest";

import {
  GRAVITY_CONTROL,
  GRAVITY_SLIDER_FORMER_HIGH,
  GRAVITY_SLIDER_MAX,
} from "../src/catalog/gravity-slider";
import {
  SCENES,
  maybeSceneById,
  type SceneControl,
  type SceneRecord,
} from "../src/catalog/scenes";

const RECREATING_CONTROL_IDS = [
  "gravity",
  "water-amount",
  "gravity",
  "gravity",
  "gravity",
  "mix-strength",
  "gravity",
  "shape",
  "softness",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
  "gravity",
] as const;

function controlLabels(controls: readonly SceneControl[]): string[] {
  return controls.map((control) => control.label);
}

function optionLabels(control: SceneControl): string[] {
  if (control.kind !== "preset") {
    return [];
  }

  return control.values.map((value) => value.label);
}

function presetValueIds(control: SceneControl): string[] {
  if (control.kind !== "preset") {
    return [];
  }

  return control.values.map((value) => value.id);
}

function recreatingControlIds(scenes: readonly SceneRecord[]): string[] {
  return scenes.flatMap((scene) =>
    scene.controls
      .filter((control) => control.recreates)
      .map((control) => control.id),
  );
}

describe("scene catalog controls", () => {
  it("matches UI-SPEC control labels, option labels, and recreates flags", () => {
    // Arrange
    const damBreak = maybeSceneById("dam-break");
    const fountain = maybeSceneById("fountain");
    const floatOrSink = maybeSceneById("float-or-sink");
    const colorMixer = maybeSceneById("color-mixer");
    const jellyDrop = maybeSceneById("jelly-drop");
    const waterWheel = maybeSceneById("water-wheel");
    const soupStirrer = maybeSceneById("soup-stirrer");
    const impulse = maybeSceneById("impulse");
    const theoJansen = maybeSceneById("theo-jansen");

    // Act
    const recreatingIds = recreatingControlIds(SCENES);

    // Assert
    expect(damBreak).toBeDefined();
    expect(fountain).toBeDefined();
    expect(floatOrSink).toBeDefined();
    expect(colorMixer).toBeDefined();
    expect(jellyDrop).toBeDefined();
    expect(waterWheel).toBeDefined();
    expect(controlLabels(damBreak?.controls ?? [])).toEqual([
      "Water amount",
      "Gravity",
      "Drop obstacle",
      "Reset obstacle",
    ]);
    expect(optionLabels(damBreak?.controls[0] as SceneControl)).toEqual([
      "Small",
      "Medium",
      "Large",
    ]);
    expect(damBreak?.controls[1]).toEqual(GRAVITY_CONTROL);
    expect(GRAVITY_SLIDER_MAX).toBe(GRAVITY_SLIDER_FORMER_HIGH * 5);
    const gravityControls = SCENES.map((scene) =>
      scene.controls.filter((control) => control.id === "gravity"),
    );
    expect(gravityControls).toEqual(SCENES.map(() => [GRAVITY_CONTROL]));
    expect(damBreak?.controls[2]).toMatchObject({
      id: "drop-obstacle",
      kind: "action",
      recreates: false,
    });
    expect(damBreak?.controls[3]).toMatchObject({
      id: "reset-obstacle",
      kind: "action",
      recreates: false,
    });
    expect(controlLabels(fountain?.controls ?? [])).toEqual([
      "Emission rate",
      "Launch speed",
      "Aim angle",
      "Gravity",
    ]);
    expect(optionLabels(fountain?.controls[0] as SceneControl)).toEqual([
      "Off",
      "Low",
      "Medium",
      "High",
    ]);
    expect(optionLabels(fountain?.controls[1] as SceneControl)).toEqual([
      "Slow",
      "Medium",
      "Fast",
    ]);
    expect(optionLabels(fountain?.controls[2] as SceneControl)).toEqual([
      "Left",
      "Up",
      "Right",
    ]);
    expect(controlLabels(floatOrSink?.controls ?? [])).toEqual([
      "Body",
      "Drop body",
      "Gravity",
    ]);
    expect(optionLabels(floatOrSink?.controls[0] as SceneControl)).toEqual([
      "Cork",
      "Wood",
      "Stone",
    ]);
    expect(controlLabels(colorMixer?.controls ?? [])).toEqual([
      "Mix strength",
      "Stir speed",
      "Gravity",
    ]);
    expect(optionLabels(colorMixer?.controls[0] as SceneControl)).toEqual([
      "Off",
      "Gentle",
      "Strong",
    ]);
    expect(optionLabels(colorMixer?.controls[1] as SceneControl)).toEqual([
      "Off",
      "Slow",
      "Fast",
    ]);
    expect(controlLabels(jellyDrop?.controls ?? [])).toEqual([
      "Shape",
      "Softness",
      "Poke jelly",
      "Gravity",
    ]);
    expect(optionLabels(jellyDrop?.controls[0] as SceneControl)).toEqual([
      "Circle",
      "Square",
    ]);
    expect(optionLabels(jellyDrop?.controls[1] as SceneControl)).toEqual([
      "Soft",
      "Medium",
      "Firm",
    ]);
    expect(controlLabels(waterWheel?.controls ?? [])).toEqual([
      "Jet strength",
      "Emission",
      "Gravity",
    ]);
    expect(optionLabels(waterWheel?.controls[0] as SceneControl)).toEqual([
      "Weak",
      "Medium",
      "Strong",
    ]);
    expect(optionLabels(waterWheel?.controls[1] as SceneControl)).toEqual([
      "On",
      "Off",
    ]);
    expect(maybeSceneById("particles")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(maybeSceneById("liquid-timer")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(maybeSceneById("surface-tension")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(maybeSceneById("elastic-particles")?.controls).toEqual([
      GRAVITY_CONTROL,
    ]);
    expect(maybeSceneById("rigid-particles")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(maybeSceneById("soup")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(maybeSceneById("wave-machine")?.controls).toEqual([
      {
        id: "wave-speed",
        label: "Wave speed",
        kind: "range",
        recreates: false,
        surface: "hud",
        min: 0,
        max: 10,
        step: 0.1,
        defaultValue: 1,
        unit: "×",
        scale: "linear",
        ticks: [0, 5, 10],
      },
      {
        id: "wave-tilt",
        label: "Wave tilt",
        kind: "range",
        recreates: false,
        surface: "hud",
        min: 0,
        max: 30,
        step: 1,
        defaultValue: 9,
        unit: "°",
        scale: "linear",
        ticks: [0, 9, 30],
      },
      GRAVITY_CONTROL,
    ]);
    expect(maybeSceneById("liquid-tumbler")?.controls).toEqual([GRAVITY_CONTROL]);
    expect(soupStirrer?.controls).toHaveLength(2);
    expect(soupStirrer?.controls[0]).toMatchObject({
      id: "toggle-paddle-rail",
      kind: "action",
      label: "Toggle paddle rail",
      recreates: false,
    });
    expect(controlLabels(impulse?.controls ?? [])).toEqual(["Push", "Gravity"]);
    expect(impulse?.controls[0]).toMatchObject({
      id: "push-mode",
      kind: "preset",
      recreates: false,
    });
    expect(optionLabels(impulse?.controls[0] as SceneControl)).toEqual([
      "Force",
      "Impulse",
    ]);
    expect(presetValueIds(impulse?.controls[0] as SceneControl)).toEqual([
      "force",
      "impulse",
    ]);
    expect(controlLabels(theoJansen?.controls ?? [])).toEqual([
      "Motor direction",
      "Gravity",
    ]);
    expect(theoJansen?.controls[0]).toMatchObject({
      id: "motor-direction",
      kind: "preset",
      recreates: false,
    });
    expect(optionLabels(theoJansen?.controls[0] as SceneControl)).toEqual([
      "Forward",
      "Reverse",
    ]);
    expect(presetValueIds(theoJansen?.controls[0] as SceneControl)).toEqual([
      "forward",
      "reverse",
    ]);
    expect(recreatingIds).toEqual([...RECREATING_CONTROL_IDS]);
  });
});
