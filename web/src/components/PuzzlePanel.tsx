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

import {
  FIRST_PUZZLE_ID,
  maybeNextPuzzleId,
  puzzleTitleForId,
} from "../catalog/puzzle-levels";
import type { PuzzleId } from "../catalog/puzzles";
import { firstHintMemory } from "../player/first-hint-preference";
import { createHudObstacles } from "../player/hud-obstacles";
import type { PlayerStatus } from "../player/view";
import { captionModel } from "../puzzle/caption";
import type { PuzzleController } from "../puzzle/controller";
import { isRepeatedPlayToggle, keyTargetOf, puzzleKeyCommand } from "../puzzle/keyboard";
import { needsWebglSurface, particleSurface, type RenderMode } from "../render/mode";
import type { ScreenRect } from "../render/safe-frame";
import { PuzzleCaption } from "./PuzzleCaption";
import { PuzzleFillMeter, PuzzleHud, PuzzleTimer, PuzzleTitle } from "./PuzzleHud";
import {
  PuzzleOptionsSheet,
  PuzzleOptionsTrigger,
  PuzzleRenderModeSelect,
} from "./PuzzleOptionsDrawer";
import { PuzzleResultCard } from "./PuzzleResultCard";
import { PuzzleSelectionBar } from "./PuzzleSelectionBar";
import { PuzzleToolTray } from "./PuzzleToolTray";
import { PuzzleTransport } from "./PuzzleTransport";
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
const KEYBOARD_HELP_ID = "puzzle-keyboard-help";
const KEYBOARD_HELP =
  "Build area. Pick a tool from the tray, or press Enter on a tray tool to place it in the middle. Tab selects placed tools. Arrow keys move the selected tool, Shift+Arrow moves it further. R rotates counterclockwise, Shift+R clockwise. Delete removes it. Space plays or stops. Escape deselects. [ and ] move a selected valve's opening time by half a second.";

/** True when focus sits on nothing usable: gone, detached or `aria-disabled`. */
function focusNeedsRescue(maybeActive: Element | null): boolean {
  if (!(maybeActive instanceof HTMLElement) || maybeActive === document.body) {
    return true;
  }
  return !maybeActive.isConnected || maybeActive.getAttribute("aria-disabled") === "true";
}

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
    <Show
      when={props.canvasStage}
      fallback={<PuzzlePanelLayout panel={props} optionsOpen={() => false} />}
    >
      <Drawer side="bottom" open={optionsOpen()} onOpenChange={setOptionsOpen}>
        <PuzzlePanelLayout panel={props} optionsOpen={optionsOpen} />
      </Drawer>
    </Show>
  );
}

