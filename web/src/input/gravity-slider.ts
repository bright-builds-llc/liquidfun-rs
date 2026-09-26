import {
  isReadySceneId,
  maybeSceneById,
  type SceneRecord,
} from "../catalog/scenes";
import { initialRangeValue } from "../components/range-control";
import type { SceneRoute } from "../routing/hash";

/**
 * Current gravity-slider magnitude in m/s², when the scene has that control.
 *
 * An empty or rejected value uses the control's default magnitude.
 */
export function maybeGravitySliderMagnitude(
  maybeScene: SceneRecord | undefined,
  values: Readonly<Record<string, string>>,
): number | undefined {
  if (maybeScene === undefined) {
    return undefined;
  }

  const maybeControl = maybeScene.controls.find(
    (control) => control.kind === "range" && control.id === "gravity",
  );
  if (maybeControl === undefined || maybeControl.kind !== "range") {
    return undefined;
  }

  const magnitude = Number(initialRangeValue(maybeControl, values));
  if (!Number.isFinite(magnitude) || magnitude < 0) {
    return undefined;
  }
  return magnitude;
}

/** Gravity-slider magnitude for the current route, in m/s². */
export function maybeRouteGravitySliderMagnitude(
  route: SceneRoute,
  values: Readonly<Record<string, string>>,
): number | undefined {
  if (route.kind !== "scene" || !isReadySceneId(route.id)) {
    return undefined;
  }
  return maybeGravitySliderMagnitude(maybeSceneById(route.id), values);
}
