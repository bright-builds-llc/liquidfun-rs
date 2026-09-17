import { createSignal, onCleanup, onMount, Show } from "solid-js";

import type { RenderFrame } from "./physics/frame";
import { loadProofSession } from "./physics/loader";
import {
  createSceneSession,
  type SceneSession,
} from "./physics/session";
import {
  drawRenderFrame,
  resizeCanvasBackingStore,
} from "./render/canvas";
import type { Camera } from "./render/camera";

const LOADING_STATUS = "Loading Rust/WASM session…";
const RUNNING_STATUS = "Running Rust/WASM session";
const FAILURE_STATUS = "Rust/WASM session failed";
const DISPOSED_STATUS = "Rust/WASM session disposed";
const ERROR_COPY =
  "The Rust/WASM session could not start or continue. Reload the page; if it still fails, rebuild the browser proof.";
const MAX_ERROR_DETAIL_LENGTH = 240;

type FrameObservation = {
  readonly particleCount: number;
  readonly rigidShapeCount: number;
  readonly stepIndex: number;
  readonly movedFrameCount: number;
};

type ProofState =
  | { readonly kind: "loading" }
  | { readonly kind: "running"; readonly frame: FrameObservation }
  | {
      readonly kind: "failure";
      readonly maybeFrame: FrameObservation | undefined;
      readonly maybeDetails: string | undefined;
    }
  | { readonly kind: "disposed"; readonly frame: FrameObservation };

function maybeObservedFrame(
  state: ProofState,
): FrameObservation | undefined {
  if (state.kind === "running" || state.kind === "disposed") {
    return state.frame;
  }

  if (state.kind === "failure") {
    return state.maybeFrame;
  }

  return undefined;
}

function statusText(state: ProofState): string {
  switch (state.kind) {
    case "loading":
      return LOADING_STATUS;
    case "running":
      return RUNNING_STATUS;
    case "failure":
      return FAILURE_STATUS;
    case "disposed":
      return DISPOSED_STATUS;
  }
}

function maybeDevelopmentDetails(error: unknown): string | undefined {
  if (!import.meta.env.DEV) {
    return undefined;
  }

  const message =
    error instanceof Error ? error.message : "Unknown browser proof failure";
  return `Details: ${message.slice(0, MAX_ERROR_DETAIL_LENGTH)}`;
}

function maybeFailureDetails(state: ProofState): string | undefined {
  return state.kind === "failure" ? state.maybeDetails : undefined;
}

function valuesDiffer(
  previousValues: Float32Array,
  currentValues: Float32Array,
): boolean {
  if (previousValues.length !== currentValues.length) {
    return true;
  }

  for (let index = 0; index < currentValues.length; index += 1) {
    if (previousValues[index] !== currentValues[index]) {
      return true;
    }
  }

  return false;
}

function frameMoved(
  previousFrame: RenderFrame,
  currentFrame: RenderFrame,
): boolean {
  return (
    valuesDiffer(
      previousFrame.particlePositions,
      currentFrame.particlePositions,
    ) ||
    valuesDiffer(previousFrame.rigidCircles, currentFrame.rigidCircles)
  );
}

function observeFrame(
  frame: RenderFrame,
  maybePreviousFrame: RenderFrame | undefined,
  previousMovedFrameCount: number,
): FrameObservation {
  const movedFrameCount =
    maybePreviousFrame !== undefined &&
    frameMoved(maybePreviousFrame, frame)
      ? previousMovedFrameCount + 1
      : previousMovedFrameCount;

  return {
    particleCount: frame.particleCount,
    rigidShapeCount: frame.rigidShapeCount,
    stepIndex: frame.stepIndex,
    movedFrameCount,
  };
}

type ProofPageProps = {
  readonly state: ProofState;
  readonly assignCanvas: (canvas: HTMLCanvasElement) => void;
  readonly onDispose: () => void;
};

