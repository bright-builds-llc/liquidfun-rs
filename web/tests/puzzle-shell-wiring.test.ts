import { createRoot } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi, type Mock } from "vitest";

import { worldBoundsForViewport } from "../src/catalog/portrait-bounds";
import { worldBoundsForTarget } from "../src/catalog/puzzle-bounds";
import type { PuzzleId } from "../src/catalog/puzzles";
import { attachCanvasPointer } from "../src/input/canvas-pointer";
import { createTiltBinding } from "../src/input/tilt-binding";
import { loadSceneSession } from "../src/physics/loader";
import type { RenderFrame } from "../src/physics/frame";
import { loadPuzzleSession } from "../src/physics/puzzle-loader";
import {
  connectResizeObserver,
  createFrameClock,
  type FrameLoopDeps,
} from "../src/player/frame-loop";
import { createPuzzleRuntime } from "../src/player/puzzle-runtime";
import { bindLifecycle, handleHashChange } from "../src/player/scene-lifecycle";
import type { SceneRuntime } from "../src/player/scene-runtime";
import { bindSurface } from "../src/player/scene-surface";
import type { PlayerView } from "../src/player/view";
import { EMPTY_EDITOR, applyInspection, editorFromPlacements } from "../src/puzzle/editor";
import type { PuzzleState } from "../src/puzzle/reduce-puzzle";
import { createCamera, IDENTITY_CAMERA_VIEW } from "../src/render/camera";
import { drawGlassCup, glassCup } from "../src/render/glass-cup";
import { drawPuzzleTools, toolOverlay } from "../src/render/puzzle-tools";
import { presentSceneFrame } from "../src/render/present-frame";
import type { PlayableRoute } from "../src/routing/playable-route";

vi.mock("../src/render/present-frame", () => ({ presentSceneFrame: vi.fn() }));
vi.mock("../src/render/glass-cup", async (importOriginal) => {
  const original = await importOriginal<typeof import("../src/render/glass-cup")>();
  return { ...original, drawGlassCup: vi.fn() };
});
vi.mock("../src/render/puzzle-tools", async (importOriginal) => {
  const original = await importOriginal<typeof import("../src/render/puzzle-tools")>();
  return { ...original, drawPuzzleTools: vi.fn() };
});
vi.mock("../src/input/canvas-pointer", async (importOriginal) => {
  const original = await importOriginal<typeof import("../src/input/canvas-pointer")>();
  return { ...original, attachCanvasPointer: vi.fn() };
});
vi.mock("../src/physics/puzzle-loader", () => ({
  loadPuzzleSession: vi.fn(() => new Promise(() => undefined)),
}));
vi.mock("../src/physics/loader", () => ({
  loadSceneSession: vi.fn(() => new Promise(() => undefined)),
}));

const PUZZLE_ROUTE: PlayableRoute = { kind: "puzzle", id: "first-pour" };
const SCENE_ROUTE: PlayableRoute = { kind: "scene", id: "dam-break" };
const GLASS = glassCup({ minX: 1.6, minY: 0, maxX: 3.4, maxY: 1.8 }, 540, 900, [{ fraction: 0.6, label: "60%" }]);
const FRAME = { stepIndex: 0 } as unknown as RenderFrame;

type ResizeCallback = () => void;
let resizeCallbacks: ResizeCallback[];
let observed: unknown[];

function fakeContext(): CanvasRenderingContext2D {
  return {
    setTransform: vi.fn(),
    fillRect: vi.fn(),
  } as unknown as CanvasRenderingContext2D;
}

function fakeCanvas(context = fakeContext()) {
  const classes = new Set(["canvas-interactive", "canvas-panning"]);
  const canvas = {
    width: 0,
    height: 0,
    dataset: {} as DOMStringMap,
    classList: {
      remove: vi.fn((...names: string[]) => names.forEach((name) => classes.delete(name))),
      contains: (name: string) => classes.has(name),
    },
    getContext: vi.fn(() => context),
    getBoundingClientRect: () => ({ width: 390, height: 844 }),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  };
  return { canvas: canvas as unknown as HTMLCanvasElement, classes, context };
}

