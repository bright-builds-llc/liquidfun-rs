import { parsePointerKind } from "../input/pointer";
import { parseRenderFrame, type RenderFrame } from "./frame";

export type WorkerEnvelope = {
  readonly version: 1;
  readonly generation: number;
  readonly id: number;
};
export type WorkerOperation =
  | { readonly op: "init" }
  | {
      readonly op: "advance";
      readonly count: number;
      readonly budgeted: boolean;
    }
  | { readonly op: "advance-only"; readonly count: number }
  | { readonly op: "capture" }
  | { readonly op: "control"; readonly name: string; readonly value: string }
  | { readonly op: "action"; readonly name: string }
  | {
      readonly op: "pointer";
      readonly kind: string;
      readonly x: number;
      readonly y: number;
    }
  | { readonly op: "gravity"; readonly x: number; readonly y: number }
  | { readonly op: "restore-gravity" }
  | { readonly op: "dispose" };
export type WorkerRequest = WorkerEnvelope & WorkerOperation;
export type WorkerComputeTiming = {
  readonly advanceMs: number;
  readonly captureMs: number;
  readonly parseMs: number;
};
export type WorkerReply = WorkerEnvelope &
  (
    | { readonly kind: "ready" }
    | {
        readonly kind: "result";
        readonly value: boolean | null;
        readonly maybeGravityAccepted?: boolean;
      }
    | { readonly kind: "disposed" }
    | {
        readonly kind: "frame";
        readonly frame: RenderFrame;
        readonly ran: number;
        readonly timing: WorkerComputeTiming;
      }
    | {
        readonly kind: "error";
        readonly error: { readonly name: string; readonly message: string };
      }
  );

function record(value: unknown): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value))
    throw new Error("Invalid live worker message");
  return value as Record<string, unknown>;
}

function safeInteger(
  value: unknown,
  maximum = Number.MAX_SAFE_INTEGER,
): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < 1 ||
    value > maximum
  )
    throw new Error("Invalid live worker integer");
  return value;
}

function finite(value: unknown): number {
  if (typeof value !== "number" || !Number.isFinite(value))
    throw new Error("Invalid live worker numeric value");
  return value;
}

function text(value: unknown): string {
  if (typeof value !== "string" || !value.length || value.length > 2048)
    throw new Error("Invalid live worker string");
  return value;
}

function envelope(value: Record<string, unknown>): WorkerEnvelope {
  if (value.version !== 1) throw new Error("Unsupported live worker protocol");
  return {
    version: 1,
    generation: safeInteger(value.generation),
    id: safeInteger(value.id),
  };
}

/** Validates the bounded protocol before it can mutate the worker-owned world. */
export function parseWorkerRequest(value: unknown): WorkerRequest {
  const message = record(value),
    header = envelope(message);
  switch (message.op) {
    case "init":
    case "capture":
    case "restore-gravity":
    case "dispose":
      return { ...header, op: message.op };
    case "advance-only":
      return {
        ...header,
        op: message.op,
        count: safeInteger(message.count, 4),
      };
    case "advance":
      if (typeof message.budgeted !== "boolean")
        throw new Error("Invalid advance budget flag");
      return {
        ...header,
        op: message.op,
        count: safeInteger(message.count, 4),
        budgeted: message.budgeted,
      };
    case "control":
      return {
        ...header,
        op: message.op,
        name: text(message.name),
        value: text(message.value),
      };
    case "action":
      return { ...header, op: message.op, name: text(message.name) };
    case "gravity":
      return {
        ...header,
        op: message.op,
        x: finite(message.x),
        y: finite(message.y),
      };
    case "pointer": {
      const kind = text(message.kind);
      if (parsePointerKind(kind) === undefined)
        throw new Error("Invalid worker pointer kind");
      return {
        ...header,
        op: message.op,
        kind,
        x: finite(message.x),
        y: finite(message.y),
      };
    }
    default:
      throw new Error("Unsupported live worker operation");
  }
}

/** Uses the same strict frame parser as direct WASM without copying transferred lanes again. */
export function validateTransferredFrame(value: unknown): RenderFrame {
  const frame = record(value);
  const parsed = parseRenderFrame({
    stepIndex: () => frame.stepIndex as number,
    particleCount: () => frame.particleCount as number,
    rigidShapeCount: () => frame.rigidShapeCount as number,
    maxSpeed: () => frame.maxSpeed as number,
    stuckCandidateCount: () => frame.stuckCandidateCount as number,
    bodyContactCount: () => frame.bodyContactCount as number,
    particlePositions: () => frame.particlePositions as Float32Array,
    particleColors: () => frame.particleColors as Uint8Array,
    particleRadii: () => frame.particleRadii as Float32Array,
    rigidSegments: () => frame.rigidSegments as Float32Array,
    rigidCircles: () => frame.rigidCircles as Float32Array,
    circleLabels: () => frame.circleLabels as readonly string[],
    free: () => undefined,
  });
  frameTransferables(parsed);
  return parsed;
}

export function parseWorkerReply(value: unknown): WorkerReply {
  const message = record(value),
    header = envelope(message);
  switch (message.kind) {
    case "ready":
    case "disposed":
      return { ...header, kind: message.kind };
    case "result":
      if (message.value !== null && typeof message.value !== "boolean")
        throw new Error("Invalid worker mutation result");
      if (
        message.maybeGravityAccepted !== undefined &&
        typeof message.maybeGravityAccepted !== "boolean"
      )
        throw new Error("Invalid gravity sample status");
      return {
        ...header,
        kind: message.kind,
        value: message.value,
        ...(message.maybeGravityAccepted === undefined
          ? {}
          : { maybeGravityAccepted: message.maybeGravityAccepted as boolean }),
      };
    case "frame": {
      const timing = record(message.timing);
      const metrics = {
        advanceMs: finite(timing.advanceMs),
        captureMs: finite(timing.captureMs),
        parseMs: finite(timing.parseMs),
      };
      if (Object.values(metrics).some((number) => number < 0))
        throw new Error("Invalid worker timing");
      const ran = message.ran === 0 ? 0 : safeInteger(message.ran, 4);
      return {
        ...header,
        kind: message.kind,
        frame: validateTransferredFrame(message.frame),
        ran,
        timing: metrics,
      };
    }
    case "error": {
      const error = record(message.error);
      return {
        ...header,
        kind: message.kind,
        error: { name: text(error.name), message: text(error.message) },
      };
    }
    default:
      throw new Error("Unsupported live worker reply");
  }
}

/** Live WASM memory is never transferable; every lane must have its own ordinary buffer. */
export function frameTransferables(frame: RenderFrame): ArrayBuffer[] {
  const lanes = [
    frame.particlePositions,
    frame.particleColors,
    frame.particleRadii,
    frame.rigidSegments,
    frame.rigidCircles,
  ];
  if (
    lanes.some(
      (lane) =>
        !(lane.buffer instanceof ArrayBuffer) ||
        lane.byteOffset !== 0 ||
        lane.buffer.byteLength !== lane.byteLength,
    )
  ) {
    throw new Error(
      "Live worker frame lanes must own tightly bounded transferable ArrayBuffers",
    );
  }
  const buffers = lanes.map((lane) => lane.buffer);
  return [...new Set(buffers as ArrayBuffer[])];
}
