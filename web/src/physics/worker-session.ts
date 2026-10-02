import { parsePointerKind } from "../input/pointer";
import type {
  LiveSceneSession,
  WorkerAdvance,
  WorkerTiming,
} from "./live-session";
import {
  parseWorkerReply,
  parseWorkerRequest,
  type WorkerOperation,
  type WorkerReply,
  type WorkerRequest,
} from "./worker-messages";

export type WorkerPort = {
  postMessage(message: WorkerRequest): void;
  terminate(): void;
  onmessage: ((event: MessageEvent<unknown>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  onmessageerror: ((event: MessageEvent<unknown>) => void) | null;
};

type Pending = {
  readonly request: WorkerRequest;
  readonly resolve: (reply: WorkerReply) => void;
  readonly reject: (error: Error) => void;
  maybeSentAt: number | undefined;
};

export const MAX_PENDING_WORKER_REQUESTS = 64;
const DISPOSED = "Rust/WASM live worker session is disposed";

/** Creates one terminal, FIFO owner; no live WASM memory crosses the worker boundary. */
export async function createWorkerSession(
  generation: number,
  maybeFactory?: () => WorkerPort,
  maybeAbortSignal?: AbortSignal,
): Promise<LiveSceneSession> {
  if (maybeAbortSignal?.aborted) throw new Error(DISPOSED);
  const worker =
    maybeFactory?.() ??
    new Worker(new URL("./simulation-worker.ts", import.meta.url), {
      type: "module",
    });
  let nextId = 1,
    completedId = 0;
  let closed = false,
    terminated = false;
  let maybeInFlight: Pending | undefined;
  const queue: Pending[] = [];
  let maybeLastTiming: WorkerTiming | undefined;
  let maybeDisposeId: number | undefined;
  let maybeTerminationTimer: ReturnType<typeof setTimeout> | undefined;

  function terminate(): void {
    if (terminated) return;
    terminated = true;
    maybeAbortSignal?.removeEventListener("abort", dispose);
    if (maybeTerminationTimer !== undefined)
      clearTimeout(maybeTerminationTimer);
    worker.onmessage = null;
    worker.onerror = null;
    worker.onmessageerror = null;
    worker.terminate();
  }

  function rejectPending(error: Error): void {
    maybeInFlight?.reject(error);
    maybeInFlight = undefined;
    for (const pending of queue.splice(0)) pending.reject(error);
  }

  function abort(error: Error): void {
    if (closed) return;
    closed = true;
    rejectPending(error);
    terminate();
  }

  function pump(): void {
    if (closed || maybeInFlight !== undefined) return;
    const pending = queue.shift();
    if (pending === undefined) return;
    maybeInFlight = pending;
    pending.maybeSentAt = performance.now();
    try {
      worker.postMessage(pending.request);
    } catch (error) {
      abort(error instanceof Error ? error : new Error(String(error)));
    }
  }

  function send(operation: WorkerOperation): Promise<WorkerReply> {
    if (closed) return Promise.reject(new Error(DISPOSED));
    if (
      queue.length + Number(maybeInFlight !== undefined) >=
      MAX_PENDING_WORKER_REQUESTS
    ) {
      return Promise.reject(
        new Error(
          "Live worker request queue is full; operation was not accepted",
        ),
      );
    }
    let request: WorkerRequest;
    try {
      request = parseWorkerRequest({
        version: 1,
        generation,
        id: nextId++,
        ...operation,
      });
    } catch (error) {
      const failure = error instanceof Error ? error : new Error(String(error));
      abort(failure);
      return Promise.reject(failure);
    }
    return new Promise((resolve, reject) => {
      queue.push({ request, resolve, reject, maybeSentAt: undefined });
      pump();
    });
  }

  function replyMatches(reply: WorkerReply, pending: Pending): boolean {
    const op = pending.request;
    if (reply.kind === "error") return true;
    if (op.op === "init") return reply.kind === "ready";
    if (op.op === "advance")
      return (
        reply.kind === "frame" &&
        reply.ran >= 1 &&
        reply.ran <= op.count &&
        (op.budgeted || reply.ran === op.count)
      );
    if (op.op === "capture") return reply.kind === "frame" && reply.ran === 0;
    if (op.op === "control")
      return reply.kind === "result" && typeof reply.value === "boolean";
    return reply.kind === "result" && reply.value === null;
  }

  worker.onmessage = (event: MessageEvent<unknown>) => {
    if (closed) {
      const data: unknown = event.data;
      if (
        data !== null &&
        typeof data === "object" &&
        "generation" in data &&
        data.generation === generation &&
        "id" in data &&
        data.id === maybeDisposeId &&
        "kind" in data &&
        data.kind === "disposed"
      )
        terminate();
      return;
    }
    try {
      const reply = parseWorkerReply(event.data);
      if (reply.generation !== generation || reply.id <= completedId) return;
      const pending = maybeInFlight;
      if (
        pending === undefined ||
        reply.id !== pending.request.id ||
        !replyMatches(reply, pending)
      )
        throw new Error(
          "Unexpected live worker response identity or operation",
        );
      if (reply.kind === "error") {
        const failure = new Error(reply.error.message);
        failure.name = reply.error.name;
        abort(failure);
        return;
      }
      if (reply.kind === "frame")
        maybeLastTiming = {
          ...reply.timing,
          roundTripMs:
            performance.now() - (pending.maybeSentAt ?? performance.now()),
        };
      completedId = reply.id;
      maybeInFlight = undefined;
      pending.resolve(reply);
      pump();
    } catch (error) {
      abort(error instanceof Error ? error : new Error(String(error)));
    }
  };
  worker.onerror = (event) =>
    abort(new Error(event.message || "Live simulation worker failed"));
  worker.onmessageerror = () =>
    abort(new Error("Live simulation worker message could not be decoded"));

  function dispose(): void {
    if (closed) return;
    closed = true;
    rejectPending(new Error(DISPOSED));
    maybeAbortSignal?.removeEventListener("abort", dispose);
    maybeDisposeId = nextId++;
    try {
      worker.postMessage({
        version: 1,
        generation,
        id: maybeDisposeId,
        op: "dispose",
      });
    } catch {
      terminate();
      return;
    }
    // Normal disposal frees the Rust owner before acknowledgment; stuck workers are bounded.
    maybeTerminationTimer = setTimeout(terminate, 1000);
  }

  async function frame(operation: WorkerOperation): Promise<WorkerAdvance> {
    const reply = await send(operation);
    if (reply.kind !== "frame" || maybeLastTiming === undefined)
      throw new Error("Live worker did not provide a validated frame");
    return { frame: reply.frame, ran: reply.ran, timing: maybeLastTiming };
  }

  maybeAbortSignal?.addEventListener("abort", dispose, { once: true });
  await send({ op: "init" });
  if (closed || maybeAbortSignal?.aborted) throw new Error(DISPOSED);
  return {
    backend: "worker",
    get maybeLastTiming() {
      return maybeLastTiming;
    },
    nextFrame: async (count = 1) =>
      (await frame({ op: "advance", count, budgeted: false })).frame,
    advanceBudgeted: (count) => frame({ op: "advance", count, budgeted: true }),
    advanceOnly: async (count) => {
      await send({ op: "advance-only", count });
    },
    captureFrame: async () => (await frame({ op: "capture" })).frame,
    applyControl: async (name, value) => {
      const reply = await send({ op: "control", name, value });
      if (reply.kind !== "result" || typeof reply.value !== "boolean")
        throw new Error("Invalid worker control result");
      return reply.value;
    },
    applyAction: async (name) => {
      await send({ op: "action", name });
    },
    pointerAction: async (kind, x, y) => {
      if (
        parsePointerKind(kind) === undefined ||
        !Number.isFinite(x) ||
        !Number.isFinite(y)
      )
        return;
      await send({ op: "pointer", kind, x, y });
    },
    setGravity: async (x, y) => {
      if (!Number.isFinite(x) || !Number.isFinite(y)) return;
      await send({ op: "gravity", x, y });
    },
    restoreAuthoredGravity: async () => {
      await send({ op: "restore-gravity" });
    },
    dispose,
  };
}
