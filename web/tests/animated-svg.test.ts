import { describe, expect, it } from "vitest";

import { maybeSceneById } from "../src/catalog/scenes";
import { buildAnimatedSvg } from "../src/export/animated-svg";
import {
  exportProgressPercent,
  formatGenerationSeconds,
  maybeParseSvgExportSeconds,
  sampleCountForDuration,
  STEPS_PER_SVG_SAMPLE,
  svgExportFileName,
} from "../src/export/duration";
import {
  boundedExportError,
  maybeParseSvgExportRequest,
  maybeParseWorkerMessage,
} from "../src/export/messages";
import { changedPresetEntries } from "../src/export/presets";
import { projectRenderSample, type ProjectedSample } from "../src/export/project";
import { recordProjectedSamples, type ExportDriver } from "../src/export/record";
import type { RenderFrame } from "../src/physics/frame";
import { createCamera } from "../src/render/camera";

function emptyFrame(): RenderFrame {
  return {
    stepIndex: 0,
    particleCount: 0,
    rigidShapeCount: 0,
    maxSpeed: 0,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array(),
    particleColors: new Uint8Array(),
    particleRadii: new Float32Array(),
    rigidSegments: new Float32Array(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
  };
}

function particleFrame(positions: readonly number[]): RenderFrame {
  const particleCount = positions.length / 2;
  return {
    ...emptyFrame(),
    particleCount,
    particlePositions: new Float32Array(positions),
    particleColors: new Uint8Array(Array.from({ length: particleCount * 4 }, () => 255)),
    particleRadii: new Float32Array(Array.from({ length: particleCount }, () => 0.1)),
  };
}

function particle(x: number, y: number): ProjectedSample["particles"][number] {
  return { x, y, radius: 4, red: 12, green: 34, blue: 56, alpha: 1 };
}

function sample(particles: ProjectedSample["particles"]): ProjectedSample {
  return { particles, segments: [], bodies: [] };
}

const SVG_INPUT = {
  title: "Dam Break",
  durationSeconds: 1,
  viewportWidth: 320,
  viewportHeight: 180,
  renderMode: "solid" as const,
  wireframeStrokeWidth: 0.3,
};

describe("maybeParseSvgExportSeconds", () => {
  it("accepts whole seconds from 1 through 30", () => {
    // Arrange
    const raw = " 10 ";

    // Act
    const parsed = maybeParseSvgExportSeconds(raw);

    // Assert
    expect(parsed).toBe(10);
  });

  it("rejects partial, fractional, and out-of-range text", () => {
    // Arrange
    const rejected = ["", "10.", "1.5", "0", "31", "ten"];

    // Act
    const parsed = rejected.map((raw) => maybeParseSvgExportSeconds(raw));

    // Assert
    expect(parsed).toEqual([undefined, undefined, undefined, undefined, undefined, undefined]);
  });
});

describe("sampleCountForDuration", () => {
  it("includes the initial frame and one sample every twentieth of a second", () => {
    // Arrange
    const durationSeconds = 10;

    // Act
    const count = sampleCountForDuration(durationSeconds);

    // Assert
    expect(count).toBe(201);
    expect(STEPS_PER_SVG_SAMPLE).toBe(3);
  });
});

describe("formatGenerationSeconds", () => {
  it("prints elapsed time in seconds with one decimal", () => {
    // Arrange
    const elapsedMs = 1800;

    // Act
    const text = formatGenerationSeconds(elapsedMs);

    // Assert
    expect(text).toBe("1.8 s");
  });
});

describe("svgExportFileName", () => {
  it("names the download from the scene id and duration", () => {
    // Arrange
    const sceneId = "dam-break";

    // Act
    const fileName = svgExportFileName(sceneId, 10);

    // Assert
    expect(fileName).toBe("dam-break-10s.svg");
  });
});

describe("exportProgressPercent", () => {
  it("floors the percent and caps it at 100", () => {
    // Arrange
    const completed = 1;
    const total = 4;

    // Act
    const percent = exportProgressPercent(completed, total);

    // Assert
    expect(percent).toBe(25);
    expect(exportProgressPercent(8, 4)).toBe(100);
  });
});

describe("changedPresetEntries", () => {
  it("applies recreating presets before live ones and drops actions", () => {
    // Arrange
    const maybeDamBreak = maybeSceneById("dam-break");
    const maybeFountain = maybeSceneById("fountain");
    expect(maybeDamBreak).toBeDefined();
    expect(maybeFountain).toBeDefined();
    if (maybeDamBreak === undefined || maybeFountain === undefined) {
      return;
    }

    // Act
    const damBreak = changedPresetEntries(maybeDamBreak, {
      gravity: "high",
      "water-amount": "small",
      "drop-obstacle": "ignored",
    });
    const fountain = changedPresetEntries(maybeFountain, {
      "aim-angle": "left",
      "emission-rate": "high",
    });

    // Assert
    expect(damBreak).toEqual([
      { name: "water-amount", value: "small" },
      { name: "gravity", value: "high" },
    ]);
    expect(fountain).toEqual([
      { name: "emission-rate", value: "high" },
      { name: "aim-angle", value: "left" },
    ]);
  });
});

describe("projectRenderSample", () => {
  it("projects world y upward and keeps only the rendered particle cap", () => {
    // Arrange
    const camera = createCamera(960, 540);
    const frame = particleFrame([-1, 0, 1, 2]);

    // Act
    const projected = projectRenderSample(frame, camera, 1);

    // Assert
    expect(projected.particles).toHaveLength(1);
    expect(projected.particles[0]?.x).toBeLessThan(
      projectRenderSample(frame, camera, 2).particles[1]?.x ?? 0,
    );
    expect(projectRenderSample(particleFrame([0, 0, 0, 1]), camera, 2).particles[0]?.y).toBeGreaterThan(
      projectRenderSample(particleFrame([0, 0, 0, 1]), camera, 2).particles[1]?.y ?? 0,
    );
  });
});

describe("recordProjectedSamples", () => {
  it("captures the initial frame before stepping three engine steps per sample", () => {
    // Arrange
    const log: string[] = [];
    const driver: ExportDriver = {
      applyControl(name, value) {
        log.push(`control:${name}=${value}`);
      },
      advance(stepCount) {
        log.push(`advance:${stepCount}`);
      },
      captureFrame() {
        log.push("capture");
        return emptyFrame();
      },
    };

    // Act
    const samples = recordProjectedSamples(
      driver,
      {
        durationSeconds: 1,
        controls: [{ name: "gravity", value: "high" }],
        viewportWidth: 960,
        viewportHeight: 540,
        zoom: 1,
        panX: 0,
        panY: 0,
        maxRenderedParticles: 4,
      },
      () => undefined,
    );

    // Assert
    expect(samples).toHaveLength(21);
    expect(log[0]).toBe("control:gravity=high");
    expect(log[1]).toBe("capture");
    expect(log.filter((entry) => entry === "advance:3")).toHaveLength(20);
    expect(log.filter((entry) => entry === "capture")).toHaveLength(21);
  });

  it("runs beforeSample before that sample's engine advance", () => {
    // Arrange
    const log: string[] = [];
    const driver: ExportDriver = {
      applyControl() {
        log.push("control");
      },
      advance(stepCount) {
        log.push(`advance:${stepCount}`);
      },
      captureFrame() {
        log.push("capture");
        return emptyFrame();
      },
    };

    // Act
    recordProjectedSamples(
      driver,
      {
        durationSeconds: 1,
        controls: [],
        viewportWidth: 320,
        viewportHeight: 180,
        zoom: 1,
        panX: 0,
        panY: 0,
        maxRenderedParticles: 1,
        beforeSample(sampleIndex) {
          log.push(`before:${sampleIndex}`);
        },
      },
      () => undefined,
    );

    // Assert
    expect(log.slice(0, 5)).toEqual([
      "before:0",
      "capture",
      "before:1",
      "advance:3",
      "capture",
    ]);
  });
});

describe("buildAnimatedSvg", () => {
  it("leaves a particle that does not move as a static circle", () => {
    // Arrange
    const samples = [sample([particle(10, 20)]), sample([particle(10, 20)])];

    // Act
    const svg = buildAnimatedSvg({ ...SVG_INPUT, samples });

    // Assert
    expect(svg).toContain(`<svg xmlns="http://www.w3.org/2000/svg"`);
    expect(svg).toContain("<title>Dam Break</title>");
    expect(svg).toContain(`<circle cx="10" cy="20" r="4" fill="rgba(12,34,56,1)"/>`);
    expect(svg).not.toContain("<animate");
  });

  it("animates a moving particle with SMIL cx and cy tracks", () => {
    // Arrange
    const samples = [sample([particle(10, 20)]), sample([particle(30, 40)])];

    // Act
    const svg = buildAnimatedSvg({ ...SVG_INPUT, samples });

    // Assert
    expect(svg).toContain(`attributeName="cx" values="10;30"`);
    expect(svg).toContain(`attributeName="cy" values="20;40"`);
    expect(svg).toContain(`dur="1s"`);
    expect(svg).toContain(`repeatCount="indefinite"`);
  });

  it("hides a particle slot until that particle exists", () => {
    // Arrange
    const samples = [sample([]), sample([particle(8, 9)])];

    // Act
    const svg = buildAnimatedSvg({ ...SVG_INPUT, samples });

    // Assert
    expect(svg).toContain(`attributeName="opacity" values="0;1"`);
  });

  it("draws wireframe particles as strokes", () => {
    // Arrange
    const samples = [sample([particle(1, 2)])];

    // Act
    const svg = buildAnimatedSvg({
      ...SVG_INPUT,
      samples,
      renderMode: "wireframe",
    });

    // Assert
    expect(svg).toContain(`fill="none"`);
    expect(svg).toContain(`stroke="rgba(12,34,56,1)"`);
    expect(svg).toContain(`stroke-width="0.3"`);
  });

  it("escapes rigid-body labels", () => {
    // Arrange
    const samples: ProjectedSample[] = [
      {
        particles: [],
        segments: [{ x1: 1, y1: 2, x2: 3, y2: 4 }],
        bodies: [{ x: 40, y: 50, radius: 30, label: "A&B" }],
      },
    ];

    // Act
    const svg = buildAnimatedSvg({ ...SVG_INPUT, samples });

    // Assert
    expect(svg).toContain(">A&amp;B</text>");
    expect(svg).toContain("<line ");
    expect(svg).not.toContain(`attributeName="x1"`);
  });
});

describe("maybeParseSvgExportRequest", () => {
  it("rejects an unknown scene id", () => {
    // Arrange
    const request = {
      sceneId: "not-a-scene",
      title: "Dam Break",
      durationSeconds: 10,
      controls: [],
      viewportWidth: 960,
      viewportHeight: 540,
      zoom: 1,
      panX: 0,
      panY: 0,
      renderMode: "solid",
      wireframeStrokeWidth: 0.3,
      maxRenderedParticles: 4000,
    };

    // Act
    const parsed = maybeParseSvgExportRequest(request);

    // Assert
    expect(parsed).toBeUndefined();
  });
});

describe("boundedExportError", () => {
  it("shortens a long failure so the page can display it", () => {
    // Arrange
    const error = new Error("x".repeat(600));

    // Act
    const message = boundedExportError(error);
    const parsed = maybeParseWorkerMessage({ type: "error", message });

    // Assert
    expect(message.length).toBe(500);
    expect(parsed).toEqual({ type: "error", message });
  });
});

describe("maybeParseWorkerMessage", () => {
  it("accepts a completed svg message", () => {
    // Arrange
    const message = { type: "complete", svg: "<svg></svg>", elapsedMs: 12.5 };

    // Act
    const parsed = maybeParseWorkerMessage(message);

    // Assert
    expect(parsed).toEqual(message);
  });
});