function ProofPage(props: ProofPageProps) {
  const maybeFrame = () => maybeObservedFrame(props.state);
  const maybeDetails = () => maybeFailureDetails(props.state);
  const hasFrame = () => maybeFrame() !== undefined;

  return (
    <main
      aria-labelledby="proof-title"
      data-wasm-initialized={hasFrame() ? "true" : undefined}
      data-step-index={maybeFrame()?.stepIndex}
      data-moved-frame-count={maybeFrame()?.movedFrameCount}
    >
      <header class="page-header">
        <h1 id="proof-title">LiquidFun Rust/WASM browser proof</h1>
        <p>
          Live particle and rigid-body state produced by the Rust engine and
          drawn with Canvas 2D.
        </p>
      </header>

      <section
        class="proof-panel"
        aria-labelledby="session-status-title"
      >
        <h2 id="session-status-title">Session status</h2>

        <div class="status-row">
          <output
            class={`session-status session-status--${props.state.kind}`}
            aria-live="polite"
            aria-atomic="true"
          >
            <span class="status-dot" aria-hidden="true" />
            {statusText(props.state)}
          </output>

          <dl class="session-metadata">
            <div>
              <dt>Runtime</dt>
              <dd>Rust engine · WebAssembly</dd>
            </div>
            <div>
              <dt class="visually-hidden">Particle count</dt>
              <dd>
                Particles: {maybeFrame()?.particleCount ?? "—"}
              </dd>
            </div>
            <div>
              <dt class="visually-hidden">Rigid shape count</dt>
              <dd>
                Rigid shapes: {maybeFrame()?.rigidShapeCount ?? "—"}
              </dd>
            </div>
          </dl>
        </div>

        <Show when={props.state.kind === "failure"}>
          <div class="error-message" role="alert">
            <p>{ERROR_COPY}</p>
            <Show when={maybeDetails()}>
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
              aria-label="Live Rust physics scene: particles moving in a basin around a rigid body."
            >
              Canvas is required to display the Rust/WASM physics proof.
            </canvas>
            <Show when={!hasFrame()}>
              <div class="empty-state">
                <strong>Waiting for first Rust frame</strong>
                <span>
                  The viewport will update after the WebAssembly session
                  starts.
                </span>
              </div>
            </Show>
          </div>
          <figcaption>
            Live frame from owned Rust/WASM particle and rigid geometry.
          </figcaption>
        </figure>

        <button
          type="button"
          disabled={props.state.kind !== "running"}
          onClick={props.onDispose}
        >
          Dispose session
        </button>
      </section>
    </main>
  );
}

