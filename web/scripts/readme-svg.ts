import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { worldBoundsForScene } from "../src/catalog/scenes";
import { buildAnimatedSvg } from "../src/export/animated-svg";
import { recordProjectedSamples, type ExportDriver } from "../src/export/record";
import type { SvgExportRequest } from "../src/export/messages";
import { parseRenderFrame } from "../src/physics/frame";
import type { GeneratedProofSession } from "../src/physics/session";
import {
  assertReadmeSvgPlanCoverage,
  README_SVG_PLANS,
  readmeSvgRepoPath,
  readmeSvgRequest,
  readmeWebpRepoPath,
  type ReadmeSvgCue,
  type ReadmeSvgPlan,
} from "./readme-svg/plans";
import { README_RASTER_FONT_FILE, rasterizeAnimatedSvg } from "./readme-svg/raster";
import { upsertReadmeSvgGallery } from "./readme-svg/section";
import {
  assertReadmeWebpByteLimit,
  README_WEBP_FPS,
  README_WEBP_PRESET,
  README_WEBP_QUALITY,
} from "./readme-svg/webp";

type GeneratedWasm = {
  default: (input?: { module_or_path?: BufferSource }) => Promise<unknown>;
  ProofSession: new (sceneId: string) => GeneratedProofSession;
};

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const logPath = resolve(repoRoot, "target/readme-svg/readme-svg.log");

function note(message: string): void {
  console.log(`[readme-svg] ${message}`);
}

async function main(): Promise<void> {
  await mkdir(dirname(logPath), { recursive: true });
  await writeFile(logPath, "");
  assertReadmeSvgPlanCoverage();
  const generated = await loadWasm();
  let svgCount = 0;
  let webpCount = 0;

  for (const plan of README_SVG_PLANS) {
    const request = readmeSvgRequest(plan);
    const svgChanged = await writeSceneSvg(generated, plan, request);
    if (svgChanged) {
      svgCount += 1;
    }
    const webpChanged = await writeSceneWebp(plan, svgChanged);
    if (webpChanged) {
      webpCount += 1;
    }
  }

  const readmePath = resolve(repoRoot, "README.md");
  const readmeChanged = await writeIfChanged(
    readmePath,
    upsertReadmeSvgGallery(await readFile(readmePath, "utf8"), README_SVG_PLANS),
  );
  const summary = previewSummary(svgCount, webpCount, readmeChanged);
  note(summary);
  await appendLog(summary);
}

function previewSummary(svgCount: number, webpCount: number, readmeChanged: boolean): string {
  if (svgCount === 0 && webpCount === 0 && !readmeChanged) {
    return "README scene SVGs, WebPs, and gallery already match";
  }

  const parts = [
    `${svgCount} SVG file${svgCount === 1 ? "" : "s"}`,
    `${webpCount} WebP file${webpCount === 1 ? "" : "s"}`,
  ];
  if (readmeChanged) {
    parts.push("the README gallery");
  }
  return `updated ${parts.join(" and ")}`;
}

async function writeSceneSvg(
  generated: GeneratedWasm,
  plan: ReadmeSvgPlan,
  request: SvgExportRequest,
): Promise<boolean> {
  const session = new generated.ProofSession(request.sceneId);
  try {
    note(
      `recording ${request.sceneId} for ${request.durationSeconds}s at ${request.viewportWidth}x${request.viewportHeight}`,
    );
    const samples = recordProjectedSamples(
      driverFor(session),
      {
        durationSeconds: request.durationSeconds,
        controls: request.controls,
        viewportWidth: request.viewportWidth,
        viewportHeight: request.viewportHeight,
        zoom: request.zoom,
        panX: request.panX,
        panY: request.panY,
        maxRenderedParticles: request.maxRenderedParticles,
        worldBounds: worldBoundsForScene(request.sceneId),
        beforeSample(sampleIndex) {
          applyPlanCues(session, plan, sampleIndex);
        },
      },
      (completed, total) => {
        if (completed === 1 || completed === total || completed % 40 === 0) {
          note(`${request.sceneId} sampled ${completed} of ${total}`);
        }
      },
    );
    const svg = buildAnimatedSvg({
      title: request.title,
      samples,
      durationSeconds: request.durationSeconds,
      viewportWidth: request.viewportWidth,
      viewportHeight: request.viewportHeight,
      renderMode: request.renderMode,
      wireframeStrokeWidth: request.wireframeStrokeWidth,
    });
    if (!svg.startsWith("<svg")) {
      throw new Error(`${request.sceneId} export did not produce an SVG document.`);
    }

    const svgPath = resolve(repoRoot, readmeSvgRepoPath(plan.id));
    await mkdir(dirname(svgPath), { recursive: true });
    return await writeIfChanged(svgPath, svg);
  } finally {
    try {
      session.free();
    } catch {
      // The export result was already reported.
    }
  }
}

