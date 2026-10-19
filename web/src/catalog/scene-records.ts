/** Catalog entry: spreads the family record arrays into SCENES in catalog order. */
import { CLASSIC_SCENE_RECORDS } from "./scene-records-classic";
import { CURIOSITY_SCENE_RECORDS } from "./scene-records-curiosity";
import { MACHINE_SCENE_RECORDS } from "./scene-records-machines";
import { TESTBED_SCENE_RECORDS } from "./scene-records-testbed";
import type { SceneRecord } from "./scenes";

export {
  ARCHIMEDES_SCREW_VIEW_BOUNDS,
  CARTESIAN_DIVER_VIEW_BOUNDS,
  FAKE_FERROFLUID_VIEW_BOUNDS,
  LIQUID_TUMBLER_VIEW_BOUNDS,
  RAYLEIGH_TAYLOR_VIEW_BOUNDS,
  TESLA_VALVE_VIEW_BOUNDS,
  THEO_JANSEN_VIEW_BOUNDS,
  WASHING_MACHINE_VIEW_BOUNDS,
  WAVE_MACHINE_VIEW_BOUNDS,
} from "./scene-view-bounds";

// A new scene family adds its own module and appends its array here, keeping catalog order.
export const SCENES: readonly SceneRecord[] = [
  ...CLASSIC_SCENE_RECORDS,
  ...TESTBED_SCENE_RECORDS,
  ...MACHINE_SCENE_RECORDS,
  ...CURIOSITY_SCENE_RECORDS,
];
