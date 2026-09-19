import { describe, expect, it } from "vitest";

import {
  SCENE_IDS,
  SCENES,
  isReadySceneId,
  maybeSceneById,
  type SceneControl,
  type SceneId,
  type SceneRecord,
} from "../src/catalog/scenes";

const UI_SPEC_DESCRIPTIONS: Readonly<Record<SceneId, string>> = {
  "dam-break": "Release a block of water into a basin and drop one obstacle.",
  fountain: "Aim a bounded stream into a bowl until particle count plateaus.",
  "float-or-sink":
    "Drop cork, wood, or stone into a pool and watch native body response.",
  "color-mixer":
    "Stir two colored groups and watch contact-driven particle-color mixing.",
  "jelly-drop": "Drop an elastic particle shape onto obstacles, then poke it.",
  "water-wheel":
    "Vary a jet that turns a pinned paddle wheel through native coupling.",
};

const UI_SPEC_HINTS: Readonly<Record<SceneId, string>> = {
  "dam-break":
    "Drag the obstacle to a new place in the basin. Labeled controls also work from the keyboard.",
  fountain:
    "Drag on the canvas to aim the stream. Labeled controls also work from the keyboard.",
  "float-or-sink":
    "Click or tap the canvas to drop the selected body at that horizontal position. Labeled controls also work from the keyboard.",
  "color-mixer":
    "Drag on the canvas to stir the colored groups. Labeled controls also work from the keyboard.",
  "jelly-drop":
    "Click or tap the canvas to poke the jelly at that location. Labeled controls also work from the keyboard.",
  "water-wheel":
    "Drag on the canvas to aim the jet. Labeled controls also work from the keyboard.",
};

const KEYBOARD_REMINDER = "Labeled controls also work from the keyboard.";
const FORBIDDEN_HINT_PHRASES = [
  "Press Space to pause",
  "Live frame from this repository's Rust engine",
] as const;

const LOCKED_ACTION_LABELS = [
  "Drop obstacle",
  "Aim angle",
  "Drop body",
  "Stir speed",
  "Poke jelly",
  "Jet strength",
] as const;

const RECREATING_CONTROL_IDS = [
  "water-amount",
  "gravity",
  "mix-strength",
  "shape",
  "softness",
] as const;

const SHOWCASE_HREF = "https://google.github.io/liquidfun/";
const PARTICLE_GUIDE_HREF =
  "https://google.github.io/liquidfun/Programmers-Guide/html/md__chapter11__particles.html";
const FAUCET_HREF =
  "https://github.com/google/liquidfun/blob/7f20402173fd143a3988c921bc384459c6a858f2/liquidfun/Box2D/Testbed/Tests/Faucet.h";

function controlLabels(controls: readonly SceneControl[]): string[] {
  return controls.map((control) => control.label);
}

function optionLabels(control: SceneControl): string[] {
  if (control.kind !== "preset") {
    return [];
  }

  return control.values.map((value) => value.label);
}

