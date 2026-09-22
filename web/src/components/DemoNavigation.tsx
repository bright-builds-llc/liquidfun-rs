import { createEffect, For, on, onCleanup } from "solid-js";

import { ScenePreview } from "../catalog/previews";
import type { SceneId } from "../catalog/scenes";
import { sceneNavigationItems } from "../player/navigation";
import { scheduleSidebarReveal } from "./sidebar-scroll";

export type DemoNavigationProps = {
  readonly label: string;
  readonly maybeCurrentSceneId: SceneId | undefined;
  readonly revealCurrent: boolean;
  readonly onNavigate?: (() => void) | undefined;
};

/** Semantic direct-link navigation for every playground demo. */
export function DemoNavigation(props: DemoNavigationProps) {
  let maybeNavigation: HTMLElement | undefined;

  createEffect(
    on(
      () => [props.revealCurrent, props.maybeCurrentSceneId] as const,
      ([revealCurrent]) => {
        if (!revealCurrent) {
          return;
        }
        const navigation = maybeNavigation;
        if (!navigation) {
          return;
        }
        onCleanup(scheduleSidebarReveal(navigation));
      },
    ),
  );

  return (
    <nav
      ref={maybeNavigation}
      class="demo-navigation"
      aria-label={props.label}
    >
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
                <span class="demo-nav-preview">
                  <ScenePreview sceneId={item.id} />
                </span>
                <span class="demo-nav-preview-caption">Static preview</span>
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
