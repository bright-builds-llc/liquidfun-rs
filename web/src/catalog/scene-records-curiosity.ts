/** Catalog records for the curiosity scenes, in catalog order. */
import { SCREW_SPEED_CONTROL } from "./archimedes-screw-speed";
import { MAGNET_STRENGTH_CONTROL } from "./fake-ferrofluid-strength";
import { withGravitySlider } from "./gravity-slider";
import { WEIGHT_RATIO_CONTROL } from "./rayleigh-taylor-ratio";
import {
  SHOWCASE,
  action,
  option,
  runtimePreset,
  sceneSource,
} from "./scene-record-shared";
import {
  ARCHIMEDES_SCREW_VIEW_BOUNDS,
  CARTESIAN_DIVER_VIEW_BOUNDS,
  FAKE_FERROFLUID_VIEW_BOUNDS,
  RAYLEIGH_TAYLOR_VIEW_BOUNDS,
} from "./scene-view-bounds";
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
  {
    id: "archimedes-screw",
    title: "Archimedes Screw",
    ready: true,
    description:
      "Slanted vanes slide up a tilted tube and carry pockets of water from the low reservoir into the upper basin, which overflows back down a chute. Flat vanes stand in for a turning helix in this 2D side view. This is an original experimental scene.",
    interactionHint:
      "Use Screw speed to speed up or stop the screw, and Direction to run it in reverse. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      SCREW_SPEED_CONTROL,
      runtimePreset("screw-direction", "Direction", [
        option("forward", "Forward"),
        option("reverse", "Reverse"),
      ]),
    ]),
    viewBounds: ARCHIMEDES_SCREW_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("archimedes_screw.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "fake-ferrofluid",
    title: "Fake Ferrofluid",
    ready: true,
    description:
      "A dark liquid puddle sits in a shallow dish over a magnet and rises into a row of separate spikes. A scripted magnetic pull lifts the liquid into the spikes. The physics engine has no magnetism, and the magnet is only a drawn marker. This is an original experimental scene.",
    interactionHint:
      "Press and drag to move the magnet, and the spikes follow. Let go and it switches off and returns under the dish. Use Magnet to switch it on or off, and Magnet strength to change the pull. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([
      MAGNET_STRENGTH_CONTROL,
      action("magnet", "Magnet"),
    ]),
    viewBounds: FAKE_FERROFLUID_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("fake_ferrofluid.rs"),
      inspiration: [SHOWCASE],
    },
  },
  {
    id: "rayleigh-taylor",
    title: "Rayleigh–Taylor Instability",
    ready: true,
    description:
      "A heavy orange layer rests on a light blue one in a closed tank. Both layers are the same particle liquid, and the orange one is given extra weight, so a small ripple grows into fingers that sink while blue bubbles rise, and the layers turn over. This is an original experimental scene.",
    interactionHint:
      "Watch the ripple grow into fingers. Pick a Weight ratio to run it again with that ratio, or use Reset scene to start over with the default settings. Labeled controls also work from the keyboard.",
    controls: withGravitySlider([WEIGHT_RATIO_CONTROL]),
    viewBounds: RAYLEIGH_TAYLOR_VIEW_BOUNDS,
    credits: {
      implementationPath: sceneSource("rayleigh_taylor.rs"),
      inspiration: [SHOWCASE],
    },
  },
];
