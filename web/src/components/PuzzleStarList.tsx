import { Index } from "solid-js";

import type { StarRowModel } from "../puzzle/star-format";
import { StarIcon } from "./PuzzleIcons";

export type PuzzleStarListProps = {
  readonly rows: readonly StarRowModel[];
  /** "Stars" before the end, "Stars this run" once ended (`starListLabel`). */
  readonly label: string;
};

/**
 * Star checklist and result rows (D-16, UI-D12): Win, Brim, Thrifty. The
 * glyph and visible label are decorative; a visually hidden span carries the
 * full row text and state phrase, so color is never the only cue.
 */
export function PuzzleStarList(props: PuzzleStarListProps) {
  return (
    <ul class="puzzle-stars" role="list" aria-label={props.label}>
      <Index each={props.rows}>
        {(row) => (
          <li class="puzzle-star-row" data-criterion={row().criterion} data-glyph={row().glyph}>
            <StarIcon filled={row().glyph === "earned"} />
            <span class="puzzle-star-label" aria-hidden="true">
              {row().label}
            </span>
            <span class="visually-hidden">{row().hiddenText}</span>
          </li>
        )}
      </Index>
    </ul>
  );
}
