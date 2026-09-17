import { createEffect, createSignal, onCleanup, Show } from "solid-js";

import { maybeSceneById } from "./catalog/scenes";
import { CatalogNav } from "./components/CatalogNav";
import {
  FallbackPanel,
  type FallbackPanelProps,
} from "./components/FallbackPanel";
import { PlayerPanel, type PlayerStatus } from "./components/PlayerPanel";
import { SiteFooter } from "./components/SiteFooter";
import { acceptedStepCount } from "./physics/clock";
import type { RenderFrame } from "./physics/frame";
import { loadProofSession } from "./physics/loader";
import {
  createSceneSession,
  type SceneSession,
} from "./physics/session";
import { isStaleGeneration, nextGeneration } from "./player/generation";
import { observeFrame, type FrameObservation } from "./player/observe";
import {
  drawRenderFrame,
  resizeCanvasBackingStore,
} from "./render/canvas";
import type { Camera } from "./render/camera";
import { maybeParseSceneRoute, type SceneRoute } from "./routing/hash";

const PAGE_HEADING = "liquidfun-rs playground";
const PAGE_SUMMARY =
  "Play experimental Rust physics scenes in the browser. Dam Break is ready; the other five names are listed honestly until they ship.";
const DEFAULT_TITLE = "liquidfun-rs playground";
const DAM_BREAK_TITLE = "Dam Break · liquidfun-rs playground";
const DAM_BREAK_ID = "dam-break";
const MAX_ERROR_DETAIL_LENGTH = 240;
const MILLISECONDS_PER_SECOND = 1000;

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

function isDamBreakRoute(route: SceneRoute): boolean {
  return route.kind === "scene" && route.id === DAM_BREAK_ID;
}

function titleForRoute(route: SceneRoute): string {
  if (isDamBreakRoute(route)) {
    return DAM_BREAK_TITLE;
  }

  if (route.kind !== "scene") {
    return DEFAULT_TITLE;
  }

  const maybeScene = maybeSceneById(route.id);
  if (maybeScene === undefined) {
    return DEFAULT_TITLE;
  }

  return `${maybeScene.title} · liquidfun-rs playground`;
}

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

function maybeDevelopmentDetails(error: unknown): string | undefined {
  if (!import.meta.env.DEV) {
    return undefined;
  }

  const message =
    error instanceof Error ? error.message : "Unknown Dam Break failure";
  return `Details: ${message.slice(0, MAX_ERROR_DETAIL_LENGTH)}`;
}

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function isUsableViewport(width: number, height: number): boolean {
  return Number.isFinite(width) && Number.isFinite(height) && width > 0 && height > 0;
}

