import { Show } from "solid-js";

import { maybeSceneById } from "../catalog/scenes";
import { DEFAULT_SCENE_HASH, DEFAULT_SCENE_ID } from "../routing/hash";
import { SceneControlsSheet, SceneControlsTrigger } from "./CanvasStage";
import { SiteFooter } from "./SiteFooter";
import { Drawer } from "./ui/drawer";

const DEFAULT_SCENE_TITLE = maybeSceneById(DEFAULT_SCENE_ID)?.title ?? "Wave Machine";
const UNKNOWN_HEADING = "Scene not found";
const UNKNOWN_BODY = `This playground link does not match a known scene. Open ${DEFAULT_SCENE_TITLE}, or choose a demo from the navigation list.`;
const OPEN_DEFAULT_SCENE = `Open ${DEFAULT_SCENE_TITLE}`;
const SHEET_DESCRIPTION = "Project details for this playground.";

export type FallbackPanelProps = {
  readonly canvasStage: boolean;
};

/** Unknown-hash copy with a hash-only link back to the default scene. */
export function FallbackPanel(props: FallbackPanelProps) {
  return (
    <section
      class="fallback-panel"
      classList={{ "fallback-panel--canvas-stage": props.canvasStage }}
      aria-labelledby="player-title"
    >
      <h2 id="player-title">{UNKNOWN_HEADING}</h2>
      <p class="fallback-copy">{UNKNOWN_BODY}</p>
      <a class="product-control" href={DEFAULT_SCENE_HASH}>
        {OPEN_DEFAULT_SCENE}
      </a>
      <Show when={props.canvasStage}>
        <Drawer side="bottom">
          <SceneControlsTrigger />
          <SceneControlsSheet description={SHEET_DESCRIPTION}>
            <SiteFooter />
          </SceneControlsSheet>
        </Drawer>
      </Show>
    </section>
  );
}
