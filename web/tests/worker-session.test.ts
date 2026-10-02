import { describe, expect, it } from "vitest";
import {
  createWorkerSession,
  MAX_PENDING_WORKER_REQUESTS,
  type WorkerPort,
} from "../src/physics/worker-session";
import type { WorkerRequest } from "../src/physics/worker-messages";

class FakeWorker implements WorkerPort {
  readonly sent: WorkerRequest[] = [];
  terminated = 0;
  onmessage: WorkerPort["onmessage"] = null;
  onerror: WorkerPort["onerror"] = null;
  onmessageerror: WorkerPort["onmessageerror"] = null;
  postMessage(request: WorkerRequest) {
    this.sent.push(request);
  }
  terminate() {
    this.terminated += 1;
  }
  reply(index: number, payload: object, maybeGeneration?: number) {
    const request = this.sent[index];
    if (request === undefined) throw new Error("Missing fake request");
    this.onmessage?.({
      data: {
        version: 1,
        generation: maybeGeneration ?? request.generation,
        id: request.id,
        ...payload,
      },
    } as MessageEvent<unknown>);
  }
  disposeAck() {
    this.reply(this.sent.length - 1, { kind: "disposed" });
  }
}

const frame = () => ({
  stepIndex: 1,
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
});

async function open() {
  const worker = new FakeWorker();
  const pending = createWorkerSession(1, () => worker);
  worker.reply(0, { kind: "ready" });
  return { worker, session: await pending };
}

describe("FIFO live worker session ownership", () => {
  it("delivers advances and accepted mutations in their original order with one request in flight", async () => {
    // Arrange
    const { worker, session } = await open();

    // Act
    const advanced = session.nextFrame(1);
    const controlled = session.applyControl("flow-direction", "-1");
    const restored = session.restoreAuthoredGravity();

    // Assert
    expect(worker.sent.map((request) => request.op)).toEqual([
      "init",
      "advance",
    ]);
    worker.reply(1, {
      kind: "frame",
      frame: frame(),
      ran: 1,
      timing: { advanceMs: 2, captureMs: 0.1, parseMs: 0.1 },
    });
    await advanced;
    expect(worker.sent.map((request) => request.op)).toEqual([
      "init",
      "advance",
      "control",
    ]);
    worker.reply(2, { kind: "result", value: false });
    expect(await controlled).toBe(false);
    expect(worker.sent[3]?.op).toBe("restore-gravity");
    worker.reply(3, { kind: "result", value: null });
    await restored;
    expect(session.maybeLastTiming?.advanceMs).toBe(2);
    session.dispose();
    worker.disposeAck();
    expect(worker.terminated).toBe(1);
  });

  it("rejects pending work immediately on disposal and ignores late snapshots", async () => {
    // Arrange
    const { worker, session } = await open();
    const result = session.nextFrame().catch((error: unknown) => error);
    const mutation = session
      .applyControl("flow-rate", "0")
      .catch((error: unknown) => error);

    // Act
    session.dispose();
    session.dispose();
    worker.reply(1, {
      kind: "frame",
      frame: frame(),
      ran: 1,
      timing: { advanceMs: 1, captureMs: 0, parseMs: 0 },
    });
    worker.disposeAck();

    // Assert
    expect(await result).toBeInstanceOf(Error);
    expect(await mutation).toBeInstanceOf(Error);
    expect(worker.sent.map((request) => request.op)).toEqual([
      "init",
      "advance",
      "dispose",
    ]);
    await expect(session.nextFrame()).rejects.toThrow("disposed");
    expect(worker.terminated).toBe(1);
  });

  it("rejects new work before acceptance when the bounded FIFO queue is full", async () => {
    // Arrange
    const { worker, session } = await open();
    const pending = Array.from(
      { length: MAX_PENDING_WORKER_REQUESTS },
      (_, index) =>
        session.setGravity(0, -index).catch((error: unknown) => error),
    );

    // Act / Assert
    await expect(session.setGravity(0, -10)).rejects.toThrow("not accepted");
    expect(worker.sent).toHaveLength(2);
    session.dispose();
    worker.disposeAck();
    await Promise.all(pending);
    expect(worker.terminated).toBe(1);
  });

  it("cancels a worker still initializing when its generation is abandoned", async () => {
    // Arrange
    const worker = new FakeWorker(),
      abort = new AbortController();
    const result = createWorkerSession(1, () => worker, abort.signal).catch(
      (error: unknown) => error,
    );

    // Act
    abort.abort();
    worker.disposeAck();

    // Assert
    expect(await result).toBeInstanceOf(Error);
    expect(worker.sent.map((request) => request.op)).toEqual([
      "init",
      "dispose",
    ]);
    expect(worker.terminated).toBe(1);
  });

  it("surfaces a malformed reply and releases all pending ownership", async () => {
    // Arrange
    const { worker, session } = await open();
    const result = session.nextFrame().catch((error: unknown) => error);

    // Act
    worker.reply(1, {
      kind: "frame",
      frame: { ...frame(), particleCount: 1 },
      ran: 1,
      timing: { advanceMs: 1, captureMs: 0, parseMs: 0 },
    });

    // Assert
    expect(await result).toBeInstanceOf(Error);
    expect(worker.terminated).toBe(1);
    await expect(session.applyControl("flow-rate", "0")).rejects.toThrow(
      "disposed",
    );
  });

  it("rejects creation when ready is followed by abandonment before the awaiting factory resumes", async () => {
    // Arrange
    const worker = new FakeWorker(),
      abort = new AbortController();
    const result = createWorkerSession(1, () => worker, abort.signal).catch(
      (error: unknown) => error,
    );

    // Act
    worker.reply(0, { kind: "ready" });
    abort.abort();
    worker.disposeAck();

    // Assert
    expect(await result).toBeInstanceOf(Error);
    expect(worker.terminated).toBe(1);
  });
});
