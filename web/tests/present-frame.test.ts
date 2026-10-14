import { beforeEach, describe, expect, it, vi } from "vitest";

import { createCamera } from "../src/render/camera";
import { drawRenderFrame } from "../src/render/canvas";
import { presentSceneFrame } from "../src/render/present-frame";
import type { RenderMode } from "../src/render/mode";
import { drawShadedBlob } from "../src/render/webgl-particles";
import { drawWater } from "../src/render/webgl-water";
import { particleFrame } from "./webgl-render-fixture";

vi.mock("../src/render/webgl-water", () => ({ drawWater: vi.fn() }));
vi.mock("../src/render/webgl-particles", () => ({ drawShadedBlob: vi.fn() }));
vi.mock("../src/render/canvas", () => ({ drawRenderFrame: vi.fn() }));

const water = vi.mocked(drawWater);
const blob = vi.mocked(drawShadedBlob);
const canvasDraw = vi.mocked(drawRenderFrame);
const camera = createCamera(640, 480);
const context = {} as CanvasRenderingContext2D;
const particleCanvas = {} as HTMLCanvasElement;

function present(mode: RenderMode) {
  return presentSceneFrame(
    context,
    particleFrame(),
    camera,
    mode,
    0.3,
    1_000,
    particleCanvas,
    1,
  );
}

/** The `webglCovered` argument of the single 2D draw. */
function coveredArgument(): unknown {
  return canvasDraw.mock.calls[0]?.[6];
}

describe("presentSceneFrame fallback chain", () => {
  beforeEach(() => {
    water.mockReset();
    blob.mockReset();
    canvasDraw.mockReset();
  });

  it("reports Water and skips the shaded blob when Water draws", () => {
    // Arrange
    water.mockReturnValue(true);
    blob.mockReturnValue(true);

    // Act
    const drawn = present("water");

    // Assert
    expect(drawn).toBe("webgl-water");
    expect(blob).not.toHaveBeenCalled();
    expect(coveredArgument()).toBe(true);
  });

  it("falls back from Water to the shaded blob", () => {
    // Arrange
    water.mockReturnValue(false);
    blob.mockReturnValue(true);

    // Act
    const drawn = present("water");

    // Assert
    expect(drawn).toBe("webgl");
    expect(coveredArgument()).toBe(true);
  });

  it("falls back from Water to the canvas soft blob when both WebGL draws fail", () => {
    // Arrange
    water.mockReturnValue(false);
    blob.mockReturnValue(false);

    // Act
    const drawn = present("water");

    // Assert
    expect(drawn).toBe("canvas");
    expect(coveredArgument()).toBe(false);
  });

  it("never attempts Water for the shaded blob", () => {
    // Arrange
    blob.mockReturnValue(true);

    // Act
    const drawn = present("shaded-blob");

    // Assert
    expect(water).not.toHaveBeenCalled();
    expect(drawn).toBe("webgl");
  });

  it("draws non-WebGL modes on the canvas only", () => {
    // Arrange
    const modes: readonly RenderMode[] = [
      "circle-wireframe",
      "triangle-wireframe",
      "solid",
      "soft-blob",
      "contour",
    ];

    // Act
    const drawn = modes.map(present);

    // Assert
    expect(drawn).toEqual(modes.map(() => "canvas"));
    expect(water).not.toHaveBeenCalled();
    expect(blob).not.toHaveBeenCalled();
    expect(canvasDraw.mock.calls.map((call) => call[6])).toEqual(
      modes.map(() => false),
    );
  });

  it("draws the 2D frame exactly once per presented frame", () => {
    // Arrange
    water.mockReturnValue(false);
    blob.mockReturnValue(false);

    // Act
    present("water");

    // Assert
    expect(canvasDraw).toHaveBeenCalledTimes(1);
  });
});
