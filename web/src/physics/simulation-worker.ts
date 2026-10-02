import { FRAME_STEP_BUDGET_MS } from "./clock";
import { parseRenderFrame } from "./frame";
import { loadSceneSession } from "./loader";
import type { GeneratedProofSession } from "./session";
import {
  frameTransferables,
  parseWorkerRequest,
  type WorkerReply,
  type WorkerRequest,
} from "./worker-messages";
import { applyWorkerGravitySample } from "./worker-gravity";

type WorkerScope = {
  onmessage: ((event: MessageEvent<unknown>) => void) | null;
  postMessage(message: WorkerReply, transfer?: Transferable[]): void;
  close(): void;
};
const scope = globalThis as unknown as WorkerScope;
let maybeSession: GeneratedProofSession | undefined;
let maybeGeneration: number | undefined;
let lastId = 0,
  busy = false,
  disposed = false;

function release(): void {
  const session = maybeSession;
  maybeSession = undefined;
  disposed = true;
  session?.free();
}

function sendFrame(
  request: WorkerRequest,
  ran: number,
  advanceMs: number,
): void {
  const session = maybeSession;
  if (session === undefined) throw new Error("Worker session is unavailable");
  const began = performance.now(),
    raw = session.captureFrame(),
    captured = performance.now();
  let frame;
  try {
    frame = parseRenderFrame(raw);
  } finally {
    raw.free();
  }
  const parsed = performance.now();
  scope.postMessage(
    {
      version: 1,
      generation: request.generation,
      id: request.id,
      kind: "frame",
      frame,
      ran,
      timing: {
        advanceMs,
        captureMs: captured - began,
        parseMs: parsed - captured,
      },
    },
    frameTransferables(frame),
  );
}

async function execute(request: WorkerRequest): Promise<void> {
  if (request.op === "init") {
    if (maybeSession !== undefined || maybeGeneration !== undefined)
      throw new Error("Worker initialization is exactly once");
    maybeGeneration = request.generation;
    const session = await loadSceneSession("tesla-valve");
    if (disposed) {
      session.free();
      return;
    }
    maybeSession = session;
    scope.postMessage({
      version: 1,
      generation: request.generation,
      id: request.id,
      kind: "ready",
    });
    return;
  }
  if (
    disposed ||
    request.generation !== maybeGeneration ||
    maybeSession === undefined
  )
    throw new Error("Worker generation/session is unavailable");
  const session = maybeSession;
  if (request.op === "dispose") {
    release();
    scope.postMessage({
      version: 1,
      generation: request.generation,
      id: request.id,
      kind: "disposed",
    });
    scope.close();
    return;
  }
  if (request.op === "capture") {
    sendFrame(request, 0, 0);
    return;
  }
  if (request.op === "advance") {
    const began = performance.now();
    let ran = 0;
    do {
      session.advance(1);
      ran += 1;
    } while (
      ran < request.count &&
      (!request.budgeted || performance.now() - began < FRAME_STEP_BUDGET_MS)
    );
    sendFrame(request, ran, performance.now() - began);
    return;
  }
  let value: boolean | null = null;
  let maybeGravityAccepted: boolean | undefined;
  switch (request.op) {
    case "advance-only":
      session.advance(request.count);
      break;
    case "control":
      value = session.applyControl(request.name, request.value);
      break;
    case "action":
      session.applyAction(request.name);
      break;
    case "pointer":
      session.pointerAction(request.kind, request.x, request.y);
      break;
    case "gravity":
      maybeGravityAccepted = applyWorkerGravitySample(
        session,
        request.x,
        request.y,
      );
      break;
    case "restore-gravity":
      session.restoreAuthoredGravity();
      break;
  }
  scope.postMessage({
    version: 1,
    generation: request.generation,
    id: request.id,
    kind: "result",
    value,
    ...(maybeGravityAccepted === undefined ? {} : { maybeGravityAccepted }),
  });
}

scope.onmessage = (event) => {
  let maybeRequest: WorkerRequest | undefined;
  try {
    const request = parseWorkerRequest(event.data);
    maybeRequest = request;
    if (
      request.op === "dispose" &&
      request.generation === maybeGeneration &&
      request.id > lastId
    ) {
      lastId = request.id;
      release();
      scope.postMessage({
        version: 1,
        generation: request.generation,
        id: request.id,
        kind: "disposed",
      });
      scope.close();
      return;
    }
    if (busy || request.id <= lastId)
      throw new Error("Worker requests must be single-flight and ordered");
    busy = true;
    lastId = request.id;
    void execute(request)
      .catch((error: unknown) => fail(request, error))
      .finally(() => {
        busy = false;
      });
  } catch (error) {
    if (maybeRequest !== undefined) fail(maybeRequest, error);
    else {
      release();
      throw error;
    }
  }
};

function fail(request: WorkerRequest, cause: unknown): void {
  try {
    release();
  } catch (cleanupError) {
    cause = new Error("Worker operation and cleanup failed", {
      cause: new AggregateError([cause, cleanupError]),
    });
  }
  const error = cause instanceof Error ? cause : new Error(String(cause));
  scope.postMessage({
    version: 1,
    generation: request.generation,
    id: request.id,
    kind: "error",
    error: {
      name: error.name.slice(0, 2048),
      message: error.message.slice(0, 2048),
    },
  });
  scope.close();
}
