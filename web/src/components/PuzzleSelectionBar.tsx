import type { JSX } from "solid-js";

import { selectionBarEnabled, valveStepperModel } from "../puzzle/hud-format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import {
  MinusIcon,
  PlusIcon,
  RemoveIcon,
  RotateClockwiseIcon,
  RotateCounterclockwiseIcon,
} from "./PuzzleIcons";

/**
 * Id of the valve readout that describes the stepper group (Phase 44 D-04).
 * A fixed id is safe because the bar mounts once per puzzle route, like
 * `#puzzle-editor-status`.
 */
const VALVE_TIME_ID = "puzzle-valve-time";

export type PuzzleSelectionBarProps = {
  readonly state: PuzzleState;
  /** +1 rotates counterclockwise 15°, −1 clockwise (D-19). */
  readonly onRotate: (direction: 1 | -1) => void;
  readonly onRemove: () => void;
  /** −1 opens the selected valve 0.5 s earlier, +1 later (Phase 40 D-05). */
  readonly onStepValveTime: (direction: 1 | -1) => void;
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
 * Selection bar leaf (D-19, Phase 40 D-05): the valve open-time stepper,
 * rotate counterclockwise, rotate clockwise and Remove for the selected tool.
 * Mounted for the whole route; the buttons are `aria-disabled` unless they
 * apply, so focus survives a cleared selection and the obstacle set never
 * changes.
 */
export function PuzzleSelectionBar(props: PuzzleSelectionBarProps) {
  const enabled = () => selectionBarEnabled(props.state);
  const valve = () => valveStepperModel(props.state);
  return (
    <div
      class="puzzle-selection-bar"
      data-hud-obstacle
      role="toolbar"
      aria-label="Selected tool"
      aria-orientation="horizontal"
    >
      <div
        class="puzzle-valve-stepper"
        role="group"
        aria-label="Valve opening time"
        aria-describedby={valve().enabled ? VALVE_TIME_ID : undefined}
      >
        <SelectionButton
          name="Open the valve 0.5 s earlier"
          enabled={valve().earlierEnabled}
          onPress={() => props.onStepValveTime(-1)}
        >
          <MinusIcon />
        </SelectionButton>
        <span
          id={VALVE_TIME_ID}
          class="puzzle-valve-time"
          data-enabled={valve().enabled ? "true" : "false"}
        >
          {valve().text}
        </span>
        <SelectionButton
          name="Open the valve 0.5 s later"
          enabled={valve().laterEnabled}
          onPress={() => props.onStepValveTime(1)}
        >
          <PlusIcon />
        </SelectionButton>
      </div>
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
