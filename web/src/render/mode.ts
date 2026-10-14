/** Supported browser particle presentation modes. */
export type RenderMode =
  | "circle-wireframe"
  | "triangle-wireframe"
  | "solid"
  | "soft-blob"
  | "contour"
  | "shaded-blob"
  | "water";

/** Circle presentation used for rigid bodies. */
export type CircleRenderMode = "wireframe" | "solid";

/**
 * Particle presentation stored in an animated SVG.
 *
 * Circle wireframes stay `"wireframe"` so existing exports keep one stroked
 * circle per particle. Triangle wireframes add a polygon outline.
 */
export type SvgRenderMode = CircleRenderMode | "triangle-wireframe";

/** How a particle mode is drawn. */
export type ParticleSurface =
  | "disc"
  | "triangle"
  | "metaball"
  | "contour"
  | "webgl"
  | "webgl-water";

export type RenderModeOption = {
  readonly value: RenderMode;
  readonly label: string;
};

export type RenderModeGroup = {
  readonly label: string;
  readonly options: readonly RenderModeOption[];
};

/** First visit, before a particle preference is stored. */
export const DEFAULT_RENDER_MODE: RenderMode = "shaded-blob";
export const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";

/** Water option text when the capability probe fails (D-05). */
export const WATER_UNAVAILABLE_LABEL = "Water (unavailable)";

/** Controls-sheet groups. Both wireframes share the stroke-width slider. */
export const RENDER_MODE_GROUPS: readonly RenderModeGroup[] = [
  {
    label: "Wireframe",
    options: [
      { value: "circle-wireframe", label: "Circle wireframe" },
      { value: "triangle-wireframe", label: "Triangle wireframe" },
    ],
  },
  {
    label: "Circles",
    options: [{ value: "solid", label: "Solid" }],
  },
  {
    label: "Surface",
    options: [
      { value: "soft-blob", label: "Soft blob" },
      { value: "contour", label: "Contour" },
      { value: "shaded-blob", label: "Shaded blob" },
      { value: "water", label: "Water" },
    ],
  },
];

type RenderModeStorage = Pick<Storage, "getItem" | "setItem">;
export type RenderModeStorageProvider = () => RenderModeStorage;

/** Parses one persisted or control-provided rendering token. */
export function maybeParseRenderMode(
  maybeValue: string | null,
): RenderMode | undefined {
  if (maybeValue === "wireframe") {
    return "circle-wireframe";
  }

  if (
    maybeValue === "circle-wireframe" ||
    maybeValue === "triangle-wireframe" ||
    maybeValue === "solid" ||
    maybeValue === "soft-blob" ||
    maybeValue === "contour" ||
    maybeValue === "shaded-blob" ||
    maybeValue === "water"
  ) {
    return maybeValue;
  }

  return undefined;
}

/** Parses the circle vocabulary used by rigid-body drawing. */
export function maybeParseCircleRenderMode(
  maybeValue: string | null,
): CircleRenderMode | undefined {
  if (maybeValue === "wireframe" || maybeValue === "solid") {
    return maybeValue;
  }

  return undefined;
}

/** Parses the particle vocabulary stored on an animated SVG request. */
export function maybeParseSvgRenderMode(
  maybeValue: string | null,
): SvgRenderMode | undefined {
  if (maybeValue === "triangle-wireframe") {
    return maybeValue;
  }

  return maybeParseCircleRenderMode(maybeValue);
}

/** Circle and triangle outlines both use the wireframe stroke width. */
export function isWireframeRenderMode(mode: RenderMode): boolean {
  return mode === "circle-wireframe" || mode === "triangle-wireframe";
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

/** Both wireframe modes stroke rigid circles and walls. */
export function rigidRenderMode(mode: RenderMode): CircleRenderMode {
  if (isWireframeRenderMode(mode)) {
    return "wireframe";
  }

  return "solid";
}

/**
 * Maps a playground mode onto the animated SVG particle vocabulary.
 *
 * Surface modes, including Water, export as filled circles. Triangle wireframes keep a polygon
 * outline; every other wireframe stays a stroked circle.
 */
export function svgRenderMode(mode: RenderMode): SvgRenderMode {
  if (mode === "triangle-wireframe") {
    return "triangle-wireframe";
  }

  return rigidRenderMode(mode);
}

/**
 * Canvas painter mode for an animated-SVG export mode.
 *
 * SVG circle wireframes use the token `"wireframe"`. The canvas painter calls
 * that same stroked-circle path `"circle-wireframe"`.
 */
export function canvasRenderMode(mode: SvgRenderMode): RenderMode {
  if (mode === "wireframe") {
    return "circle-wireframe";
  }

  return mode;
}

/** Surface modes draw every particle. Stride would leave holes in the blob. */
export function usesParticleStride(mode: RenderMode): boolean {
  const surface = particleSurface(mode);
  return surface === "disc" || surface === "triangle";
}

/** Selects the particle painter for a stored mode. */
export function particleSurface(mode: RenderMode): ParticleSurface {
  switch (mode) {
    case "circle-wireframe":
    case "solid":
      return "disc";
    case "triangle-wireframe":
      return "triangle";
    case "soft-blob":
      return "metaball";
    case "contour":
      return "contour";
    case "shaded-blob":
      return "webgl";
    case "water":
      return "webgl-water";
  }
}

/**
 * Shaded blob and Water draw on the particle canvas. Both WebGL modes share
 * that one canvas and its context (D-04).
 */
export function needsWebglSurface(mode: RenderMode): boolean {
  const surface = particleSurface(mode);
  return surface === "webgl" || surface === "webgl-water";
}

/**
 * The particle speed lane is requested only while Water is the requested
 * mode (D-11). It follows the requested mode, not the surface that drew.
 */
export function wantsParticleSpeeds(mode: RenderMode): boolean {
  return particleSurface(mode) === "webgl-water";
}

/** How a picker option reads and whether it can be chosen. */
export type RenderModeOptionView = {
  readonly label: string;
  readonly disabled: boolean;
};

/**
 * Picker presentation for one option. Only Water depends on the capability
 * probe; it reads "Water (unavailable)" and is disabled when the probe fails.
 */
export function renderModeOptionView(
  option: RenderModeOption,
  waterAvailable: boolean,
): RenderModeOptionView {
  if (option.value !== "water" || waterAvailable) {
    return { label: option.label, disabled: false };
  }

  return { label: WATER_UNAVAILABLE_LABEL, disabled: true };
}