/** One-session Dam Break playground shell with hash routing and bounded playback. */
export function App() {
  const [route, setRoute] = createSignal(
    maybeParseSceneRoute(window.location.hash),
  );
  const [view, setView] = createSignal<PlayerView>(
    isDamBreakRoute(maybeParseSceneRoute(window.location.hash))
      ? { kind: "loading" }
      : { kind: "fallback" },
  );

  let generation = 0;
  let maybeCanvas: HTMLCanvasElement | undefined;
  let maybeContext: CanvasRenderingContext2D | undefined;
  let maybeSession: SceneSession | undefined;
  let maybeAnimationFrameId: number | undefined;
  let maybeLastTimestamp: number | undefined;
  let maybePreviousFrame: RenderFrame | undefined;
  let maybeCamera: Camera | undefined;
  let maybeResizeObserver: ResizeObserver | undefined;

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

  function abandonDamBreak(): void {
    incrementGeneration();
    disconnectResizeObserver();
    cancelPendingFrame();
    disposeOwnedSession();
    maybeCanvas = undefined;
    maybeContext = undefined;
    maybeCamera = undefined;
    maybePreviousFrame = undefined;
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
        fail(new Error("Dam Break session owner is unavailable"));
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
        if (maybeSession === undefined && isDamBreakRoute(route())) {
          void startDamBreak();
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

  async function startDamBreak(): Promise<void> {
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
      const generatedSession = await loadProofSession();
      if (isStaleGeneration(started, generation)) {
        createSceneSession(generatedSession).dispose();
        return;
      }

      const ownedSession = createSceneSession(generatedSession);
      if (isStaleGeneration(started, generation)) {
        ownedSession.dispose();
        return;
      }

      maybeSession = ownedSession;
      const camera = maybeCamera;
      if (camera === undefined) {
        fail(new Error("Canvas camera is unavailable"));
        return;
      }

      const frame = ownedSession.nextFrame();
      drawRenderFrame(context, frame, camera);
      const observation = observeFrame(frame, undefined, 0);
      maybePreviousFrame = frame;

      if (prefersReducedMotion()) {
        setView({ kind: "paused", frame: observation });
        return;
      }

      setView({ kind: "playing", frame: observation });
      scheduleFrame(context);
    } catch (error) {
      if (isStaleGeneration(started, generation)) {
        return;
      }

      fail(error);
    }
  }

  function assignCanvas(canvas: HTMLCanvasElement): void {
    maybeCanvas = canvas;

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

    cancelPendingFrame();
    maybeLastTimestamp = undefined;
    setView({ kind: "paused", frame: current.frame });
  }

  function recreateScene(): void {
    if (!isDamBreakRoute(route())) {
      return;
    }

    void startDamBreak();
  }

  function onHashChange(): void {
    const nextRoute = maybeParseSceneRoute(window.location.hash);
    if (!isDamBreakRoute(nextRoute)) {
      abandonDamBreak();
    } else if (view().kind === "fallback") {
      setView({ kind: "loading" });
    }

    setRoute(nextRoute);
  }

  function onVisibilityChange(): void {
    maybeLastTimestamp = undefined;
  }

  window.addEventListener("hashchange", onHashChange);
  document.addEventListener("visibilitychange", onVisibilityChange);

  createEffect(() => {
    const currentRoute = route();
    document.title = titleForRoute(currentRoute);
    if (isDamBreakRoute(currentRoute)) {
      if (view().kind === "fallback") {
        setView({ kind: "loading" });
      }
      return;
    }

    abandonDamBreak();
  });

  onCleanup(() => {
    window.removeEventListener("hashchange", onHashChange);
    document.removeEventListener("visibilitychange", onVisibilityChange);
    abandonDamBreak();
  });

  const maybeFrame = () => maybeObservedFrame(view());
  const maybeCurrentSceneId = () => {
    const currentRoute = route();
    return isDamBreakRoute(currentRoute) ? DAM_BREAK_ID : undefined;
  };
  const maybeFailureDetails = () => {
    const current = view();
    return current.kind === "failure" ? current.maybeDetails : undefined;
  };
  const maybeSceneAttr = () => {
    const currentRoute = route();
    return currentRoute.kind === "scene" ? currentRoute.id : undefined;
  };

  return (
    <main
      aria-labelledby="site-title"
      data-playback={view().kind}
      data-scene={maybeSceneAttr()}
      data-step-index={maybeFrame()?.stepIndex}
    >
      <header class="page-header">
        <h1 id="site-title">{PAGE_HEADING}</h1>
        <p>{PAGE_SUMMARY}</p>
      </header>

      <CatalogNav maybeCurrentSceneId={maybeCurrentSceneId()} />

      <Show
        when={isDamBreakRoute(route())}
        fallback={<FallbackPanel {...fallbackProps(route())} />}
      >
        <PlayerPanel
          status={playerStatus(view())}
          maybeDetails={maybeFailureDetails()}
          assignCanvas={assignCanvas}
          onPlay={playScene}
          onPause={pauseScene}
          onReset={recreateScene}
          onRetry={recreateScene}
        />
      </Show>

      <SiteFooter />
    </main>
  );
}