function PuzzlePanelLayout(layoutProps: {
  readonly panel: PuzzlePanelProps;
  readonly optionsOpen: () => boolean;
}) {
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

  // D-20: Dismiss or placing the first tool on the first level hides the hint
  // for good. The leaf keeps its box, so only focus needs care: a focused
  // Dismiss button is removed, so focus moves to the canvas.
  const [hintDismissed, setHintDismissed] = createSignal(firstHintMemory.dismissed());
  const dismissHint = () => {
    const maybeActive = document.activeElement;
    const dismissFocused =
      maybeActive instanceof HTMLElement && maybeActive.classList.contains("puzzle-caption-dismiss");
    firstHintMemory.dismiss();
    setHintDismissed(true);
    if (dismissFocused) {
      maybeSceneCanvas?.focus();
    }
  };
  createEffect(() => {
    if (hintDismissed() || props.levelId !== FIRST_PUZZLE_ID) {
      return;
    }
    const state = props.controller.state();
    if (state.kind === "building" && state.editor.tools.length > 0) {
      dismissHint();
    }
  });
  const caption = (): JSX.Element => (
    <PuzzleCaption
      model={captionModel(props.levelId, hintDismissed())}
      onDismiss={dismissHint}
    />
  );

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

  // UI-D8, extended: only while focus is in the panel. Entering ended focuses
  // the primary result action. Entering building or running moves focus to
  // Play/Stop only when the focused control became disabled or was removed,
  // so focus is never taken from the canvas or a live control.
  createEffect(
    on(
      phase,
      (kind) => {
        if (!focusInPanel) {
          return;
        }
        if (kind === "ended") {
          queueMicrotask(() => maybePrimaryAction?.focus());
          return;
        }
        const editing = kind === "building" || kind === "running";
        if (editing && focusNeedsRescue(document.activeElement)) {
          maybePlayButton?.focus();
        }
      },
      { defer: true },
    ),
  );

  // D-23: one panel-level key map; preventDefault only for handled keys.
  const handleKeyDown = (event: KeyboardEvent) => {
    if (layoutProps.optionsOpen()) {
      return;
    }
    const maybeCommand = puzzleKeyCommand({
      key: event.key,
      shiftKey: event.shiftKey,
      ctrlKey: event.ctrlKey,
      metaKey: event.metaKey,
      altKey: event.altKey,
      altGraph: event.getModifierState("AltGraph"),
      target: keyTargetOf(event.target),
    });
    if (maybeCommand === undefined) {
      return;
    }
    if (isRepeatedPlayToggle(maybeCommand, event.repeat)) {
      event.preventDefault();
      return;
    }
    if (props.controller.command(maybeCommand)) {
      event.preventDefault();
    }
  };
  const armed = () => {
    const state = props.controller.state();
    return state.kind === "building" && state.editor.maybeArmedKind !== undefined;
  };

  const resultCard = (): JSX.Element => (
    <PuzzleResultCard
      state={props.controller.state()}
      maybeNextId={maybeNextId()}
      maybeParTools={props.controller.maybeLevelInfo()?.parTools}
      maybeBestStars={props.controller.maybeBestStars()}
      onRetry={() => props.controller.retry()}
      onEdit={() => props.controller.edit()}
      primaryRef={(element) => {
        maybePrimaryAction = element;
      }}
    />
  );
  const toolTray = (): JSX.Element => (
    <PuzzleToolTray
      state={props.controller.state()}
      onToggleArm={(kind) => props.controller.toggleArm(kind)}
      onPlaceAtCenter={(kind) => props.controller.placeAtCenter(kind)}
    />
  );
  const selectionBar = (): JSX.Element => (
    <PuzzleSelectionBar
      state={props.controller.state()}
      onRotate={(direction) => props.controller.rotateSelected(direction)}
      onRemove={() => props.controller.removeSelected()}
      onStepValveTime={(direction) => props.controller.stepValveTime(direction)}
    />
  );
  const transport = (): JSX.Element => (
    <PuzzleTransport
      state={props.controller.state()}
      failed={props.status === "failed"}
      onPlay={() => props.controller.play()}
      onStop={() => props.controller.stop()}
      onCycleSpeed={() => props.controller.cycleSpeed()}
      onClear={() => props.controller.clear()}
      playRef={(element) => {
        maybePlayButton = element;
      }}
    />
  );

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
      onKeyDown={handleKeyDown}
    >
      <Show when={!props.canvasStage}>
        <div class="status-row">
          <PuzzleTitle title={title()} />
          <PuzzleFillMeter state={props.controller.state()} />
          <PuzzleTimer state={props.controller.state()} />
          {caption()}
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
            classList={{ "puzzle-canvas--armed": armed() }}
            width="960"
            height="540"
            tabindex="0"
            role="application"
            aria-label={`${title()} puzzle: Rust particles pour toward a glass cup.`}
            aria-describedby={KEYBOARD_HELP_ID}
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
              caption={caption()}
              toolTray={toolTray()}
              selectionBar={selectionBar()}
              transport={transport()}
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
              {transport()}
              {toolTray()}
              {selectionBar()}
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

      <p id={KEYBOARD_HELP_ID} class="visually-hidden">
        {KEYBOARD_HELP}
      </p>
      <div id="puzzle-editor-status" class="visually-hidden" aria-live="polite" aria-atomic="true">
        {props.controller.announcement()}
      </div>
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
