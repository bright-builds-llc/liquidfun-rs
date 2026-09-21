import { STEP_SECONDS } from "../physics/clock";
import type { RenderFrame } from "../physics/frame";

export type DebugRow = {
  readonly label: string;
  readonly value: string;
};

/** Formats the player debug readout from one copied frame. */
export function debugRows(
  frame: RenderFrame,
  stepsThisFrame: number,
): readonly DebugRow[] {
  return [
    { label: "Step", value: String(frame.stepIndex) },
    {
      label: "Time",
      value: `${(frame.stepIndex * STEP_SECONDS).toFixed(2)} s`,
    },
    { label: "Particles", value: String(frame.particleCount) },
    { label: "Rigid shapes", value: String(frame.rigidShapeCount) },
    { label: "Steps this frame", value: String(stepsThisFrame) },
    { label: "Max speed", value: `${frame.maxSpeed.toFixed(3)} m/s` },
    { label: "Stuck", value: String(frame.stuckCandidateCount) },
    { label: "Body contacts", value: String(frame.bodyContactCount) },
  ];
}
