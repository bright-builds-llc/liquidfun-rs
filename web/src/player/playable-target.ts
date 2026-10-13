import { puzzleTitleForId } from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";
import type { SceneId } from "../catalog/scenes";
import type { PlayableRoute } from "../routing/playable-route";
import { DEFAULT_TITLE, maybeReadySceneId, titleForRoute } from "./runtime";

/** Document title of the level-select grid (UI-D26). */
const PUZZLES_TITLE = "Puzzles · liquidfun-rs playground";

/** Something the player can load: a ready scene or an allowlisted level (D-02). */
export type PlayableTarget =
  | { readonly kind: "scene"; readonly id: SceneId }
  | { readonly kind: "puzzle"; readonly id: PuzzleId };

/** Loadable target for a route, or undefined for the grid, empty and unknown hashes. */
export function maybePlayableTarget(route: PlayableRoute): PlayableTarget | undefined {
  if (route.kind === "puzzle") {
    return { kind: "puzzle", id: route.id };
  }
  if (route.kind === "puzzles" || route.kind === "unknown-puzzle") {
    return undefined;
  }

  const maybeId = maybeReadySceneId(route);
  return maybeId === undefined ? undefined : { kind: "scene", id: maybeId };
}

/** True only when both targets exist and share kind and id. */
export function samePlayableTarget(
  maybeA: PlayableTarget | undefined,
  maybeB: PlayableTarget | undefined,
): boolean {
  if (maybeA === undefined || maybeB === undefined) {
    return false;
  }

  return maybeA.kind === maybeB.kind && maybeA.id === maybeB.id;
}

/** Document title for any playable route; scene routes keep their existing titles. */
export function titleForPlayableRoute(route: PlayableRoute): string {
  if (route.kind === "puzzle") {
    return `${puzzleTitleForId(route.id)} · puzzle · liquidfun-rs playground`;
  }
  if (route.kind === "puzzles") {
    return PUZZLES_TITLE;
  }
  if (route.kind === "unknown-puzzle") {
    return DEFAULT_TITLE;
  }

  return titleForRoute(route);
}

/** Stable key that changes whenever the route names a different page. */
export function playableRouteIdentity(route: PlayableRoute): string {
  switch (route.kind) {
    case "scene":
      return `scene:${route.id}`;
    case "unknown":
      return `unknown:${route.maybeRaw}`;
    case "empty":
      return "empty";
    case "puzzles":
      return "puzzles";
    case "puzzle":
      return `puzzle:${route.id}`;
    case "unknown-puzzle":
      return `unknown-puzzle:${route.maybeRaw}`;
    default:
      return assertNever(route);
  }
}

/** `aria-current` of the sidebar Puzzles link: the page itself, or the section on a level route (UI-D24). */
export function puzzlesNavCurrent(route: PlayableRoute): "page" | "true" | undefined {
  if (route.kind === "puzzles") {
    return "page";
  }
  if (route.kind === "puzzle") {
    return "true";
  }

  return undefined;
}

function assertNever(route: never): never {
  throw new Error(`Unhandled playable route: ${JSON.stringify(route)}`);
}
