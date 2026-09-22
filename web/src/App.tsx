import { createEffect, createSignal, onCleanup } from "solid-js";

import { maybeSceneById, type SceneId } from "./catalog/scenes";
import { PlaygroundStage } from "./components/PlaygroundStage";
import {
  attachCanvasPointer,
  forwardScenePointer,
  syncCanvasInteractive,
  type CanvasPointerHandlers,
} from "./input/canvas-pointer";
import {
  changeTiltGravity,
  createTiltBinding,
} from "./input/tilt-binding";
import type { TiltDebug } from "./input/tilt-gravity";
import type { PointerKind } from "./input/pointer";
import type { RenderFrame } from "./physics/frame";
import { loadSceneSession } from "./physics/loader";
import { createSceneSession, type SceneSession } from "./physics/session";
import {
  cancelPendingFrame,
  connectResizeObserver,
  createFrameClock,
  disconnectResizeObserver,
  presentOwnedFrame,
  scheduleFrame,
  type FrameLoopDeps,
} from "./player/frame-loop";
import { isStaleGeneration, nextGeneration } from "./player/generation";
import {
  constructionEntriesForScene,
  isReadySceneRoute,
  maybeDevelopmentDetails,
  maybeReadySceneId,
  titleForRoute,
} from "./player/runtime";
import { isUsableViewport } from "./player/viewport";
import { maybeObservedFrame, playerStatus, type PlayerView } from "./player/view";
import type { FpsTick } from "./components/fps-meter";
import { drawRenderFrame } from "./render/canvas";
import {
  IDENTITY_CAMERA_VIEW,
  createCamera,
  zoomCameraView,
  type Camera,
  type CameraView,
} from "./render/camera";
import {
  DEFAULT_RENDERED_PARTICLE_LIMIT,
  maybeParseRenderedParticleLimit,
} from "./render/particle-limit";
import { loadRenderMode, persistRenderMode, type RenderMode } from "./render/mode";
import {
  loadWireframeStrokeWidth,
  persistWireframeStrokeWidth,
} from "./render/stroke-width";
import { normalizeSceneRoute } from "./routing/hash";

