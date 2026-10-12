import type { PuzzleId } from "../catalog/puzzles";
import type { WorldBounds } from "../render/camera";
import { parseRenderFrame, type RenderFrame } from "./frame";
import type { GeneratedProofSession, SceneSession } from "./session";

const DISPOSED_MESSAGE = "Rust/WASM session is disposed";
const FAILED_MESSAGE = "Rust/WASM session failed";
const INVALID_INFO_MESSAGE = "Invalid puzzle level info";
const MIN_STEPS_PER_FRAME = 1;
const MAX_STEPS_PER_FRAME = 4;
const BOUNDS_LANE_LENGTH = 4;

/** Rejection raised by every scene input method of a puzzle session (D-16). */
export const PUZZLE_SCENE_INPUT_MESSAGE = "Puzzle sessions accept no scene input";

/** Generated `PuzzleLevelInfo` surface. */
export interface RawPuzzleLevelInfo {
  levelId(): string;
  title(): string;
  targetCount(): number;
  fullCount(): number;
  settleSteps(): number;
  stepLimit(): number;
  cupInterior(): Float32Array;
  buildArea(): Int32Array;
  gridMeters(): number;
  toolKinds(): Uint8Array;
  toolCounts(): Uint32Array;
  free(): void;
}

/** A generated session built by `ProofSession.newPuzzle`. */
export interface GeneratedPuzzleSession extends GeneratedProofSession {
  levelInfo(): RawPuzzleLevelInfo;
}

/** Level numbers parsed once from `levelInfo()`; lengths in meters. */
export type PuzzleLevelInfo = {
  readonly levelId: PuzzleId;
  readonly title: string;
  readonly targetCount: number;
  readonly fullCount: number;
  readonly settleSteps: number;
  readonly stepLimit: number;
  readonly gridMeters: number;
  readonly cupInterior: WorldBounds;
  readonly buildArea: WorldBounds;
};

/** Direct-backend puzzle owner: SceneSession plus a step-0 capture and parsed level info. */
export interface PuzzleSession extends SceneSession {
  readonly levelInfo: PuzzleLevelInfo;
  /** Captures the current frame without advancing the world (D-07). */
  captureFrame(): RenderFrame;
}

function invalidInfo(): never {
  throw new Error(INVALID_INFO_MESSAGE);
}

function parsePositiveCount(value: number): number {
  if (!Number.isSafeInteger(value) || value < 1) {
    return invalidInfo();
  }

  return value;
}

function boundsLanes(lanes: Float32Array | Int32Array): [number, number, number, number] {
  if (lanes.length !== BOUNDS_LANE_LENGTH) {
    return invalidInfo();
  }

  const [minX, minY, maxX, maxY] = lanes;
  if (minX === undefined || minY === undefined || maxX === undefined || maxY === undefined) {
    return invalidInfo();
  }

  return [minX, minY, maxX, maxY];
}

function parseCupInterior(lanes: Float32Array): WorldBounds {
  const [minX, minY, maxX, maxY] = boundsLanes(lanes);
  const finite = [minX, minY, maxX, maxY].every(Number.isFinite);
  if (!finite || minX >= maxX || minY >= maxY) {
    return invalidInfo();
  }

  return { minX, minY, maxX, maxY };
}

function parseBuildArea(lanes: Int32Array, gridMeters: number): WorldBounds {
  const [minX, minY, maxX, maxY] = boundsLanes(lanes);
  const integral = [minX, minY, maxX, maxY].every(Number.isSafeInteger);
  if (!integral || minX > maxX || minY > maxY) {
    return invalidInfo();
  }

  return {
    minX: minX * gridMeters,
    minY: minY * gridMeters,
    maxX: maxX * gridMeters,
    maxY: maxY * gridMeters,
  };
}

/** Parses generated level info; throws `Invalid puzzle level info` (fail closed). Does not free `raw`. */
export function parsePuzzleLevelInfo(
  raw: RawPuzzleLevelInfo,
  expectedId: PuzzleId,
): PuzzleLevelInfo {
  if (raw.levelId() !== expectedId) {
    return invalidInfo();
  }

  const title = raw.title();
  if (title.length === 0) {
    return invalidInfo();
  }

  const targetCount = parsePositiveCount(raw.targetCount());
  const fullCount = parsePositiveCount(raw.fullCount());
  if (targetCount > fullCount) {
    return invalidInfo();
  }

  const gridMeters = raw.gridMeters();
  if (!Number.isFinite(gridMeters) || gridMeters <= 0) {
    return invalidInfo();
  }

  return {
    levelId: expectedId,
    title,
    targetCount,
    fullCount,
    settleSteps: parsePositiveCount(raw.settleSteps()),
    stepLimit: parsePositiveCount(raw.stepLimit()),
    gridMeters,
    cupInterior: parseCupInterior(raw.cupInterior()),
    buildArea: parseBuildArea(raw.buildArea(), gridMeters),
  };
}

function readLevelInfo(
  generated: GeneratedPuzzleSession,
  expectedId: PuzzleId,
): PuzzleLevelInfo {
  const rawInfo = generated.levelInfo();
  try {
    return parsePuzzleLevelInfo(rawInfo, expectedId);
  } finally {
    rawInfo.free();
  }
}

function isAcceptedStepCount(stepCount: number): boolean {
  return (
    Number.isSafeInteger(stepCount) &&
    stepCount >= MIN_STEPS_PER_FRAME &&
    stepCount <= MAX_STEPS_PER_FRAME
  );
}

function rejectSceneInput(): never {
  throw new Error(PUZZLE_SCENE_INPUT_MESSAGE);
}

/** Wraps one generated puzzle session; frees it and throws if its level info is invalid. */
export function createPuzzleSession(
  generated: GeneratedPuzzleSession,
  expectedId: PuzzleId,
): PuzzleSession {
  let levelInfo: PuzzleLevelInfo;
  try {
    levelInfo = readLevelInfo(generated, expectedId);
  } catch (error) {
    generated.free();
    throw error;
  }

  let disposed = false;

  // Poisons the owner; a failing free is reported alongside the original cause.
  function failWith(cause: unknown): never {
    disposed = true;
    try {
      generated.free();
    } catch (freeError) {
      throw new Error(FAILED_MESSAGE, {
        cause: new AggregateError([cause, freeError]),
      });
    }

    throw new Error(FAILED_MESSAGE, { cause });
  }

  function requireLive(): void {
    if (disposed) {
      throw new Error(DISPOSED_MESSAGE);
    }
  }

  function captureParsed(): RenderFrame {
    const rawFrame = generated.captureFrame();
    try {
      return parseRenderFrame(rawFrame);
    } finally {
      rawFrame.free();
    }
  }

  function captureFrame(): RenderFrame {
    requireLive();
    try {
      return captureParsed();
    } catch (cause) {
      return failWith(cause);
    }
  }

  function nextFrame(stepCount = 1): RenderFrame {
    requireLive();
    if (!isAcceptedStepCount(stepCount)) {
      return failWith(new RangeError(`Rejected step count ${String(stepCount)}`));
    }

    try {
      generated.advance(stepCount);
      return captureParsed();
    } catch (cause) {
      return failWith(cause);
    }
  }

  function dispose(): void {
    if (disposed) {
      return;
    }

    disposed = true;
    generated.free();
  }

  return {
    levelInfo,
    captureFrame,
    nextFrame,
    applyControl: rejectSceneInput,
    applyAction: rejectSceneInput,
    pointerAction: rejectSceneInput,
    setGravity: rejectSceneInput,
    restoreAuthoredGravity: rejectSceneInput,
    dispose,
  };
}
