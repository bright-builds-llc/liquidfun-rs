import { createRoot, createSignal, type Setter } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { PUZZLE_PORTRAIT_BOUNDS } from "../src/catalog/puzzle-bounds";
import type { RawProofFrame } from "../src/physics/frame";
import { loadPuzzleSession } from "../src/physics/puzzle-loader";
import type {
  GeneratedPuzzleSession,
  RawPuzzleLevelInfo,
} from "../src/physics/puzzle-session";
import type { PlayerSession } from "../src/physics/live-session";
import { createFrameClock } from "../src/player/frame-loop";
import {
  beginPuzzleRoute,
  createPuzzleController,
  editPuzzle,
  leavePuzzle,
  playPuzzle,
  retryPuzzle,
  startPuzzleUnlessLoading,
  tryAgainPuzzle,
  type PuzzleHost,
} from "../src/player/puzzle-lifecycle";
import { createPuzzleRuntime } from "../src/player/puzzle-runtime";
import type { PlayerView } from "../src/player/view";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import { INITIAL_PUZZLE_STATE, type PuzzleState } from "../src/puzzle/reduce-puzzle";
import { IDENTITY_CAMERA_VIEW, createCamera } from "../src/render/camera";

vi.mock("../src/physics/puzzle-loader", () => ({ loadPuzzleSession: vi.fn() }));

const PARTICLE_COUNT = 4;
const ENDED_STATE: PuzzleState = {
  kind: "ended",
  stepIndex: 840,
  latch: {
    outcome: "lost",
    outcomeStep: 720,
    decidedStep: 720,
    goalCount: 0,
    peakHeldCount: 0,
    maybeBrimStep: undefined,
    goalTarget: 540,
    goalFull: 900,
    brimCount: 810,
    stepLimit: 720,
  },
  tailEndStep: 840,
  held: true,
  editor: EMPTY_EDITOR,
  speed: 1,
};

function fakeLevelInfo(title: string): RawPuzzleLevelInfo {
  return {
    levelId: () => "first-pour",
    title: () => title,
    targetCount: () => 540,
    fullCount: () => 900,
    brimCount: () => 810,
    parTools: () => 1,
    settleSteps: () => 30,
    stepLimit: () => 720,
    cupInterior: () => new Float32Array([1.6, 0, 3.4, 1.8]),
    buildArea: () => new Int32Array([-20, 44, 40, 84]),
    gridMeters: () => 0.05,
    toolKinds: () => new Uint8Array([1]),
    toolCounts: () => new Uint32Array([2]),
    free: vi.fn(),
  };
}

function rawFrame(stepIndex: number): RawProofFrame {
  return {
    stepIndex: () => stepIndex,
    particleCount: () => PARTICLE_COUNT,
    rigidShapeCount: () => 0,
    maxSpeed: () => 0,
    stuckCandidateCount: () => 0,
    bodyContactCount: () => 0,
    particlePositions: () => new Float32Array([0, 0, 0.1, 0, 0.2, 0, 0.3, 0]),
    particleColors: () => new Uint8Array(PARTICLE_COUNT * 4).fill(200),
    particleRadii: () => new Float32Array(PARTICLE_COUNT).fill(0.05),
    rigidSegments: () => new Float32Array(),
    rigidCircles: () => new Float32Array(),
    circleLabels: () => [],
    goalCount: () => 0,
    goalTarget: () => 540,
    goalFull: () => 900,
    outcome: () => 0,
    outcomeStep: () => undefined,
    stepLimit: () => 720,
    peakHeldCount: () => 0,
    brimCount: () => 810,
    brimStep: () => undefined,
    decidedStep: () => undefined,
    free: () => undefined,
  };
}

function fakeGenerated(title = "First Pour") {
  let step = 0;
  const generated = {
    advance: vi.fn((count: number) => {
      step += count;
    }),
    captureFrame: vi.fn(() => rawFrame(step)),
    applyControl: vi.fn(() => false),
    applyAction: vi.fn(),
    pointerAction: vi.fn(),
    setGravity: vi.fn(),
    restoreAuthoredGravity: vi.fn(),
    levelInfo: vi.fn(() => fakeLevelInfo(title)),
    free: vi.fn(),
  } satisfies GeneratedPuzzleSession;
  return generated;
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => {
    resolve = yes;
  });
  return { promise, resolve };
}

function setter<T>(): Setter<T> {
  return vi.fn() as unknown as Setter<T>;
}

