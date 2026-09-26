import { Show, type JSX } from "solid-js";

import type { PlayerStatus } from "../player/view";
import type { RenderFrame } from "../physics/frame";
import {
  RENDER_MODE_GROUPS,
  maybeParseRenderMode,
  needsWebglSurface,
  particleSurface,
  usesParticleStride,
  type RenderMode,
} from "../render/mode";
import {
  formatWireframeStrokeWidth,
  maybeParseWireframeStrokeWidth,
  WIREFRAME_STROKE_WIDTH_MAX,
  WIREFRAME_STROKE_WIDTH_MIN,
  WIREFRAME_STROKE_WIDTH_STEP,
} from "../render/stroke-width";
import { formatTiltDebug, type TiltDebug } from "../input/tilt-gravity";
import { DebugReadout } from "./DebugReadout";
import { GravityArrow } from "./GravityArrow";
import type { FpsTick } from "./fps-meter";
import { ViewportTools } from "./ViewportTools";
import { CanvasHud, PlaybackButtons, SceneControlsSheet } from "./CanvasStage";
import {
  HudControlSlider,
  type HudControlSliderProps,
} from "./SceneControls";
import { SiteFooter } from "./SiteFooter";
import { Drawer } from "./ui/drawer";

export type PlayerPanelProps = {
  readonly canvasStage: boolean;
  readonly sceneTitle: string;
  readonly status: PlayerStatus;
  readonly maybeDetails?: string | undefined;
  readonly interactionHint: string;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly assignParticleSurface: (canvas: HTMLCanvasElement) => void;
  readonly onPlay: () => void;
  readonly onPause: () => void;
  readonly onReset: () => void;
  readonly onRetry: () => void;
  readonly renderMode: RenderMode;
  readonly onRenderModeChange: (mode: RenderMode) => void;
  readonly wireframeStrokeWidth: number;
  readonly onWireframeStrokeWidthChange: (width: number) => void;
  readonly debugEnabled: boolean;
  readonly onDebugEnabledChange: (enabled: boolean) => void;
  readonly maybeDebugFrame: RenderFrame | undefined;
  readonly stepsThisFrame: number;
  readonly fpsTicks: readonly FpsTick[];
  readonly renderedParticleDraft: string;
  readonly onRenderedParticleDraft: (raw: string) => void;
  readonly panEnabled: boolean;
  readonly tiltGravityEnabled: boolean;
  readonly tiltDebug: TiltDebug;
  readonly onTiltGravityEnabledChange: (enabled: boolean) => void;
  readonly onZoomIn: () => void;
  readonly onZoomOut: () => void;
  readonly onResetZoom: () => void;
  readonly onPanEnabledChange: (enabled: boolean) => void;
  readonly hudControls: HudControlSliderProps;
  readonly children?: JSX.Element;
};

const RUNTIME_LABEL = "Runtime";
const RUNTIME_VALUE = "Rust engine · WebAssembly";
const PLAYING_STATUS = "Playing";
const PAUSED_STATUS = "Paused";
const LOADING_OVERLAY_BODY =
  "Starting the Rust WebAssembly session. The scene appears when the first frame is ready.";
const SHEET_DESCRIPTION =
  "Particle rendering, playback options, scene parameters, and project details.";

function statusText(status: PlayerStatus, sceneTitle: string): string {
  switch (status) {
    case "loading":
      return `Loading ${sceneTitle}…`;
    case "playing":
      return PLAYING_STATUS;
    case "paused":
      return PAUSED_STATUS;
    case "failed":
      return `${sceneTitle} failed`;
  }
}

function failureCopy(sceneTitle: string): string {
  return `${sceneTitle} could not start or continue. Use Retry scene to recreate it, or Reset scene to return to the documented initial state.`;
}

/** Presentational player chrome for any ready scene. */
export function PlayerPanel(props: PlayerPanelProps) {
  return (
    <Show
      when={props.canvasStage}
      fallback={<PlayerPanelLayout canvasStage={false} panel={props} />}
    >
      <Drawer side="bottom">
        <PlayerPanelLayout canvasStage panel={props} />
      </Drawer>
    </Show>
  );
}