/** Minimal semantic shell for the Rust/WASM Canvas proof. */
export function App() {
  const [state, setState] = createSignal<ProofState>({ kind: "loading" });
  let maybeCanvas: HTMLCanvasElement | undefined;
  let maybeSession: SceneSession | undefined;
  let maybeAnimationFrameId: number | undefined;
  let maybePreviousFrame: RenderFrame | undefined;
  let maybeCamera: Camera | undefined;
  let maybeResizeObserver: ResizeObserver | undefined;
  let stopped = false;

  function cancelPendingFrame(): void {
    if (maybeAnimationFrameId === undefined) {
      return;
    }

    cancelAnimationFrame(maybeAnimationFrameId);
    maybeAnimationFrameId = undefined;
  }

  function disconnectResizeObserver(): void {
    const maybeObserver = maybeResizeObserver;
    maybeResizeObserver = undefined;
    maybeObserver?.disconnect();
  }

  function stopResources(): void {
    if (stopped) {
      return;
    }

    stopped = true;
    disconnectResizeObserver();
    cancelPendingFrame();

    const maybeOwnedSession = maybeSession;
    maybeSession = undefined;
    if (maybeOwnedSession === undefined) {
      return;
    }

    try {
      maybeOwnedSession.dispose();
    } catch {
      // Disposal remains terminal even if generated cleanup reports a failure.
    }
  }

  function fail(error: unknown): void {
    if (stopped) {
      return;
    }

    const maybeFrame = maybeObservedFrame(state());
    stopResources();
    setState({
      kind: "failure",
      maybeFrame,
      maybeDetails: maybeDevelopmentDetails(error),
    });
  }

  function scheduleFrame(context: CanvasRenderingContext2D): void {
    if (stopped) {
      return;
    }

    maybeAnimationFrameId = requestAnimationFrame(() => {
      maybeAnimationFrameId = undefined;
      if (stopped) {
        return;
      }

      const maybeOwnedSession = maybeSession;
      if (maybeOwnedSession === undefined) {
        fail(new Error("Rust/WASM session owner is unavailable"));
        return;
      }
      const camera = maybeCamera;
      if (camera === undefined) {
        fail(new Error("Canvas camera is unavailable"));
        return;
      }

      try {
        const frame = maybeOwnedSession.nextFrame();
        drawRenderFrame(context, frame, camera);

        const previousObservation = maybeObservedFrame(state());
        const observation = observeFrame(
          frame,
          maybePreviousFrame,
          previousObservation?.movedFrameCount ?? 0,
        );
        maybePreviousFrame = frame;
        setState({ kind: "running", frame: observation });
        scheduleFrame(context);
      } catch (error) {
        fail(error);
      }
    });
  }

  async function startSession(
    context: CanvasRenderingContext2D,
  ): Promise<void> {
    try {
      const generatedSession = await loadProofSession();
      const ownedSession = createSceneSession(generatedSession);
      if (stopped) {
        ownedSession.dispose();
        return;
      }

      maybeSession = ownedSession;
      const camera = maybeCamera;
      if (camera === undefined) {
        fail(new Error("Canvas camera is unavailable"));
        return;
      }
      const frame = ownedSession.nextFrame();
      drawRenderFrame(context, frame, camera);
      const observation = observeFrame(frame, undefined, 0);
      maybePreviousFrame = frame;
      setState({ kind: "running", frame: observation });
      scheduleFrame(context);
    } catch (error) {
      fail(error);
    }
  }

  function disposeSession(): void {
    const currentState = state();
    if (currentState.kind !== "running") {
      return;
    }

    stopResources();
    setState({ kind: "disposed", frame: currentState.frame });
  }

  onMount(() => {
    const canvas = maybeCanvas;
    if (canvas === undefined) {
      fail(new Error("Canvas element is unavailable"));
      return;
    }

    try {
      const bounds = canvas.getBoundingClientRect();
      maybeCamera = resizeCanvasBackingStore(
        canvas,
        bounds.width,
        bounds.height,
        window.devicePixelRatio,
      );
      const maybeContext = canvas.getContext("2d");
      if (maybeContext === null) {
        fail(new Error("Canvas 2D is unavailable"));
        return;
      }

      maybeResizeObserver = new ResizeObserver(() => {
        if (stopped) {
          return;
        }

        try {
          const resizedBounds = canvas.getBoundingClientRect();
          const resizedCamera = resizeCanvasBackingStore(
            canvas,
            resizedBounds.width,
            resizedBounds.height,
            window.devicePixelRatio,
          );
          maybeCamera = resizedCamera;
          const maybeFrame = maybePreviousFrame;
          if (maybeFrame !== undefined) {
            drawRenderFrame(maybeContext, maybeFrame, resizedCamera);
          }
        } catch (error) {
          fail(error);
        }
      });
      maybeResizeObserver.observe(canvas);

      void startSession(maybeContext);
    } catch (error) {
      fail(error);
    }
  });

  onCleanup(stopResources);

  return (
    <ProofPage
      state={state()}
      assignCanvas={(canvas) => {
        maybeCanvas = canvas;
      }}
      onDispose={disposeSession}
    />
  );
}
