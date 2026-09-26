import { SCENE_IDS, type SceneId } from "../catalog/scenes";
import { VIEWPORT_INSET } from "../render/camera";
import { maybeParseRenderedParticleLimit } from "../render/particle-limit";
import {
  maybeParseCircleRenderMode,
  type CircleRenderMode,
} from "../render/mode";
import { maybeParseWireframeStrokeWidth } from "../render/stroke-width";
import { MAX_SVG_EXPORT_SECONDS, maybeParseSvgExportSeconds } from "./duration";
import type { ExportControl } from "./record";

const MAX_CONTROLS = 32;
const MAX_CONTROL_TEXT = 64;
const MAX_ERROR_TEXT = 500;
const MAX_SVG_CHARACTERS = 80_000_000;
const MAX_ELAPSED_MS = 3_600_000;
const MIN_VIEWPORT = VIEWPORT_INSET * 2 + 1;

export type SvgExportRequest = {
  readonly sceneId: SceneId;
  readonly title: string;
  readonly durationSeconds: number;
  readonly controls: readonly ExportControl[];
  readonly viewportWidth: number;
  readonly viewportHeight: number;
  readonly zoom: number;
  readonly panX: number;
  readonly panY: number;
  readonly renderMode: CircleRenderMode;
  readonly wireframeStrokeWidth: number;
  readonly maxRenderedParticles: number;
};

export type SvgExportWorkerMessage =
  | {
      readonly type: "sampling";
      readonly completed: number;
      readonly total: number;
    }
  | { readonly type: "assembling"; readonly total: number }
  | {
      readonly type: "complete";
      readonly svg: string;
      readonly elapsedMs: number;
    }
  | { readonly type: "error"; readonly message: string };

export function maybeParseSvgExportRequest(value: unknown): SvgExportRequest | undefined {
  if (!isRecord(value)) {
    return undefined;
  }

  const maybeSceneId = sceneIdFrom(value.sceneId);
  const maybeTitle = maybeShortText(value.title);
  const maybeDuration =
    typeof value.durationSeconds === "number"
      ? maybeParseSvgExportSeconds(String(value.durationSeconds))
      : undefined;
  const maybeControls = controlsFrom(value.controls);
  const maybeWidth = maybeViewport(value.viewportWidth);
  const maybeHeight = maybeViewport(value.viewportHeight);
  const maybeZoom = maybeFinite(value.zoom);
  const maybePanX = maybeFinite(value.panX);
  const maybePanY = maybeFinite(value.panY);
  const maybeRenderMode = renderModeFrom(value.renderMode);
  const maybeStroke =
    typeof value.wireframeStrokeWidth === "number"
      ? maybeParseWireframeStrokeWidth(String(value.wireframeStrokeWidth))
      : undefined;
  const maybeParticleLimit =
    typeof value.maxRenderedParticles === "number"
      ? maybeParseRenderedParticleLimit(String(value.maxRenderedParticles))
      : undefined;

  if (
    maybeSceneId === undefined ||
    maybeTitle === undefined ||
    maybeDuration === undefined ||
    maybeControls === undefined ||
    maybeWidth === undefined ||
    maybeHeight === undefined ||
    maybeZoom === undefined ||
    maybePanX === undefined ||
    maybePanY === undefined ||
    maybeRenderMode === undefined ||
    maybeStroke === undefined ||
    maybeParticleLimit === undefined
  ) {
    return undefined;
  }

  return {
    sceneId: maybeSceneId,
    title: maybeTitle,
    durationSeconds: maybeDuration,
    controls: maybeControls,
    viewportWidth: maybeWidth,
    viewportHeight: maybeHeight,
    zoom: maybeZoom,
    panX: maybePanX,
    panY: maybePanY,
    renderMode: maybeRenderMode,
    wireframeStrokeWidth: maybeStroke,
    maxRenderedParticles: maybeParticleLimit,
  };
}

