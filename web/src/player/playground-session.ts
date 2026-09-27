import { createEffect, createSignal, onCleanup } from "solid-js";

import { syncCanvasInteractive } from "../input/canvas-pointer";
import { maybeRouteGravitySliderMagnitude } from "../input/gravity-slider";
import { createTiltBinding } from "../input/tilt-binding";
import type { PointerKind } from "../input/pointer";
import type { RenderFrame } from "../physics/frame";
import { createFrameClock } from "./frame-loop";
import { isReadySceneRoute, titleForRoute } from "./runtime";
import { playerStatus, type PlayerView } from "./view";
import { createAppearancePreferences } from "./appearance";
import type { FpsTick } from "../components/fps-meter";
import {
  DEFAULT_RENDERED_PARTICLE_LIMIT,
} from "../render/particle-limit";
import { readCanvasStage } from "./fullscreen-capability";
import { bindVisualViewport } from "./visual-viewport";
import { normalizeSceneRoute } from "../routing/hash";
import type { SceneRuntime } from "./scene-runtime";
import {
  bindLifecycle,
  handleHashChange,
  handleVisibilityChange,
} from "./scene-lifecycle";
import { bindSurface } from "./scene-surface";

function createShell(): SceneRuntime {
  const initialRoute = normalizeSceneRoute(window.location.hash);
  if (initialRoute.maybeReplacementHash !== undefined) {
    window.history.replaceState(
      window.history.state,
      "",
      initialRoute.maybeReplacementHash,
    );
  }

  const [route, setRoute] = createSignal(initialRoute.route);
  const [view, setView] = createSignal<PlayerView>(
    isReadySceneRoute(initialRoute.route)
      ? { kind: "loading" }
      : { kind: "fallback" },
  );
  const canvasStage = readCanvasStage(document.fullscreenEnabled, navigator);
  const [debugEnabled, setDebugEnabled] = createSignal(!canvasStage);
  const [maybeDebugFrame, setMaybeDebugFrame] = createSignal<
    RenderFrame | undefined
  >();
  const [stepsThisFrame, setStepsThisFrame] = createSignal(0);
  const [fpsTicks, setFpsTicks] = createSignal<readonly FpsTick[]>([]);
  const [lastPointerKind, setLastPointerKind] = createSignal<
    PointerKind | undefined
  >();
  const [pointerAccepted, setPointerAccepted] = createSignal(0);
  const [resetGeneration, setResetGeneration] = createSignal(1);
  const [renderedParticleDraft, setRenderedParticleDraft] = createSignal(
    String(DEFAULT_RENDERED_PARTICLE_LIMIT),
  );
  const [maxRenderedParticles, setMaxRenderedParticles] = createSignal(
    DEFAULT_RENDERED_PARTICLE_LIMIT,
  );
  const [panEnabled, setPanEnabled] = createSignal(false);
  const [tiltGravityEnabled, setTiltGravityEnabled] = createSignal(false);
  const [tiltDebug, setTiltDebug] = createSignal({ kind: "idle" as const });
  const releaseVisualViewport = canvasStage
    ? bindVisualViewport(document.documentElement)
    : undefined;
  if (canvasStage) {
    document.documentElement.dataset.canvasStage = "true";
  }

  const session = {
    generation: 0,
    constructionValues: {},
    maybeCanvas: undefined,
    maybeParticleCanvas: undefined,
    maybeContext: undefined,
    maybeSession: undefined,
    maybeCanvasPointer: undefined,
    clock: createFrameClock(),
    tiltBinding: createTiltBinding(),
    canvasStage,
    releaseVisualViewport,
    route,
    setRoute,
    view,
    setView,
    debugEnabled,
    setDebugEnabled,
    maybeDebugFrame,
    setMaybeDebugFrame,
    stepsThisFrame,
    setStepsThisFrame,
    fpsTicks,
    setFpsTicks,
    lastPointerKind,
    setLastPointerKind,
    pointerAccepted,
    setPointerAccepted,
    resetGeneration,
    setResetGeneration,
    renderedParticleDraft,
    setRenderedParticleDraft,
    maxRenderedParticles,
    setMaxRenderedParticles,
    panEnabled,
    setPanEnabled,
    tiltGravityEnabled,
    setTiltGravityEnabled,
    tiltDebug,
    setTiltDebug,
  } as SceneRuntime;
  session.gravitySliderMagnitude = () =>
    maybeRouteGravitySliderMagnitude(session.route(), session.constructionValues);
  return session;
}

function bindNavigation(session: SceneRuntime): void {
  const onHashChange = () => handleHashChange(session);
  const onVisibilityChange = () => handleVisibilityChange(session);
  window.addEventListener("hashchange", onHashChange);
  document.addEventListener("visibilitychange", onVisibilityChange);

  createEffect(() => {
    syncCanvasInteractive(
      session.maybeCanvas,
      playerStatus(session.view()) === "playing" ||
        playerStatus(session.view()) === "paused",
    );
  });

  createEffect(() => {
    const currentRoute = session.route();
    document.title = titleForRoute(currentRoute);
    if (isReadySceneRoute(currentRoute)) {
      if (session.view().kind === "fallback") {
        session.setView({ kind: "loading" });
      }
      return;
    }

    session.abandonScene();
  });

  onCleanup(() => {
    session.releaseVisualViewport?.();
    delete document.documentElement.dataset.canvasStage;
    window.removeEventListener("hashchange", onHashChange);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    session.tiltBinding.stop?.();
    session.abandonScene();
  });
}

/** Builds the one-session playground and registers its browser listeners. */
export function createPlaygroundSession(): SceneRuntime {
  const session = createShell();
  bindLifecycle(session);
  bindSurface(session);
  session.appearance = createAppearancePreferences(
    () => window.localStorage,
    () => {
      session.paintHeldFrame();
    },
  );
  bindNavigation(session);
  return session;
}
