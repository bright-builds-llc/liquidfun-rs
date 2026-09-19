import { createEffect, createSignal, onCleanup, Show } from "solid-js";

import { maybeSceneById, type SceneId } from "./catalog/scenes";
import { CatalogNav } from "./components/CatalogNav";
import {
  FallbackPanel,
  type FallbackPanelProps,
} from "./components/FallbackPanel";
import { PlayerPanel, type PlayerStatus } from "./components/PlayerPanel";
import { SceneControls } from "./components/SceneControls";
import { SceneCredits } from "./components/SceneCredits";
import { SiteFooter } from "./components/SiteFooter";
import {
  attachCanvasPointer,
  forwardScenePointer,
  syncCanvasInteractive,
  type CanvasPointerHandlers,
} from "./input/canvas-pointer";
import type { PointerKind } from "./input/pointer";
import { acceptedStepCount } from "./physics/clock";
import type { RenderFrame } from "./physics/frame";
import { loadSceneSession } from "./physics/loader";
import {
  createSceneSession,
  type SceneSession,
} from "./physics/session";
import { isStaleGeneration, nextGeneration } from "./player/generation";
import { observeFrame, type FrameObservation } from "./player/observe";
import {
  PAGE_HEADING,
  constructionEntriesForScene,
  isReadySceneRoute,
  maybeDevelopmentDetails,
  maybeReadySceneId,
  sceneTitleForId,
  titleForRoute,
} from "./player/runtime";
import {
  drawRenderFrame,
  resizeCanvasBackingStore,
} from "./render/canvas";
import type { Camera } from "./render/camera";
import { maybeParseSceneRoute, type SceneRoute } from "./routing/hash";

const MILLISECONDS_PER_SECOND = 1000;
const PAGE_SUMMARY =
  "Play experimental Rust physics scenes in the browser. All six demos run this repository's engine through WebAssembly.";

type PlayerView =
  | { readonly kind: "fallback" }
  | { readonly kind: "loading" }
  | { readonly kind: "playing"; readonly frame: FrameObservation }
  | { readonly kind: "paused"; readonly frame: FrameObservation }
  | {
      readonly kind: "failure";
      readonly maybeFrame: FrameObservation | undefined;
      readonly maybeDetails: string | undefined;
    };

function fallbackProps(route: SceneRoute): FallbackPanelProps {
  if (route.kind === "empty") {
    return { kind: "empty" };
  }

  if (route.kind === "unknown") {
    return { kind: "unknown" };
  }

  const maybeScene = maybeSceneById(route.id);
  return {
    kind: "not-ready",
    sceneTitle: maybeScene?.title ?? route.id,
  };
}

function maybeObservedFrame(view: PlayerView): FrameObservation | undefined {
  if (view.kind === "playing" || view.kind === "paused") {
    return view.frame;
  }

  if (view.kind === "failure") {
    return view.maybeFrame;
  }

  return undefined;
}

