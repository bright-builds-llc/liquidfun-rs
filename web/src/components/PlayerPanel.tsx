import { Show } from "solid-js";

export type PlayerStatus = "loading" | "playing" | "paused" | "failed";

export type PlayerPanelProps = {
  readonly status: PlayerStatus;
  readonly maybeDetails?: string | undefined;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly onPlay: () => void;
  readonly onPause: () => void;
  readonly onReset: () => void;
  readonly onRetry: () => void;
};

const PLAYER_HEADING = "Dam Break";
const RUNTIME_LABEL = "Runtime";
const RUNTIME_VALUE = "Rust engine · WebAssembly";
const LOADING_STATUS = "Loading Dam Break…";
const PLAYING_STATUS = "Playing";
const PAUSED_STATUS = "Paused";
const FAILED_STATUS = "Dam Break failed";
const CANVAS_CAPTION =
  "Live frame from this repository's Rust engine, drawn with Canvas 2D.";
const CANVAS_NAME =
  "Live Dam Break scene: Rust particles and a rigid body in a basin.";
const CANVAS_FALLBACK = "Canvas is required to display the Dam Break scene.";
const PLAY_LABEL = "Play scene";
const PAUSE_LABEL = "Pause scene";
const RESET_LABEL = "Reset scene";
const RETRY_LABEL = "Retry scene";
const FAILURE_COPY =
  "Dam Break could not start or continue. Use Retry scene to recreate it, or Reset scene to return to the documented initial state.";
const LOADING_OVERLAY_HEADING = "Loading Dam Break";
const LOADING_OVERLAY_BODY =
  "Starting the Rust WebAssembly session. The scene appears when the first frame is ready.";

function statusText(status: PlayerStatus): string {
  switch (status) {
    case "loading":
      return LOADING_STATUS;
    case "playing":
      return PLAYING_STATUS;
    case "paused":
      return PAUSED_STATUS;
    case "failed":
      return FAILED_STATUS;
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

/** Presentational Dam Break player chrome without a WASM session. */
export function PlayerPanel(props: PlayerPanelProps) {
  return (
    <section class="player-panel" aria-labelledby="player-title">
      <div class="status-row">
        <h2 id="player-title">{PLAYER_HEADING}</h2>
        <output
          class={`session-status session-status--${props.status}`}
          aria-live="polite"
          aria-atomic="true"
        >
          <span class="status-dot" aria-hidden="true" />
          {statusText(props.status)}
        </output>
        <dl class="player-identity">
          <dt>{RUNTIME_LABEL}</dt>
          <dd>{RUNTIME_VALUE}</dd>
        </dl>
      </div>

      <Show when={props.status === "failed"}>
        <div class="error-message" role="alert">
          <p>{FAILURE_COPY}</p>
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
            aria-label={CANVAS_NAME}
          >
            {CANVAS_FALLBACK}
          </canvas>
          <Show when={props.status === "loading"}>
            <div class="empty-state">
              <strong>{LOADING_OVERLAY_HEADING}</strong>
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
    </section>
  );
}
