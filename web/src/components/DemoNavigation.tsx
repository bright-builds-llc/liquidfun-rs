import { For } from "solid-js";

import type { SceneId } from "../catalog/scenes";
import { sceneNavigationItems } from "../player/navigation";

export type DemoNavigationProps = {
  readonly label: string;
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly onNavigate?: (() => void) | undefined;
};

/** Semantic direct-link navigation for every playground demo. */
export function DemoNavigation(props: DemoNavigationProps) {
  return (
    <nav class="demo-navigation" aria-label={props.label}>
      <ul class="demo-nav-list">
        <For each={sceneNavigationItems(props.maybeCurrentSceneId)}>
          {(item) => (
            <li>
              <a
                class="demo-nav-link"
                href={item.href}
                aria-current={item.isCurrent ? "page" : undefined}
                onClick={() => props.onNavigate?.()}
              >
                <span class="demo-nav-title">{item.title}</span>
                <span class="demo-nav-description">{item.description}</span>
              </a>
            </li>
          )}
        </For>
      </ul>
    </nav>
  );
}
