import type { JSX } from "solid-js";

import { selectionBarEnabled } from "../puzzle/hud-format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { RemoveIcon, RotateClockwiseIcon, RotateCounterclockwiseIcon } from "./PuzzleIcons";

export type PuzzleSelectionBarProps = {
  readonly state: PuzzleState;
  /** +1 rotates counterclockwise 15°, −1 clockwise (D-19). */
  readonly onRotate: (direction: 1 | -1) => void;
  readonly onRemove: () => void;
};

function SelectionButton(props: {
  readonly name: string;
  readonly enabled: boolean;
  readonly onPress: () => void;
  readonly children: JSX.Element;
}) {
  return (
    <button
      type="button"
      class="product-control puzzle-icon-button"
      aria-label={props.name}
      title={props.name}
      aria-disabled={props.enabled ? undefined : "true"}
      onClick={() => {
        if (props.enabled) {
          props.onPress();
        }
      }}
    >
      {props.children}
    </button>
  );
}

/**
 * Selection bar leaf (D-19): rotate counterclockwise, rotate clockwise and
 * Remove for the selected tool. Mounted for the whole route; the buttons are
 * `aria-disabled` unless building with a selection, so Remove keeps focus
 * after it clears the selection.
 */
export function PuzzleSelectionBar(props: PuzzleSelectionBarProps) {
  const enabled = () => selectionBarEnabled(props.state);
  return (
    <div
      class="puzzle-selection-bar"
      data-hud-obstacle
      role="toolbar"
      aria-label="Selected tool"
      aria-orientation="horizontal"
    >
      <SelectionButton
        name="Rotate counterclockwise 15°"
        enabled={enabled()}
        onPress={() => props.onRotate(1)}
      >
        <RotateCounterclockwiseIcon />
      </SelectionButton>
      <SelectionButton
        name="Rotate clockwise 15°"
        enabled={enabled()}
        onPress={() => props.onRotate(-1)}
      >
        <RotateClockwiseIcon />
      </SelectionButton>
      <SelectionButton name="Remove selected tool" enabled={enabled()} onPress={props.onRemove}>
        <RemoveIcon />
      </SelectionButton>
    </div>
  );
}
