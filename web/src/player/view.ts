import type { FrameObservation } from "./observe";

export type PlayerStatus = "loading" | "playing" | "paused" | "failed";

export type PlayerView =
  | { readonly kind: "fallback" }
  | { readonly kind: "loading" }
  | { readonly kind: "playing"; readonly frame: FrameObservation }
  | { readonly kind: "paused"; readonly frame: FrameObservation }
  | {
      readonly kind: "failure";
      readonly maybeFrame: FrameObservation | undefined;
      readonly maybeDetails: string | undefined;
    };

/** Returns the latest observation carried by a player view, when present. */
export function maybeObservedFrame(
  view: PlayerView,
): FrameObservation | undefined {
  if (view.kind === "playing" || view.kind === "paused") {
    return view.frame;
  }

  if (view.kind === "failure") {
    return view.maybeFrame;
  }

  return undefined;
}

/** Maps application player state to shared player chrome status. */
export function playerStatus(view: PlayerView): PlayerStatus {
  switch (view.kind) {
    case "playing":
      return "playing";
    case "paused":
      return "paused";
    case "failure":
      return "failed";
    default:
      return "loading";
  }
}
