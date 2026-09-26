import { For, Show, createSignal } from "solid-js";

import type { SceneControl } from "../catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  initialRangeValue,
  maybeMagnitudeForSliderPosition,
  maybeParseRangeControlValue,
  rangeTickMarks,
  sliderBounds,
  sliderPositionForMagnitude,
} from "./range-control";
import { CONSTRUCTION_RESET_HINT, initialPresetValue } from "./scene-controls";

export { CONSTRUCTION_RESET_HINT, constructionHintVisible } from "./scene-controls";

export type SceneControlsProps = {
  readonly controls: readonly SceneControl[];
  readonly disabled: boolean;
  readonly maybeValues?: Readonly<Record<string, string>> | undefined;
  readonly onApplyControl: (name: string, value: string) => void;
  readonly onApplyAction: (name: string) => void;
};

function PresetControl(props: {
  readonly control: Extract<SceneControl, { kind: "preset" }>;
  readonly disabled: boolean;
  readonly maybeValues?: Readonly<Record<string, string>> | undefined;
  readonly onApply: (name: string, value: string) => void;
}) {
  const [pendingValue, setPendingValue] = createSignal(
    initialPresetValue(props.control, props.maybeValues),
  );

  function onSelectChange(value: string): void {
    setPendingValue(value);
    props.onApply(props.control.id, value);
  }

  return (
    <div class="scene-control">
      <label class="scene-control-label">
        {props.control.label}
        <select
          class="scene-control-select"
          disabled={props.disabled}
          value={pendingValue()}
          onChange={(event) => onSelectChange(event.currentTarget.value)}
        >
          <For each={props.control.values}>
            {(option) => <option value={option.id}>{option.label}</option>}
          </For>
        </select>
      </label>
      <Show when={props.control.recreates}>
        <p class="construction-reset-hint">{CONSTRUCTION_RESET_HINT}</p>
      </Show>
    </div>
  );
}

function RangeControl(props: {
  readonly control: Extract<SceneControl, { kind: "range" }>;
  readonly disabled: boolean;
  readonly maybeValues?: Readonly<Record<string, string>> | undefined;
  readonly onApply: (name: string, value: string) => void;
}) {
  const inputId = `scene-control-${props.control.id}`;
  const bounds = sliderBounds(props.control);
  const ticks = rangeTickMarks(props.control);
  const initialValue = initialRangeValue(props.control, props.maybeValues);
  const [pendingValue, setPendingValue] = createSignal(initialValue);
  const [sliderPosition, setSliderPosition] = createSignal(
    sliderPositionForMagnitude(props.control, Number(initialValue)),
  );
  let committedValue = initialValue;

  function showMagnitude(magnitude: string): void {
    setPendingValue(magnitude);
    setSliderPosition(
      sliderPositionForMagnitude(props.control, Number(magnitude)),
    );
  }

  function commitMagnitude(magnitude: string): void {
    showMagnitude(magnitude);
    if (magnitude === committedValue) {
      return;
    }

    committedValue = magnitude;
    props.onApply(props.control.id, magnitude);
  }

  function onRangeInput(position: string): void {
    const maybeMagnitude = maybeMagnitudeForSliderPosition(
      props.control,
      position,
    );
    if (maybeMagnitude === undefined) {
      return;
    }

    setSliderPosition(Number(position));
    setPendingValue(maybeMagnitude);
  }

  function onRangeCommit(position: string): void {
    const maybeMagnitude = maybeMagnitudeForSliderPosition(
      props.control,
      position,
    );
    if (maybeMagnitude === undefined) {
      showMagnitude(committedValue);
      return;
    }

    commitMagnitude(maybeMagnitude);
  }

  function onRangeKeyDown(event: KeyboardEvent): void {
    if (props.control.scale !== "logarithmic") {
      return;
    }

    const direction = logarithmicKeyDirection(event.key);
    if (direction === undefined) {
      return;
    }

    event.preventDefault();
    const current = Number(pendingValue());
    if (!Number.isFinite(current)) {
      return;
    }

    const next = Math.min(
      props.control.max,
      Math.max(props.control.min, current + direction * props.control.step),
    );
    const maybeMagnitude = maybeParseRangeControlValue(
      props.control,
      String(next),
    );
    if (maybeMagnitude === undefined) {
      return;
    }

    commitMagnitude(maybeMagnitude);
  }

  return (
    <div class="scene-control">
      <div class="scene-control-label">
        <span class="scene-control-readout">
          <label for={inputId}>{props.control.label}</label>
          <output for={inputId}>
            {formatRangeReadout(pendingValue(), props.control.unit)}
          </output>
        </span>
        <div class="scene-control-slider">
          <input
            id={inputId}
            class="scene-control-range"
            type="range"
            min={bounds.min}
            max={bounds.max}
            step={bounds.step}
            disabled={props.disabled}
            value={sliderPosition()}
            aria-valuemin={props.control.min}
            aria-valuemax={props.control.max}
            aria-valuenow={Number(pendingValue())}
            aria-valuetext={formatRangeValueText(
              pendingValue(),
              props.control.unit,
            )}
            onInput={(event) => onRangeInput(event.currentTarget.value)}
            onChange={(event) => onRangeCommit(event.currentTarget.value)}
            onKeyDown={onRangeKeyDown}
          />
          <div class="scene-control-ticks" aria-hidden="true">
            <For each={ticks}>
              {(tick) => (
                <span
                  class="scene-control-tick"
                  style={{
                    left: `calc(${tick.ratio} * (100% - var(--range-thumb)) + (var(--range-thumb) / 2))`,
                  }}
                >
                  <span class="scene-control-tick-mark" />
                  <span>{tick.label}</span>
                </span>
              )}
            </For>
          </div>
        </div>
      </div>
      <Show when={props.control.recreates}>
        <p class="construction-reset-hint">{CONSTRUCTION_RESET_HINT}</p>
      </Show>
    </div>
  );
}

function logarithmicKeyDirection(key: string): number | undefined {
  if (key === "ArrowRight" || key === "ArrowUp") {
    return 1;
  }
  if (key === "ArrowLeft" || key === "ArrowDown") {
    return -1;
  }

  return undefined;
}

function ActionControl(props: {
  readonly control: Extract<SceneControl, { kind: "action" }>;
  readonly disabled: boolean;
  readonly onApply: (name: string) => void;
}) {
  return (
    <div class="scene-control">
      <button
        class="product-control"
        type="button"
        disabled={props.disabled}
        onClick={() => props.onApply(props.control.id)}
      >
        {props.control.label}
      </button>
    </div>
  );
}

/** Labeled scene presets, sliders, and actions. Every change applies immediately. */
export function SceneControls(props: SceneControlsProps) {
  if (props.controls.length === 0) {
    return null;
  }

  return (
    <div class="scene-controls">
      <For each={props.controls}>
        {(control) => {
          if (control.kind === "preset") {
            return (
              <PresetControl
                control={control}
                disabled={props.disabled}
                maybeValues={props.maybeValues}
                onApply={props.onApplyControl}
              />
            );
          }
          if (control.kind === "range") {
            return (
              <RangeControl
                control={control}
                disabled={props.disabled}
                maybeValues={props.maybeValues}
                onApply={props.onApplyControl}
              />
            );
          }

          return (
            <ActionControl
              control={control}
              disabled={props.disabled}
              onApply={props.onApplyAction}
            />
          );
        }}
      </For>
    </div>
  );
}