function applyPlanCues(
  session: GeneratedProofSession,
  plan: ReadmeSvgPlan,
  sampleIndex: number,
): void {
  for (const cue of plan.cues) {
    if (cue.atSample !== sampleIndex) {
      continue;
    }
    applyCue(session, plan.id, cue);
  }
}

function applyCue(
  session: GeneratedProofSession,
  sceneId: string,
  cue: ReadmeSvgCue,
): void {
  if (cue.kind === "action") {
    note(`${sceneId} action ${cue.name} at sample ${cue.atSample}`);
    session.applyAction(cue.name);
    return;
  }

  note(`${sceneId} pointer up at sample ${cue.atSample}`);
  session.pointerAction("up", cue.worldX, cue.worldY);
}

async function loadWasm(): Promise<GeneratedWasm> {
  const directory = resolve(repoRoot, "web/src/generated/liquidfun-wasm");
  const jsPath = resolve(directory, "liquidfun_wasm.js");
  const wasmPath = resolve(directory, "liquidfun_wasm_bg.wasm");
  try {
    await access(jsPath);
    const wasmBytes = await readFile(wasmPath);
    const generated = (await import(pathToFileURL(jsPath).href)) as GeneratedWasm;
    await generated.default({ module_or_path: wasmBytes });
    return generated;
  } catch (error) {
    const detail = error instanceof Error ? error.message : "unknown error";
    throw new Error(
      `Could not load the generated WASM package (${detail}). Run \`bun scripts/web-build.ts wasm\` from the repository root first.`,
      { cause: error },
    );
  }
}

function driverFor(session: GeneratedProofSession): ExportDriver {
  return {
    applyControl(name, value) {
      session.applyControl(name, value);
    },
    advance(stepCount) {
      session.advance(stepCount);
    },
    captureFrame() {
      const rawFrame = session.captureFrame();
      try {
        return parseRenderFrame(rawFrame);
      } finally {
        rawFrame.free();
      }
    },
  };
}

async function writeSceneWebp(plan: ReadmeSvgPlan, svgChanged: boolean): Promise<boolean> {
  const svgPath = resolve(repoRoot, readmeSvgRepoPath(plan.id));
  const webpPath = resolve(repoRoot, readmeWebpRepoPath(plan.id));
  if (!svgChanged && (await exists(webpPath))) {
    note(`${plan.id} WebP already matches the SVG`);
    return false;
  }

  note(
    `rasterizing ${plan.id} at ${README_WEBP_FPS} fps, quality ${README_WEBP_QUALITY}, preset ${README_WEBP_PRESET}`,
  );
  const webp = await rasterizeAnimatedSvg(await readFile(svgPath, "utf8"), {
    framesPerSecond: README_WEBP_FPS,
    quality: README_WEBP_QUALITY,
    preset: README_WEBP_PRESET,
    fontFile: README_RASTER_FONT_FILE,
    onFrame(completed, total) {
      note(`${plan.id} rasterized ${completed} of ${total}`);
    },
  });
  assertReadmeWebpByteLimit(plan.id, webp.length);
  return writeBytesIfChanged(webpPath, webp);
}

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch (error) {
    if (isMissingFile(error)) {
      return false;
    }
    throw error;
  }
}

async function writeBytesIfChanged(path: string, next: Uint8Array): Promise<boolean> {
  const maybeCurrent = await readOptionalBytes(path);
  if (maybeCurrent !== undefined && maybeCurrent.equals(Buffer.from(next))) {
    return false;
  }

  await writeFile(path, next);
  note(`wrote ${path} (${next.length} bytes)`);
  return true;
}

async function readOptionalBytes(path: string): Promise<Buffer | undefined> {
  try {
    return await readFile(path);
  } catch (error) {
    if (isMissingFile(error)) {
      return undefined;
    }
    throw error;
  }
}

async function writeIfChanged(path: string, next: string): Promise<boolean> {
  const maybeCurrent = await readOptionalText(path);
  if (maybeCurrent === next) {
    return false;
  }

  await writeFile(path, next);
  note(`wrote ${path}`);
  return true;
}

async function readOptionalText(path: string): Promise<string | undefined> {
  try {
    return await readFile(path, "utf8");
  } catch (error) {
    if (isMissingFile(error)) {
      return undefined;
    }
    throw error;
  }
}

function isMissingFile(error: unknown): boolean {
  return (
    typeof error === "object" &&
    error !== null &&
    "code" in error &&
    error.code === "ENOENT"
  );
}

async function appendLog(message: string): Promise<void> {
  await writeFile(logPath, `${message}\n`, { flag: "a" });
}

try {
  await main();
} catch (error) {
  const detail = error instanceof Error ? error.message : "README SVG export failed.";
  console.error(`[readme-svg] ${detail}`);
  process.exitCode = 1;
}
