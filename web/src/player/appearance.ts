import { createSignal, type Accessor } from "solid-js";

import {
  loadDensityShading,
  persistDensityShading,
} from "../render/density-shade";
import {
  loadRenderMode,
  persistRenderMode,
  type RenderMode,
} from "../render/mode";
import {
  loadWireframeStrokeWidth,
  persistWireframeStrokeWidth,
} from "../render/stroke-width";

type AppearanceStorage = Pick<Storage, "getItem" | "setItem">;

export type AppearancePreferences = {
  readonly renderMode: Accessor<RenderMode>;
  readonly wireframeStrokeWidth: Accessor<number>;
  readonly densityShading: Accessor<boolean>;
  readonly changeRenderMode: (mode: RenderMode) => void;
  readonly changeWireframeStrokeWidth: (width: number) => void;
  readonly changeDensityShading: (enabled: boolean) => void;
};

/** Loads particle appearance preferences and repaints after each change. */
export function createAppearancePreferences(
  storageProvider: () => AppearanceStorage,
  repaint: () => void,
): AppearancePreferences {
  const [renderMode, setRenderMode] = createSignal(loadRenderMode(storageProvider));
  const [wireframeStrokeWidth, setWireframeStrokeWidth] = createSignal(
    loadWireframeStrokeWidth(storageProvider),
  );
  const [densityShading, setDensityShading] = createSignal(
    loadDensityShading(storageProvider),
  );

  return {
    renderMode,
    wireframeStrokeWidth,
    densityShading,
    changeRenderMode(nextMode) {
      setRenderMode(nextMode);
      persistRenderMode(storageProvider, nextMode);
      repaint();
    },
    changeWireframeStrokeWidth(nextWidth) {
      setWireframeStrokeWidth(nextWidth);
      persistWireframeStrokeWidth(storageProvider, nextWidth);
      repaint();
    },
    changeDensityShading(enabled) {
      setDensityShading(enabled);
      persistDensityShading(storageProvider, enabled);
      repaint();
    },
  };
}