function createHost() {
  return createRoot((dispose) => {
    const [view, setView] = createSignal<PlayerView>({ kind: "fallback" });
    const clock = createFrameClock();
    clock.viewportWidth = 390;
    clock.viewportHeight = 844;
    clock.cameraView = { zoom: 2, panX: 10, panY: -4 };
    const canvas = { dataset: {} as DOMStringMap } as HTMLCanvasElement;
    const host: PuzzleHost = {
      generation: 0,
      clock,
      maybeCanvas: canvas,
      maybeContext: {} as CanvasRenderingContext2D,
      maybeSession: undefined,
      maybeCanvasPointer: undefined,
      tiltBinding: { request: 0, stop: vi.fn(), maybeLive: undefined },
      constructionValues: { width: "3" },
      view,
      setView,
      refreshCamera: () => {
        clock.maybeCamera = createCamera(
          clock.viewportWidth,
          clock.viewportHeight,
          clock.cameraView,
          clock.worldBounds,
        );
      },
      drawSceneFrame: vi.fn(),
      failScene: vi.fn(),
      setMaybeDebugFrame: setter(),
      setStepsThisFrame: setter(),
      setFpsTicks: setter(),
      setPanEnabled: setter(),
      setTiltGravityEnabled: setter(),
      setTiltDebug: setter(),
      puzzle: createPuzzleRuntime(),
    };
    return { host, canvas, dispose };
  });
}

async function flushMicrotasks(): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, 0));
}

let callbacks: Map<number, FrameRequestCallback>;
let rafId: number;
const loader = vi.mocked(loadPuzzleSession);

