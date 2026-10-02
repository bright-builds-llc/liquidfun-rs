import { TESLA_VALVE_VIEW_BOUNDS } from "../../src/catalog/scene-records";
import { parseRenderFrame, type RenderFrame } from "../../src/physics/frame";
import { loadSceneSession } from "../../src/physics/loader";
import type { GeneratedProofSession } from "../../src/physics/session";
import { resizeCanvasBackingStore } from "../../src/render/canvas";
import type { Camera } from "../../src/render/camera";
import { presentSceneFrame } from "../../src/render/present-frame";
import { drawShadedBlob } from "../../src/render/webgl-particles";
import type {
  CaseReplicate,
  RendererIdentity,
  TimedSample,
} from "../../scripts/bench-tesla/contracts";
import { fingerprint } from "../../scripts/bench-tesla/fingerprint";
import {
  canonicalJson,
  DEFAULT_WORKLOAD,
  SMOKE_WORKLOAD,
  summarize,
  type BenchmarkWorkload,
} from "../../scripts/bench-tesla/model";
import { measureLiveCadence } from "./live";
import { compareWorkerDirect } from "./worker-quality";

export type BenchmarkSurface = {
  readonly context: CanvasRenderingContext2D;
  readonly particles: HTMLCanvasElement;
  readonly camera: Camera;
  readonly workload: BenchmarkWorkload;
  readonly liveBackend: "direct" | "worker";
};

export function capture(session: GeneratedProofSession): RenderFrame {
  const raw = session.captureFrame();
  try {
    return parseRenderFrame(raw);
  } finally {
    raw.free();
  }
}

export function warm(session: GeneratedProofSession, steps: number): void {
  for (
    let remaining = steps;
    remaining > 0;
    remaining -= Math.min(4, remaining)
  ) {
    session.advance(Math.min(4, remaining));
  }
}

export function present(surface: BenchmarkSurface, frame: RenderFrame): void {
  const settings = surface.workload;
  presentSceneFrame(
    surface.context,
    frame,
    surface.camera,
    settings.renderMode,
    settings.wireframeStrokeWidth,
    settings.maxRenderedParticles,
    surface.particles,
    settings.devicePixelRatio,
    settings.densityShading,
  );
}

function createSurface(
  workload: BenchmarkWorkload,
  liveBackend: "direct" | "worker",
): BenchmarkSurface {
  const walls = document.querySelector("#walls");
  const particles = document.querySelector("#particles");
  if (
    !(walls instanceof HTMLCanvasElement) ||
    !(particles instanceof HTMLCanvasElement)
  )
    throw new Error("Benchmark canvases are missing");
  if (
    window.devicePixelRatio !== workload.devicePixelRatio ||
    innerWidth !== workload.viewport.width ||
    innerHeight !== workload.viewport.height
  )
    throw new Error("Browser viewport/DPR does not match workload");
  if (
    canonicalJson(TESLA_VALVE_VIEW_BOUNDS) !==
    canonicalJson(workload.cameraBounds)
  )
    throw new Error(
      "Scene camera bounds do not match the frozen benchmark workload",
    );
  for (const canvas of [walls, particles]) {
    canvas.style.width = `${workload.viewport.width}px`;
    canvas.style.height = `${workload.viewport.height}px`;
  }
  const camera = resizeCanvasBackingStore(
    walls,
    workload.viewport.width,
    workload.viewport.height,
    workload.devicePixelRatio,
    workload.cameraView,
    workload.cameraBounds,
  );
  const context = walls.getContext("2d");
  if (context === null) throw new Error("Benchmark Canvas2D is unavailable");
  return { context, particles, camera, workload, liveBackend };
}

