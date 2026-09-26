import type { SceneRecord } from "../catalog/scenes";
import { maybeAcceptedControlValue } from "../components/scene-controls";

export type PresetEntry = {
  readonly name: string;
  readonly value: string;
};

/**
 * Presets the user has changed, with world-recreating controls first.
 *
 * A fresh export session already starts from the scene defaults. Replaying a
 * recreating control after a live one would rebuild the world and drop the
 * live change, so construction presets are applied before runtime presets.
 */
export function changedPresetEntries(
  scene: SceneRecord,
  values: Readonly<Record<string, string>>,
): readonly PresetEntry[] {
  const recreating: PresetEntry[] = [];
  const live: PresetEntry[] = [];

  for (const control of scene.controls) {
    const maybeValue = maybeAcceptedControlValue(control, values[control.id]);
    if (maybeValue === undefined) {
      continue;
    }

    const entry = { name: control.id, value: maybeValue };
    if (control.recreates) {
      recreating.push(entry);
      continue;
    }

    live.push(entry);
  }

  return [...recreating, ...live];
}