function fakeRuntime(initialRoute: PlayableRoute) {
  let route = initialRoute;
  let view: PlayerView = { kind: "loading" };
  const runtime = {
    generation: 0,
    constructionValues: {},
    maybeCanvas: undefined,
    maybeParticleCanvas: undefined,
    maybeContext: undefined,
    maybeSession: undefined,
    maybeCanvasPointer: undefined,
    clock: createFrameClock(),
    tiltBinding: createTiltBinding(),
    route: () => route,
    setRoute: (next: PlayableRoute) => {
      route = next;
    },
    view: () => view,
    setView: vi.fn((next: PlayerView) => {
      view = next;
    }),
    appearance: {
      renderMode: () => "circles",
      wireframeStrokeWidth: () => 1,
      densityShading: () => false,
    },
    maxRenderedParticles: () => 1000,
    gravitySliderMagnitude: () => undefined,
    refreshCamera: vi.fn(),
    setMaybeDebugFrame: vi.fn(),
    setStepsThisFrame: vi.fn(),
    setFpsTicks: vi.fn(),
    setRenderedParticleDraft: vi.fn(),
    setMaxRenderedParticles: vi.fn(),
    setMaybePixelsPerMeter: vi.fn(),
    setPanEnabled: vi.fn(),
    setTiltGravityEnabled: vi.fn(),
    setTiltDebug: vi.fn(),
  };
  return runtime as unknown as SceneRuntime;
}

function withPuzzle(session: SceneRuntime): SceneRuntime {
  createRoot(() => {
    session.puzzle = createPuzzleRuntime();
  });
  return session;
}

function throwingPuzzle(session: SceneRuntime): SceneRuntime {
  Object.defineProperty(session, "puzzle", {
    get() {
      throw new Error("scene routes must not read session.puzzle");
    },
  });
  return session;
}

function buildingWithOnePlank(): PuzzleState {
  const editor = editorFromPlacements([{ kind: "plank", x: 8, y: 64, angle: 0, openTick: 0 }]);
  const quad = [
    { x: 0.3, y: 3.2 },
    { x: 0.5, y: 3.2 },
    { x: 0.5, y: 3.25 },
    { x: 0.3, y: 3.25 },
  ] as const;
  return {
    kind: "building",
    status: {
      goalCount: 0,
      goalTarget: 540,
      goalFull: 900,
      outcome: "running",
      stepLimit: 720,
      peakHeldCount: 0,
      brimCount: 810,
    },
    editor: applyInspection(
      editor,
      { revision: editor.revision, statuses: [0], outlines: [quad] },
      "commit",
    ),
    speed: 1,
  };
}

function camera() {
  return createCamera(390, 844, IDENTITY_CAMERA_VIEW, worldBoundsForViewport("dam-break", 390, 844));
}

function resizeDeps(route: PlayableRoute): FrameLoopDeps & {
  maybeStartPuzzle: Mock<(id: PuzzleId) => void>;
} {
  return {
    view: () => ({ kind: "loading" }),
    fail: vi.fn(),
    maybeSession: () => undefined,
    route: () => route,
    startScene: vi.fn(),
    maybeStartPuzzle: vi.fn<(id: PuzzleId) => void>(),
    drawSceneFrame: vi.fn(),
    setMaybeDebugFrame: vi.fn(),
    setStepsThisFrame: vi.fn(),
    setFpsTicks: vi.fn(),
    setMaybePixelsPerMeter: vi.fn(),
    setView: vi.fn(),
  };
}

function setHash(hash: string): void {
  vi.stubGlobal("window", {
    location: { hash },
    history: { state: null, replaceState: vi.fn() },
    devicePixelRatio: 1,
  });
}

