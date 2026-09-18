import { Show, type JSX } from "solid-js";

export type PlayerStatus = "loading" | "playing" | "paused" | "failed";

export type PlayerPanelProps = {
  readonly sceneTitle: string;
  readonly status: PlayerStatus;
  readonly maybeDetails?: string | undefined;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly onPlay: () => void;
  readonly onPause: () => void;
  readonly onReset: () => void;
  readonly onRetry: () => void;
  readonly children?: JSX.Element;
};

const RUNTIME_LABEL = "Runtime";
const RUNTIME_VALUE = "Rust engine · WebAssembly";
const PLAYING_STATUS = "Playing";
const PAUSED_STATUS = "Paused";
const CANVAS_CAPTION =
  "Live frame from this repository's Rust engine, drawn with Canvas 2D.";
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
            {(details) => <p class="error-details">{details()}</p>}
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
          >
            {`Canvas is required to display the ${props.sceneTitle} scene.`}
          </canvas>
          <Show when={props.status === "loading"}>
            <div class="empty-state">
              <strong>{`Loading ${props.sceneTitle}`}</strong>
              <span>{LOADING_OVERLAY_BODY}</span>
            </div>
          </Show>
        </div>
        <figcaption>{CANVAS_CAPTION}</figcaption>
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
      </div>

      {props.children}
    </section>
  );
}
