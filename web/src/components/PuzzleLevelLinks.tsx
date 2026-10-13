import "../styles/puzzle-select.css";

import { Show } from "solid-js";

import { PROGRESS_NOTE, PUZZLES_HASH } from "../catalog/puzzle-levels";

/**
 * "All levels" link plus the degraded-storage note. Rendered in the options
 * sheet and the form-factor options row only, never on the canvas HUD
 * (D-04, D-15).
 */
export function PuzzleLevelLinks(props: { readonly degraded: boolean }) {
  return (
    <div class="puzzle-level-links">
      <a class="product-control" href={PUZZLES_HASH}>
        All levels
      </a>
      <Show when={props.degraded}>
        <p class="puzzle-progress-note" data-progress-note>
          {PROGRESS_NOTE}
        </p>
      </Show>
    </div>
  );
}
