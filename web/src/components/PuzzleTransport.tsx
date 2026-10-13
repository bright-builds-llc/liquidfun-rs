import { Show, createMemo } from "solid-js";

import { transportModel } from "../puzzle/hud-format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { ClearIcon, PlayIcon, StopIcon } from "./PuzzleIcons";

export type PuzzleTransportProps = {
  readonly state: PuzzleState;
  readonly onPlay: () => void;
  readonly onStop: () => void;
  readonly onCycleSpeed: () => void;
  readonly onClear: () => void;
  readonly playRef: (element: HTMLButtonElement) => void;
};

const CLEAR_NAME = "Clear all tools";
const STATUS_LINE_ID = "puzzle-status-line";

/**
 * Run controls (D-21, D-24): Play/Stop (one element, so focus is never
 * lost), the speed cycle and Clear. Every control stays focusable and uses
 * `aria-disabled`; presses are ignored while disabled. Landscape hides the
 * text labels by CSS and keeps the icon-only buttons named by `aria-label`.
 */
export function PuzzleTransport(props: PuzzleTransportProps) {
  const model = createMemo(() => transportModel(props.state));
  const playName = () => (model().mode === "play" ? "Play" : "Stop");

  return (
    <div class="puzzle-transport" data-hud-obstacle role="group" aria-label="Run controls">
      <button
        ref={(element) => props.playRef(element)}
        type="button"
        class="product-control puzzle-play"
        classList={{ "product-control--accent": model().playAccent }}
        data-mode={model().mode}
        aria-label={playName()}
        title={model().maybeDisabledReason ?? playName()}
        aria-disabled={model().playEnabled ? undefined : "true"}
        aria-describedby={model().maybeDisabledReason === undefined ? undefined : STATUS_LINE_ID}
        onClick={() => {
          if (!model().playEnabled) {
            return;
          }
          if (model().mode === "play") {
            props.onPlay();
          } else {
            props.onStop();
          }
        }}
      >
        <Show when={model().mode === "play"} fallback={<StopIcon />}>
          <PlayIcon />
        </Show>
        <span class="puzzle-transport-text">{playName()}</span>
      </button>
      <button
        type="button"
        class="product-control puzzle-speed"
        aria-label={model().speedAccessibleName}
        title={model().speedTitle}
        aria-disabled={model().speedEnabled ? undefined : "true"}
        onClick={() => {
          if (model().speedEnabled) {
            props.onCycleSpeed();
          }
        }}
      >
        {model().speedText}
      </button>
      <button
        type="button"
        class="product-control puzzle-clear"
        aria-label={CLEAR_NAME}
        title={CLEAR_NAME}
        aria-disabled={model().clearEnabled ? undefined : "true"}
        onClick={() => {
          if (model().clearEnabled) {
            props.onClear();
          }
        }}
      >
        <ClearIcon />
        <span class="puzzle-transport-text">Clear</span>
      </button>
    </div>
  );
}
