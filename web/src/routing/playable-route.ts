import { PUZZLE_IDS, type PuzzleId } from "../catalog/puzzles";
import { maybeParseSceneRoute, normalizeSceneRoute, type SceneRoute } from "./hash";

/** A scene route, an allowlisted puzzle level, or a puzzle hash naming no level. */
export type PlayableRoute =
  | SceneRoute
  | { readonly kind: "puzzle"; readonly id: PuzzleId }
  | { readonly kind: "unknown-puzzle"; readonly maybeRaw: string };

export type NormalizedPlayableRoute = {
  readonly route: PlayableRoute;
  readonly maybeReplacementHash: string | undefined;
};

/**
 * Parses `#/puzzle/<id>` against the level allowlist and delegates every
 * other hash to the unchanged scene parser (D-01, D-03). The raw text of an
 * unknown puzzle hash is kept verbatim for text-only rendering.
 */
export function maybeParsePlayableRoute(hash: string): PlayableRoute {
  const parts = hash
    .replace(/^#/, "")
    .split("/")
    .filter((part) => part.length > 0);
  if (parts[0] !== "puzzle") {
    return maybeParseSceneRoute(hash);
  }

  const maybeId = parts.length === 2 ? PUZZLE_IDS.find((id) => id === parts[1]) : undefined;
  if (maybeId === undefined) {
    return { kind: "unknown-puzzle", maybeRaw: parts.slice(1).join("/") };
  }

  return { kind: "puzzle", id: maybeId };
}

/** Like `normalizeSceneRoute`, but puzzle routes never get a replacement hash. */
export function normalizePlayableRoute(hash: string): NormalizedPlayableRoute {
  const route = maybeParsePlayableRoute(hash);
  if (route.kind === "puzzle" || route.kind === "unknown-puzzle") {
    return { route, maybeReplacementHash: undefined };
  }

  return normalizeSceneRoute(hash);
}
