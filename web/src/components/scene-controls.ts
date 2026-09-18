import type { SceneControl } from "../catalog/scenes";

export const APPLY_SETTING_LABEL = "Apply setting";
export const CONSTRUCTION_RESET_HINT =
  "Changing this setting recreates the scene from its documented initial state.";

export const DEFAULT_PRESET_VALUES: Readonly<Record<string, string>> = {
  "water-amount": "medium",
  gravity: "normal",
  "emission-rate": "medium",
  "launch-speed": "medium",
  "aim-angle": "up",
  body: "cork",
  "mix-strength": "strong",
  "stir-speed": "slow",
  shape: "circle",
  softness: "medium",
  "jet-strength": "medium",
  emission: "on",
};

/** True only for construction presets that recreate the world. */
export function constructionHintVisible(control: SceneControl): boolean {
  return control.kind === "preset" && control.recreates;
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
