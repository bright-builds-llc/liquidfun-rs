/** Catalog records for the curiosity scenes, in catalog order. */
import { withGravitySlider } from "./gravity-slider";
import { SHOWCASE, action, sceneSource } from "./scene-record-shared";
import { CARTESIAN_DIVER_VIEW_BOUNDS } from "./scene-view-bounds";
import type { SceneRecord } from "./scenes";

export const CURIOSITY_SCENE_RECORDS: readonly SceneRecord[] = [
  {
    id: "cartesian-diver",
    title: "Cartesian Diver",
    ready: true,
    description:
      "A diver floats in a closed bottle of particle water. Squeeze the bottle and it sinks, then let go and it rises. Switching the diver to a heavier density stands in for its squeezed air bubble. The bottle dent and the bubble are drawn, not simulated. This is an original experimental scene.",
    interactionHint:
      "Press and hold the bottle, or use Squeeze, to sink the diver. Let go, or use Squeeze again, and it rises. Presses outside the bottle do nothing. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([action("squeeze", "Squeeze")]),
    viewBounds: CARTESIAN_DIVER_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("cartesian_diver.rs"),
      inspiration: [SHOWCASE],
    },
  },
];
