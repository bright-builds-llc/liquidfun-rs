import init, { ProofSession } from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";
import { parseRenderFrame } from "../physics/frame";
import { buildAnimatedSvg } from "./animated-svg";
import {
  boundedExportError,
  maybeParseSvgExportRequest,
  type SvgExportWorkerMessage,
} from "./messages";
import { recordProjectedSamples, type ExportDriver } from "./record";

type ExportWorkerScope = {
  addEventListener(type: "message", listener: (event: MessageEvent<unknown>) => void): void;
  postMessage(message: SvgExportWorkerMessage): void;
};

const scope = globalThis as unknown as ExportWorkerScope;

let maybeWasmReady: Promise<void> | undefined;

scope.addEventListener("message", (event: MessageEvent<unknown>) => {
  void runExport(event.data);
});

async function runExport(data: unknown): Promise<void> {
  const started = performance.now();
  const maybeRequest = maybeParseSvgExportRequest(data);
  if (maybeRequest === undefined) {
    post({ type: "error", message: "The SVG export request was invalid." });
    return;
  }

  let maybeSession: ProofSession | undefined;
  try {
    await ensureWasm();
    const session = new ProofSession(maybeRequest.sceneId);
    maybeSession = session;
    const samples = recordProjectedSamples(driverFor(session), maybeRequest, (completed, total) => {
      post({ type: "sampling", completed, total });
    });
    post({ type: "assembling", total: samples.length });
    const svg = buildAnimatedSvg({
      title: maybeRequest.title,
      samples,
      durationSeconds: maybeRequest.durationSeconds,
      viewportWidth: maybeRequest.viewportWidth,
      viewportHeight: maybeRequest.viewportHeight,
      renderMode: maybeRequest.renderMode,
      wireframeStrokeWidth: maybeRequest.wireframeStrokeWidth,
    });
    post({
      type: "complete",
      svg,
      elapsedMs: performance.now() - started,
    });
  } catch (error) {
    post({ type: "error", message: boundedExportError(error) });
  } finally {
    try {
      maybeSession?.free();
    } catch {
      // The export result was already reported.
    }
  }
}

function driverFor(session: ProofSession): ExportDriver {
  return {
    applyControl(name: string, value: string): void {
      session.applyControl(name, value);
    },
    advance(stepCount: number): void {
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

function ensureWasm(): Promise<void> {
  maybeWasmReady ??= init({ module_or_path: wasmUrl })
    .then(() => undefined)
    .catch((error: unknown) => {
      maybeWasmReady = undefined;
      throw error;
    });
  return maybeWasmReady;
}

function post(message: SvgExportWorkerMessage): void {
  scope.postMessage(message);
}
