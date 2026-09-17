import type { RenderFrame } from "../physics/frame";

export type FrameObservation = {
  readonly particleCount: number;
  readonly rigidShapeCount: number;
  readonly stepIndex: number;
  readonly movedFrameCount: number;
};

function valuesDiffer(
  previousValues: Float32Array,
  currentValues: Float32Array,
): boolean {
  if (previousValues.length !== currentValues.length) {
    return true;
  }

  for (let index = 0; index < currentValues.length; index += 1) {
    if (previousValues[index] !== currentValues[index]) {
      return true;
    }
  }

  return false;
}

function frameMoved(
  previousFrame: RenderFrame,
  currentFrame: RenderFrame,
): boolean {
  return (
    valuesDiffer(
      previousFrame.particlePositions,
      currentFrame.particlePositions,
    ) || valuesDiffer(previousFrame.rigidCircles, currentFrame.rigidCircles)
  );
}

export function observeFrame(
  frame: RenderFrame,
  maybePreviousFrame: RenderFrame | undefined,
  previousMovedFrameCount: number,
): FrameObservation {
  const movedFrameCount =
    maybePreviousFrame !== undefined && frameMoved(maybePreviousFrame, frame)
      ? previousMovedFrameCount + 1
      : previousMovedFrameCount;

  return {
    particleCount: frame.particleCount,
    rigidShapeCount: frame.rigidShapeCount,
    stepIndex: frame.stepIndex,
    movedFrameCount,
  };
}
