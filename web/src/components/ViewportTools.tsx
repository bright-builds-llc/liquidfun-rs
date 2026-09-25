import { createSignal, onCleanup, onMount, Show } from "solid-js";

export type ViewportToolsProps = {
  readonly showFullscreen: boolean;
  readonly panEnabled: boolean;
  readonly onZoomIn: () => void;
  readonly onZoomOut: () => void;
  readonly onResetZoom: () => void;
  readonly onPanEnabledChange: (enabled: boolean) => void;
};

type FullscreenDocument = Document & {
  webkitFullscreenElement?: Element | null;
  webkitExitFullscreen?: () => Promise<void> | void;
};

type FullscreenTarget = HTMLElement & {
  webkitRequestFullscreen?: () => Promise<void> | void;
};

function currentFullscreenElement(): Element | null {
  const fullscreenDocument = document as FullscreenDocument;
  return document.fullscreenElement ?? fullscreenDocument.webkitFullscreenElement ?? null;
}

/** Zoom, pan, and fullscreen controls anchored to the canvas. */
export function ViewportTools(props: ViewportToolsProps) {
  const [fullscreen, setFullscreen] = createSignal(false);
  let maybeTools: HTMLDivElement | undefined;

  function syncFullscreen(): void {
    const frame = maybeTools?.closest(".viewport-frame") ?? null;
    setFullscreen(frame !== null && currentFullscreenElement() === frame);
  }

  onMount(() => {
    document.addEventListener("fullscreenchange", syncFullscreen);
    document.addEventListener("webkitfullscreenchange", syncFullscreen);
  });

  onCleanup(() => {
    document.removeEventListener("fullscreenchange", syncFullscreen);
    document.removeEventListener("webkitfullscreenchange", syncFullscreen);
  });

  function toggleFullscreen(): void {
    const frame = maybeTools?.closest(".viewport-frame");
    if (!(frame instanceof HTMLElement)) {
      return;
    }

    if (currentFullscreenElement() === frame) {
      const fullscreenDocument = document as FullscreenDocument;
      if (typeof document.exitFullscreen === "function") {
        void document.exitFullscreen();
        return;
      }
      void fullscreenDocument.webkitExitFullscreen?.();
      return;
    }

    const target = frame as FullscreenTarget;
    const request =
      typeof frame.requestFullscreen === "function"
        ? frame.requestFullscreen()
        : target.webkitRequestFullscreen?.();
    void Promise.resolve(request).catch(() => {
      setFullscreen(false);
    });
  }

  return (
    <div class="viewport-tools" ref={maybeTools}>
      <button type="button" aria-label="Zoom in" onClick={() => props.onZoomIn()}>
        <ZoomInIcon />
      </button>
      <button type="button" aria-label="Zoom out" onClick={() => props.onZoomOut()}>
        <ZoomOutIcon />
      </button>
      <button type="button" aria-label="Reset zoom" onClick={() => props.onResetZoom()}>
        <ResetZoomIcon />
      </button>
      <button
        type="button"
        aria-label="Pan canvas"
        aria-pressed={props.panEnabled}
        onClick={() => props.onPanEnabledChange(!props.panEnabled)}
      >
        <PanIcon />
      </button>
      <Show when={props.showFullscreen}>
        <button
          type="button"
          aria-label={fullscreen() ? "Exit full screen" : "Full screen"}
          aria-pressed={fullscreen()}
          onClick={toggleFullscreen}
        >
          {fullscreen() ? <ExitFullscreenIcon /> : <FullscreenIcon />}
        </button>
      </Show>
    </div>
  );
}

function ZoomInIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M11 5v12M5 11h12" />
    </svg>
  );
}

function ZoomOutIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 11h12" />
    </svg>
  );
}

function ResetZoomIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <rect x="6" y="6" width="12" height="12" />
    </svg>
  );
}

function FullscreenIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M8 4H4v4M16 4h4v4M20 16v4h-4M4 16v4h4" />
    </svg>
  );
}

function ExitFullscreenIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 8h4V4M16 4v4h4M20 16h-4v4M8 20v-4H4" />
    </svg>
  );
}

function PanIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 4v16M4 12h16M12 4l-2.5 2.5M12 4l2.5 2.5M12 20l-2.5-2.5M12 20l2.5-2.5M4 12l2.5-2.5M4 12l2.5 2.5M20 12l-2.5-2.5M20 12l-2.5 2.5" />
    </svg>
  );
}
