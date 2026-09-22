export type ViewportToolsProps = {
  readonly panEnabled: boolean;
  readonly onZoomIn: () => void;
  readonly onZoomOut: () => void;
  readonly onResetZoom: () => void;
  readonly onPanEnabledChange: (enabled: boolean) => void;
};

/** Zoom and pan controls anchored to the canvas. */
export function ViewportTools(props: ViewportToolsProps) {
  return (
    <div class="viewport-tools">
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

function PanIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 4v16M4 12h16M12 4l-2.5 2.5M12 4l2.5 2.5M12 20l-2.5-2.5M12 20l2.5-2.5M4 12l2.5-2.5M4 12l2.5 2.5M20 12l-2.5-2.5M20 12l-2.5 2.5" />
    </svg>
  );
}
