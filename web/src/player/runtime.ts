import {
  isReadySceneId,
  maybeSceneById,
  type SceneId,
  type SceneRecord,
} from "../catalog/scenes";
import type { SceneRoute } from "../routing/hash";

export const PAGE_SUMMARY =
  "Play experimental Rust physics scenes in the browser. All six demos run this repository's engine through WebAssembly.";
export const DEFAULT_TITLE = "liquidfun-rs playground";
export const MAX_ERROR_DETAIL_LENGTH = 240;

/** True when the hash names an allowlisted ready scene. */
export function isReadySceneRoute(route: SceneRoute): boolean {
  return route.kind === "scene" && isReadySceneId(route.id);
}

/** Ready scene id from a route, or undefined for empty/unknown hashes. */
export function maybeReadySceneId(route: SceneRoute): SceneId | undefined {
  if (route.kind !== "scene" || !isReadySceneId(route.id)) {
    return undefined;
  }

  return route.id;
}

/** Document title for the current hash route. */
export function titleForRoute(route: SceneRoute): string {
  if (route.kind !== "scene") {
    return DEFAULT_TITLE;
  }

  const maybeScene = maybeSceneById(route.id);
  if (maybeScene === undefined) {
    return DEFAULT_TITLE;
  }

  return `${maybeScene.title} · liquidfun-rs playground`;
}

/** Locked player heading for a catalog id. */
export function sceneTitleForId(id: SceneId): string {
  return maybeSceneById(id)?.title ?? id;
}

export function sceneControlsIdentity(
  sceneId: SceneId,
  generation: number,
): string {
  return `${sceneId}:${generation}`;
}

/** Last applied construction presets until Reset or scene switch clears the bag. */
export function constructionEntriesForScene(
  scene: SceneRecord,
  values: Readonly<Record<string, string>>,
): readonly { readonly name: string; readonly value: string }[] {
  return scene.controls.flatMap((control) => {
    if (control.kind !== "preset" || !control.recreates) {
      return [];
    }

    const maybeValue = values[control.id];
    if (maybeValue === undefined) {
      return [];
    }

    return [{ name: control.id, value: maybeValue }];
  });
}

export function maybeDevelopmentDetails(error: unknown): string | undefined {
  if (!import.meta.env.DEV) {
    return undefined;
  }

  const message =
    error instanceof Error ? error.message : "Unknown scene failure";
  return `Details: ${message.slice(0, MAX_ERROR_DETAIL_LENGTH)}`;
}
