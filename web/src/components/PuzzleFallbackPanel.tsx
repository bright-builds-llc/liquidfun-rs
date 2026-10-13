import { Show } from "solid-js";

import { PUZZLES_HASH } from "../catalog/puzzle-levels";
import { truncateRaw } from "../puzzle/fallback-text";
import { DEFAULT_SCENE_HASH } from "../routing/hash";
import { PuzzleOptionsSheet, PuzzleOptionsTrigger } from "./PuzzleOptionsDrawer";
import { Drawer } from "./ui/drawer";

const EMPTY_RAW_BODY =
  "This puzzle link does not name a level. Pick one from All levels, or go back to the scenes.";

export type PuzzleFallbackPanelProps = {
  readonly canvasStage: boolean;
  readonly maybeRaw: string;
  readonly progressDegraded: boolean;
};

/**
 * Unknown-level copy (D-05, Phase 41 D-04). The raw id is attacker-controlled
 * hash text, so it renders through a text binding only and is truncated
 * (T-38-18); the links use fixed hashes, never the raw id.
 */
export function PuzzleFallbackPanel(props: PuzzleFallbackPanelProps) {
  const body = () =>
    props.maybeRaw === ""
      ? EMPTY_RAW_BODY
      : `No level is called “${truncateRaw(props.maybeRaw)}”. Pick one from All levels, or go back to the scenes.`;

  return (
    <section
      class="fallback-panel"
      classList={{ "fallback-panel--canvas-stage": props.canvasStage }}
      aria-labelledby="player-title"
    >
      <h2 id="player-title">Level not found</h2>
      <p class="fallback-copy">{body()}</p>
      <div class="control-row">
        <a class="product-control product-control--accent" href={PUZZLES_HASH}>
          All levels
        </a>
        <a class="product-control" href={DEFAULT_SCENE_HASH}>
          Back to scenes
        </a>
      </div>
      <Show when={props.canvasStage}>
        <Drawer side="bottom">
          <PuzzleOptionsTrigger />
          <PuzzleOptionsSheet progressDegraded={props.progressDegraded} />
        </Drawer>
      </Show>
    </section>
  );
}
