import type { Point } from "../render/camera";

export type PointerKind = "down" | "move" | "up" | "cancel";

const POINTER_KINDS = new Set<PointerKind>(["down", "move", "up", "cancel"]);

export function parsePointerKind(raw: string): PointerKind | undefined {
  if (!POINTER_KINDS.has(raw as PointerKind)) {
    return undefined;
  }

  return raw as PointerKind;
}

export type ClientRectLike = {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
};

export function cssPointFromClient(
  clientX: number,
  clientY: number,
  rect: ClientRectLike,
): Point | undefined {
  if (
    !Number.isFinite(clientX) ||
    !Number.isFinite(clientY) ||
    !Number.isFinite(rect.left) ||
    !Number.isFinite(rect.top) ||
    !Number.isFinite(rect.width) ||
    !Number.isFinite(rect.height) ||
    rect.width <= 0 ||
    rect.height <= 0
  ) {
    return undefined;
  }

  const point = {
    x: clientX - rect.left,
    y: clientY - rect.top,
  };
  if (!Number.isFinite(point.x) || !Number.isFinite(point.y)) {
    return undefined;
  }

  return point;
}

export type GestureState = {
  readonly maybePointerId: number | undefined;
};

export type PointerEventName =
  | "pointerdown"
  | "pointermove"
  | "pointerup"
  | "pointercancel"
  | "lostpointercapture";

export type GestureDecision = {
  readonly next: GestureState;
  readonly maybeKind: PointerKind | undefined;
  readonly shouldCapture: boolean;
  readonly shouldRelease: boolean;
};

export function emptyGestureState(): GestureState {
  return { maybePointerId: undefined };
}

function ignorePointerEvent(state: GestureState): GestureDecision {
  return {
    next: state,
    maybeKind: undefined,
    shouldCapture: false,
    shouldRelease: false,
  };
}

function finishCapturedGesture(
  state: GestureState,
  pointerId: number,
  maybeKind: PointerKind,
): GestureDecision {
  if (state.maybePointerId !== pointerId) {
    return ignorePointerEvent(state);
  }

  return {
    next: emptyGestureState(),
    maybeKind,
    shouldCapture: false,
    shouldRelease: true,
  };
}

export function reducePointerEvent(
  state: GestureState,
  event: { readonly name: PointerEventName; readonly pointerId: number },
): GestureDecision {
  if (event.name === "pointerdown") {
    return {
      next: { maybePointerId: event.pointerId },
      maybeKind: "down",
      shouldCapture: true,
      shouldRelease: false,
    };
  }

  if (event.name === "pointermove") {
    if (state.maybePointerId !== event.pointerId) {
      return ignorePointerEvent(state);
    }

    return {
      next: state,
      maybeKind: "move",
      shouldCapture: false,
      shouldRelease: false,
    };
  }

  if (event.name === "pointerup") {
    return finishCapturedGesture(state, event.pointerId, "up");
  }

  return finishCapturedGesture(state, event.pointerId, "cancel");
}
