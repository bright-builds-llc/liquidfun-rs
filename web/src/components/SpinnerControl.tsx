import { createSignal, type JSX } from "solid-js";

import type { SceneControl } from "../catalog/scenes";
import {
  formatRangeReadout,
  formatRangeValueText,
  initialRangeValue,
  maybeMagnitudeForSliderPosition,
  maybeParseRangeControlValue,
  maybeStepRangeValue,
  sliderBounds,
  sliderPositionForMagnitude,
} from "./range-control";

type SpinnerRange = Extract<SceneControl, { kind: "range" }>;

export function SpinnerControl(props: {
  readonly control: SpinnerRange;
  readonly disabled: boolean;
  readonly maybeValues?: Readonly<Record<string, string>> | undefined;
  readonly onApply: (name: string, value: string) => void;
}): JSX.Element {
  const inputId = `scene-control-${props.control.id}`;
  const bounds = sliderBounds(props.control);
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
    if (!props.control.recreates) {
      commitMagnitude(maybeMagnitude);
    }
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
            class="scene-control-range"
            type="range"
            min={bounds.min}
            max={bounds.max}
            step={bounds.step}
            disabled={props.disabled}
            value={sliderPosition()}
            aria-valuemin={props.control.min}
            aria-valuemax={props.control.max}
            aria-valuenow={Number(parsedValue())}
            aria-valuetext={formatRangeValueText(
              parsedValue(),
              props.control.unit,
            )}
            onInput={(event) => onRangeInput(event.currentTarget.value)}
            onChange={(event) => onRangeCommit(event.currentTarget.value)}
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
