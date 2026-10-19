import { option } from "./scene-record-shared";
import type { SceneControl } from "./scenes";

/** Recreating preset: picking a ratio rebuilds the scene with that extra weight on the heavy layer. */
export const WEIGHT_RATIO_CONTROL: SceneControl = {
  id: "weight-ratio",
  label: "Weight ratio",
  kind: "preset",
  recreates: true,
  values: [option("1.2", "1.2×"), option("1.5", "1.5×"), option("2", "2×"), option("3", "3×")],
};
