import { SCENES, type SceneId } from "../catalog/scenes";

export type SceneNavigationItem = {
  readonly id: SceneId;
  readonly title: string;
  readonly description: string;
  readonly href: `#/scene/${SceneId}`;
  readonly isCurrent: boolean;
};

export function sceneNavigationItems(
  maybeCurrentSceneId: SceneId | undefined,
): readonly SceneNavigationItem[] {
  return SCENES.map((scene) => ({
    id: scene.id,
    title: scene.title,
    description: scene.description,
    href: `#/scene/${scene.id}`,
    isCurrent: maybeCurrentSceneId === scene.id,
  }));
}