function PlayerPanelLayout(layoutProps: {
  readonly canvasStage: boolean;
  readonly panel: PlayerPanelProps;
}) {
  const props = layoutProps.panel;
  return (
    <section
      class="player-panel"
      classList={{ "player-panel--canvas-stage": layoutProps.canvasStage }}
      aria-labelledby="player-title"
    >
      <Show when={!layoutProps.canvasStage}>
        <div class="status-row">
          <h2 id="player-title">{props.sceneTitle}</h2>
          <output
            class={`session-status session-status--${props.status}`}
            aria-live="polite"
            aria-atomic="true"
          >
            <span class="status-dot" aria-hidden="true" />
            {statusText(props.status, props.sceneTitle)}
          </output>
          <RuntimeIdentity />
        </div>
      </Show>

      <Show when={!layoutProps.canvasStage && props.status === "failed"}>
        <FailureMessage
          sceneTitle={props.sceneTitle}
          maybeDetails={props.maybeDetails}
        />
      </Show>

      <figure>
        <div class="viewport-frame">
          <canvas
            class="scene-canvas"
            data-particle-surface={particleSurface(props.renderMode)}
            ref={(canvas) => props.assignCanvas(canvas)}
            width="960"
            height="540"
            role="img"
            aria-label={`Live ${props.sceneTitle} scene: Rust particles and rigid bodies.`}
            aria-describedby="scene-interaction-hint"
          >
            {`Canvas is required to display the ${props.sceneTitle} scene.`}
          </canvas>
          <canvas
            class="particle-surface"
            data-active={needsWebglSurface(props.renderMode) ? "true" : "false"}
            ref={(canvas) => props.assignParticleSurface(canvas)}
            width="960"
            height="540"
            aria-hidden="true"
          />
          <Show when={props.status === "loading"}>
            <div class="empty-state">
              <strong>{`Loading ${props.sceneTitle}`}</strong>
              <span>{LOADING_OVERLAY_BODY}</span>
            </div>
          </Show>
          <Show when={layoutProps.canvasStage && props.status === "failed"}>
            <div class="empty-state">
              <strong>{statusText(props.status, props.sceneTitle)}</strong>
              <span>{failureCopy(props.sceneTitle)}</span>
            </div>
          </Show>
          <DebugReadout
            enabled={props.debugEnabled}
            maybeFrame={props.maybeDebugFrame}
            stepsThisFrame={props.stepsThisFrame}
            fpsTicks={props.fpsTicks}
            onClose={() => props.onDebugEnabledChange(false)}
          />
          <Show when={!layoutProps.canvasStage}>
            <GravityArrow
              tiltGravityEnabled={props.tiltGravityEnabled}
              tiltDebug={props.tiltDebug}
            />
          </Show>
          <Show when={layoutProps.canvasStage}>
            <figcaption id="scene-interaction-hint" class="visually-hidden">
              {props.interactionHint}
            </figcaption>
          </Show>
          <Show
            when={layoutProps.canvasStage}
            fallback={
              <ViewportTools
                showFullscreen
                panEnabled={props.panEnabled}
                onZoomIn={props.onZoomIn}
                onZoomOut={props.onZoomOut}
                onResetZoom={props.onResetZoom}
                onPanEnabledChange={props.onPanEnabledChange}
              />
            }
          >
            <CanvasHud
              compact
              sceneTitle={props.sceneTitle}
              status={props.status}
              statusLabel={statusText(props.status, props.sceneTitle)}
              fpsTicks={props.fpsTicks}
              onPlay={props.onPlay}
              onPause={props.onPause}
              onReset={props.onReset}
              onRetry={props.onRetry}
              panEnabled={props.panEnabled}
              onZoomIn={props.onZoomIn}
              onZoomOut={props.onZoomOut}
              onResetZoom={props.onResetZoom}
              onPanEnabledChange={props.onPanEnabledChange}
              tiltGravityEnabled={props.tiltGravityEnabled}
              tiltDebug={props.tiltDebug}
              onTiltGravityEnabledChange={props.onTiltGravityEnabledChange}
            >
              <HudControlSlider {...props.hudControls} />
            </CanvasHud>
          </Show>
        </div>
        <Show when={!layoutProps.canvasStage}>
          <figcaption id="scene-interaction-hint">
            {props.interactionHint}
          </figcaption>
        </Show>
      </figure>

      <Show
        when={layoutProps.canvasStage}
        fallback={
          <>
            <div class="playback-stack">
              <HudControlSlider {...props.hudControls} />
              <div class="control-row">
                <PlaybackButtons
                  compact={false}
                  status={props.status}
                  onPlay={props.onPlay}
                  onPause={props.onPause}
                  onReset={props.onReset}
                  onRetry={props.onRetry}
                />
                <PlayerOptions panel={props} />
              </div>
            </div>
            <TiltPane panel={props} />
            {props.children}
          </>
        }
      >
        <SceneControlsSheet description={SHEET_DESCRIPTION}>
          <Show when={props.status === "failed"}>
            <FailureMessage
              sceneTitle={props.sceneTitle}
              maybeDetails={props.maybeDetails}
            />
          </Show>
          <p class="scene-interaction-hint">{props.interactionHint}</p>
          <RuntimeIdentity />
          <div class="control-row">
            <PlayerOptions panel={props} />
          </div>
          <TiltPane panel={props} />
          {props.children}
          <SiteFooter />
        </SceneControlsSheet>
      </Show>
    </section>
  );
}

