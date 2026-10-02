import { maybeSceneById, type SceneId } from "../catalog/scenes";
import { changedPresetEntries } from "../export/presets";
import type { SvgExportRequest } from "../export/messages";
import { controlStoresAppliedValue } from "../components/scene-controls";
import {
  attachCanvasPointer,
  forwardScenePointer,
} from "../input/canvas-pointer";
import type { PointerKind } from "../input/pointer";
import {
  changeTiltGravity,
  reapplyStoredTiltGravity,
} from "../input/tilt-binding";
import type { RenderFrame } from "../physics/frame";
import {
  cancelPendingFrame,
  connectResizeObserver,
  presentOwnedFrame,
} from "./frame-loop";
import { maybeReadySceneId } from "./runtime";
import { maybeObservedFrame } from "./view";
import { isUsableViewport } from "./viewport";
import { presentSceneFrame } from "../render/present-frame";
import {
  IDENTITY_CAMERA_VIEW,
  createCamera,
  zoomCameraView,
  type Camera,
  type CameraView,
} from "../render/camera";
import { maybeParseRenderedParticleLimit } from "../render/particle-limit";
import { svgRenderMode } from "../render/mode";
import type { SceneRuntime } from "./scene-runtime";
import { isWorkerSession } from "../physics/live-session";
import { applyWorkerAction, applyWorkerControl } from "./worker-controls";

function paintHeldFrame(session: SceneRuntime): void {
  const maybeFrame = session.clock.maybePreviousFrame;
  const context = session.maybeContext;
  const camera = session.clock.maybeCamera;
  if (
    maybeFrame === undefined ||
    context === undefined ||
    camera === undefined
  ) {
    return;
  }

  drawSceneFrame(session, context, maybeFrame, camera);
}

function drawSceneFrame(
  session: SceneRuntime,
  context: CanvasRenderingContext2D,
  frame: RenderFrame,
  camera: Camera,
): void {
  presentSceneFrame(
    context,
    frame,
    camera,
    session.appearance.renderMode(),
    session.appearance.wireframeStrokeWidth(),
    session.maxRenderedParticles(),
    session.maybeParticleCanvas,
    window.devicePixelRatio,
    session.appearance.densityShading(),
  );
}

function refreshCamera(session: SceneRuntime): void {
  if (
    !isUsableViewport(session.clock.viewportWidth, session.clock.viewportHeight)
  ) {
    return;
  }

  session.clock.maybeCamera = createCamera(
    session.clock.viewportWidth,
    session.clock.viewportHeight,
    session.clock.cameraView,
    session.clock.worldBounds,
  );
  session.setMaybePixelsPerMeter(session.clock.maybeCamera.scale);
}

function changeRenderedParticleDraft(session: SceneRuntime, raw: string): void {
  session.setRenderedParticleDraft(raw);
  const maybeLimit = maybeParseRenderedParticleLimit(raw);
  if (maybeLimit === undefined) {
    return;
  }

  session.setMaxRenderedParticles(maybeLimit);
  paintHeldFrame(session);
}

function applyCameraView(session: SceneRuntime, nextView: CameraView): void {
  session.clock.cameraView = nextView;
  refreshCamera(session);
  paintHeldFrame(session);
}

function changePanEnabled(session: SceneRuntime, enabled: boolean): void {
  session.setPanEnabled(enabled);
  session.maybeCanvas?.classList.toggle("canvas-panning", enabled);
  if (!enabled) {
    session.maybeCanvasPointer?.cancel();
  }
}

function panBy(session: SceneRuntime, deltaX: number, deltaY: number): void {
  applyCameraView(session, {
    zoom: session.clock.cameraView.zoom,
    panX: session.clock.cameraView.panX + deltaX,
    panY: session.clock.cameraView.panY + deltaY,
  });
}

