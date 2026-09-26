import { Show, createSignal, onCleanup, onMount, type JSX } from "solid-js";

import type { PlayerStatus } from "../player/view";
import { GitHubSourceLink } from "./SiteHeader";
import { ViewportTools } from "./ViewportTools";
import {
  FPS_COUNTER_WINDOW_MS,
  fpsOverWindow,
  fpsTone,
  renderedFpsLabel,
  type FpsTick,
} from "./fps-meter";
import {
  DrawerContent,
  DrawerDescription,
  DrawerLabel,
  DrawerTrigger,
} from "./ui/drawer";

const PLAY_LABEL = "Play scene";
const PAUSE_LABEL = "Pause scene";
const RESET_LABEL = "Reset scene";
const RETRY_LABEL = "Retry scene";
const CONTROLS_LABEL = "Scene controls";

export type PlaybackButtonsProps = {
  readonly compact: boolean;
  readonly status: PlayerStatus;
  readonly onPlay: () => void;
  readonly onPause: () => void;
  readonly onReset: () => void;
  readonly onRetry: () => void;
};

export type CanvasHudProps = PlaybackButtonsProps & {
  readonly sceneTitle: string;
  readonly statusLabel: string;
  readonly fpsTicks: readonly FpsTick[];
  readonly panEnabled: boolean;
  readonly onZoomIn: () => void;
  readonly onZoomOut: () => void;
  readonly onResetZoom: () => void;
  readonly onPanEnabledChange: (enabled: boolean) => void;
};

function playDisabled(status: PlayerStatus): boolean {
  return status !== "paused";
}

function pauseDisabled(status: PlayerStatus): boolean {
  return status !== "playing";
}

function resetDisabled(status: PlayerStatus): boolean {
  return status === "loading";
}

/** Play, pause, reset, and retry. Compact mode is the canvas HUD. */
export function PlaybackButtons(props: PlaybackButtonsProps) {
  return (
    <>
      <Show when={props.status === "failed"}>
        <button
          class={
            props.compact
              ? "hud-button"
              : "product-control product-control--accent"
          }
          type="button"
          aria-label={props.compact ? RETRY_LABEL : undefined}
          onClick={() => props.onRetry()}
        >
          <Show when={props.compact} fallback={RETRY_LABEL}>
            <RetryIcon />
          </Show>
        </button>
      </Show>
      <button
        class={
          props.compact ? "hud-button" : "product-control product-control--accent"
        }
        type="button"
        disabled={playDisabled(props.status)}
        aria-label={props.compact ? PLAY_LABEL : undefined}
        onClick={() => props.onPlay()}
      >
        <Show when={props.compact} fallback={PLAY_LABEL}>
          <PlayIcon />
        </Show>
      </button>
      <button
        class={props.compact ? "hud-button" : "product-control"}
        type="button"
        disabled={pauseDisabled(props.status)}
        aria-label={props.compact ? PAUSE_LABEL : undefined}
        onClick={() => props.onPause()}
      >
        <Show when={props.compact} fallback={PAUSE_LABEL}>
          <PauseIcon />
        </Show>
      </button>
      <button
        class={props.compact ? "hud-button" : "product-control"}
        type="button"
        disabled={resetDisabled(props.status)}
        aria-label={props.compact ? RESET_LABEL : undefined}
        onClick={() => props.onReset()}
      >
        <Show when={props.compact} fallback={RESET_LABEL}>
          <ResetIcon />
        </Show>
      </button>
    </>
  );
}

/** Title, status, transport, and zoom controls over the canvas. */
export function CanvasHud(props: CanvasHudProps) {
  return (
    <>
      <div class="canvas-hud-title">
        <RenderedFps fpsTicks={props.fpsTicks} />
        <h2 id="player-title">{props.sceneTitle}</h2>
        <output
          class={`session-status session-status--${props.status}`}
          aria-live="polite"
          aria-atomic="true"
        >
          <span class="status-dot" aria-hidden="true" />
          {props.statusLabel}
        </output>
      </div>
      <div class="canvas-hud-bottom">
        <div class="canvas-transport">
          <PlaybackButtons
            compact
            status={props.status}
            onPlay={props.onPlay}
            onPause={props.onPause}
            onReset={props.onReset}
            onRetry={props.onRetry}
          />
        </div>
        <ViewportTools
          showFullscreen={false}
          panEnabled={props.panEnabled}
          onZoomIn={props.onZoomIn}
          onZoomOut={props.onZoomOut}
          onResetZoom={props.onResetZoom}
          onPanEnabledChange={props.onPanEnabledChange}
        />
      </div>
      <SceneControlsTrigger />
    </>
  );
}

export function SceneControlsTrigger() {
  return (
    <DrawerTrigger
      class="canvas-controls-trigger hud-button"
      aria-label={CONTROLS_LABEL}
    >
      <ControlsIcon />
    </DrawerTrigger>
  );
}

export function SceneControlsSheet(props: {
  readonly description: string;
  readonly children: JSX.Element;
}) {
  return (
    <DrawerContent class="canvas-controls-sheet">
      <DrawerLabel>{CONTROLS_LABEL}</DrawerLabel>
      <DrawerDescription class="visually-hidden">
        {props.description}
      </DrawerDescription>
      <div class="canvas-controls-scroll">
        <div class="canvas-controls-body">
          <GitHubSourceLink />
          {props.children}
        </div>
      </div>
    </DrawerContent>
  );
}

function RenderedFps(props: { readonly fpsTicks: readonly FpsTick[] }) {
  const [nowMs, setNowMs] = createSignal(performance.now());

  onMount(() => {
    const id = window.setInterval(() => setNowMs(performance.now()), 100);
    onCleanup(() => window.clearInterval(id));
  });

  const fps = () =>
    fpsOverWindow(props.fpsTicks, nowMs(), FPS_COUNTER_WINDOW_MS).renderFps;

  return (
    <output class="canvas-fps" data-fps-tone={fpsTone(fps())}>
      {renderedFpsLabel(props.fpsTicks, nowMs())}
    </output>
  );
}

function PlayIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 6v12l10-6z" />
    </svg>
  );
}

function PauseIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 6v12M16 6v12" />
    </svg>
  );
}

function ResetIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 12a8 8 0 1 0 2-5.3M4 4v4h4" />
    </svg>
  );
}

function RetryIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M20 12a8 8 0 1 1-2.3-5.7M20 4v4h-4" />
    </svg>
  );
}

function ControlsIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 8h16M4 16h16M8 6v4M16 14v4" />
    </svg>
  );
}
