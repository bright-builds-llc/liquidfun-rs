import "../styles/puzzle.css";
import "../styles/puzzle-editor.css";

import {
  Show,
  createEffect,
  createSignal,
  on,
  onCleanup,
  onMount,
  type JSX,
} from "solid-js";

import { maybeNextPuzzleId, puzzleTitleForId } from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";
import { createHudObstacles } from "../player/hud-obstacles";
import type { PlayerStatus } from "../player/view";
import type { PuzzleController } from "../puzzle/controller";
import { needsWebglSurface, particleSurface, type RenderMode } from "../render/mode";
import type { ScreenRect } from "../render/safe-frame";
import {
  PuzzleFillMeter,
  PuzzleHud,
  PuzzlePlayButton,
  PuzzleTimer,
  PuzzleTitle,
} from "./PuzzleHud";
import {
  PuzzleOptionsSheet,
  PuzzleOptionsTrigger,
  PuzzleRenderModeSelect,
} from "./PuzzleOptionsDrawer";
import { PuzzleResultCard } from "./PuzzleResultCard";
import { ScaleLegend } from "./ScaleLegend";
import { Drawer } from "./ui/drawer";

export type PuzzlePanelProps = {
  readonly canvasStage: boolean;
  readonly levelId: PuzzleId;
  readonly controller: PuzzleController;
  readonly status: PlayerStatus;
  readonly maybeFailureDetails: string | undefined;
  readonly renderMode: RenderMode;
  readonly onRenderModeChange: (mode: RenderMode) => void;
  readonly maybePixelsPerMeter: number | undefined;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly assignParticleSurface: (canvas: HTMLCanvasElement) => void;
  readonly onHudObstaclesChange: (obstacles: readonly ScreenRect[]) => void;
};

type PuzzleLayout = "landscape" | "portrait";

const LOADING_BODY = "Starting the physics engine.";
const FAILURE_HEADING = "This level could not start";
const FAILURE_BODY =
  "Something went wrong while building the level. Try again, or open a scene from the list.";

/** Same orientation rule the camera frame uses: portrait iff taller than wide. */
function layoutFor(width: number, height: number): PuzzleLayout {
  return height > width ? "portrait" : "landscape";
}

/**
 * Puzzle player chrome for one level: canvas markup, fixed HUD slots in both
 * layouts, loading and failure overlays, and the puzzle options drawer. It
 * reads and commands the puzzle only through `PuzzleController`.
 */
export function PuzzlePanel(props: PuzzlePanelProps) {
  const [optionsOpen, setOptionsOpen] = createSignal(false);
  // The sheet covers most of the canvas, so a new level starts with it closed.
  createEffect(
    on(
      () => props.levelId,
      () => {
        setOptionsOpen(false);
      },
      { defer: true },
    ),
  );

  return (
    <Show when={props.canvasStage} fallback={<PuzzlePanelLayout panel={props} />}>
      <Drawer side="bottom" open={optionsOpen()} onOpenChange={setOptionsOpen}>
        <PuzzlePanelLayout panel={props} />
      </Drawer>
    </Show>
  );
}