function playerStatus(view: PlayerView): PlayerStatus {
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

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function isUsableViewport(width: number, height: number): boolean {
  return Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0;
}

/** One-session playground shell with hash routing and bounded playback. */
export function App() {
  const [route, setRoute] = createSignal(
    maybeParseSceneRoute(window.location.hash),
  );
  const [view, setView] = createSignal<PlayerView>(
    isReadySceneRoute(maybeParseSceneRoute(window.location.hash))
      ? { kind: "loading" }
      : { kind: "fallback" },
  );
  const [lastPointerKind, setLastPointerKind] =
    createSignal<PointerKind | undefined>();
  const [pointerAccepted, setPointerAccepted] = createSignal(0);

  let generation = 0;
  let constructionValues: Record<string, string> = {};
  let maybeCanvas: HTMLCanvasElement | undefined;
  let maybeContext: CanvasRenderingContext2D | undefined;
  let maybeSession: SceneSession | undefined;
  let maybeAnimationFrameId: number | undefined;
  let maybeLastTimestamp: number | undefined;
  let maybePreviousFrame: RenderFrame | undefined;
  let maybeCamera: Camera | undefined;
  let maybeResizeObserver: ResizeObserver | undefined;
  let maybeCanvasPointer: CanvasPointerHandlers | undefined;

  function incrementGeneration(): number {
    generation = nextGeneration(generation);
    return generation;
  }

  function cancelPendingFrame(): void {
    if (maybeAnimationFrameId === undefined) {
      return;
    }

    cancelAnimationFrame(maybeAnimationFrameId);
    maybeAnimationFrameId = undefined;
  }

  function disconnectResizeObserver(): void {
    const maybeObserver = maybeResizeObserver;
    maybeResizeObserver = undefined;
    maybeObserver?.disconnect();
  }

  function disposeOwnedSession(): void {
    maybeCanvasPointer?.cancel();
    const maybeOwnedSession = maybeSession;
    maybeSession = undefined;
    maybeLastTimestamp = undefined;
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
    disconnectResizeObserver();
    cancelPendingFrame();
    disposeOwnedSession();
    maybeCanvas = undefined;
    maybeContext = undefined;
    maybeCamera = undefined;
    maybePreviousFrame = undefined;
    constructionValues = {};
    setView({ kind: "fallback" });
  }

  function fail(error: unknown): void {
    const maybeFrame = maybeObservedFrame(view());
    cancelPendingFrame();
    disposeOwnedSession();
    setView({
      kind: "failure",
      maybeFrame,
      maybeDetails: maybeDevelopmentDetails(error),
    });
  }

  function scheduleFrame(context: CanvasRenderingContext2D): void {
    maybeAnimationFrameId = requestAnimationFrame((timestamp) => {
      maybeAnimationFrameId = undefined;
      if (view().kind !== "playing") {
        return;
      }

      if (document.hidden) {
        maybeLastTimestamp = undefined;
        scheduleFrame(context);
        return;
      }

      const maybeOwnedSession = maybeSession;
      if (maybeOwnedSession === undefined) {
        fail(new Error("Scene session owner is unavailable"));
        return;
      }

      const camera = maybeCamera;
      if (camera === undefined) {
        fail(new Error("Canvas camera is unavailable"));
        return;
      }

      const maybePreviousTimestamp = maybeLastTimestamp;
      maybeLastTimestamp = timestamp;
      if (maybePreviousTimestamp === undefined) {
        scheduleFrame(context);
        return;
      }

      const elapsedSeconds =
        (timestamp - maybePreviousTimestamp) / MILLISECONDS_PER_SECOND;
      const stepCount = acceptedStepCount(elapsedSeconds);
      if (stepCount === 0) {
        scheduleFrame(context);
        return;
      }

      try {
        const frame = maybeOwnedSession.nextFrame(stepCount);
        drawRenderFrame(context, frame, camera);
        const observation = observeFrame(
          frame,
          maybePreviousFrame,
          maybeObservedFrame(view())?.movedFrameCount ?? 0,
        );
        maybePreviousFrame = frame;
        setView({ kind: "playing", frame: observation });
        scheduleFrame(context);
      } catch (error) {
        fail(error);
      }
    });
  }

  function presentOwnedFrame(
    ownedSession: SceneSession,
    context: CanvasRenderingContext2D,
    resetObservation: boolean,
  ): void {
    const camera = maybeCamera;
    if (camera === undefined) {
      fail(new Error("Canvas camera is unavailable"));
      return;
    }

    const frame = ownedSession.nextFrame();
    drawRenderFrame(context, frame, camera);
    const observation = observeFrame(
      frame,
      resetObservation ? undefined : maybePreviousFrame,
      resetObservation ? 0 : maybeObservedFrame(view())?.movedFrameCount ?? 0,
    );
    maybePreviousFrame = frame;

    if (prefersReducedMotion()) {
      setView({ kind: "paused", frame: observation });
      return;
    }

    setView({ kind: "playing", frame: observation });
    scheduleFrame(context);
  }

  function connectResizeObserver(
    canvas: HTMLCanvasElement,
    context: CanvasRenderingContext2D,
  ): void {
    disconnectResizeObserver();
    maybeResizeObserver = new ResizeObserver(() => {
      const resizedBounds = canvas.getBoundingClientRect();
      if (!isUsableViewport(resizedBounds.width, resizedBounds.height)) {
        return;
      }

      try {
        const resizedCamera = resizeCanvasBackingStore(
          canvas,
          resizedBounds.width,
          resizedBounds.height,
          window.devicePixelRatio,
        );
        maybeCamera = resizedCamera;
        const maybeReadyId = maybeReadySceneId(route());
        if (maybeSession === undefined && maybeReadyId !== undefined) {
          void startScene(maybeReadyId);
          return;
        }

        const maybeFrame = maybePreviousFrame;
        if (maybeFrame !== undefined) {
          drawRenderFrame(context, maybeFrame, resizedCamera);
        }
      } catch (error) {
        fail(error);
      }
    });
    maybeResizeObserver.observe(canvas);
  }

  async function startScene(id: SceneId): Promise<void> {
    const started = incrementGeneration();
    cancelPendingFrame();
    disposeOwnedSession();
    maybePreviousFrame = undefined;
    maybeLastTimestamp = undefined;
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
      presentOwnedFrame(ownedSession, context, true);
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
      maybeCamera: () => maybeCamera,
      send: sendPointer,
    });

    const maybeNextContext = canvas.getContext("2d");
    if (maybeNextContext === null) {
      fail(new Error("Canvas 2D is unavailable"));
      return;
    }

    maybeContext = maybeNextContext;
    connectResizeObserver(canvas, maybeNextContext);
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

    maybeLastTimestamp = undefined;
    setView({ kind: "playing", frame: current.frame });
    scheduleFrame(context);
  }

  function pauseScene(): void {
    const current = view();
    if (current.kind !== "playing") {
      return;
    }

    maybeCanvasPointer?.cancel();
    cancelPendingFrame();
    maybeLastTimestamp = undefined;
    setView({ kind: "paused", frame: current.frame });
  }

  function recreateScene(): void {
    const maybeReadyId = maybeReadySceneId(route());
    if (maybeReadyId === undefined) {
      return;
    }

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
        cancelPendingFrame();
        setView({ kind: "loading" });
      }

      const recreated = maybeOwnedSession.applyControl(name, value);
      if (!recreated) {
        return;
      }

      maybePreviousFrame = undefined;
      maybeLastTimestamp = undefined;
      presentOwnedFrame(maybeOwnedSession, context, true);
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

      presentOwnedFrame(maybeOwnedSession, context, false);
      const maybeFrame = maybeObservedFrame(view());
      if (maybeFrame !== undefined && view().kind === "playing") {
        cancelPendingFrame();
        setView({ kind: "paused", frame: maybeFrame });
      }
    } catch (error) {
      fail(error);
    }
  }

  function onHashChange(): void {
    const previousRoute = route();
    const nextRoute = maybeParseSceneRoute(window.location.hash);
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
    maybeLastTimestamp = undefined;
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
    abandonScene();
  });

  const maybeFrame = () => maybeObservedFrame(view());
  const maybeCurrentSceneId = () => maybeReadySceneId(route());
  const maybeFailureDetails = () => {
    const current = view();
    return current.kind === "failure" ? current.maybeDetails : undefined;
  };
  const maybeSceneAttr = () => {
    const currentRoute = route();
    return currentRoute.kind === "scene" ? currentRoute.id : undefined;
  };
  const maybeCurrentScene = () => {
    const maybeId = maybeCurrentSceneId();
    return maybeId === undefined ? undefined : maybeSceneById(maybeId);
  };
  const sceneControlsDisabled = () => {
    const status = playerStatus(view());
    return status === "loading" || status === "failed";
  };

  return (
    <main
      aria-labelledby="site-title"
      data-playback={view().kind}
      data-scene={maybeSceneAttr()}
      data-step-index={maybeFrame()?.stepIndex}
      data-last-pointer-kind={lastPointerKind()}
      data-pointer-accepted={pointerAccepted()}
    >
      <header class="page-header">
        <h1 id="site-title">{PAGE_HEADING}</h1>
        <p>{PAGE_SUMMARY}</p>
      </header>

      <CatalogNav maybeCurrentSceneId={maybeCurrentSceneId()} />

      <Show
        when={maybeCurrentSceneId()}
        fallback={<FallbackPanel {...fallbackProps(route())} />}
      >
        {(sceneId) => {
          const scene = () => maybeCurrentScene();
          return (
            <PlayerPanel
              sceneTitle={sceneTitleForId(sceneId())}
              status={playerStatus(view())}
              maybeDetails={maybeFailureDetails()}
              assignCanvas={assignCanvas}
              onPlay={playScene}
              onPause={pauseScene}
              onReset={recreateScene}
              onRetry={recreateScene}
            >
              <Show when={scene()}>
                {(currentScene) => (
                  <>
                    <SceneControls
                      controls={currentScene().controls}
                      disabled={sceneControlsDisabled()}
                      maybeValues={constructionValues}
                      onApplyControl={applySceneControl}
                      onApplyAction={applySceneAction}
                    />
                    <SceneCredits
                      implementationPath={currentScene().credits.implementationPath}
                      inspiration={currentScene().credits.inspiration}
                    />
                  </>
                )}
              </Show>
            </PlayerPanel>
          );
        }}
      </Show>

      <SiteFooter />
    </main>
  );
}
