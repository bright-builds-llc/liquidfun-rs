import { For } from "solid-js";

import { SCENES, type SceneId, type SceneRecord } from "../catalog/scenes";

export type CatalogNavProps = {
  readonly maybeCurrentSceneId?: SceneId;
};

function catalogItemClass(scene: SceneRecord, isCurrent: boolean): string {
  if (isCurrent) {
    return "catalog-item catalog-item--current";
  }

  if (!scene.ready) {
    return "catalog-item catalog-item--not-ready";
  }

  return "catalog-item";
}

/** Honest six-name catalog with a hash-only Dam Break link. */
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
              scene.id === "dam-break" &&
              props.maybeCurrentSceneId === "dam-break";
            const chip = scene.ready ? "Ready" : "Not ready yet";

            return (
              <li class={catalogItemClass(scene, isCurrent)}>
                {scene.id === "dam-break" ? (
                  <a
                    href="#/scene/dam-break"
                    aria-current={isCurrent ? "page" : undefined}
                  >
                    {scene.title}
                  </a>
                ) : (
                  <a href={`#/scene/${scene.id}`}>{scene.title}</a>
                )}
                <span class="scene-chip">{chip}</span>
              </li>
            );
          }}
        </For>
      </ul>
    </nav>
  );
}
