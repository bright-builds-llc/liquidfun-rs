import { For, Show, createSignal } from "solid-js";

import type { SceneControl } from "../catalog/scenes";
import { initialPresetValue } from "./scene-controls";

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
        <p class="construction-reset-hint">Changing this setting recreates the scene from its documented initial state.</p>
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

/** Labeled scene presets and actions. Every change applies immediately. */
export function SceneControls(props: SceneControlsProps) {
  if (props.controls.length === 0) {
    return null;
  }

  return (
    <div class="scene-controls">
      <For each={props.controls}>
        {(control) =>
          control.kind === "preset" ? (
            <PresetControl
              control={control}
              disabled={props.disabled}
              maybeValues={props.maybeValues}
              onApply={props.onApplyControl}
            />
          ) : (
            <ActionControl
              control={control}
              disabled={props.disabled}
              onApply={props.onApplyAction}
            />
          )
        }
      </For>
    </div>
  );
}