export function rendererIdentity(
  surface: BenchmarkSurface,
  frame: RenderFrame,
): RendererIdentity {
  const active = drawShadedBlob(
    surface.particles,
    frame,
    surface.camera,
    surface.workload.devicePixelRatio,
    true,
  );
  const gl = surface.particles.getContext("webgl2");
  const info = gl?.getExtension("WEBGL_debug_renderer_info");
  const parameter = (maybeName: number | undefined) => {
    if (gl === null || maybeName === undefined) return "unavailable";
    const value: unknown = gl.getParameter(maybeName);
    return typeof value === "string" && value.length ? value : "unavailable";
  };
  const unmaskedRenderer = parameter(info?.UNMASKED_RENDERER_WEBGL);
  return {
    backend: active ? "webgl2" : "canvas-metaball",
    contextLost: gl?.isContextLost() ?? "unavailable",
    vendor: parameter(gl?.VENDOR),
    renderer: parameter(gl?.RENDERER),
    version: parameter(gl?.VERSION),
    unmaskedVendor: parameter(info?.UNMASKED_VENDOR_WEBGL),
    unmaskedRenderer,
    softwareClassification:
      unmaskedRenderer === "unavailable"
        ? "unavailable"
        : /swiftshader|llvmpipe|software|softpipe/i.test(unmaskedRenderer)
          ? "software"
          : "not-identified-as-software",
  };
}

async function runCase(
  workload: BenchmarkWorkload,
  caseId: string,
  repetition: number,
  liveBackend: "direct" | "worker" = "direct",
): Promise<CaseReplicate> {
  if (
    canonicalJson(workload) !==
    canonicalJson(
      workload.purpose === "smoke" ? SMOKE_WORKLOAD : DEFAULT_WORKLOAD,
    )
  )
    throw new Error("Unknown benchmark profile");
  const scenario = workload.cases.find((entry) => entry.id === caseId);
  if (
    scenario === undefined ||
    !Number.isInteger(repetition) ||
    repetition < 1 ||
    repetition > workload.repetitions
  )
    throw new Error("Unknown benchmark case/repetition");
  if (liveBackend !== "direct" && liveBackend !== "worker")
    throw new Error("Unknown actual live backend");
  const surface = createSurface(workload, liveBackend);
  const fixed = await measureFixed(surface, scenario, repetition);
  const live = await measureLiveCadence(surface, scenario);
  return { ...fixed, live };
}

async function measureFixed(
  surface: BenchmarkSurface,
  scenario: BenchmarkWorkload["cases"][number],
  repetition: number,
): Promise<Omit<CaseReplicate, "live">> {
  const workload = surface.workload;
  const session = await loadSceneSession("tesla-valve");
  try {
    session.applyControl("flow-rate", String(workload.rate));
    session.applyControl("flow-direction", scenario.direction);
    warm(session, scenario.warmupSteps);
    const first = capture(session);
    const start = await fingerprint(first);
    const renderer = rendererIdentity(surface, first);
    // Initialize shader and surface resources outside the measured trajectory.
    present(surface, first);
    present(surface, first);
    const samples: TimedSample[] = [];
    let final = first;
    for (let index = 0; index < workload.sampleCount; index += 1) {
      const began = performance.now();
      session.advance(1);
      const advanced = performance.now();
      const raw = session.captureFrame();
      const captured = performance.now();
      try {
        final = parseRenderFrame(raw);
      } finally {
        raw.free();
      }
      const copied = performance.now();
      present(surface, final);
      const rendered = performance.now();
      samples.push({
        stepIndex: final.stepIndex,
        particleCount: final.particleCount,
        advanceMs: advanced - began,
        captureMs: captured - advanced,
        copyValidateMs: copied - captured,
        renderSubmitMs: rendered - copied,
      });
    }
    const end = await fingerprint(final);
    if (
      canonicalJson(rendererIdentity(surface, final)) !==
      canonicalJson(renderer)
    )
      throw new Error("Renderer backend changed within the case");
    const summary = {
      advanceMs: summarize(samples.map((sample) => sample.advanceMs)),
      captureMs: summarize(samples.map((sample) => sample.captureMs)),
      copyValidateMs: summarize(samples.map((sample) => sample.copyValidateMs)),
      renderSubmitMs: summarize(samples.map((sample) => sample.renderSubmitMs)),
    };
    return {
      caseId: scenario.id,
      repetition,
      fixedStepBackend: "direct",
      renderer,
      start,
      end,
      samples,
      summary,
    };
  } finally {
    session.free();
  }
}

declare global {
  interface Window {
    teslaBenchmark: {
      runCase: typeof runCase;
      compareWorkerDirect: typeof compareWorkerDirect;
    };
  }
}

window.teslaBenchmark = { runCase, compareWorkerDirect };
