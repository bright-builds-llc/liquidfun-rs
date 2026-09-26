/** Supported browser particle presentation modes. */
export type RenderMode =
  | "wireframe"
  | "solid"
  | "soft-blob"
  | "contour"
  | "shaded-blob";

/** Circle presentation used for rigid bodies and animated SVG. */
export type CircleRenderMode = "wireframe" | "solid";

/** How a particle mode is drawn. */
export type ParticleSurface = "disc" | "metaball" | "contour" | "webgl";

export type RenderModeOption = {
  readonly value: RenderMode;
  readonly label: string;
};

export type RenderModeGroup = {
  readonly label: string;
  readonly options: readonly RenderModeOption[];
};

export const DEFAULT_RENDER_MODE: RenderMode = "wireframe";
export const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";

/** Controls-sheet groups. Circle modes share one disc painter. */
export const RENDER_MODE_GROUPS: readonly RenderModeGroup[] = [
  {
    label: "Circles",
    options: [
      { value: "wireframe", label: "Wireframe" },
      { value: "solid", label: "Solid" },
    ],
  },
  {
    label: "Surface",
    options: [
      { value: "soft-blob", label: "Soft blob" },
      { value: "contour", label: "Contour" },
      { value: "shaded-blob", label: "Shaded blob" },
    ],
  },
];

type RenderModeStorage = Pick<Storage, "getItem" | "setItem">;
export type RenderModeStorageProvider = () => RenderModeStorage;

/** Parses one persisted or control-provided rendering token. */
export function maybeParseRenderMode(
  maybeValue: string | null,
): RenderMode | undefined {
  if (
    maybeValue === "wireframe" ||
    maybeValue === "solid" ||
    maybeValue === "soft-blob" ||
    maybeValue === "contour" ||
    maybeValue === "shaded-blob"
  ) {
    return maybeValue;
  }

  return undefined;
}

/** Parses the circle vocabulary used by SVG export. */
export function maybeParseCircleRenderMode(
  maybeValue: string | null,
): CircleRenderMode | undefined {
  if (maybeValue === "wireframe" || maybeValue === "solid") {
    return maybeValue;
  }

  return undefined;
}

/** Loads a render preference while containing unavailable browser storage. */
export function loadRenderMode(
  storageProvider: RenderModeStorageProvider,
): RenderMode {
  try {
    return (
      maybeParseRenderMode(
        storageProvider().getItem(RENDER_MODE_STORAGE_KEY),
      ) ?? DEFAULT_RENDER_MODE
    );
  } catch {
    return DEFAULT_RENDER_MODE;
  }
}

/** Persists a render preference without turning storage failure into app failure. */
export function persistRenderMode(
  storageProvider: RenderModeStorageProvider,
  mode: RenderMode,
): void {
  try {
    storageProvider().setItem(RENDER_MODE_STORAGE_KEY, mode);
  } catch {
    // The in-memory preference remains valid when storage is unavailable.
  }
}

/** Rigid geometry follows wireframe only when particles are wireframe. */
export function rigidRenderMode(mode: RenderMode): CircleRenderMode {
  if (mode === "wireframe") {
    return "wireframe";
  }

  return "solid";
}

/** Animated SVG keeps one circle per particle. Surface modes export as solid. */
export function circleExportMode(mode: RenderMode): CircleRenderMode {
  return rigidRenderMode(mode);
}

/** Surface modes draw every particle. Stride would leave holes in the blob. */
export function usesParticleStride(mode: RenderMode): boolean {
  return particleSurface(mode) === "disc";
}

/** Selects the particle painter for a stored mode. */
export function particleSurface(mode: RenderMode): ParticleSurface {
  switch (mode) {
    case "wireframe":
    case "solid":
      return "disc";
    case "soft-blob":
      return "metaball";
    case "contour":
      return "contour";
    case "shaded-blob":
      return "webgl";
  }
}

/** Shaded blob is the only mode that needs a second canvas. */
export function needsWebglSurface(mode: RenderMode): boolean {
  return particleSurface(mode) === "webgl";
}
