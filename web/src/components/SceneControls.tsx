import { For, Show, createSignal } from "solid-js";

import type { SceneControl } from "../catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  initialRangeValue,
  maybeParseRangeControlValue,
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
  const initialValue = initialRangeValue(props.control, props.maybeValues);
  const [pendingValue, setPendingValue] = createSignal(initialValue);
  let committedValue = initialValue;

  function onRangeInput(value: string): void {
    setPendingValue(value);
  }

  function onRangeCommit(value: string): void {
    const maybeAccepted = maybeParseRangeControlValue(props.control, value);
    if (maybeAccepted === undefined) {
      setPendingValue(committedValue);
      return;
    }
    setPendingValue(maybeAccepted);
    if (maybeAccepted === committedValue) {
      return;
    }

    committedValue = maybeAccepted;
    props.onApply(props.control.id, maybeAccepted);
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
        <input
          id={inputId}
          class="scene-control-range"
          type="range"
          min={props.control.min}
          max={props.control.max}
          step={props.control.step}
          disabled={props.disabled}
          value={pendingValue()}
          aria-valuetext={formatRangeValueText(
            pendingValue(),
            props.control.unit,
          )}
          onInput={(event) => onRangeInput(event.currentTarget.value)}
          onChange={(event) => onRangeCommit(event.currentTarget.value)}
        />
      </div>
      <div class="scene-control-scale" aria-hidden="true">
        <span>{props.control.min}</span>
        <span>{props.control.max}</span>
      </div>
      <Show when={props.control.recreates}>
        <p class="construction-reset-hint">{CONSTRUCTION_RESET_HINT}</p>
      </Show>
    </div>
  );
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
