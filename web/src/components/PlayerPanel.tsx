import { Show, type JSX } from "solid-js";

import type { PlayerStatus } from "../player/view";
import type { RenderFrame } from "../physics/frame";
import {
  maybeParseRenderMode,
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

export type PlayerPanelProps = {
  readonly sceneTitle: string;
  readonly status: PlayerStatus;
  readonly maybeDetails?: string | undefined;
  readonly interactionHint: string;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
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
  readonly children?: JSX.Element;
};

const RUNTIME_LABEL = "Runtime";
const RUNTIME_VALUE = "Rust engine · WebAssembly";
const PLAYING_STATUS = "Playing";
const PAUSED_STATUS = "Paused";
const PLAY_LABEL = "Play scene";
const PAUSE_LABEL = "Pause scene";
const RESET_LABEL = "Reset scene";
const RETRY_LABEL = "Retry scene";
const LOADING_OVERLAY_BODY =
  "Starting the Rust WebAssembly session. The scene appears when the first frame is ready.";

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

function playDisabled(status: PlayerStatus): boolean {
  return status !== "paused";
}

function pauseDisabled(status: PlayerStatus): boolean {
  return status !== "playing";
}

function resetDisabled(status: PlayerStatus): boolean {
  return status === "loading";
}

/** Presentational player chrome for any ready scene. */
export function PlayerPanel(props: PlayerPanelProps) {
  return (
    <section class="player-panel" aria-labelledby="player-title">
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
        <dl class="player-identity">
          <dt>{RUNTIME_LABEL}</dt>
          <dd>{RUNTIME_VALUE}</dd>
        </dl>
      </div>

      <Show when={props.status === "failed"}>
        <div class="error-message" role="alert">
          <p>
            {`${props.sceneTitle} could not start or continue. Use Retry scene to recreate it, or Reset scene to return to the documented initial state.`}
          </p>
          <Show when={props.maybeDetails}>
            {(details) => (
              <details class="failure-debug" open>
                <summary>Debug details</summary>
                <pre>{details()}</pre>
              </details>
            )}
          </Show>
        </div>
      </Show>

      <figure>
        <div class="viewport-frame">
          <canvas
            ref={(canvas) => props.assignCanvas(canvas)}
            width="960"
            height="540"
            role="img"
            aria-label={`Live ${props.sceneTitle} scene: Rust particles and rigid bodies.`}
            aria-describedby="scene-interaction-hint"
          >
            {`Canvas is required to display the ${props.sceneTitle} scene.`}
          </canvas>
          <Show when={props.status === "loading"}>
            <div class="empty-state">
              <strong>{`Loading ${props.sceneTitle}`}</strong>
              <span>{LOADING_OVERLAY_BODY}</span>
            </div>
          </Show>
          <DebugReadout
            enabled={props.debugEnabled}
            maybeFrame={props.maybeDebugFrame}
            stepsThisFrame={props.stepsThisFrame}
            fpsTicks={props.fpsTicks}
          />
          <GravityArrow
            debugEnabled={props.debugEnabled}
            tiltGravityEnabled={props.tiltGravityEnabled}
            tiltDebug={props.tiltDebug}
          />
          <ViewportTools
            panEnabled={props.panEnabled}
            onZoomIn={props.onZoomIn}
            onZoomOut={props.onZoomOut}
            onResetZoom={props.onResetZoom}
            onPanEnabledChange={props.onPanEnabledChange}
          />
        </div>
        <figcaption id="scene-interaction-hint">{props.interactionHint}</figcaption>
      </figure>

      <div class="control-row">
        <Show when={props.status === "failed"}>
          <button
            class="product-control product-control--accent"
            type="button"
            onClick={() => props.onRetry()}
          >
            {RETRY_LABEL}
          </button>
        </Show>
        <button
          class="product-control product-control--accent"
          type="button"
          disabled={playDisabled(props.status)}
          onClick={() => props.onPlay()}
        >
          {PLAY_LABEL}
        </button>
        <button
          class="product-control"
          type="button"
          disabled={pauseDisabled(props.status)}
          onClick={() => props.onPause()}
        >
          {PAUSE_LABEL}
        </button>
        <button
          class="product-control"
          type="button"
          disabled={resetDisabled(props.status)}
          onClick={() => props.onReset()}
        >
          {RESET_LABEL}
        </button>
        <label class="render-mode-control">
          Rendering
          <select
            value={props.renderMode}
            onChange={(event) => {
              const maybeMode = maybeParseRenderMode(
                event.currentTarget.value,
              );
              if (maybeMode !== undefined) {
                props.onRenderModeChange(maybeMode);
              }
            }}
          >
            <option value="wireframe">Wireframe</option>
            <option value="solid">Solid</option>
          </select>
        </label>
        <label class="stroke-width-control">
          Wireframe stroke
          <input
            id="wireframe-stroke"
            type="range"
            min={WIREFRAME_STROKE_WIDTH_MIN}
            max={WIREFRAME_STROKE_WIDTH_MAX}
            step={WIREFRAME_STROKE_WIDTH_STEP}
            value={props.wireframeStrokeWidth}
            onInput={(event) => {
              const maybeWidth = maybeParseWireframeStrokeWidth(
                event.currentTarget.value,
              );
              if (maybeWidth !== undefined) {
                props.onWireframeStrokeWidthChange(maybeWidth);
              }
            }}
          />
          <output for="wireframe-stroke">
            {formatWireframeStrokeWidth(props.wireframeStrokeWidth)}
          </output>
        </label>
        <label class="rendered-particle-control">
          Rendered particles
          <input
            type="text"
            inputMode="numeric"
            spellcheck={false}
            value={props.renderedParticleDraft}
            onInput={(event) => {
              props.onRenderedParticleDraft(event.currentTarget.value);
            }}
          />
        </label>
        <label class="debug-toggle">
          <input
            type="checkbox"
            checked={props.debugEnabled}
            onChange={(event) => {
              props.onDebugEnabledChange(event.currentTarget.checked);
            }}
          />
          Debug info
        </label>
      </div>

      <section class="tilt-pane" aria-labelledby="tilt-pane-title">
        <h3 id="tilt-pane-title">Phone tilt gravity</h3>
        <label class="player-check">
          <input
            type="checkbox"
            checked={props.tiltGravityEnabled}
            onChange={(event) => {
              props.onTiltGravityEnabledChange(event.currentTarget.checked);
            }}
          />
          Use phone accelerometer
        </label>
        <p
          class="tilt-debug"
          classList={{
            "tilt-debug-problem": props.tiltDebug.kind === "problem",
          }}
          role="status"
        >
          {formatTiltDebug(props.tiltDebug)}
        </p>
      </section>

      {props.children}
    </section>
  );
}