function sendPointer(
  session: SceneRuntime,
  kind: PointerKind,
  worldX: number,
  worldY: number,
): void {
  const owner = session.maybeSession,
    generation = session.generation;
  forwardScenePointer(
    session.maybeSession,
    session.view().kind,
    kind,
    worldX,
    worldY,
    session.setLastPointerKind,
    session.setPointerAccepted,
    session.failScene,
    () => session.maybeSession === owner && session.generation === generation,
  );
}

function assignCanvas(session: SceneRuntime, canvas: HTMLCanvasElement): void {
  session.maybeCanvasPointer?.detach();
  session.maybeCanvas = canvas;
  session.maybeCanvasPointer = attachCanvasPointer({
    canvas,
    maybeCamera: () => session.clock.maybeCamera,
    send: (kind, worldX, worldY) => sendPointer(session, kind, worldX, worldY),
    panEnabled: () => session.panEnabled(),
    onPanBy: (deltaX, deltaY) => panBy(session, deltaX, deltaY),
  });

  const maybeNextContext = canvas.getContext("2d");
  if (maybeNextContext === null) {
    session.failScene(new Error("Canvas 2D is unavailable"));
    return;
  }

  session.maybeContext = maybeNextContext;
  connectResizeObserver(
    session.clock,
    canvas,
    maybeNextContext,
    session.frameDeps(),
  );
}

function reapplyLiveControls(session: SceneRuntime, sceneId: SceneId): void {
  const maybeScene = maybeSceneById(sceneId);
  const maybeOwnedSession = session.maybeSession;
  if (maybeScene === undefined || maybeOwnedSession === undefined) {
    return;
  }

  for (const control of maybeScene.controls) {
    if (control.recreates || control.kind === "action") {
      continue;
    }

    const maybeValue = session.constructionValues[control.id];
    if (maybeValue === undefined) {
      continue;
    }

    maybeOwnedSession.applyControl(control.id, maybeValue);
  }
}

function applySceneControl(
  session: SceneRuntime,
  name: string,
  value: string,
): void {
  const maybeOwnedSession = session.maybeSession;
  const context = session.maybeContext;
  const maybeReadyId = maybeReadySceneId(session.route());
  if (
    maybeOwnedSession === undefined ||
    context === undefined ||
    maybeReadyId === undefined
  ) {
    session.failScene(new Error("Scene session owner is unavailable"));
    return;
  }

  const maybeControl = maybeSceneById(maybeReadyId)?.controls.find(
    (control) => control.id === name,
  );
  const recreates = maybeControl?.recreates === true;
  if (isWorkerSession(maybeOwnedSession)) {
    const generation = session.generation;
    void applyWorkerControl(session, name, value).catch((error: unknown) => {
      if (
        session.maybeSession === maybeOwnedSession &&
        session.generation === generation
      )
        session.failScene(error);
    });
    return;
  }

  try {
    if (recreates) {
      session.maybeCanvasPointer?.cancel();
      cancelPendingFrame(session.clock);
      session.setView({ kind: "loading" });
    }

    const recreated = maybeOwnedSession.applyControl(name, value);
    if (controlStoresAppliedValue(maybeControl)) {
      session.constructionValues = {
        ...session.constructionValues,
        [name]: value,
      };
    }
    reapplyStoredTiltGravity(
      session.tiltBinding,
      () => session.maybeSession,
      session.gravitySliderMagnitude,
      session.setTiltDebug,
    );
    if (!recreated) {
      return;
    }

    reapplyLiveControls(session, maybeReadyId);
    session.clock.maybePreviousFrame = undefined;
    session.setMaybeDebugFrame(undefined);
    session.setStepsThisFrame(0);
    session.setFpsTicks([]);
    session.clock.maybeLastTimestamp = undefined;
    presentOwnedFrame(
      session.clock,
      maybeOwnedSession,
      context,
      true,
      session.frameDeps(),
    );
  } catch (error) {
    session.failScene(error);
  }
}

