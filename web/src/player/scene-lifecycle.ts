import { maybeSceneById, type SceneId } from "../catalog/scenes";
import { worldBoundsForViewport } from "../catalog/portrait-bounds";
import {
  constructionEntriesForScene,
  formatFailureDetails,
  maybeReadySceneId,
} from "./runtime";
import { isStaleGeneration, nextGeneration } from "./generation";
import { maybeObservedFrame } from "./view";
import { loadSceneSession } from "../physics/loader";
import { createSceneSession } from "../physics/session";
import {
  cancelPendingFrame,
  disconnectResizeObserver,
  presentOwnedFrame,
  scheduleFrame,
  type FrameLoopDeps,
} from "./frame-loop";
import { reapplyStoredTiltGravity } from "../input/tilt-binding";
import {
  DEFAULT_RENDERED_PARTICLE_LIMIT,
} from "../render/particle-limit";
import { IDENTITY_CAMERA_VIEW } from "../render/camera";
import { normalizeSceneRoute } from "../routing/hash";
import type { SceneRuntime } from "./scene-runtime";

function incrementGeneration(session: SceneRuntime): number {
  session.generation = nextGeneration(session.generation);
  return session.generation;
}

function disposeOwnedSession(session: SceneRuntime): void {
  session.maybeCanvasPointer?.cancel();
  const maybeOwnedSession = session.maybeSession;
  session.maybeSession = undefined;
  session.clock.maybeLastTimestamp = undefined;
  if (maybeOwnedSession === undefined) {
    return;
  }

  try {
    maybeOwnedSession.dispose();
  } catch {
    // Disposal remains terminal even if generated cleanup reports a failure.
  }
}

function abandonScene(session: SceneRuntime): void {
  incrementGeneration(session);
  session.maybeCanvasPointer?.detach();
  session.maybeCanvasPointer = undefined;
  disconnectResizeObserver(session.clock);
  cancelPendingFrame(session.clock);
  disposeOwnedSession(session);
  session.maybeCanvas = undefined;
  session.maybeParticleCanvas = undefined;
  session.maybeContext = undefined;
  session.clock.maybeCamera = undefined;
  session.clock.maybePreviousFrame = undefined;
  session.setMaybeDebugFrame(undefined);
  session.setStepsThisFrame(0);
  session.constructionValues = {};
  session.setView({ kind: "fallback" });
}

function failScene(session: SceneRuntime, error: unknown): void {
  const maybeFrame = maybeObservedFrame(session.view());
  cancelPendingFrame(session.clock);
  disposeOwnedSession(session);
  session.setView({
    kind: "failure",
    maybeFrame,
    maybeDetails: formatFailureDetails(error),
  });
}

function frameDeps(session: SceneRuntime): FrameLoopDeps {
  return {
    view: session.view,
    fail: (error) => failScene(session, error),
    maybeSession: () => session.maybeSession,
    route: session.route,
    startScene: (id) => {
      void beginScene(session, id);
    },
    drawSceneFrame: (context, frame, camera) => {
      session.drawSceneFrame(context, frame, camera);
    },
    setMaybeDebugFrame: session.setMaybeDebugFrame,
    setStepsThisFrame: session.setStepsThisFrame,
    setFpsTicks: session.setFpsTicks,
    setView: session.setView,
  };
}

