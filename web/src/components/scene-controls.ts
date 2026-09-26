import type { SceneControl } from "../catalog/scenes";
import { maybeParseRangeControlValue } from "./range-control";

export const CONSTRUCTION_RESET_HINT =
  "Changing this setting recreates the scene from its documented initial state.";

export const DEFAULT_PRESET_VALUES: Readonly<Record<string, string>> = {
  "water-amount": "medium",
  "emission-rate": "medium",
  "launch-speed": "medium",
  "aim-angle": "up",
  body: "wood",
  "mix-strength": "strong",
  "stir-speed": "slow",
  shape: "circle",
  softness: "medium",
  "jet-strength": "medium",
  emission: "on",
};

/** True when changing the control rebuilds the scene from its initial state. */
export function constructionHintVisible(control: SceneControl): boolean {
  return control.recreates;
}

export type ControlSurface = "hud" | "panel";

/** Splits catalog controls between the canvas HUD and the scene panel. */
export function sceneControlsForSurface(
  controls: readonly SceneControl[],
  surface: ControlSurface,
): readonly SceneControl[] {
  return controls.filter((control) => controlSurface(control) === surface);
}

function controlSurface(control: SceneControl): ControlSurface {
  if (control.kind === "range" && control.surface === "hud") {
    return "hud";
  }

  return "panel";
}

/** True for controls whose applied text is stored and replayed. */
export function controlStoresAppliedValue(
  maybeControl: SceneControl | undefined,
): boolean {
  return maybeControl?.kind === "preset" || maybeControl?.kind === "range";
}

/** Allowlisted preset id or canonical range token. */
export function maybeAcceptedControlValue(
  control: SceneControl,
  maybeValue: string | undefined,
): string | undefined {
  if (maybeValue === undefined) {
    return undefined;
  }
  if (control.kind === "preset") {
    return control.values.some((option) => option.id === maybeValue)
      ? maybeValue
      : undefined;
  }
  if (control.kind === "range") {
    return maybeParseRangeControlValue(control, maybeValue);
  }

  return undefined;
}

export function initialPresetValue(
  control: Extract<SceneControl, { kind: "preset" }>,
  maybeValues: Readonly<Record<string, string>> | undefined,
): string {
  const maybeCurrent = maybeValues?.[control.id];
  if (
    maybeCurrent !== undefined &&
    control.values.some((value) => value.id === maybeCurrent)
  ) {
    return maybeCurrent;
  }

  const maybeDefault = DEFAULT_PRESET_VALUES[control.id];
  if (
    maybeDefault !== undefined &&
    control.values.some((value) => value.id === maybeDefault)
  ) {
    return maybeDefault;
  }

  return control.values[0]?.id ?? "";
}