beforeEach(() => {
  resizeCallbacks = [];
  observed = [];
  vi.stubGlobal(
    "ResizeObserver",
    class {
      constructor(callback: ResizeCallback) {
        resizeCallbacks.push(callback);
      }
      observe(target: unknown) {
        observed.push(target);
      }
      disconnect() {
        return undefined;
      }
    },
  );
  vi.stubGlobal("window", { devicePixelRatio: 1 });
  vi.stubGlobal("requestAnimationFrame", vi.fn(() => 1));
  vi.stubGlobal("cancelAnimationFrame", vi.fn());
  vi.mocked(presentSceneFrame).mockClear();
  vi.mocked(drawGlassCup).mockClear();
  vi.mocked(drawPuzzleTools).mockClear();
  vi.mocked(attachCanvasPointer).mockClear();
  vi.mocked(loadPuzzleSession).mockClear();
  vi.mocked(loadSceneSession).mockClear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("glass overlay hook", () => {
  it("draws the glass after presenting the frame on a puzzle route", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    bindSurface(session);
    session.puzzle.maybeGlass = GLASS;
    const context = fakeContext();
    const view = camera();

    // Act
    session.drawSceneFrame(context, FRAME, view);

    // Assert
    expect(drawGlassCup).toHaveBeenCalledWith(context, view, GLASS);
    const presentOrder = vi.mocked(presentSceneFrame).mock.invocationCallOrder[0] ?? 0;
    const glassOrder = vi.mocked(drawGlassCup).mock.invocationCallOrder[0] ?? 0;
    expect(presentOrder).toBeGreaterThan(0);
    expect(glassOrder).toBeGreaterThan(presentOrder);
  });

  it("draws the tools between the frame and the glass (D-22)", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    bindSurface(session);
    session.puzzle.maybeGlass = GLASS;
    session.puzzle.current = buildingWithOnePlank();
    const context = fakeContext();
    const view = camera();

    // Act
    session.drawSceneFrame(context, FRAME, view);

    // Assert
    expect(drawPuzzleTools).toHaveBeenCalledWith(
      context,
      view,
      toolOverlay(session.puzzle.current),
    );
    const presentOrder = vi.mocked(presentSceneFrame).mock.invocationCallOrder[0] ?? 0;
    const toolsOrder = vi.mocked(drawPuzzleTools).mock.invocationCallOrder[0] ?? 0;
    const glassOrder = vi.mocked(drawGlassCup).mock.invocationCallOrder[0] ?? 0;
    expect(toolsOrder).toBeGreaterThan(presentOrder);
    expect(glassOrder).toBeGreaterThan(toolsOrder);
  });

  it("never reads puzzle state or draws the glass on a scene route", () => {
    // Arrange
    const session = throwingPuzzle(fakeRuntime(SCENE_ROUTE));
    bindSurface(session);

    // Act
    session.drawSceneFrame(fakeContext(), FRAME, camera());

    // Assert
    expect(presentSceneFrame).toHaveBeenCalledTimes(1);
    expect(drawGlassCup).not.toHaveBeenCalled();
    expect(drawPuzzleTools).not.toHaveBeenCalled();
  });

  it("redraws the glass when the held frame is repainted", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    bindSurface(session);
    session.puzzle.maybeGlass = GLASS;
    session.maybeContext = fakeContext();
    session.clock.maybePreviousFrame = FRAME;
    session.clock.maybeCamera = camera();

    // Act
    session.paintHeldFrame();

    // Assert
    expect(drawGlassCup).toHaveBeenCalledTimes(1);
  });
});

describe("assignPuzzleCanvas", () => {
  it("attaches no pointer handler and drops the previous one and the crosshair", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    session.frameDeps = () => resizeDeps(PUZZLE_ROUTE);
    bindSurface(session);
    const previous = { detach: vi.fn(), cancel: vi.fn() };
    session.maybeCanvasPointer = previous as unknown as SceneRuntime["maybeCanvasPointer"];
    const { canvas, classes } = fakeCanvas();

    // Act
    session.assignPuzzleCanvas(canvas);

    // Assert
    expect(attachCanvasPointer).not.toHaveBeenCalled();
    expect(previous.detach).toHaveBeenCalledTimes(1);
    expect(session.maybeCanvasPointer).toBeUndefined();
    expect(classes.has("canvas-interactive")).toBe(false);
  });

  it("binds a separate editor input and detaches the previous one", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    session.frameDeps = () => resizeDeps(PUZZLE_ROUTE);
    bindSurface(session);
    const previous = { detach: vi.fn(), cancel: vi.fn() };
    session.puzzle.maybeEditorInput = previous;
    const { canvas } = fakeCanvas();

    // Act
    session.assignPuzzleCanvas(canvas);

    // Assert
    expect(previous.detach).toHaveBeenCalledTimes(1);
    expect(session.puzzle.maybeEditorInput).toBeDefined();
    expect(session.puzzle.maybeEditorInput).not.toBe(previous);
    expect(canvas.addEventListener).toHaveBeenCalledWith("pointerdown", expect.any(Function));
    expect(attachCanvasPointer).not.toHaveBeenCalled();
    expect(session.maybeCanvasPointer).toBeUndefined();
  });

  it("stores the 2D context and connects the resize observer", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    session.frameDeps = () => resizeDeps(PUZZLE_ROUTE);
    bindSurface(session);
    const { canvas, context } = fakeCanvas();

    // Act
    session.assignPuzzleCanvas(canvas);

    // Assert
    expect(session.maybeCanvas).toBe(canvas);
    expect(session.maybeContext).toBe(context);
    expect(session.clock.maybeResizeObserver).toBeDefined();
    expect(observed).toEqual([canvas]);
  });
});