async function beginScene(session: SceneRuntime, id: SceneId): Promise<void> {
  const started = incrementGeneration(session);
  cancelPendingFrame(session.clock);
  disposeOwnedSession(session);
  session.clock.maybePreviousFrame = undefined;
  session.setMaybeDebugFrame(undefined);
  session.setStepsThisFrame(0);
  session.setFpsTicks([]);
  session.clock.maybeLastTimestamp = undefined;
  session.setRenderedParticleDraft(String(DEFAULT_RENDERED_PARTICLE_LIMIT));
  session.setMaxRenderedParticles(DEFAULT_RENDERED_PARTICLE_LIMIT);
  session.clock.worldBounds = worldBoundsForViewport(
    id,
    session.clock.viewportWidth,
    session.clock.viewportHeight,
  );
  session.clock.cameraView = IDENTITY_CAMERA_VIEW;
  session.refreshCamera();
  session.setView({ kind: "loading" });

  const canvas = session.maybeCanvas;
  const context = session.maybeContext;
  if (canvas === undefined || context === undefined) {
    failScene(session, new Error("Canvas element is unavailable"));
    return;
  }

  try {
    const generatedSession = await loadSceneSession(id);
    if (isStaleGeneration(started, session.generation)) {
      createSceneSession(generatedSession).dispose();
      return;
    }

    const ownedSession = createSceneSession(generatedSession);
    if (isStaleGeneration(started, session.generation)) {
      ownedSession.dispose();
      return;
    }

    const maybeScene = maybeSceneById(id);
    if (maybeScene !== undefined) {
      for (const entry of constructionEntriesForScene(
        maybeScene,
        session.constructionValues,
      )) {
        ownedSession.applyControl(entry.name, entry.value);
      }
    }

    session.maybeSession = ownedSession;
    reapplyStoredTiltGravity(
      session.tiltBinding,
      () => session.maybeSession,
      session.gravitySliderMagnitude,
      session.setTiltDebug,
    );
    presentOwnedFrame(
      session.clock,
      ownedSession,
      context,
      true,
      frameDeps(session),
    );
  } catch (error) {
    if (isStaleGeneration(started, session.generation)) {
      return;
    }

    failScene(session, error);
  }
}

function playScene(session: SceneRuntime): void {
  const current = session.view();
  if (current.kind !== "paused") {
    return;
  }

  const context = session.maybeContext;
  if (context === undefined) {
    failScene(session, new Error("Canvas 2D is unavailable"));
    return;
  }

  session.clock.maybeLastTimestamp = undefined;
  session.setView({ kind: "playing", frame: current.frame });
  scheduleFrame(session.clock, context, frameDeps(session));
}

function pauseScene(session: SceneRuntime): void {
  const current = session.view();
  if (current.kind !== "playing") {
    return;
  }

  session.maybeCanvasPointer?.cancel();
  cancelPendingFrame(session.clock);
  session.clock.maybeLastTimestamp = undefined;
  session.setStepsThisFrame(0);
  session.setView({ kind: "paused", frame: current.frame });
}

function recreateScene(session: SceneRuntime): void {
  const maybeReadyId = maybeReadySceneId(session.route());
  if (maybeReadyId === undefined) {
    return;
  }

  session.constructionValues = {};
  session.setResetGeneration((current) => current + 1);
  void beginScene(session, maybeReadyId);
}

/** Applies a playground hash change to the live session. */
export function handleHashChange(session: SceneRuntime): void {
  const previousRoute = session.route();
  const normalized = normalizeSceneRoute(window.location.hash);
  if (normalized.maybeReplacementHash !== undefined) {
    window.history.replaceState(
      window.history.state,
      "",
      normalized.maybeReplacementHash,
    );
  }
  const nextRoute = normalized.route;
  session.setRoute(nextRoute);

  const maybeNextId = maybeReadySceneId(nextRoute);
  if (maybeNextId === undefined) {
    abandonScene(session);
    return;
  }

  if (maybeReadySceneId(previousRoute) === maybeNextId) {
    return;
  }

  session.constructionValues = {};
  if (session.maybeCanvas !== undefined && session.maybeContext !== undefined) {
    void beginScene(session, maybeNextId);
    return;
  }

  session.setView({ kind: "loading" });
}

/** Attaches scene start, transport, and hash routing to a playground session. */
export function bindLifecycle(session: SceneRuntime): void {
  session.frameDeps = () => frameDeps(session);
  session.beginScene = (id) => beginScene(session, id);
  session.playScene = () => playScene(session);
  session.pauseScene = () => pauseScene(session);
  session.recreateScene = () => recreateScene(session);
  session.abandonScene = () => abandonScene(session);
  session.failScene = (error) => failScene(session, error);
}

/** Drops the frame clock when the tab is hidden and resumes cleanly. */
export function handleVisibilityChange(session: SceneRuntime): void {
  session.clock.maybeLastTimestamp = undefined;
  if (document.hidden) {
    session.maybeCanvasPointer?.cancel();
  }
}