beforeEach(() => {
  callbacks = new Map();
  rafId = 0;
  vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
    rafId += 1;
    callbacks.set(rafId, callback);
    return rafId;
  });
  vi.stubGlobal("cancelAnimationFrame", (id: number) => callbacks.delete(id));
  vi.stubGlobal("document", { hidden: false });
  vi.stubGlobal("performance", { now: () => 0 });
  vi.stubGlobal("window", {});
  loader.mockReset();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("beginPuzzleRoute", () => {
  it("lands in building on a never-advanced step-0 frame with glass and direct backend", async () => {
    // Arrange
    const generated = fakeGenerated();
    loader.mockResolvedValue(generated);
    const { host, canvas } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const state = host.puzzle.state();
    expect(state.kind).toBe("building");
    expect(state.kind === "building" && state.status.outcome).toBe("running");
    expect(host.view()).toMatchObject({ kind: "paused", frame: { stepIndex: 0 } });
    expect(generated.advance).not.toHaveBeenCalled();
    expect(host.puzzle.maybeGlass?.targetFraction).toBe(0.6);
    expect(canvas.dataset.simulationBackend).toBe("direct");
    expect(host.failScene).not.toHaveBeenCalled();
  });

  it("stops tilt, turns pan off, resets the camera and clears construction values", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    const stopTilt = host.tiltBinding.stop;

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(stopTilt).toHaveBeenCalledTimes(1);
    expect(host.setTiltGravityEnabled).toHaveBeenCalledWith(false);
    expect(host.setPanEnabled).toHaveBeenCalledWith(false);
    expect(host.clock.cameraView).toBe(IDENTITY_CAMERA_VIEW);
    expect(host.constructionValues).toEqual({});
    expect(host.clock.worldBounds).toEqual(PUZZLE_PORTRAIT_BOUNDS["first-pour"]);
  });

  it("builds with empty placement lanes by default", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const lanes = loader.mock.calls[0]?.[1];
    expect(Array.from(lanes?.kinds ?? [9])).toEqual([]);
    expect(Array.from(lanes?.poses ?? [9])).toEqual([]);
  });

  it("builds with the test seam's placements when the window global is set", async () => {
    // Arrange
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: {
        level: "first-pour",
        placements: [{ kind: "plank", x: 8, y: 64, angle: 23 }],
      },
    });
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const lanes = loader.mock.calls[0]?.[1];
    expect(Array.from(lanes?.kinds ?? [])).toEqual([1]);
    expect(Array.from(lanes?.poses ?? [])).toEqual([8, 64, 23]);
  });

  it("fails visibly on an invalid test seam without building", async () => {
    // Arrange
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: "x" },
    });
    const { host } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(host.failScene).toHaveBeenCalledTimes(1);
    expect(loader).not.toHaveBeenCalled();
  });

  it("fails closed and frees the session when the wasm title differs from the title table", async () => {
    // Arrange
    const generated = fakeGenerated("Wrong");
    loader.mockResolvedValue(generated);
    const { host } = createHost();

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    const error = vi.mocked(host.failScene).mock.calls[0]?.[0];
    expect(error).toBeInstanceOf(Error);
    expect(error instanceof Error && error.message).toContain("title");
    expect(generated.free).toHaveBeenCalledTimes(1);
    expect(host.maybeSession).toBeUndefined();
  });

  it("frees a stale build's session and keeps the newer one", async () => {
    // Arrange
    const first = deferred<GeneratedPuzzleSession>();
    const second = deferred<GeneratedPuzzleSession>();
    const staleGenerated = fakeGenerated();
    const currentGenerated = fakeGenerated();
    loader.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const { host } = createHost();
    const staleBuild = beginPuzzleRoute(host, "first-pour");
    const currentBuild = beginPuzzleRoute(host, "first-pour");

    // Act
    second.resolve(currentGenerated);
    await currentBuild;
    first.resolve(staleGenerated);
    await staleBuild;

    // Assert
    expect(staleGenerated.free).toHaveBeenCalledTimes(1);
    expect(currentGenerated.free).not.toHaveBeenCalled();
    expect(host.puzzle.maybeOwned).toBe(host.maybeSession);
    expect(host.puzzle.state().kind).toBe("building");
    expect(host.failScene).not.toHaveBeenCalled();
  });

  it("supersedes an in-flight build even when the newer entry fails", async () => {
    // Arrange
    const pending = deferred<GeneratedPuzzleSession>();
    const staleGenerated = fakeGenerated();
    loader.mockReturnValueOnce(pending.promise);
    const { host } = createHost();
    const staleBuild = beginPuzzleRoute(host, "first-pour");
    vi.stubGlobal("window", {
      __liquidfunPuzzleTestPlacements: { level: "first-pour", placements: "x" },
    });
    await beginPuzzleRoute(host, "first-pour");

    // Act
    pending.resolve(staleGenerated);
    await staleBuild;

    // Assert
    expect(staleGenerated.free).toHaveBeenCalledTimes(1);
    expect(host.maybeSession).toBeUndefined();
    expect(host.puzzle.state().kind).toBe("loading");
  });

  it("routes a throwing previous-session dispose to failScene without rejecting", async () => {
    // Arrange
    const disposeError = new Error("dispose failed");
    const { host } = createHost();
    host.maybeSession = {
      dispose: () => {
        throw disposeError;
      },
    } as unknown as PlayerSession;

    // Act
    const result = beginPuzzleRoute(host, "first-pour");

    // Assert
    await expect(result).resolves.toBeUndefined();
    expect(host.failScene).toHaveBeenCalledWith(disposeError);
    expect(loader).not.toHaveBeenCalled();
  });
});

describe("puzzle commands", () => {
  it("Play moves building to running, sets the view playing and schedules a frame", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");

    // Act
    playPuzzle(host);

    // Assert
    expect(host.puzzle.state().kind).toBe("running");
    expect(host.view().kind).toBe("playing");
    expect(callbacks.size).toBe(1);
  });

  it("Play does nothing outside building", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    host.puzzle.current = ENDED_STATE;
    host.puzzle.setState(ENDED_STATE);

    // Act
    playPuzzle(host);

    // Assert
    expect(host.puzzle.state()).toBe(ENDED_STATE);
    expect(callbacks.size).toBe(0);
  });

  it("Retry builds a fresh session with the same placements and lands in running", async () => {
    // Arrange
    const first = fakeGenerated();
    const second = fakeGenerated();
    loader.mockResolvedValueOnce(first).mockResolvedValueOnce(second);
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    const placements = host.puzzle.placements;
    host.puzzle.current = ENDED_STATE;

    // Act
    retryPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(2);
    expect(first.free).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state().kind).toBe("running");
    expect(host.puzzle.placements).toBe(placements);
    expect(callbacks.size).toBe(1);
  });

  it("Edit builds a fresh session with the same placements and lands in building", async () => {
    // Arrange
    const first = fakeGenerated();
    const second = fakeGenerated();
    loader.mockResolvedValueOnce(first).mockResolvedValueOnce(second);
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    const placements = host.puzzle.placements;
    host.puzzle.current = ENDED_STATE;

    // Act
    editPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(2);
    expect(first.free).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state().kind).toBe("building");
    expect(host.puzzle.placements).toBe(placements);
    expect(host.view().kind).toBe("paused");
  });

  it("Retry reaches failScene without an unhandled rejection when dispose throws", async () => {
    // Arrange
    const generated = fakeGenerated();
    const disposeError = new Error("dispose failed");
    generated.free.mockImplementation(() => {
      throw disposeError;
    });
    loader.mockResolvedValue(generated);
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    host.puzzle.current = ENDED_STATE;
    const unhandled = vi.fn();
    process.on("unhandledRejection", unhandled);

    // Act
    retryPuzzle(host);
    await flushMicrotasks();
    process.off("unhandledRejection", unhandled);

    // Assert
    expect(host.failScene).toHaveBeenCalledWith(disposeError);
    expect(unhandled).not.toHaveBeenCalled();
    expect(loader).toHaveBeenCalledTimes(1);
  });

  it("Try again rebuilds into building only from the failure view", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    host.setView({ kind: "failure", maybeFrame: undefined, maybeDetails: undefined });
    host.puzzle.setMaybeLevelId("first-pour");

    // Act
    tryAgainPuzzle(host);
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state().kind).toBe("building");
  });
});