function RuntimeIdentity() {
  return (
    <dl class="player-identity">
      <dt>{RUNTIME_LABEL}</dt>
      <dd>{RUNTIME_VALUE}</dd>
    </dl>
  );
}

function FailureMessage(props: {
  readonly sceneTitle: string;
  readonly maybeDetails: string | undefined;
}) {
  return (
    <div class="error-message" role="alert">
      <p>{failureCopy(props.sceneTitle)}</p>
      <Show when={props.maybeDetails}>
        {(details) => (
          <details class="failure-debug" open>
            <summary>Debug details</summary>
            <pre>{details()}</pre>
          </details>
        )}
      </Show>
    </div>
  );
}

function PlayerOptions(props: { readonly panel: PlayerPanelProps }) {
  const panel = props.panel;
  return (
    <>
      <label class="render-mode-control">
        Particles
        <select
          value={panel.renderMode}
          onChange={(event) => {
            const maybeMode = maybeParseRenderMode(event.currentTarget.value);
            if (maybeMode !== undefined) {
              panel.onRenderModeChange(maybeMode);
            }
          }}
        >
          {RENDER_MODE_GROUPS.map((group) => (
            <optgroup label={group.label}>
              {group.options.map((option) => (
                <option value={option.value}>{option.label}</option>
              ))}
            </optgroup>
          ))}
        </select>
      </label>
      <Show when={panel.renderMode === "wireframe"}>
        <label class="stroke-width-control">
          Wireframe stroke
          <input
            id="wireframe-stroke"
            type="range"
            min={WIREFRAME_STROKE_WIDTH_MIN}
            max={WIREFRAME_STROKE_WIDTH_MAX}
            step={WIREFRAME_STROKE_WIDTH_STEP}
            value={panel.wireframeStrokeWidth}
            onInput={(event) => {
              const maybeWidth = maybeParseWireframeStrokeWidth(
                event.currentTarget.value,
              );
              if (maybeWidth !== undefined) {
                panel.onWireframeStrokeWidthChange(maybeWidth);
              }
            }}
          />
          <output for="wireframe-stroke">
            {formatWireframeStrokeWidth(panel.wireframeStrokeWidth)}
          </output>
        </label>
      </Show>
      <label class="rendered-particle-control">
        Rendered particles
        <input
          type="text"
          inputMode="numeric"
          spellcheck={false}
          value={panel.renderedParticleDraft}
          onInput={(event) => {
            panel.onRenderedParticleDraft(event.currentTarget.value);
          }}
        />
      </label>
      <Show when={!usesParticleStride(panel.renderMode)}>
        <p class="particle-cap-note">Surface modes draw every particle.</p>
      </Show>
      <label class="debug-toggle">
        <input
          type="checkbox"
          checked={panel.debugEnabled}
          onChange={(event) => {
            panel.onDebugEnabledChange(event.currentTarget.checked);
          }}
        />
        Debug info
      </label>
    </>
  );
}

function TiltPane(props: { readonly panel: PlayerPanelProps }) {
  const panel = props.panel;
  return (
    <section class="tilt-pane" aria-labelledby="tilt-pane-title">
      <h3 id="tilt-pane-title">Phone tilt gravity</h3>
      <label class="player-check">
        <input
          type="checkbox"
          checked={panel.tiltGravityEnabled}
          onChange={(event) => {
            panel.onTiltGravityEnabledChange(event.currentTarget.checked);
          }}
        />
        Use phone accelerometer
      </label>
      <p class="tilt-debug">
        Tilt sets the direction. On a scene with a gravity slider, that slider is
        the strength of one standard g.
      </p>
      <p
        class="tilt-debug"
        classList={{
          "tilt-debug-problem": panel.tiltDebug.kind === "problem",
        }}
        role="status"
      >
        {formatTiltDebug(panel.tiltDebug)}
      </p>
    </section>
  );
}
