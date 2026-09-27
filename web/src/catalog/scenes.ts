import { WORLD_BOUNDS, type WorldBounds } from "../render/camera";
import {
  LIQUID_TUMBLER_VIEW_BOUNDS,
  SCENES,
  THEO_JANSEN_VIEW_BOUNDS,
  WAVE_MACHINE_VIEW_BOUNDS,
} from "./scene-records";

export const SCENE_IDS = [
  "wave-machine",
  "dam-break",
  "fountain",
  "float-or-sink",
  "color-mixer",
  "jelly-drop",
  "water-wheel",
  "particles",
  "liquid-timer",
  "surface-tension",
  "elastic-particles",
  "rigid-particles",
  "soup",
  "soup-stirrer",
  "impulse",
  "theo-jansen",
  "liquid-tumbler",
  "drawing-particles",
  "sparky",
  "hydraulic-fountain",
] as const;

export type SceneId = (typeof SCENE_IDS)[number];

export type SceneControl =
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "preset";
      readonly recreates: boolean;
      readonly values: readonly { readonly id: string; readonly label: string }[];
    }
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "range";
      readonly recreates: boolean;
      readonly min: number;
      readonly max: number;
      readonly step: number;
      readonly defaultValue: number;
      readonly unit: string;
      readonly scale: "linear" | "logarithmic";
      readonly ticks: readonly number[];
      /** `hud` draws the slider above the play row. Other controls stay in the panel. */
      readonly surface?: "hud";
    }
  | {
      readonly id: string;
      readonly label: string;
      readonly kind: "action";
      readonly recreates: false;
    };

export type SceneCredits = {
  readonly implementationPath: string;
  readonly inspiration: readonly { readonly label: string; readonly href: string }[];
};

export type SceneRecord = {
  readonly id: SceneId;
  readonly title: string;
  readonly ready: boolean;
  readonly description: string;
  readonly interactionHint: string;
  readonly controls: readonly SceneControl[];
  readonly credits: SceneCredits;
  /**
   * World rectangle fitted to the canvas.
   *
   * Omitted scenes use the shared 12 m by 9 m frame.
   */
  readonly viewBounds?: WorldBounds;
};

export {
  LIQUID_TUMBLER_VIEW_BOUNDS,
  SCENES,
  THEO_JANSEN_VIEW_BOUNDS,
  WAVE_MACHINE_VIEW_BOUNDS,
};

export function maybeSceneById(id: string): SceneRecord | undefined {
  return SCENES.find((scene) => scene.id === id);
}

/** World rectangle the canvas fits for one catalog scene. */
export function worldBoundsForScene(id: SceneId): WorldBounds {
  return maybeSceneById(id)?.viewBounds ?? WORLD_BOUNDS;
}

export function isReadySceneId(id: SceneId): boolean {
  const maybeScene = maybeSceneById(id);
  return maybeScene?.ready === true;
}