export function maybeParseWorkerMessage(value: unknown): SvgExportWorkerMessage | undefined {
  if (!isRecord(value)) {
    return undefined;
  }

  if (value.type === "sampling") {
    const maybeProgress = progressFrom(value.completed, value.total);
    if (maybeProgress === undefined) {
      return undefined;
    }
    return { type: "sampling", ...maybeProgress };
  }

  if (value.type === "assembling") {
    const maybeTotal = maybeCount(value.total);
    if (maybeTotal === undefined || maybeTotal <= 0) {
      return undefined;
    }
    return { type: "assembling", total: maybeTotal };
  }

  if (value.type === "complete") {
    return maybeComplete(value.svg, value.elapsedMs);
  }

  if (value.type === "error") {
    const maybeMessage = maybeBoundedText(value.message, MAX_ERROR_TEXT);
    if (maybeMessage === undefined) {
      return undefined;
    }
    return { type: "error", message: maybeMessage };
  }

  return undefined;
}

function maybeComplete(svg: unknown, elapsedMs: unknown): SvgExportWorkerMessage | undefined {
  if (typeof svg !== "string" || !svg.startsWith("<svg") || svg.length > MAX_SVG_CHARACTERS) {
    return undefined;
  }

  if (
    typeof elapsedMs !== "number" ||
    !Number.isFinite(elapsedMs) ||
    elapsedMs < 0 ||
    elapsedMs > MAX_ELAPSED_MS
  ) {
    return undefined;
  }

  return { type: "complete", svg, elapsedMs };
}

function progressFrom(
  completed: unknown,
  total: unknown,
): { readonly completed: number; readonly total: number } | undefined {
  const maybeCompleted = maybeCount(completed);
  const maybeTotal = maybeCount(total);
  if (
    maybeCompleted === undefined ||
    maybeTotal === undefined ||
    maybeTotal <= 0 ||
    maybeCompleted > maybeTotal ||
    maybeCompleted < 0
  ) {
    return undefined;
  }

  const maximumSamples = MAX_SVG_EXPORT_SECONDS * 20 + 1;
  if (maybeTotal > maximumSamples) {
    return undefined;
  }

  return { completed: maybeCompleted, total: maybeTotal };
}

function controlsFrom(value: unknown): readonly ExportControl[] | undefined {
  if (!Array.isArray(value) || value.length > MAX_CONTROLS) {
    return undefined;
  }

  const controls: ExportControl[] = [];
  for (const entry of value) {
    if (!isRecord(entry)) {
      return undefined;
    }
    const maybeName = maybeShortText(entry.name);
    const maybeValue = maybeShortText(entry.value);
    if (maybeName === undefined || maybeValue === undefined) {
      return undefined;
    }
    controls.push({ name: maybeName, value: maybeValue });
  }

  return controls;
}

function sceneIdFrom(value: unknown): SceneId | undefined {
  if (typeof value !== "string" || !(SCENE_IDS as readonly string[]).includes(value)) {
    return undefined;
  }

  return value as SceneId;
}

function renderModeFrom(value: unknown): CircleRenderMode | undefined {
  if (typeof value !== "string") {
    return undefined;
  }

  return maybeParseCircleRenderMode(value);
}

function maybeViewport(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isFinite(value) || value < MIN_VIEWPORT) {
    return undefined;
  }

  return value;
}

function maybeFinite(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    return undefined;
  }

  return value;
}

function maybeCount(value: unknown): number | undefined {
  if (typeof value !== "number" || !Number.isSafeInteger(value)) {
    return undefined;
  }

  return value;
}

/** Keeps a worker failure inside the message the page is willing to display. */
export function boundedExportError(error: unknown): string {
  if (!(error instanceof Error) || error.message.length === 0) {
    return "SVG export failed.";
  }

  if (error.message.length <= MAX_ERROR_TEXT) {
    return error.message;
  }

  return `${error.message.slice(0, MAX_ERROR_TEXT - 1)}…`;
}

function maybeShortText(value: unknown): string | undefined {
  return maybeBoundedText(value, MAX_CONTROL_TEXT);
}

function maybeBoundedText(value: unknown, maximum: number): string | undefined {
  if (typeof value !== "string" || value.length === 0 || value.length > maximum) {
    return undefined;
  }

  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
