import { Show } from "solid-js";

import type { CaptionModel } from "../puzzle/caption";

export type PuzzleCaptionProps = {
  readonly model: CaptionModel;
  readonly onDismiss: () => void;
};

/**
 * Caption HUD leaf (D-19, D-21). The leaf stays mounted with one fixed box per
 * layout; only its children swap between the first-level hint and the level
 * caption, so dismissing never changes the obstacle set or refits the camera.
 */
export function PuzzleCaption(props: PuzzleCaptionProps) {
  return (
    <div
      class="puzzle-caption puzzle-chip"
      data-hud-obstacle
      data-hint={props.model.showDismiss ? "true" : "false"}
    >
      <p class="puzzle-caption-text" title={props.model.text}>
        {props.model.text}
      </p>
      <Show when={props.model.showDismiss}>
        <button
          type="button"
          class="product-control puzzle-caption-dismiss"
          aria-label="Dismiss hint"
          onClick={() => props.onDismiss()}
        >
          Got it
        </button>
      </Show>
    </div>
  );
}
