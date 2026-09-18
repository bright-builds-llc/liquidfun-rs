import { For } from "solid-js";

import { ScenePreview } from "../catalog/previews";
import { SCENES, isReadySceneId, type SceneId } from "../catalog/scenes";

const PREVIEW_CAPTION = "Static preview";
const OPEN_LABEL = "Open";

export type CatalogNavProps = {
  readonly maybeCurrentSceneId?: SceneId | undefined;
};

function catalogCardClass(isCurrent: boolean): string {
  if (isCurrent) {
    return "catalog-card catalog-card--current";
  }

  return "catalog-card";
}

/** Six-card catalog with static SVG previews and hash-only Open links. */
export function CatalogNav(props: CatalogNavProps) {
  return (
    <nav class="catalog-nav" aria-labelledby="scene-nav-title">
      <p id="scene-nav-title" class="catalog-heading">
        Scenes
      </p>
      <ul class="catalog-list">
        <For each={SCENES}>
          {(scene) => {
            const isCurrent =
              props.maybeCurrentSceneId === scene.id &&
              isReadySceneId(scene.id);

            return (
              <li class={catalogCardClass(isCurrent)}>
                <ScenePreview sceneId={scene.previewId} />
                <p class="catalog-preview-caption">{PREVIEW_CAPTION}</p>
                <p class="catalog-card-title">{scene.title}</p>
                <p class="catalog-card-description">{scene.description}</p>
                <a
                  class="product-control catalog-open"
                  href={`#/scene/${scene.id}`}
                  aria-current={isCurrent ? "page" : undefined}
                >
                  {OPEN_LABEL}
                </a>
              </li>
            );
          }}
        </For>
      </ul>
    </nav>
  );
}
