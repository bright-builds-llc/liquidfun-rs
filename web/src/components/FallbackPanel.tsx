import { Show } from "solid-js";

import { SceneControlsSheet, SceneControlsTrigger } from "./CanvasStage";
import { SiteFooter } from "./SiteFooter";
import { Drawer } from "./ui/drawer";

const UNKNOWN_HEADING = "Scene not found";
const UNKNOWN_BODY =
  "This playground link does not match a known scene. Open Dam Break, or choose a demo from the navigation list.";
const OPEN_DAM_BREAK = "Open Dam Break";
const SHEET_DESCRIPTION = "Project details for this playground.";

export type FallbackPanelProps = {
  readonly canvasStage: boolean;
};

/** Unknown-hash copy with a hash-only Open Dam Break control. */
export function FallbackPanel(props: FallbackPanelProps) {
  return (
    <section
      class="fallback-panel"
      classList={{ "fallback-panel--canvas-stage": props.canvasStage }}
      aria-labelledby="player-title"
    >
      <h2 id="player-title">{UNKNOWN_HEADING}</h2>
      <p class="fallback-copy">{UNKNOWN_BODY}</p>
      <a class="product-control" href="#/scene/dam-break">
        {OPEN_DAM_BREAK}
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
