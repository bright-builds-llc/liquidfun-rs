import { describe, expect, it } from "vitest";

import {
  LIQUID_TUMBLER_VIEW_BOUNDS,
  SCENE_IDS,
  SCENES,
  isReadySceneId,
  maybeSceneById,
  worldBoundsForScene,
  type SceneId,
} from "../src/catalog/scenes";
import { WORLD_BOUNDS } from "../src/render/camera";

const WATCH_FIRST_HINT =
  "This scene is watch-first. Use Play scene, Pause scene, and Reset scene.";

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
  particles: "Watch water fall in an open basin while a ball drops into it.",
  "liquid-timer":
    "Watch tensile, viscous liquid drain through shelves into bottom columns.",
  "surface-tension":
    "Watch three colored tensile groups bead and bleed color when a ball hits them.",
  "elastic-particles":
    "Watch three soft particle clumps deform when a ball falls on them.",
  "rigid-particles":
    "Watch three colored rigid clumps stay solid when a ball hits them.",
  soup: "Watch a basin of liquid hold floating solid bits.",
  "soup-stirrer": "Watch a paddle stir soup, and free or restore its rail.",
  impulse: "Click or tap inside the box to shove the whole particle blob.",
  "wave-machine": "Watch a motorized tank rock and slosh the water inside.",
  "theo-jansen":
    "Watch a walker move under a particle load and reverse its motor.",
  "liquid-tumbler":
    "A drinking glass at real size: 74 mm wide, 0.8 mm particles, and Earth gravity. Phone tilt drives this water on a glass clock.",
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
  particles: WATCH_FIRST_HINT,
  "liquid-timer": WATCH_FIRST_HINT,
  "surface-tension": WATCH_FIRST_HINT,
  "elastic-particles": WATCH_FIRST_HINT,
  "rigid-particles": WATCH_FIRST_HINT,
  soup: WATCH_FIRST_HINT,
  "soup-stirrer":
    "Click or tap the canvas, or use Toggle paddle rail, to free the paddle from its rail or put it back. Labeled controls also work from the keyboard.",
  impulse:
    "Click or tap inside the box to shove the particle blob. Use Push to choose force or impulse. Clicks outside the box do nothing. Labeled controls also work from the keyboard.",
  "wave-machine":
    "Use Wave speed to rock the tank. It starts stopped, and 1× matches the original motor. Labeled controls also work from the keyboard.",
  "theo-jansen":
    "Use Motor direction to walk forward or reverse under the particle load. Labeled controls also work from the keyboard.",
  "liquid-tumbler": WATCH_FIRST_HINT,
};

const KEYBOARD_REMINDER = "Labeled controls also work from the keyboard.";
const WATCH_FIRST_SCENE_IDS = [
  "particles",
  "liquid-timer",
  "surface-tension",
  "elastic-particles",
  "rigid-particles",
  "soup",
  "liquid-tumbler",
] as const;
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

describe("SCENES", () => {
  it("lists seventeen locked scenes in the approved order", () => {
    // Arrange
    const expectedIds = [
      "wave-machine",
      "dam-break",
      "fountain",
      "float-or-sink",
      "color-mixer",
      "jelly-drop",
      "water-wheel",
      "particles",
      "liquid-timer",
      "surface-tension",
      "elastic-particles",
      "rigid-particles",
      "soup",
      "soup-stirrer",
      "impulse",
      "theo-jansen",
      "liquid-tumbler",
    ] as const;

    // Act
    const ids = SCENES.map((scene) => scene.id);

    // Assert
    expect(SCENES).toHaveLength(17);
    expect(ids).toEqual([...SCENE_IDS]);
    expect(ids).toEqual([...expectedIds]);
  });

  it("uses the locked titles for each scene", () => {
    // Arrange
    const expectedTitles = [
      "Wave Machine",
      "Dam Break",
      "Fountain",
      "Float or Sink",
      "Color Mixer",
      "Jelly Drop",
      "Water Wheel",
      "Particles",
      "Liquid Timer",
      "Surface Tension",
      "Elastic Particles",
      "Rigid Particles",
      "Soup",
      "Soup Stirrer",
      "Impulse",
      "Theo Jansen",
      "Liquid Tumbler",
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

  it("keeps locked descriptions after all scenes are ready", () => {
    // Arrange
    const readyCount = SCENES.filter((scene) => scene.ready).length;

    // Act
    const descriptions = SCENES.map((scene) => scene.description);

    // Assert
    expect(readyCount).toBe(17);
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

  it("keeps the keyboard reminder on interactive scenes and omits leftover copy", () => {
    // Arrange
    const interactiveHints = SCENES.filter(
      (scene) =>
        !(WATCH_FIRST_SCENE_IDS as readonly string[]).includes(scene.id),
    ).map((scene) => scene.interactionHint);
    const allHints = SCENES.map((scene) => scene.interactionHint);

    // Act
    const missingReminder = interactiveHints.filter(
      (hint) => !hint.includes(KEYBOARD_REMINDER),
    );
    const forbiddenHits = allHints.filter((hint) =>
      FORBIDDEN_HINT_PHRASES.some((phrase) => hint.includes(phrase)),
    );

    // Assert
    expect(missingReminder).toEqual([]);
    expect(forbiddenHits).toEqual([]);
  });

  it("keeps the locked labeled control path for pointer scenes", () => {
    // Arrange
    const labels = SCENES.flatMap((scene) =>
      scene.controls.map((control) => control.label),
    );

    // Act
    const missingLabels = LOCKED_ACTION_LABELS.filter(
      (label) => !labels.includes(label),
    );

    // Assert
    expect(missingLabels).toEqual([]);
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
    });
    expect(maybeScene?.controls).toHaveLength(4);
    expect(maybeScene?.credits.implementationPath).toBe(
      "crates/liquidfun-wasm/src/scene/dam_break.rs",
    );
  });

  it("frames Liquid Tumbler on the glass and leaves other scenes on the shared bounds", () => {
    // Arrange
    const tumbler = maybeSceneById("liquid-tumbler");
    const damBreak = maybeSceneById("dam-break");

    // Act
    const tumblerBounds = worldBoundsForScene("liquid-tumbler");
    const damBreakBounds = worldBoundsForScene("dam-break");

    // Assert
    expect(tumbler?.viewBounds).toEqual(LIQUID_TUMBLER_VIEW_BOUNDS);
    expect(tumblerBounds).toEqual(LIQUID_TUMBLER_VIEW_BOUNDS);
    expect(damBreak?.viewBounds).toBeUndefined();
    expect(damBreakBounds).toEqual(WORLD_BOUNDS);
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
  it("is true for all seventeen approved ids", () => {
    // Arrange
    const ids = SCENE_IDS;

    // Act
    const readyFlags = ids.map((id) => isReadySceneId(id));

    // Assert
    expect(readyFlags).toEqual(Array.from({ length: 17 }, () => true));
  });
});