function PuzzlePanelLayout(layoutProps: { readonly panel: PuzzlePanelProps }) {
  const props = layoutProps.panel;
  const title = () => puzzleTitleForId(props.levelId);
  const maybeNextId = () => maybeNextPuzzleId(props.levelId);
  const phase = () => props.controller.state().kind;
  const [layout, setLayout] = createSignal<PuzzleLayout>(
    layoutFor(window.innerWidth, window.innerHeight),
  );

  let maybeViewportFrame: HTMLDivElement | undefined;
  let maybeSceneCanvas: HTMLCanvasElement | undefined;
  let maybePlayButton: HTMLButtonElement | undefined;
  let maybePrimaryAction: HTMLElement | undefined;
  let focusInPanel = false;

  onMount(() => {
    const viewportFrame = maybeViewportFrame;
    if (viewportFrame === undefined) {
      return;
    }

    // Layout depends only on viewport geometry, never on the puzzle phase.
    const layoutObserver = new ResizeObserver(() => {
      setLayout(layoutFor(viewportFrame.clientWidth, viewportFrame.clientHeight));
    });
    layoutObserver.observe(viewportFrame);
    onCleanup(() => layoutObserver.disconnect());

    if (!props.canvasStage || maybeSceneCanvas === undefined) {
      return;
    }

    const obstacles = createHudObstacles(viewportFrame, maybeSceneCanvas);
    createEffect(on(obstacles, (rects) => props.onHudObstaclesChange(rects)));
  });

  // UI-D8: a phase change disables or removes the focused control, so move
  // focus to the control that replaces it, but only while focus is in the panel.
  createEffect(
    on(
      phase,
      (kind) => {
        if (!focusInPanel) {
          return;
        }
        if (kind === "building" || kind === "running") {
          maybePlayButton?.focus();
        } else if (kind === "ended") {
          queueMicrotask(() => maybePrimaryAction?.focus());
        }
      },
      { defer: true },
    ),
  );

  const resultCard = (): JSX.Element => (
    <PuzzleResultCard
      state={props.controller.state()}
      maybeNextId={maybeNextId()}
      onRetry={() => props.controller.retry()}
      onEdit={() => props.controller.edit()}
      primaryRef={(element) => {
        maybePrimaryAction = element;
      }}
    />
  );
  const assignPlayButton = (element: HTMLButtonElement) => {
    maybePlayButton = element;
  };

  return (
    <section
      class="puzzle-panel player-panel"
      classList={{
        "puzzle-panel--canvas-stage": props.canvasStage,
        "player-panel--canvas-stage": props.canvasStage,
      }}
      aria-labelledby="player-title"
      data-pixels-per-meter={props.maybePixelsPerMeter}
      data-puzzle-layout={layout()}
      onFocusIn={() => {
        focusInPanel = true;
      }}
      onFocusOut={(event) => {
        const maybeNext = event.relatedTarget;
        if (maybeNext instanceof Node && !event.currentTarget.contains(maybeNext)) {
          focusInPanel = false;
        }
      }}
    >
      <Show when={!props.canvasStage}>
        <div class="status-row">
          <PuzzleTitle title={title()} />
          <PuzzleFillMeter state={props.controller.state()} />
          <PuzzleTimer state={props.controller.state()} />
        </div>
      </Show>

      <figure>
        <div
          class="viewport-frame"
          ref={(frame) => {
            maybeViewportFrame = frame;
          }}
        >
          <canvas
            class="scene-canvas"
            data-particle-surface={particleSurface(props.renderMode)}
            ref={(canvas) => {
              maybeSceneCanvas = canvas;
              props.assignCanvas(canvas);
            }}
            width="960"
            height="540"
            role="img"
            aria-label={`${title()} puzzle: Rust particles pour toward a glass cup.`}
          >
            {`Canvas is required to display the ${title()} puzzle.`}
          </canvas>
          <canvas
            class="particle-surface"
            data-active={needsWebglSurface(props.renderMode) ? "true" : "false"}
            ref={(canvas) => props.assignParticleSurface(canvas)}
            width="960"
            height="540"
            aria-hidden="true"
          />
          <Show when={phase() === "loading" && props.status !== "failed"}>
            <div class="empty-state">
              <strong>{`Loading ${title()}`}</strong>
              <span>{LOADING_BODY}</span>
            </div>
          </Show>
          <FailureOverlay
            failed={props.status === "failed"}
            maybeDetails={props.maybeFailureDetails}
            onTryAgain={() => props.controller.tryAgain()}
          />
          <Show
            when={props.canvasStage}
            fallback={
              <div class="puzzle-legend-slot">
                <ScaleLegend maybePixelsPerMeter={props.maybePixelsPerMeter} />
              </div>
            }
          >
            <PuzzleHud
              title={title()}
              state={props.controller.state()}
              maybePixelsPerMeter={props.maybePixelsPerMeter}
              onPlay={() => props.controller.play()}
              playRef={assignPlayButton}
              resultCard={resultCard()}
              optionsTrigger={<PuzzleOptionsTrigger />}
            />
          </Show>
        </div>
      </figure>

      <Show
        when={props.canvasStage}
        fallback={
          <>
            <div class="control-row">
              <PuzzlePlayButton
                state={props.controller.state()}
                onPlay={() => props.controller.play()}
                ref={assignPlayButton}
              />
              {resultCard()}
            </div>
            <div class="control-row">
              <PuzzleRenderModeSelect
                renderMode={props.renderMode}
                onRenderModeChange={props.onRenderModeChange}
              />
            </div>
          </>
        }
      >
        <PuzzleOptionsSheet>
          <PuzzleRenderModeSelect
            renderMode={props.renderMode}
            onRenderModeChange={props.onRenderModeChange}
          />
        </PuzzleOptionsSheet>
      </Show>
    </section>
  );
}

/**
 * Failure overlay. The assertive region stays mounted and empty until a
 * failure, so the heading is announced when it appears.
 */
function FailureOverlay(props: {
  readonly failed: boolean;
  readonly maybeDetails: string | undefined;
  readonly onTryAgain: () => void;
}) {
  return (
    <div class="empty-state puzzle-failure" data-failed={props.failed ? "true" : "false"}>
      <div aria-live="assertive" aria-atomic="true">
        <Show when={props.failed}>
          <strong class="puzzle-failure-heading">{FAILURE_HEADING}</strong>
        </Show>
      </div>
      <Show when={props.failed}>
        <span>{FAILURE_BODY}</span>
        <button
          type="button"
          class="product-control product-control--accent"
          onClick={() => props.onTryAgain()}
        >
          Try again
        </button>
        <Show when={props.maybeDetails}>
          {(details) => (
            <details class="failure-debug">
              <summary>Failure details</summary>
              <pre>{details()}</pre>
            </details>
          )}
        </Show>
      </Show>
    </div>
  );
}
