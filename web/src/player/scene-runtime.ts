import type { Accessor, Setter } from "solid-js";

import type { SceneId } from "../catalog/scenes";
import type { SvgExportRequest } from "../export/messages";
import type { FpsTick } from "../components/fps-meter";
import type { CanvasPointerHandlers } from "../input/canvas-pointer";
import type { TiltBinding } from "../input/tilt-binding";
import type { TiltDebug } from "../input/tilt-gravity";
import type { PointerKind } from "../input/pointer";
import type { RenderFrame } from "../physics/frame";
import type { SceneSession } from "../physics/session";
import type { FrameClock, FrameLoopDeps } from "./frame-loop";
import type { PlayerView } from "./view";
import type { AppearancePreferences } from "./appearance";
import type { Camera } from "../render/camera";
import type { SceneRoute } from "../routing/hash";

/** Mutable playground session shared by the shell, lifecycle, and surface. */
export type SceneRuntime = {
  generation: number;
  constructionValues: Record<string, string>;
  maybeCanvas: HTMLCanvasElement | undefined;
  maybeParticleCanvas: HTMLCanvasElement | undefined;
  maybeContext: CanvasRenderingContext2D | undefined;
  maybeSession: SceneSession | undefined;
  maybeCanvasPointer: CanvasPointerHandlers | undefined;
  clock: FrameClock;
  tiltBinding: TiltBinding;
  canvasStage: boolean;
  releaseVisualViewport: (() => void) | undefined;
  route: Accessor<SceneRoute>;
  setRoute: Setter<SceneRoute>;
  view: Accessor<PlayerView>;
  setView: Setter<PlayerView>;
  appearance: AppearancePreferences;
  debugEnabled: Accessor<boolean>;
  setDebugEnabled: Setter<boolean>;
  maybeDebugFrame: Accessor<RenderFrame | undefined>;
  setMaybeDebugFrame: Setter<RenderFrame | undefined>;
  stepsThisFrame: Accessor<number>;
  setStepsThisFrame: Setter<number>;
  fpsTicks: Accessor<readonly FpsTick[]>;
  setFpsTicks: Setter<readonly FpsTick[]>;
  lastPointerKind: Accessor<PointerKind | undefined>;
  setLastPointerKind: Setter<PointerKind | undefined>;
  pointerAccepted: Accessor<number>;
  setPointerAccepted: Setter<number>;
  resetGeneration: Accessor<number>;
  setResetGeneration: Setter<number>;
  renderedParticleDraft: Accessor<string>;
  setRenderedParticleDraft: Setter<string>;
  maxRenderedParticles: Accessor<number>;
  setMaxRenderedParticles: Setter<number>;
  panEnabled: Accessor<boolean>;
  setPanEnabled: Setter<boolean>;
  maybePixelsPerMeter: Accessor<number | undefined>;
  setMaybePixelsPerMeter: Setter<number | undefined>;
  tiltGravityEnabled: Accessor<boolean>;
  setTiltGravityEnabled: Setter<boolean>;
  tiltDebug: Accessor<TiltDebug>;
  setTiltDebug: Setter<TiltDebug>;
  gravitySliderMagnitude: () => number | undefined;
  frameDeps: () => FrameLoopDeps;
  refreshCamera: () => void;
  paintHeldFrame: () => void;
  drawSceneFrame: (
    context: CanvasRenderingContext2D,
    frame: RenderFrame,
    camera: Camera,
  ) => void;
  zoomIn: () => void;
  zoomOut: () => void;
  resetCameraView: () => void;
  changePanEnabled: (enabled: boolean) => void;
  changeRenderedParticleDraft: (raw: string) => void;
  assignCanvas: (canvas: HTMLCanvasElement) => void;
  assignParticleSurface: (canvas: HTMLCanvasElement) => void;
  beginScene: (id: SceneId) => Promise<void>;
  playScene: () => void;
  pauseScene: () => void;
  recreateScene: () => void;
  abandonScene: () => void;
  failScene: (error: unknown) => void;
  applySceneControl: (name: string, value: string) => void;
  applySceneAction: (name: string) => void;
  createSvgExportRequest: (
    durationSeconds: number,
  ) => SvgExportRequest | undefined;
  changeTiltGravityEnabled: (enabled: boolean) => void;
};