describe("resize observer dispatch", () => {
  it("refits a puzzle with puzzle bounds and starts it through maybeStartPuzzle", () => {
    // Arrange
    const clock = createFrameClock();
    const { canvas, context } = fakeCanvas();
    const deps = resizeDeps(PUZZLE_ROUTE);
    connectResizeObserver(clock, canvas, context, deps);

    // Act
    resizeCallbacks[0]?.();

    // Assert
    expect(clock.worldBounds).toEqual(
      worldBoundsForTarget({ kind: "puzzle", id: "first-pour" }, 390, 844),
    );
    expect(deps.maybeStartPuzzle).toHaveBeenCalledWith("first-pour");
    expect(deps.startScene).not.toHaveBeenCalled();
  });

  it("refits a scene with its scene bounds and starts it through startScene", () => {
    // Arrange
    const clock = createFrameClock();
    const { canvas, context } = fakeCanvas();
    const deps = resizeDeps(SCENE_ROUTE);
    connectResizeObserver(clock, canvas, context, deps);

    // Act
    resizeCallbacks[0]?.();

    // Assert
    expect(clock.worldBounds).toEqual(worldBoundsForViewport("dam-break", 390, 844));
    expect(deps.startScene).toHaveBeenCalledWith("dam-break");
    expect(deps.maybeStartPuzzle).not.toHaveBeenCalled();
  });
});

describe("frameDeps.maybeStartPuzzle", () => {
  it("does not restart a level whose Retry rebuild is still loading", () => {
    // Arrange
    const session = withPuzzle(fakeRuntime(PUZZLE_ROUTE));
    bindLifecycle(session);
    session.puzzle.setMaybeLevelId("first-pour");
    const loading: PuzzleState = {
      kind: "loading",
      pendingRun: true,
      editor: EMPTY_EDITOR,
      speed: 1,
    };
    session.puzzle.current = loading;
    const maybeStart = session.frameDeps().maybeStartPuzzle;
    if (maybeStart === undefined) {
      throw new Error("maybeStartPuzzle is not bound");
    }

    // Act
    maybeStart("first-pour");

    // Assert
    expect(loadPuzzleSession).not.toHaveBeenCalled();
    expect(session.puzzle.current).toBe(loading);
  });
});

describe("handleHashChange puzzle arm", () => {
  function readyRuntime(route: PlayableRoute) {
    const session = withPuzzle(fakeRuntime(route));
    bindLifecycle(session);
    const { canvas, context } = fakeCanvas();
    session.maybeCanvas = canvas;
    session.maybeContext = context;
    session.beginPuzzleRoute = vi.fn(() => Promise.resolve());
    session.leavePuzzle = vi.fn();
    return { session, canvas };
  }

  it("begins the puzzle route from a scene without abandoning the canvas", () => {
    // Arrange
    const { session, canvas } = readyRuntime(SCENE_ROUTE);
    setHash("#/puzzle/first-pour");

    // Act
    handleHashChange(session);

    // Assert
    expect(session.beginPuzzleRoute).toHaveBeenCalledWith("first-pour");
    expect(session.maybeCanvas).toBe(canvas);
    expect(session.setView).not.toHaveBeenCalledWith({ kind: "fallback" });
  });

  it("leaves the puzzle before the unchanged scene path runs", () => {
    // Arrange
    const { session } = readyRuntime(PUZZLE_ROUTE);
    setHash("#/scene/dam-break");

    // Act
    handleHashChange(session);

    // Assert
    const leaveOrder = vi.mocked(session.leavePuzzle).mock.invocationCallOrder[0] ?? 0;
    const loadOrder = vi.mocked(loadSceneSession).mock.invocationCallOrder[0] ?? 0;
    expect(leaveOrder).toBeGreaterThan(0);
    expect(loadOrder).toBeGreaterThan(leaveOrder);
    expect(loadSceneSession).toHaveBeenCalledWith("dam-break");
  });

  it("does nothing when the same puzzle hash is applied again", () => {
    // Arrange
    const { session } = readyRuntime(PUZZLE_ROUTE);
    setHash("#/puzzle/first-pour");

    // Act
    handleHashChange(session);

    // Assert
    expect(session.beginPuzzleRoute).not.toHaveBeenCalled();
    expect(session.leavePuzzle).not.toHaveBeenCalled();
    expect(session.setView).not.toHaveBeenCalled();
  });
});