function recreatingControlIds(scenes: readonly SceneRecord[]): string[] {
  return scenes.flatMap((scene) =>
    scene.controls
      .filter((control) => control.recreates)
      .map((control) => control.id),
  );
}

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

  it("marks every approved scene ready", () => {
    // Arrange
    const readyScenes = SCENES.filter((scene) => scene.ready);

    // Act
    const readyIds = readyScenes.map((scene) => scene.id);
    const notReadyIds = SCENES.filter((scene) => !scene.ready).map(
      (scene) => scene.id,
    );

    // Assert
    expect(readyIds).toEqual([...SCENE_IDS]);
    expect(notReadyIds).toEqual([]);
  });

  it("keeps static preview ids and locked descriptions after all scenes are ready", () => {
    // Arrange
    const readyCount = SCENES.filter((scene) => scene.ready).length;

    // Act
    const previewIds = SCENES.map((scene) => scene.previewId);
    const descriptions = SCENES.map((scene) => scene.description);

    // Assert
    expect(readyCount).toBe(6);
    expect(previewIds).toEqual([...SCENE_IDS]);
    expect(descriptions).toEqual(
      SCENE_IDS.map((id) => UI_SPEC_DESCRIPTIONS[id]),
    );
  });

  it("uses the locked UI-SPEC description for each scene", () => {
    // Arrange
    const expected = SCENE_IDS.map((id) => UI_SPEC_DESCRIPTIONS[id]);

    // Act
    const descriptions = SCENES.map((scene) => scene.description);

    // Assert
    expect(descriptions).toEqual(expected);
  });

  it("uses the locked UI-SPEC interactionHint for each scene", () => {
    // Arrange
    const expected = SCENE_IDS.map((id) => UI_SPEC_HINTS[id]);

    // Act
    const hints = SCENES.map((scene) => scene.interactionHint);

    // Assert
    expect(hints).toEqual(expected);
  });

  it("keeps the keyboard reminder and omits Space-to-pause or Canvas 2D leftover copy", () => {
    // Arrange
    const hints = SCENES.map((scene) => scene.interactionHint);

    // Act
    const missingReminder = hints.filter(
      (hint) => !hint.includes(KEYBOARD_REMINDER),
    );
    const forbiddenHits = hints.filter((hint) =>
      FORBIDDEN_HINT_PHRASES.some((phrase) => hint.includes(phrase)),
    );

    // Assert
    expect(missingReminder).toEqual([]);
    expect(forbiddenHits).toEqual([]);
  });

  it("keeps the locked labeled control path for pointer scenes", () => {
    // Arrange
    const labels = SCENES.flatMap((scene) => controlLabels(scene.controls));

    // Act
    const missingLabels = LOCKED_ACTION_LABELS.filter(
      (label) => !labels.includes(label),
    );

    // Assert
    expect(missingLabels).toEqual([]);
  });

  it("matches UI-SPEC control labels, option labels, and recreates flags", () => {
    // Arrange
    const damBreak = maybeSceneById("dam-break");
    const fountain = maybeSceneById("fountain");
    const floatOrSink = maybeSceneById("float-or-sink");
    const colorMixer = maybeSceneById("color-mixer");
    const jellyDrop = maybeSceneById("jelly-drop");
    const waterWheel = maybeSceneById("water-wheel");

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
    expect(optionLabels(damBreak?.controls[1] as SceneControl)).toEqual([
      "Low",
      "Normal",
      "High",
    ]);
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
    ]);
    expect(optionLabels(floatOrSink?.controls[0] as SceneControl)).toEqual([
      "Cork",
      "Wood",
      "Stone",
    ]);
    expect(controlLabels(colorMixer?.controls ?? [])).toEqual([
      "Mix strength",
      "Stir speed",
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
    expect(recreatingIds).toEqual([...RECREATING_CONTROL_IDS]);
  });

  it("uses each scene id as its preview id", () => {
    // Arrange
    const expected = [...SCENE_IDS];

    // Act
    const previewIds = SCENES.map((scene) => scene.previewId);

    // Assert
    expect(previewIds).toEqual(expected);
  });

  it("credits this repo's scene modules and host-locked inspiration", () => {
    // Arrange
    const expectedPaths: Readonly<Record<SceneId, string>> = {
      "dam-break": "crates/liquidfun-wasm/src/scene/dam_break.rs",
      fountain: "crates/liquidfun-wasm/src/scene/fountain.rs",
      "float-or-sink": "crates/liquidfun-wasm/src/scene/float_or_sink.rs",
      "color-mixer": "crates/liquidfun-wasm/src/scene/color_mixer.rs",
      "jelly-drop": "crates/liquidfun-wasm/src/scene/jelly_drop.rs",
      "water-wheel": "crates/liquidfun-wasm/src/scene/water_wheel.rs",
    };

    // Act
    const implementationPaths = SCENES.map(
      (scene) => scene.credits.implementationPath,
    );
    const inspirationById = Object.fromEntries(
      SCENES.map((scene) => [
        scene.id,
        scene.credits.inspiration.map((item) => item.href),
      ]),
    );

    // Assert
    expect(implementationPaths).toEqual(
      SCENE_IDS.map((id) => expectedPaths[id]),
    );
    expect(
      implementationPaths.some((path) => path.includes("google/liquidfun")),
    ).toBe(false);
    expect(inspirationById["dam-break"]).toEqual([SHOWCASE_HREF]);
    expect(inspirationById.fountain).toEqual([FAUCET_HREF, SHOWCASE_HREF]);
    expect(inspirationById["float-or-sink"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["color-mixer"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["jelly-drop"]).toEqual([
      PARTICLE_GUIDE_HREF,
      SHOWCASE_HREF,
    ]);
    expect(inspirationById["water-wheel"]).toEqual([SHOWCASE_HREF]);
  });
});

describe("maybeSceneById", () => {
  it("returns the Dam Break record for the locked id", () => {
    // Arrange
    const id = "dam-break";

    // Act
    const maybeScene = maybeSceneById(id);

    // Assert
    expect(maybeScene).toMatchObject({
      id: "dam-break",
      title: "Dam Break",
      ready: true,
      description: UI_SPEC_DESCRIPTIONS["dam-break"],
      previewId: "dam-break",
    });
    expect(maybeScene?.controls).toHaveLength(4);
    expect(maybeScene?.credits.implementationPath).toBe(
      "crates/liquidfun-wasm/src/scene/dam_break.rs",
    );
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
  it("is true for all six approved ids", () => {
    // Arrange
    const ids = SCENE_IDS;

    // Act
    const readyFlags = ids.map((id) => isReadySceneId(id));

    // Assert
    expect(readyFlags).toEqual([true, true, true, true, true, true]);
  });
});
