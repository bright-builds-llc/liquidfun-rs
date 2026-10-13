import { Match, Show, Switch } from "solid-js";

import { maybeSceneById } from "../catalog/scenes";
import type { RenderFrame } from "../physics/frame";
import {
  maybeReadySceneId,
  sceneControlsIdentity,
  sceneTitleForId,
} from "../player/runtime";
import { playableRouteIdentity } from "../player/playable-target";
import { maybeObservedFrame, playerStatus, type PlayerView } from "../player/view";
import type { PuzzleProgress } from "../player/puzzle-progress";
import type { PuzzleController } from "../puzzle/controller";
import { puzzleDebugAttributes } from "../puzzle/debug-attributes";
import type { FpsTick } from "./fps-meter";
import type { TiltDebug } from "../input/tilt-gravity";
import type { PointerKind } from "../input/pointer";
import type { RenderMode } from "../render/mode";
import type { ScreenRect } from "../render/safe-frame";
import type { PlayableRoute } from "../routing/playable-route";
import type { SvgExportRequest } from "../export/messages";
import { sceneControlsForSurface } from "./scene-controls";
import { AnimatedSvgPane } from "./AnimatedSvgPane";
import { FallbackPanel } from "./FallbackPanel";
import { PlayerPanel } from "./PlayerPanel";
import { PlayerSceneChrome } from "./PlayerSceneChrome";
import { PlaygroundShell } from "./PlaygroundShell";
import { PuzzleFallbackPanel } from "./PuzzleFallbackPanel";
import { PuzzlePanel } from "./PuzzlePanel";
import { PuzzleSelectPanel } from "./PuzzleSelectPanel";
import type { HudControlSliderProps } from "./SceneControls";

export type PlaygroundStageProps = {
  readonly canvasStage: boolean;
  readonly route: () => PlayableRoute;
  readonly view: () => PlayerView;
  readonly lastPointerKind: () => PointerKind | undefined;
  readonly pointerAccepted: () => number;
  readonly renderMode: () => RenderMode;
  readonly wireframeStrokeWidth: () => number;
  readonly densityShading: () => boolean;
  readonly renderedParticleDraft: () => string;
  readonly panEnabled: () => boolean;
  readonly maybePixelsPerMeter: () => number | undefined;
  readonly tiltGravityEnabled: () => boolean;
  readonly tiltDebug: () => TiltDebug;
  readonly debugEnabled: () => boolean;
  readonly maybeDebugFrame: () => RenderFrame | undefined;
  readonly stepsThisFrame: () => number;
  readonly fpsTicks: () => readonly FpsTick[];
  readonly resetGeneration: () => number;
  readonly constructionValues: Readonly<Record<string, string>>;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly assignParticleSurface: (canvas: HTMLCanvasElement) => void;
  readonly onPlay: () => void;
  readonly onPause: () => void;
  readonly onReset: () => void;
  readonly onRenderModeChange: (mode: RenderMode) => void;
  readonly onWireframeStrokeWidthChange: (width: number) => void;
  readonly onDensityShadingChange: (enabled: boolean) => void;
  readonly onDebugEnabledChange: (enabled: boolean) => void;
  readonly onRenderedParticleDraft: (raw: string) => void;
  readonly onZoomIn: () => void;
  readonly onZoomOut: () => void;
  readonly onResetZoom: () => void;
  readonly onPanEnabledChange: (enabled: boolean) => void;
  readonly onHudObstaclesChange: (obstacles: readonly ScreenRect[]) => void;
  readonly onTiltGravityEnabledChange: (enabled: boolean) => void;
  readonly onApplyControl: (name: string, value: string) => void;
  readonly onApplyAction: (name: string) => void;
  readonly onCreateSvgExportRequest: (durationSeconds: number) => SvgExportRequest | undefined;
  readonly puzzleController: PuzzleController;
  /** Persisted progress for the `#/puzzles` grid (Phase 41 D-05). */
  readonly puzzleProgress: PuzzleProgress;
  /** Puzzle canvas ref: no pointer handlers (D-16). */
  readonly assignPuzzleCanvas: (canvas: HTMLCanvasElement) => void;
};

