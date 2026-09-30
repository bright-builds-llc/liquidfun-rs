import { createSignal, type JSX } from "solid-js";

import type { SceneControl } from "../catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  initialRangeValue,
  maybeParseRangeControlValue,
  maybeStepRangeValue,
} from "./range-control";

type SpinnerRange = Extract<SceneControl, { kind: "range" }>;

export function SpinnerControl(props: {
  readonly control: SpinnerRange;
  readonly disabled: boolean;
  readonly maybeValues?: Readonly<Record<string, string>> | undefined;
  readonly onApply: (name: string, value: string) => void;
}): JSX.Element {
  const inputId = `scene-control-${props.control.id}`;
  const initialValue = initialRangeValue(props.control, props.maybeValues);
  const [pendingValue, setPendingValue] = createSignal(initialValue);
  let committedValue = initialValue;

  function commitMagnitude(magnitude: string): void {
    setPendingValue(magnitude);
    if (magnitude === committedValue) {
      return;
    }

    committedValue = magnitude;
    props.onApply(props.control.id, magnitude);
  }

  function onTyped(raw: string): void {
    setPendingValue(raw);
    const maybeMagnitude = maybeParseRangeControlValue(props.control, raw);
    if (maybeMagnitude === undefined || props.control.recreates) {
      return;
    }

    commitMagnitude(maybeMagnitude);
  }

  function onCommit(raw: string): void {
    const maybeMagnitude = maybeParseRangeControlValue(props.control, raw);
    if (maybeMagnitude === undefined) {
      setPendingValue(committedValue);
      return;
    }

    commitMagnitude(maybeMagnitude);
  }

  function step(direction: -1 | 1): void {
    const maybePending = maybeParseRangeControlValue(
      props.control,
      pendingValue(),
    );
    const maybeNext = maybeStepRangeValue(
      props.control,
      maybePending ?? committedValue,
      direction,
    );
    if (maybeNext === undefined) {
      return;
    }

    commitMagnitude(maybeNext);
  }

  const parsedValue = (): string =>
    maybeParseRangeControlValue(props.control, pendingValue()) ?? committedValue;
  const atMinimum = (): boolean => parsedValue() === boundToken(props.control.min);
  const atMaximum = (): boolean => parsedValue() === boundToken(props.control.max);

  return (
    <div class="scene-control">
      <div class="scene-control-label">
        <span class="scene-control-readout">
          <label for={inputId}>{props.control.label}</label>
          <output for={inputId}>
            {formatRangeReadout(parsedValue(), props.control.unit)}
          </output>
        </span>
        <div class="scene-control-spinner">
          <button
            class="scene-control-step"
            type="button"
            disabled={props.disabled || atMinimum()}
            aria-label={`Decrease ${props.control.label}`}
            onClick={() => step(-1)}
          >
            −
          </button>
          <input
            id={inputId}
            class="scene-control-number"
            type="number"
            inputMode="numeric"
            min={props.control.min}
            max={props.control.max}
            step={props.control.step}
            disabled={props.disabled}
            value={pendingValue()}
            aria-valuemin={props.control.min}
            aria-valuemax={props.control.max}
            aria-valuenow={Number(parsedValue())}
            aria-valuetext={formatRangeValueText(
              parsedValue(),
              props.control.unit,
            )}
            onInput={(event) => onTyped(event.currentTarget.value)}
            onChange={(event) => onCommit(event.currentTarget.value)}
          />
          <button
            class="scene-control-step"
            type="button"
            disabled={props.disabled || atMaximum()}
            aria-label={`Increase ${props.control.label}`}
            onClick={() => step(1)}
          >
            +
          </button>
        </div>
      </div>
    </div>
  );
}

function boundToken(bound: number): string | undefined {
  return maybeParseRangeControlValue(
    { min: bound, max: bound, step: 1 },
    String(bound),
  );
}