/** One-session playground shell with hash routing and bounded playback. */
export function App() {
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
  const [renderMode, setRenderMode] = createSignal<RenderMode>(
    loadRenderMode(() => window.localStorage),
  );
  const [wireframeStrokeWidth, setWireframeStrokeWidth] = createSignal(
    loadWireframeStrokeWidth(() => window.localStorage),
  );
  const [debugEnabled, setDebugEnabled] = createSignal(true);
  const [maybeDebugFrame, setMaybeDebugFrame] = createSignal<
    RenderFrame | undefined
  >();
  const [stepsThisFrame, setStepsThisFrame] = createSignal(0);
  const [fpsTicks, setFpsTicks] = createSignal<readonly FpsTick[]>([]);
  const [lastPointerKind, setLastPointerKind] =
    createSignal<PointerKind | undefined>();
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
  const [tiltDebug, setTiltDebug] = createSignal<TiltDebug>({ kind: "idle" });

  let generation = 0;
  let constructionValues: Record<string, string> = {};
  let maybeCanvas: HTMLCanvasElement | undefined;
  let maybeContext: CanvasRenderingContext2D | undefined;
  let maybeSession: SceneSession | undefined;
  const clock = createFrameClock();
  const tiltBinding = createTiltBinding();

  function frameDeps(): FrameLoopDeps {
    return {
      view,
      fail,
      maybeSession: () => maybeSession,
      route,
      startScene,
      drawSceneFrame,
      setMaybeDebugFrame,
      setStepsThisFrame,
      setFpsTicks,
      setView,
    };
  }

  let maybeCanvasPointer: CanvasPointerHandlers | undefined;

  function incrementGeneration(): number {
    generation = nextGeneration(generation);
    return generation;
  }

  function disposeOwnedSession(): void {
    maybeCanvasPointer?.cancel();
    const maybeOwnedSession = maybeSession;
    maybeSession = undefined;
    clock.maybeLastTimestamp = undefined;
    if (maybeOwnedSession === undefined) {
      return;
    }

    try {
      maybeOwnedSession.dispose();
    } catch {
      // Disposal remains terminal even if generated cleanup reports a failure.
    }
  }

  function abandonScene(): void {
    incrementGeneration();
    maybeCanvasPointer?.detach();
    maybeCanvasPointer = undefined;
    disconnectResizeObserver(clock);
    cancelPendingFrame(clock);
    disposeOwnedSession();
    maybeCanvas = undefined;
    maybeContext = undefined;
    clock.maybeCamera = undefined;
    clock.maybePreviousFrame = undefined;
    setMaybeDebugFrame(undefined);
    setStepsThisFrame(0);
    constructionValues = {};
    setView({ kind: "fallback" });
  }

  function fail(error: unknown): void {
    const maybeFrame = maybeObservedFrame(view());
    cancelPendingFrame(clock);
    disposeOwnedSession();
    setView({
      kind: "failure",
      maybeFrame,
      maybeDetails: maybeDevelopmentDetails(error),
    });
  }

  function paintHeldFrame(): void {
    const maybeFrame = clock.maybePreviousFrame;
    const context = maybeContext;
    const camera = clock.maybeCamera;
    if (
      maybeFrame === undefined ||
      context === undefined ||
      camera === undefined
    ) {
      return;
    }

    drawSceneFrame(context, maybeFrame, camera);
  }

  function drawSceneFrame(
    context: CanvasRenderingContext2D,
    frame: RenderFrame,
    camera: Camera,
  ): void {
    drawRenderFrame(
      context,
      frame,
      camera,
      renderMode(),
      wireframeStrokeWidth(),
      maxRenderedParticles(),
    );
  }

  function refreshCamera(): void {
    if (!isUsableViewport(clock.viewportWidth, clock.viewportHeight)) {
      return;
    }

    clock.maybeCamera = createCamera(clock.viewportWidth, clock.viewportHeight, clock.cameraView);
  }

  function changeRenderMode(nextMode: RenderMode): void {
    setRenderMode(nextMode);
    persistRenderMode(() => window.localStorage, nextMode);
    paintHeldFrame();
  }

  function changeWireframeStrokeWidth(nextWidth: number): void {
    setWireframeStrokeWidth(nextWidth);
    persistWireframeStrokeWidth(() => window.localStorage, nextWidth);
    paintHeldFrame();
  }

  function changeRenderedParticleDraft(raw: string): void {
    setRenderedParticleDraft(raw);
    const maybeLimit = maybeParseRenderedParticleLimit(raw);
    if (maybeLimit === undefined) {
      return;
    }

    setMaxRenderedParticles(maybeLimit);
    paintHeldFrame();
  }

  function applyCameraView(nextView: CameraView): void {
    clock.cameraView = nextView;
    refreshCamera();
    paintHeldFrame();
  }

  function zoomBy(direction: "in" | "out"): void {
    applyCameraView(zoomCameraView(clock.cameraView, direction));
  }

  function resetCameraView(): void {
    applyCameraView(IDENTITY_CAMERA_VIEW);
  }

  function changePanEnabled(enabled: boolean): void {
    setPanEnabled(enabled);
    maybeCanvas?.classList.toggle("canvas-panning", enabled);
    if (!enabled) {
      maybeCanvasPointer?.cancel();
    }
  }

  function panBy(deltaX: number, deltaY: number): void {
    applyCameraView({
      zoom: clock.cameraView.zoom,
      panX: clock.cameraView.panX + deltaX,
      panY: clock.cameraView.panY + deltaY,
    });
  }

  async function startScene(id: SceneId): Promise<void> {
    const started = incrementGeneration();
    cancelPendingFrame(clock);
    disposeOwnedSession();
    clock.maybePreviousFrame = undefined;
    setMaybeDebugFrame(undefined);
    setStepsThisFrame(0);
    setFpsTicks([]);
    clock.maybeLastTimestamp = undefined;
    clock.cameraView = IDENTITY_CAMERA_VIEW;
    refreshCamera();
    setView({ kind: "loading" });

    const canvas = maybeCanvas;
    const context = maybeContext;
    if (canvas === undefined || context === undefined) {
      fail(new Error("Canvas element is unavailable"));
      return;
    }

    try {
      const generatedSession = await loadSceneSession(id);
      if (isStaleGeneration(started, generation)) {
        createSceneSession(generatedSession).dispose();
        return;
      }

      const ownedSession = createSceneSession(generatedSession);
      if (isStaleGeneration(started, generation)) {
        ownedSession.dispose();
        return;
      }

      const maybeScene = maybeSceneById(id);
      if (maybeScene !== undefined) {
        for (const entry of constructionEntriesForScene(
          maybeScene,
          constructionValues,
        )) {
          ownedSession.applyControl(entry.name, entry.value);
        }
      }

      maybeSession = ownedSession;
      presentOwnedFrame(clock, ownedSession, context, true, frameDeps());
    } catch (error) {
      if (isStaleGeneration(started, generation)) {
        return;
      }

      fail(error);
    }
  }

  function sendPointer(kind: PointerKind, worldX: number, worldY: number): void {
    forwardScenePointer(maybeSession, view().kind, kind, worldX, worldY, setLastPointerKind, setPointerAccepted, fail);
  }

  function assignCanvas(canvas: HTMLCanvasElement): void {
    maybeCanvasPointer?.detach();
    maybeCanvas = canvas;
    maybeCanvasPointer = attachCanvasPointer({
      canvas,
      maybeCamera: () => clock.maybeCamera,
      send: sendPointer,
      panEnabled: () => panEnabled(),
      onPanBy: panBy,
    });

    const maybeNextContext = canvas.getContext("2d");
    if (maybeNextContext === null) {
      fail(new Error("Canvas 2D is unavailable"));
      return;
    }

    maybeContext = maybeNextContext;
    connectResizeObserver(clock, canvas, maybeNextContext, frameDeps());
  }

  function playScene(): void {
    const current = view();
    if (current.kind !== "paused") {
      return;
    }

    const context = maybeContext;
    if (context === undefined) {
      fail(new Error("Canvas 2D is unavailable"));
      return;
    }

    clock.maybeLastTimestamp = undefined;
    setView({ kind: "playing", frame: current.frame });
    scheduleFrame(clock, context, frameDeps());
  }

  function pauseScene(): void {
    const current = view();
    if (current.kind !== "playing") {
      return;
    }

    maybeCanvasPointer?.cancel();
    cancelPendingFrame(clock);
    clock.maybeLastTimestamp = undefined;
    setStepsThisFrame(0);
    setView({ kind: "paused", frame: current.frame });
  }

  function recreateScene(): void {
    const maybeReadyId = maybeReadySceneId(route());
    if (maybeReadyId === undefined) {
      return;
    }

    constructionValues = {};
    setResetGeneration((current) => current + 1);
    void startScene(maybeReadyId);
  }

  function applySceneControl(name: string, value: string): void {
    const maybeOwnedSession = maybeSession;
    const context = maybeContext;
    const maybeReadyId = maybeReadySceneId(route());
    if (
      maybeOwnedSession === undefined ||
      context === undefined ||
      maybeReadyId === undefined
    ) {
      fail(new Error("Scene session owner is unavailable"));
      return;
    }

    const maybeControl = maybeSceneById(maybeReadyId)?.controls.find(
      (control) => control.id === name,
    );
    const recreates =
      maybeControl?.kind === "preset" && maybeControl.recreates;

    try {
      if (recreates) {
        constructionValues = { ...constructionValues, [name]: value };
        maybeCanvasPointer?.cancel();
        cancelPendingFrame(clock);
        setView({ kind: "loading" });
      }

      const recreated = maybeOwnedSession.applyControl(name, value);
      if (!recreated) {
        return;
      }

      clock.maybePreviousFrame = undefined;
      setMaybeDebugFrame(undefined);
      setStepsThisFrame(0);
      setFpsTicks([]);
      clock.maybeLastTimestamp = undefined;
      presentOwnedFrame(clock, maybeOwnedSession, context, true, frameDeps());
    } catch (error) {
      fail(error);
    }
  }

  function applySceneAction(name: string): void {
    const maybeOwnedSession = maybeSession;
    const context = maybeContext;
    if (maybeOwnedSession === undefined || context === undefined) {
      fail(new Error("Scene session owner is unavailable"));
      return;
    }

    try {
      maybeOwnedSession.applyAction(name);
      if (view().kind !== "paused") {
        return;
      }

      presentOwnedFrame(clock, maybeOwnedSession, context, false, frameDeps());
      const maybeFrame = maybeObservedFrame(view());
      if (maybeFrame !== undefined && view().kind === "playing") {
        cancelPendingFrame(clock);
        setView({ kind: "paused", frame: maybeFrame });
      }
    } catch (error) {
      fail(error);
    }
  }

  function onHashChange(): void {
    const previousRoute = route();
    const normalized = normalizeSceneRoute(window.location.hash);
    if (normalized.maybeReplacementHash !== undefined) {
      window.history.replaceState(
        window.history.state,
        "",
        normalized.maybeReplacementHash,
      );
    }
    const nextRoute = normalized.route;
    setRoute(nextRoute);

    const maybeNextId = maybeReadySceneId(nextRoute);
    if (maybeNextId === undefined) {
      abandonScene();
      return;
    }

    if (maybeReadySceneId(previousRoute) === maybeNextId) {
      return;
    }

    constructionValues = {};
    if (maybeCanvas !== undefined && maybeContext !== undefined) {
      void startScene(maybeNextId);
      return;
    }

    setView({ kind: "loading" });
  }

  function onVisibilityChange(): void {
    clock.maybeLastTimestamp = undefined;
    if (document.hidden) {
      maybeCanvasPointer?.cancel();
    }
  }

  window.addEventListener("hashchange", onHashChange);
  document.addEventListener("visibilitychange", onVisibilityChange);

  createEffect(() => {
    syncCanvasInteractive(maybeCanvas, playerStatus(view()) === "playing" || playerStatus(view()) === "paused");
  });

  createEffect(() => {
    const currentRoute = route();
    document.title = titleForRoute(currentRoute);
    if (isReadySceneRoute(currentRoute)) {
      if (view().kind === "fallback") {
        setView({ kind: "loading" });
      }
      return;
    }

    abandonScene();
  });

  onCleanup(() => {
    window.removeEventListener("hashchange", onHashChange);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    tiltBinding.stop?.();
    abandonScene();
  });

  return (
    <PlaygroundStage
      route={route}
      view={view}
      lastPointerKind={lastPointerKind}
      pointerAccepted={pointerAccepted}
      renderMode={renderMode}
      wireframeStrokeWidth={wireframeStrokeWidth}
      renderedParticleDraft={renderedParticleDraft}
      panEnabled={panEnabled}
      tiltGravityEnabled={tiltGravityEnabled}
      tiltDebug={tiltDebug}
      debugEnabled={debugEnabled}
      maybeDebugFrame={maybeDebugFrame}
      stepsThisFrame={stepsThisFrame}
      fpsTicks={fpsTicks}
      resetGeneration={resetGeneration}
      constructionValues={constructionValues}
      assignCanvas={assignCanvas}
      onPlay={playScene}
      onPause={pauseScene}
      onReset={recreateScene}
      onRenderModeChange={changeRenderMode}
      onWireframeStrokeWidthChange={changeWireframeStrokeWidth}
      onDebugEnabledChange={setDebugEnabled}
      onRenderedParticleDraft={changeRenderedParticleDraft}
      onZoomIn={() => zoomBy("in")}
      onZoomOut={() => zoomBy("out")}
      onResetZoom={resetCameraView}
      onPanEnabledChange={changePanEnabled}
      onTiltGravityEnabledChange={(enabled) => {
        void changeTiltGravity(
          tiltBinding,
          enabled,
          () => maybeSession,
          setTiltGravityEnabled,
          setTiltDebug,
        );
      }}
      onApplyControl={applySceneControl}
      onApplyAction={applySceneAction}
    />
  );
}
