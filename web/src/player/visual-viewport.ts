export type ViewportMetrics = {
  readonly heightPx: number;
  readonly offsetTopPx: number;
};

type VisualViewportBox = {
  readonly height: number;
  readonly offsetTop: number;
};

/** Visible viewport box, including the offset Safari uses when its bars are showing. */
export function visualViewportMetrics(
  maybeViewport: VisualViewportBox | null,
  innerHeight: number,
): ViewportMetrics {
  if (maybeViewport === null) {
    return { heightPx: innerHeight, offsetTopPx: 0 };
  }

  return {
    heightPx: maybeViewport.height,
    offsetTopPx: maybeViewport.offsetTop,
  };
}

export function applyVisualViewportMetrics(target: HTMLElement): void {
  const metrics = visualViewportMetrics(
    window.visualViewport,
    window.innerHeight,
  );
  target.style.setProperty("--visual-viewport-height", `${metrics.heightPx}px`);
  target.style.setProperty(
    "--visual-viewport-offset-top",
    `${metrics.offsetTopPx}px`,
  );
}

/** Keep canvas-stage CSS variables aligned with the visible viewport. */
export function bindVisualViewport(target: HTMLElement): () => void {
  const sync = () => {
    applyVisualViewportMetrics(target);
  };
  sync();
  const maybeViewport = window.visualViewport;
  maybeViewport?.addEventListener("resize", sync);
  maybeViewport?.addEventListener("scroll", sync);
  window.addEventListener("resize", sync);
  return () => {
    maybeViewport?.removeEventListener("resize", sync);
    maybeViewport?.removeEventListener("scroll", sync);
    window.removeEventListener("resize", sync);
  };
}
