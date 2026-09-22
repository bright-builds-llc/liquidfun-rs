import type { SceneSession } from "../physics/session";
import { unprojectPoint, type Camera, type Point } from "../render/camera";
import {
  cssPointFromClient,
  emptyGestureState,
  reducePointerEvent,
  type GestureState,
  type PointerEventName,
  type PointerKind,
} from "./pointer";

export type CanvasPointerHandlers = {
  cancel(): void;
  detach(): void;
};

const POINTER_EVENT_NAMES = [
  "pointerdown",
  "pointermove",
  "pointerup",
  "pointercancel",
  "lostpointercapture",
] as const satisfies readonly PointerEventName[];

function isPointerEventName(name: string): name is PointerEventName {
  return (POINTER_EVENT_NAMES as readonly string[]).includes(name);
}

function maybeWorldFromEvent(
  canvas: HTMLCanvasElement,
  event: PointerEvent,
  maybeCamera: () => Camera | undefined,
): Point | undefined {
  const rect = canvas.getBoundingClientRect();
  const maybeCss = cssPointFromClient(event.clientX, event.clientY, rect);
  const camera = maybeCamera();
  if (maybeCss === undefined || camera === undefined) {
    return undefined;
  }

  const world = unprojectPoint(camera, maybeCss);
  if (!Number.isFinite(world.x) || !Number.isFinite(world.y)) {
    return undefined;
  }

  return world;
}

export function syncCanvasInteractive(
  maybeCanvas: HTMLCanvasElement | undefined,
  interactive: boolean,
): void {
  maybeCanvas?.classList.toggle("canvas-interactive", interactive);
}

export function forwardScenePointer(
  maybeSession: SceneSession | undefined,
  viewKind: string,
  kind: PointerKind,
  worldX: number,
  worldY: number,
  onAccepted: (kind: PointerKind) => void,
  onAcceptedCount: (update: (count: number) => number) => void,
  onFailure: (error: unknown) => void,
): void {
  if (
    maybeSession === undefined ||
    (viewKind !== "playing" && viewKind !== "paused")
  ) {
    return;
  }

  try {
    maybeSession.pointerAction(kind, worldX, worldY);
    onAccepted(kind);
    onAcceptedCount((count) => count + 1);
  } catch (error) {
    onFailure(error);
  }
}

function releaseCapturedPointer(
  canvas: HTMLCanvasElement,
  pointerId: number,
): void {
  if (!canvas.hasPointerCapture(pointerId)) {
    return;
  }

  canvas.releasePointerCapture(pointerId);
}

/** Attaches one Pointer Events pipeline that unprojects CSS points into world space. */
export function attachCanvasPointer(options: {
  readonly canvas: HTMLCanvasElement;
  readonly maybeCamera: () => Camera | undefined;
  readonly send: (kind: PointerKind, worldX: number, worldY: number) => void;
  readonly panEnabled?: () => boolean;
  readonly onPanBy?: (deltaX: number, deltaY: number) => void;
}): CanvasPointerHandlers {
  const { canvas, maybeCamera, send, panEnabled, onPanBy } = options;
  let gesture: GestureState = emptyGestureState();
  let maybeLastWorld: Point | undefined;
  let maybeLastPanCss: Point | undefined;

  function onPointerEvent(event: PointerEvent): void {
    if (!isPointerEventName(event.type)) {
      return;
    }

    const maybeWorld = maybeWorldFromEvent(canvas, event, maybeCamera);
    const decision = reducePointerEvent(gesture, {
      name: event.type,
      pointerId: event.pointerId,
    });
    gesture = decision.next;

    if (decision.shouldCapture) {
      canvas.setPointerCapture(event.pointerId);
      event.preventDefault();
    }

    if (decision.shouldRelease) {
      releaseCapturedPointer(canvas, event.pointerId);
      maybeLastPanCss = undefined;
    }

    if (panEnabled?.() === true) {
      const rect = canvas.getBoundingClientRect();
      const maybeCss = cssPointFromClient(event.clientX, event.clientY, rect);
      if (decision.shouldCapture) {
        maybeLastPanCss = maybeCss;
      }
      if (
        decision.maybeKind === "move" &&
        maybeCss !== undefined &&
        maybeLastPanCss !== undefined
      ) {
        onPanBy?.(maybeCss.x - maybeLastPanCss.x, maybeCss.y - maybeLastPanCss.y);
        maybeLastPanCss = maybeCss;
      }
      return;
    }

    if (decision.maybeKind === undefined || maybeWorld === undefined) {
      return;
    }

    maybeLastWorld = maybeWorld;
    send(decision.maybeKind, maybeWorld.x, maybeWorld.y);
  }

  function cancel(): void {
    const maybePointerId = gesture.maybePointerId;
    if (maybePointerId === undefined) {
      return;
    }

    const worldX = maybeLastWorld?.x ?? 0;
    const worldY = maybeLastWorld?.y ?? 0;
    const panning = panEnabled?.() === true || maybeLastPanCss !== undefined;
    releaseCapturedPointer(canvas, maybePointerId);
    gesture = emptyGestureState();
    maybeLastWorld = undefined;
    maybeLastPanCss = undefined;
    if (panning) {
      return;
    }

    send("cancel", worldX, worldY);
  }

  function detach(): void {
    cancel();
    for (const name of POINTER_EVENT_NAMES) {
      canvas.removeEventListener(name, onPointerEvent);
    }
  }

  for (const name of POINTER_EVENT_NAMES) {
    canvas.addEventListener(name, onPointerEvent);
  }

  return { cancel, detach };
}
