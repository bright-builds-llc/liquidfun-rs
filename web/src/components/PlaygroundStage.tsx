import { Show } from "solid-js";

import { maybeSceneById } from "../catalog/scenes";
import type { RenderFrame } from "../physics/frame";
import {
  maybeReadySceneId,
  sceneControlsIdentity,
  sceneTitleForId,
} from "../player/runtime";
import { maybeObservedFrame, playerStatus, type PlayerView } from "../player/view";
import type { FpsTick } from "./fps-meter";
import type { TiltDebug } from "../input/tilt-gravity";
import type { PointerKind } from "../input/pointer";
import type { RenderMode } from "../render/mode";
import type { SceneRoute } from "../routing/hash";
import type { SvgExportRequest } from "../export/messages";
import { sceneControlsForSurface } from "./scene-controls";
import { AnimatedSvgPane } from "./AnimatedSvgPane";
import { FallbackPanel } from "./FallbackPanel";
import { PlayerPanel } from "./PlayerPanel";
import { PlayerSceneChrome } from "./PlayerSceneChrome";
import { PlaygroundShell } from "./PlaygroundShell";
import type { HudControlSliderProps } from "./SceneControls";

export type PlaygroundStageProps = {
  readonly canvasStage: boolean;
  readonly route: () => SceneRoute;
  readonly view: () => PlayerView;
  readonly lastPointerKind: () => PointerKind | undefined;
  readonly pointerAccepted: () => number;
  readonly renderMode: () => RenderMode;
  readonly wireframeStrokeWidth: () => number;
  readonly densityShading: () => boolean;
  readonly renderedParticleDraft: () => string;
  readonly panEnabled: () => boolean;
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
  readonly onTiltGravityEnabledChange: (enabled: boolean) => void;
  readonly onApplyControl: (name: string, value: string) => void;
  readonly onApplyAction: (name: string) => void;
  readonly onCreateSvgExportRequest: (durationSeconds: number) => SvgExportRequest | undefined;
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
  const routeIdentity = () => {
    const currentRoute = props.route();
    if (currentRoute.kind === "scene") {
      return `scene:${currentRoute.id}`;
    }
    if (currentRoute.kind === "unknown") {
      return `unknown:${currentRoute.maybeRaw}`;
    }
    return "empty";
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
        data-last-pointer-kind={props.lastPointerKind()}
        data-pointer-accepted={props.pointerAccepted()}
        data-render-mode={props.renderMode()}
        data-wireframe-stroke-width={props.wireframeStrokeWidth()}
        data-density-shading={props.densityShading() ? "on" : "off"}
      >
        <Show
          when={maybeCurrentSceneId()}
          fallback={<FallbackPanel canvasStage={props.canvasStage} />}
        >
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
              onZoomIn={props.onZoomIn}
              onZoomOut={props.onZoomOut}
              onResetZoom={props.onResetZoom}
              onPanEnabledChange={props.onPanEnabledChange}
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
        </Show>
      </main>
    </PlaygroundShell>
  );
}