/** Routed playground surface for the live scene player. */
export function PlaygroundStage(props: PlaygroundStageProps) {
  const maybeFrame = () => maybeObservedFrame(props.view());
  const maybeCurrentSceneId = () => maybeReadySceneId(props.route());
  const maybeFailureDetails = () => {
    const current = props.view();
    return current.kind === "failure" ? current.maybeDetails : undefined;
  };
  const maybeSceneAttr = () => {
    const currentRoute = props.route();
    return currentRoute.kind === "scene" ? currentRoute.id : undefined;
  };
  const maybeCurrentScene = () => {
    const maybeId = maybeCurrentSceneId();
    return maybeId === undefined ? undefined : maybeSceneById(maybeId);
  };
  const sceneControlsDisabled = () => {
    const status = playerStatus(props.view());
    return status === "loading" || status === "failed";
  };
  const hudControls = (): HudControlSliderProps => {
    const maybeScene = maybeCurrentScene();
    const maybeId = maybeCurrentSceneId();
    return {
      controls:
        maybeScene === undefined
          ? []
          : sceneControlsForSurface(maybeScene.controls, "hud"),
      disabled: sceneControlsDisabled(),
      identity:
        maybeId === undefined
          ? "none"
          : sceneControlsIdentity(maybeId, props.resetGeneration()),
      maybeValues: props.constructionValues,
      onApplyControl: props.onApplyControl,
    };
  };
  const routeIdentity = () => playableRouteIdentity(props.route());
  const maybePuzzleId = () => {
    const currentRoute = props.route();
    return currentRoute.kind === "puzzle" ? currentRoute.id : undefined;
  };
  // The route object, not its raw text, so an empty raw id still matches.
  const maybeUnknownPuzzle = () => {
    const currentRoute = props.route();
    return currentRoute.kind === "unknown-puzzle" ? currentRoute : undefined;
  };
  const maybeEndedPuzzle = () => {
    if (maybePuzzleId() === undefined) {
      return undefined;
    }
    const state = props.puzzleController.state();
    return state.kind === "ended" ? state : undefined;
  };
  const maybePuzzleHeld = () => {
    if (maybePuzzleId() === undefined) {
      return undefined;
    }
    return maybeEndedPuzzle()?.held === true ? "true" : "false";
  };

  return (
    <PlaygroundShell
      canvasStage={props.canvasStage}
      maybeCurrentSceneId={maybeCurrentSceneId()}
      routeIdentity={routeIdentity()}
    >
      <main
        class="playground-main"
        aria-labelledby="site-title"
        data-playback={props.view().kind}
        data-scene={maybeSceneAttr()}
        data-step-index={maybeFrame()?.stepIndex}
        data-particle-count={maybeFrame()?.particleCount ?? 0}
        data-last-pointer-kind={props.lastPointerKind()}
        data-pointer-accepted={props.pointerAccepted()}
        data-render-mode={props.renderMode()}
        data-wireframe-stroke-width={props.wireframeStrokeWidth()}
        data-density-shading={props.densityShading() ? "on" : "off"}
        data-puzzle={maybePuzzleId()}
        data-puzzle-phase={
          maybePuzzleId() === undefined ? undefined : props.puzzleController.state().kind
        }
        data-puzzle-held={maybePuzzleHeld()}
        data-puzzle-outcome={maybeEndedPuzzle()?.latch.outcome}
        {...(maybePuzzleId() === undefined
          ? {}
          : puzzleDebugAttributes(
              props.puzzleController.state(),
              props.puzzleController.maybeHeldHash(),
            ))}
      >
        <Switch fallback={<FallbackPanel canvasStage={props.canvasStage} />}>
          <Match when={maybeCurrentSceneId()}>
            {(sceneId) => (
              <PlayerPanel
                canvasStage={props.canvasStage}
                sceneTitle={sceneTitleForId(sceneId())}
                status={playerStatus(props.view())}
                maybeDetails={maybeFailureDetails()}
                interactionHint={maybeCurrentScene()?.interactionHint ?? ""}
                assignCanvas={props.assignCanvas}
                assignParticleSurface={props.assignParticleSurface}
                onPlay={props.onPlay}
                onPause={props.onPause}
                onReset={props.onReset}
                onRetry={props.onReset}
                renderMode={props.renderMode()}
                onRenderModeChange={props.onRenderModeChange}
                wireframeStrokeWidth={props.wireframeStrokeWidth()}
                onWireframeStrokeWidthChange={props.onWireframeStrokeWidthChange}
                densityShading={props.densityShading()}
                onDensityShadingChange={props.onDensityShadingChange}
                debugEnabled={props.debugEnabled()}
                onDebugEnabledChange={props.onDebugEnabledChange}
                maybeDebugFrame={props.maybeDebugFrame()}
                stepsThisFrame={props.stepsThisFrame()}
                fpsTicks={props.fpsTicks()}
                renderedParticleDraft={props.renderedParticleDraft()}
                onRenderedParticleDraft={props.onRenderedParticleDraft}
                panEnabled={props.panEnabled()}
                maybePixelsPerMeter={props.maybePixelsPerMeter()}
                onZoomIn={props.onZoomIn}
                onZoomOut={props.onZoomOut}
                onResetZoom={props.onResetZoom}
                onPanEnabledChange={props.onPanEnabledChange}
                onHudObstaclesChange={props.onHudObstaclesChange}
                hudControls={hudControls()}
                tiltGravityEnabled={props.tiltGravityEnabled()}
                tiltDebug={props.tiltDebug()}
                onTiltGravityEnabledChange={props.onTiltGravityEnabledChange}
              >
                <AnimatedSvgPane
                  disabled={sceneControlsDisabled()}
                  sceneId={sceneId()}
                  onCreateRequest={props.onCreateSvgExportRequest}
                />
                <Show when={maybeCurrentScene()}>
                  {(currentScene) => (
                    <PlayerSceneChrome
                      scene={currentScene()}
                      resetGeneration={props.resetGeneration()}
                      disabled={sceneControlsDisabled()}
                      maybeValues={props.constructionValues}
                      onApplyControl={props.onApplyControl}
                      onApplyAction={props.onApplyAction}
                    />
                  )}
                </Show>
              </PlayerPanel>
            )}
          </Match>
          <Match when={maybePuzzleId()}>
            {(levelId) => (
              <PuzzlePanel
                canvasStage={props.canvasStage}
                levelId={levelId()}
                controller={props.puzzleController}
                status={playerStatus(props.view())}
                maybeFailureDetails={maybeFailureDetails()}
                renderMode={props.renderMode()}
                onRenderModeChange={props.onRenderModeChange}
                maybePixelsPerMeter={props.maybePixelsPerMeter()}
                assignCanvas={props.assignPuzzleCanvas}
                assignParticleSurface={props.assignParticleSurface}
                onHudObstaclesChange={props.onHudObstaclesChange}
              />
            )}
          </Match>
          <Match when={props.route().kind === "puzzles"}>
            <PuzzleSelectPanel
              canvasStage={props.canvasStage}
              progress={props.puzzleProgress.current()}
              degraded={props.puzzleProgress.degraded()}
            />
          </Match>
          <Match when={maybeUnknownPuzzle()}>
            {(unknown) => (
              <PuzzleFallbackPanel
                canvasStage={props.canvasStage}
                maybeRaw={unknown().maybeRaw}
              />
            )}
          </Match>
        </Switch>
      </main>
    </PlaygroundShell>
  );
}
