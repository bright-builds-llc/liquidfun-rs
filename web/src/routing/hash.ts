import { SCENE_IDS, type SceneId } from "../catalog/scenes";

export type SceneRoute =
  | { readonly kind: "scene"; readonly id: SceneId }
  | { readonly kind: "empty" }
  | { readonly kind: "unknown"; readonly maybeRaw: string };

/** First catalog scene. An empty playground hash opens this demo. */
export const DEFAULT_SCENE_ID: SceneId = firstSceneId();
export const DEFAULT_SCENE_HASH = `#/scene/${DEFAULT_SCENE_ID}`;

function firstSceneId(): SceneId {
  const maybeId = SCENE_IDS[0];
  if (maybeId === undefined) {
    throw new Error("The scene catalog is empty.");
  }

  return maybeId;
}

export type NormalizedSceneRoute = {
  readonly route: SceneRoute;
  readonly maybeReplacementHash: string | undefined;
};

export function normalizeSceneRoute(hash: string): NormalizedSceneRoute {
  const route = maybeParseSceneRoute(hash);
  if (route.kind === "empty") {
    return {
      route: { kind: "scene", id: DEFAULT_SCENE_ID },
      maybeReplacementHash: DEFAULT_SCENE_HASH,
    };
  }

  return {
    route,
    maybeReplacementHash: undefined,
  };
}

export function maybeParseSceneRoute(hash: string): SceneRoute {
  const parts = hash
    .replace(/^#/, "")
    .split("/")
    .filter((part) => part.length > 0);
  if (parts.length === 0) {
    return { kind: "empty" };
  }

  if (parts[0] === "scene" && parts.length === 1) {
    return { kind: "empty" };
  }

  if (parts[0] !== "scene" || parts.length !== 2) {
    return { kind: "unknown", maybeRaw: parts.join("/") };
  }

  const maybeId = SCENE_IDS.find((id) => id === parts[1]);
  if (maybeId === undefined) {
    return { kind: "unknown", maybeRaw: parts[1] ?? "" };
  }

  return { kind: "scene", id: maybeId };
}
