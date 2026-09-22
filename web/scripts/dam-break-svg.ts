import { access, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { buildAnimatedSvg } from "../src/export/animated-svg";
import { recordProjectedSamples, type ExportDriver } from "../src/export/record";
import { parseRenderFrame } from "../src/physics/frame";
import type { GeneratedProofSession } from "../src/physics/session";
import { damBreakSvgRequest, DAM_BREAK_SVG_REPO_PATH } from "./dam-break-svg/request";
import { upsertDamBreakAnimatedSvgSection } from "./dam-break-svg/section";

type GeneratedWasm = {
  default: (input?: { module_or_path?: BufferSource }) => Promise<unknown>;
  ProofSession: new (sceneId: string) => GeneratedProofSession;
};

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../..");

function note(message: string): void {
  console.log(`[dam-break-svg] ${message}`);
}

async function main(): Promise<void> {
  const request = damBreakSvgRequest();
  const session = await loadSession();
  try {
    note(
      `recording ${request.sceneId} for ${request.durationSeconds}s at ${request.viewportWidth}x${request.viewportHeight}`,
    );
    const samples = recordProjectedSamples(driverFor(session), request, (completed, total) => {
      if (completed === 1 || completed === total || completed % 20 === 0) {
        note(`sampled ${completed} of ${total}`);
      }
    });
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
      throw new Error("Dam Break export did not produce an SVG document.");
    }

    const svgPath = resolve(repoRoot, DAM_BREAK_SVG_REPO_PATH);
    const readmePath = resolve(repoRoot, "README.md");
    const svgChanged = await writeIfChanged(svgPath, svg);
    const readmeChanged = await writeIfChanged(
      readmePath,
      upsertDamBreakAnimatedSvgSection(await readFile(readmePath, "utf8")),
    );
    note(
      svgChanged || readmeChanged
        ? "updated the Dam Break SVG README assets"
        : "Dam Break SVG and README already match",
    );
  } finally {
    try {
      session.free();
    } catch {
      // The export result was already reported.
    }
  }
}

async function loadSession(): Promise<GeneratedProofSession> {
  const directory = resolve(repoRoot, "web/src/generated/liquidfun-wasm");
  const jsPath = resolve(directory, "liquidfun_wasm.js");
  const wasmPath = resolve(directory, "liquidfun_wasm_bg.wasm");
  try {
    await access(jsPath);
    const wasmBytes = await readFile(wasmPath);
    const generated = (await import(pathToFileURL(jsPath).href)) as GeneratedWasm;
    await generated.default({ module_or_path: wasmBytes });
    return new generated.ProofSession("dam-break");
  } catch (error) {
    const detail = error instanceof Error ? error.message : "unknown error";
    throw new Error(
      `Could not load the generated Dam Break WASM package (${detail}). Run \`bun scripts/web-build.ts wasm\` from the repository root first.`,
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

async function writeIfChanged(path: string, next: string): Promise<boolean> {
  const maybeCurrent = await readOptionalText(path);
  if (maybeCurrent === next) {
    return false;
  }

  await writeFile(path, next);
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

try {
  await main();
} catch (error) {
  const detail = error instanceof Error ? error.message : "Dam Break SVG export failed.";
  console.error(`[dam-break-svg] ${detail}`);
  process.exitCode = 1;
}
