export const SCENE_IDS = [
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
] as const;

export type SceneId = (typeof SCENE_IDS)[number];

export type SceneRecord = {
  readonly id: SceneId;
  readonly title: string;
  readonly ready: boolean;
};

export const SCENES: readonly SceneRecord[] = [
  { id: "dam-break", title: "Dam Break", ready: true },
  { id: "fountain", title: "Fountain", ready: false },
  { id: "float-or-sink", title: "Float or Sink", ready: false },
  { id: "color-mixer", title: "Color Mixer", ready: false },
  { id: "jelly-drop", title: "Jelly Drop", ready: false },
  { id: "water-wheel", title: "Water Wheel", ready: false },
];

export function maybeSceneById(id: string): SceneRecord | undefined {
  return SCENES.find((scene) => scene.id === id);
}

export function isReadySceneId(id: SceneId): boolean {
  const maybeScene = maybeSceneById(id);
  return maybeScene?.ready === true;
}
