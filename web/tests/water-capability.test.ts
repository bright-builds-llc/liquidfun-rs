import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  cachedWaterSupport,
  probeWaterSupport,
  resetWaterSupportCacheForTests,
  waterSupportFrom,
  type WaterSupportFacts,
} from "../src/render/water-capability";
import { linkWaterPrograms } from "../src/render/water-programs";
import { glFixture, type GlFixtureOptions } from "./webgl-render-fixture";

const SUPPORTED: WaterSupportFacts = {
  webgl2: true,
  floatTargets: true,
  maxDrawBuffers: 2,
  highpFragment: 23,
  mrtComplete: true,
  programsLinked: true,
};

function probeFixture(options: GlFixtureOptions = {}) {
  const fixture = glFixture(options);
  const extension = fixture.gl.getExtension("WEBGL_lose_context");
  if (extension === null) throw new Error("fixture lacks WEBGL_lose_context");
  const loseContext = vi.spyOn(extension, "loseContext");
  vi.mocked(fixture.gl.getExtension).mockClear();
  return { ...fixture, loseContext, createCanvas: () => fixture.canvas };
}

beforeEach(() => {
  resetWaterSupportCacheForTests();
});

describe("waterSupportFrom", () => {
  it("accepts a device with every Water requirement", () => {
    // Arrange
    const facts = SUPPORTED;

    // Act
    const supported = waterSupportFrom(facts);

    // Assert
    expect(supported).toBe(true);
  });

  it.each([
    ["no WebGL2", { webgl2: false }],
    ["no float targets", { floatTargets: false }],
    ["a single draw buffer", { maxDrawBuffers: 1 }],
    ["no highp fragment precision", { highpFragment: 0 }],
    ["an incomplete MRT target", { mrtComplete: false }],
    ["unlinked programs", { programsLinked: false }],
  ] as const)("rejects %s", (_name, failing) => {
    // Arrange
    const facts: WaterSupportFacts = { ...SUPPORTED, ...failing };

    // Act
    const supported = waterSupportFrom(facts);

    // Assert
    expect(supported).toBe(false);
  });
});

describe("probeWaterSupport", () => {
  it("reports support on a capable device and releases the probe context once", () => {
    // Arrange
    const fixture = probeFixture();

    // Act
    const supported = probeWaterSupport(fixture.createCanvas);

    // Assert
    expect(supported).toBe(true);
    expect(fixture.loseContext).toHaveBeenCalledTimes(1);
  });

  it("reports no support when WebGL2 is missing", () => {
    // Arrange
    const canvas = { getContext: () => null } as unknown as HTMLCanvasElement;

    // Act
    const supported = probeWaterSupport(() => canvas);

    // Assert
    expect(supported).toBe(false);
  });

  it("reports no support without a DOM canvas", () => {
    // Arrange
    const createCanvas = () => undefined;

    // Act
    const supported = probeWaterSupport(createCanvas);

    // Assert
    expect(supported).toBe(false);
  });

  it.each([
    ["neither color-buffer extension", { maybeExtensions: [] }],
    ["a single draw buffer", { maybeMaxDrawBuffers: 1 }],
    ["no highp fragment precision", { maybeHighpPrecision: 0 }],
  ] as const)(
    "reports no support with %s and still releases the context",
    (_name, options) => {
      // Arrange
      const fixture = probeFixture(options);

      // Act
      const supported = probeWaterSupport(fixture.createCanvas);

      // Assert
      expect(supported).toBe(false);
      expect(fixture.loseContext).toHaveBeenCalledTimes(1);
    },
  );

  it("reports no support when the 2-attachment framebuffer is incomplete", () => {
    // Arrange
    const fixture = probeFixture({ maybeIncompleteMrt: true });

    // Act
    const supported = probeWaterSupport(fixture.createCanvas);

    // Assert
    expect(supported).toBe(false);
    expect(fixture.loseContext).toHaveBeenCalledTimes(1);
  });

  it("reports support when only the half-float extension is exposed (D-13)", () => {
    // Arrange
    const fixture = probeFixture({
      maybeExtensions: ["EXT_color_buffer_half_float"],
    });

    // Act
    const supported = probeWaterSupport(fixture.createCanvas);

    // Assert
    expect(supported).toBe(true);
  });

  it("reports no support and releases the context when the probe throws", () => {
    // Arrange
    const fixture = probeFixture();
    vi.mocked(fixture.gl.createFramebuffer).mockImplementation(() => {
      throw new Error("driver failure");
    });

    // Act
    const supported = probeWaterSupport(fixture.createCanvas);

    // Assert
    expect(supported).toBe(false);
    expect(fixture.loseContext).toHaveBeenCalledTimes(1);
  });
});

describe("cachedWaterSupport", () => {
  it("probes once per page", () => {
    // Arrange
    const fixture = probeFixture();
    const createCanvas = vi.fn(fixture.createCanvas);

    // Act
    const verdicts = [
      cachedWaterSupport(createCanvas),
      cachedWaterSupport(createCanvas),
      cachedWaterSupport(createCanvas),
    ];

    // Assert
    expect(verdicts).toEqual([true, true, true]);
    expect(createCanvas).toHaveBeenCalledTimes(1);
  });

  it("probes again after the test reset", () => {
    // Arrange
    const fixture = probeFixture();
    const createCanvas = vi.fn(fixture.createCanvas);
    cachedWaterSupport(createCanvas);

    // Act
    resetWaterSupportCacheForTests();
    cachedWaterSupport(createCanvas);

    // Assert
    expect(createCanvas).toHaveBeenCalledTimes(2);
  });
});

describe("linkWaterPrograms", () => {
  it("links the point and shade programs", () => {
    // Arrange
    const { gl } = glFixture();

    // Act
    const programs = linkWaterPrograms(gl);

    // Assert
    expect(programs).toEqual({
      point: expect.objectContaining({ id: expect.any(Number) }),
      shade: expect.objectContaining({ id: expect.any(Number) }),
    });
    expect(gl.createProgram).toHaveBeenCalledTimes(2);
  });

  it("returns undefined when a program fails to link", () => {
    // Arrange
    const { gl } = glFixture();
    vi.mocked(gl.getProgramParameter).mockReturnValue(false);

    // Act
    const programs = linkWaterPrograms(gl);

    // Assert
    expect(programs).toBeUndefined();
  });

  it("deletes the linked point program when the shade program fails", () => {
    // Arrange
    const { gl } = glFixture();
    vi.mocked(gl.getProgramParameter)
      .mockReturnValueOnce(true)
      .mockReturnValueOnce(false);

    // Act
    const programs = linkWaterPrograms(gl);

    // Assert
    expect(programs).toBeUndefined();
    expect(gl.deleteProgram).toHaveBeenCalledTimes(2);
  });
});