describe("startPuzzleUnlessLoading", () => {
  it("keeps an in-flight Retry rebuild instead of restarting the route", async () => {
    // Arrange
    const rebuild = deferred<GeneratedPuzzleSession>();
    loader.mockResolvedValueOnce(fakeGenerated()).mockReturnValueOnce(rebuild.promise);
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    host.puzzle.current = ENDED_STATE;
    retryPuzzle(host);
    const setState = vi.spyOn(host.puzzle, "setState");

    // Act
    startPuzzleUnlessLoading(host, "first-pour");
    rebuild.resolve(fakeGenerated());
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(2);
    expect(setState).not.toHaveBeenCalledWith(INITIAL_PUZZLE_STATE);
    expect(host.puzzle.state().kind).toBe("running");
  });

  it("keeps the failure view when a resize follows a mid-run failure", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    playPuzzle(host);
    const failure: PlayerView = {
      kind: "failure",
      maybeFrame: undefined,
      maybeDetails: "step failed",
    };
    host.maybeSession = undefined;
    host.setView(failure);

    // Act
    startPuzzleUnlessLoading(host, "first-pour");
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(1);
    expect(host.view()).toBe(failure);
    expect(host.puzzle.state().kind).toBe("running");
  });

  it("begins the route when no level is loading", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();

    // Act
    startPuzzleUnlessLoading(host, "first-pour");
    await flushMicrotasks();

    // Assert
    expect(loader).toHaveBeenCalledTimes(1);
    expect(host.puzzle.state().kind).toBe("building");
  });
});

describe("leavePuzzle and the controller", () => {
  it("leavePuzzle resets puzzle state and cancels the pending frame", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    await beginPuzzleRoute(host, "first-pour");
    playPuzzle(host);

    // Act
    leavePuzzle(host);

    // Assert
    expect(host.puzzle.state()).toBe(INITIAL_PUZZLE_STATE);
    expect(host.puzzle.current).toBe(INITIAL_PUZZLE_STATE);
    expect(host.puzzle.maybeGlass).toBeUndefined();
    expect(host.puzzle.maybeOwned).toBeUndefined();
    expect(host.puzzle.maybeLevelId()).toBeUndefined();
    expect(host.puzzle.placements).toEqual([]);
    expect(callbacks.size).toBe(0);
    expect(host.clock.maybeAnimationFrameId).toBeUndefined();
  });

  it("createPuzzleController does not read the puzzle bag until used", () => {
    // Arrange
    const { host } = createHost();
    const lazyHost = Object.defineProperty({ ...host }, "puzzle", {
      get: () => {
        throw new Error("puzzle read too early");
      },
    });

    // Act
    const create = () => createPuzzleController(lazyHost);

    // Assert
    expect(create).not.toThrow();
  });

  it("controller accessors read the live puzzle bag", async () => {
    // Arrange
    loader.mockResolvedValue(fakeGenerated());
    const { host } = createHost();
    const controller = createPuzzleController(host);

    // Act
    await beginPuzzleRoute(host, "first-pour");

    // Assert
    expect(controller.maybeLevelId()).toBe("first-pour");
    expect(controller.state().kind).toBe("building");
  });
});
