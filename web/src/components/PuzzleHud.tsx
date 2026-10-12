import { For, createMemo, type JSX } from "solid-js";

import { fillMeterModel, timerText } from "../puzzle/format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { ScaleLegend } from "./ScaleLegend";

const PLAY_LABEL = "Play";

/** Level title leaf; the full title stays available when the fixed box ellipsizes it. */
export function PuzzleTitle(props: { readonly title: string }) {
  return (
    <h2 id="player-title" class="puzzle-title" data-hud-obstacle title={props.title}>
      {props.title}
    </h2>
  );
}

/** Authoritative fill readout (D-11): percent, target, bar and goal ticks in a fixed chip. */
export function PuzzleFillMeter(props: { readonly state: PuzzleState }) {
  const model = createMemo(() => fillMeterModel(props.state));
  return (
    <div
      class="puzzle-meter puzzle-chip"
      data-hud-obstacle
      role="meter"
      aria-label="Cup fill"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={model().maybeValueNow}
      aria-valuetext={model().maybeValueText}
    >
      <div class="puzzle-meter-readout">
        <span class="puzzle-meter-percent">{model().percentText}</span>
        <span class="puzzle-meter-target">{model().targetText}</span>
      </div>
      <div class="puzzle-meter-track" aria-hidden="true">
        <div class="puzzle-meter-fill" style={{ width: `${model().fillPercent}%` }} />
        <For each={model().ticks}>
          {(tick) => (
            <span
              class="puzzle-meter-tick"
              data-star={tick.label.startsWith("★") ? "true" : "false"}
              style={{ left: `${tick.fraction * 100}%` }}
            />
          )}
        </For>
      </div>
    </div>
  );
}

/**
 * Simulation-time countdown (D-09). `role="timer"` is not live, unlike an
 * output element, so it never announces each presented frame (UI-D6).
 */
export function PuzzleTimer(props: { readonly state: PuzzleState }) {
  return (
    <div
      class="puzzle-timer puzzle-chip"
      data-hud-obstacle
      role="timer"
      aria-label="Simulation time left"
    >
      {timerText(props.state)}
    </div>
  );
}

/**
 * Puzzle transport. Outside building it stays focusable with `aria-disabled`
 * and ignores presses, so focus is never dropped by a phase change.
 */
export function PuzzlePlayButton(props: {
  readonly state: PuzzleState;
  readonly onPlay: () => void;
  readonly ref?: (element: HTMLButtonElement) => void;
}) {
  const building = () => props.state.kind === "building";
  return (
    <button
      ref={(element) => props.ref?.(element)}
      type="button"
      class="product-control puzzle-play"
      classList={{ "product-control--accent": building() }}
      aria-disabled={building() ? undefined : "true"}
      onClick={() => {
        if (building()) {
          props.onPlay();
        }
      }}
    >
      <PlayIcon />
      {PLAY_LABEL}
    </button>
  );
}

export type PuzzleHudProps = {
  readonly title: string;
  readonly state: PuzzleState;
  readonly maybePixelsPerMeter: number | undefined;
  readonly onPlay: () => void;
  readonly playRef: (element: HTMLButtonElement) => void;
  readonly resultCard: JSX.Element;
  readonly optionsTrigger: JSX.Element;
};

/**
 * Canvas-stage puzzle HUD. Every leaf is mounted for the whole route with a
 * fixed CSS box, so phase changes never change the obstacle set (D-14, D-18).
 * DOM order is the focus order: Play, result actions, options.
 */
export function PuzzleHud(props: PuzzleHudProps) {
  return (
    <div class="puzzle-hud">
      <PuzzleTitle title={props.title} />
      <PuzzleFillMeter state={props.state} />
      <PuzzleTimer state={props.state} />
      <div class="puzzle-transport" data-hud-obstacle>
        <PuzzlePlayButton state={props.state} onPlay={props.onPlay} ref={props.playRef} />
      </div>
      {props.resultCard}
      <div class="puzzle-options-slot" data-hud-obstacle>
        {props.optionsTrigger}
      </div>
      <div class="puzzle-legend-slot" data-hud-obstacle>
        <ScaleLegend maybePixelsPerMeter={props.maybePixelsPerMeter} />
      </div>
    </div>
  );
}

function PlayIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 6v12l10-6z" />
    </svg>
  );
}