function applySceneAction(session: SceneRuntime, name: string): void {
  const maybeOwnedSession = session.maybeSession;
  const context = session.maybeContext;
  if (maybeOwnedSession === undefined || context === undefined) {
    session.failScene(new Error("Scene session owner is unavailable"));
    return;
  }
  if (isWorkerSession(maybeOwnedSession)) {
    const generation = session.generation;
    void applyWorkerAction(session, name).catch((error: unknown) => {
      if (
        session.maybeSession === maybeOwnedSession &&
        session.generation === generation
      )
        session.failScene(error);
    });
    return;
  }

  try {
    maybeOwnedSession.applyAction(name);
    if (session.view().kind !== "paused") {
      return;
    }

    presentOwnedFrame(
      session.clock,
      maybeOwnedSession,
      context,
      false,
      session.frameDeps(),
    );
    const maybeFrame = maybeObservedFrame(session.view());
    if (maybeFrame !== undefined && session.view().kind === "playing") {
      cancelPendingFrame(session.clock);
      session.setView({ kind: "paused", frame: maybeFrame });
    }
  } catch (error) {
    session.failScene(error);
  }
}

function createSvgExportRequest(
  session: SceneRuntime,
  durationSeconds: number,
): SvgExportRequest | undefined {
  const maybeId = maybeReadySceneId(session.route());
  const maybeScene =
    maybeId === undefined ? undefined : maybeSceneById(maybeId);
  if (
    maybeScene === undefined ||
    !isUsableViewport(
      session.clock.viewportWidth,
      session.clock.viewportHeight,
    ) ||
    session.clock.viewportWidth <= 32 ||
    session.clock.viewportHeight <= 32
  ) {
    return undefined;
  }

  return {
    sceneId: maybeScene.id,
    title: maybeScene.title,
    durationSeconds,
    controls: changedPresetEntries(maybeScene, session.constructionValues),
    viewportWidth: session.clock.viewportWidth,
    viewportHeight: session.clock.viewportHeight,
    zoom: session.clock.cameraView.zoom,
    panX: session.clock.cameraView.panX,
    panY: session.clock.cameraView.panY,
    renderMode: svgRenderMode(session.appearance.renderMode()),
    wireframeStrokeWidth: session.appearance.wireframeStrokeWidth(),
    maxRenderedParticles: session.maxRenderedParticles(),
  };
}

/** Attaches canvas, camera, and scene-command behavior to a playground session. */
export function bindSurface(session: SceneRuntime): void {
  session.paintHeldFrame = () => paintHeldFrame(session);
  session.drawSceneFrame = (context, frame, camera) => {
    drawSceneFrame(session, context, frame, camera);
  };
  session.refreshCamera = () => refreshCamera(session);
  session.changeRenderedParticleDraft = (raw) => {
    changeRenderedParticleDraft(session, raw);
  };
  session.zoomIn = () => {
    applyCameraView(session, zoomCameraView(session.clock.cameraView, "in"));
  };
  session.zoomOut = () => {
    applyCameraView(session, zoomCameraView(session.clock.cameraView, "out"));
  };
  session.resetCameraView = () => {
    applyCameraView(session, IDENTITY_CAMERA_VIEW);
  };
  session.changePanEnabled = (enabled) => changePanEnabled(session, enabled);
  session.assignCanvas = (canvas) => assignCanvas(session, canvas);
  session.assignParticleSurface = (canvas) => {
    session.maybeParticleCanvas = canvas;
  };
  session.applySceneControl = (name, value) => {
    applySceneControl(session, name, value);
  };
  session.applySceneAction = (name) => applySceneAction(session, name);
  session.createSvgExportRequest = (durationSeconds) =>
    createSvgExportRequest(session, durationSeconds);
  session.changeTiltGravityEnabled = (enabled) => {
    const owner = session.maybeSession,
      generation = session.generation;
    const fail = (error: unknown) => {
      if (session.maybeSession === owner && session.generation === generation)
        session.failScene(error);
    };
    void changeTiltGravity(
      session.tiltBinding,
      enabled,
      () => session.maybeSession,
      session.gravitySliderMagnitude,
      session.setTiltGravityEnabled,
      session.setTiltDebug,
      fail,
    ).catch(fail);
  };
}
