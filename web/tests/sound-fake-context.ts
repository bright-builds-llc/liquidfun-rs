/**
 * Recording fake of the browser audio context for node-environment tests
 * (Phase 42). It records constructions, nodes, connections, scheduled
 * parameter calls and start/stop/disconnect calls in one shared event log.
 */

import type { SoundContext, SoundContextFactory } from "../src/audio/sound-engine";

export type ParamCall = {
  readonly method: string;
  readonly value: number | undefined;
  readonly time: number | undefined;
};

export class FakeParam {
  value = 0;
  readonly calls: ParamCall[] = [];

  setValueAtTime(value: number, time: number): this {
    this.calls.push({ method: "setValueAtTime", value, time });
    this.value = value;
    return this;
  }

  linearRampToValueAtTime(value: number, time: number): this {
    this.calls.push({ method: "linearRampToValueAtTime", value, time });
    return this;
  }

  exponentialRampToValueAtTime(value: number, time: number): this {
    this.calls.push({ method: "exponentialRampToValueAtTime", value, time });
    return this;
  }

  setTargetAtTime(value: number, time: number): this {
    this.calls.push({ method: "setTargetAtTime", value, time });
    return this;
  }

  cancelScheduledValues(time: number): this {
    this.calls.push({ method: "cancelScheduledValues", value: undefined, time });
    return this;
  }
}

export type FakeNodeKind =
  "destination" | "gain" | "oscillator" | "biquad" | "bufferSource" | "compressor";

export class FakeNode {
  readonly gain = new FakeParam();
  readonly frequency = new FakeParam();
  readonly Q = new FakeParam();
  type = "";
  buffer: unknown = null;
  onended: (() => void) | null = null;
  readonly connections: FakeNode[] = [];
  readonly starts: number[] = [];
  readonly stops: number[] = [];
  disconnects = 0;

  constructor(
    readonly kind: FakeNodeKind,
    private readonly log: string[],
  ) {}

  connect(target: FakeNode): FakeNode {
    this.connections.push(target);
    return target;
  }

  disconnect(): void {
    this.disconnects += 1;
    this.log.push(`disconnect:${this.kind}`);
  }

  start(when = 0): void {
    this.starts.push(when);
    this.log.push(`start:${this.kind}`);
  }

  stop(when = 0): void {
    this.stops.push(when);
    this.log.push(`stop:${this.kind}`);
  }
}

/** How `resume()` behaves: run at once, stay pending, reject, or throw synchronously. */
export type ResumeBehavior = "run" | "pending" | "reject" | "throw";

export const RESUME_ERROR = new Error("resume rejected");
export const RESUME_THROW_ERROR = new Error("resume threw");

export class FakeContext {
  state: string = "suspended";
  currentTime = 0;
  readonly sampleRate = 48_000;
  readonly nodes: FakeNode[] = [];
  readonly destination: FakeNode;
  resumeCalls = 0;
  suspendCalls = 0;
  closeCalls = 0;
  bufferCalls = 0;

  constructor(
    readonly log: string[],
    public resumeBehavior: ResumeBehavior,
  ) {
    this.destination = new FakeNode("destination", log);
  }

  private node(kind: FakeNodeKind): FakeNode {
    const node = new FakeNode(kind, this.log);
    this.nodes.push(node);
    return node;
  }

  resume(): Promise<void> {
    this.resumeCalls += 1;
    this.log.push("resume");
    if (this.resumeBehavior === "throw") {
      throw RESUME_THROW_ERROR;
    }
    if (this.resumeBehavior === "reject") {
      return Promise.reject(RESUME_ERROR);
    }
    if (this.resumeBehavior === "pending") {
      return new Promise(() => undefined);
    }
    this.state = "running";
    return Promise.resolve();
  }

  suspend(): Promise<void> {
    this.suspendCalls += 1;
    this.log.push("suspend");
    this.state = "suspended";
    return Promise.resolve();
  }

  close(): Promise<void> {
    this.closeCalls += 1;
    return Promise.resolve();
  }

  createGain(): FakeNode {
    return this.node("gain");
  }

  createOscillator(): FakeNode {
    return this.node("oscillator");
  }

  createBiquadFilter(): FakeNode {
    return this.node("biquad");
  }

  createBufferSource(): FakeNode {
    return this.node("bufferSource");
  }

  createDynamicsCompressor(): FakeNode {
    return this.node("compressor");
  }

  createBuffer(channels: number, length: number, sampleRate: number) {
    this.bufferCalls += 1;
    const data = new Float32Array(length);
    return { numberOfChannels: channels, length, sampleRate, getChannelData: () => data };
  }

  ofKind(kind: FakeNodeKind): FakeNode[] {
    return this.nodes.filter((node) => node.kind === kind);
  }

  /** Every node that makes sound: oscillators and buffer sources. */
  sources(): FakeNode[] {
    return this.nodes.filter((node) => node.kind === "oscillator" || node.kind === "bufferSource");
  }
}

export type FakeAudio = {
  readonly factory: SoundContextFactory;
  readonly contexts: FakeContext[];
  readonly log: string[];
  /** The one constructed context; throws when none exists. */
  readonly context: () => FakeContext;
};

export function createFakeAudio(
  options: { readonly resume?: ResumeBehavior; readonly throwOnConstruct?: boolean } = {},
): FakeAudio {
  const contexts: FakeContext[] = [];
  const log: string[] = [];
  const factory: SoundContextFactory = () => {
    log.push("construct");
    if (options.throwOnConstruct === true) {
      throw new Error("construction failed");
    }
    const context = new FakeContext(log, options.resume ?? "run");
    contexts.push(context);
    return context as unknown as SoundContext;
  };
  return {
    factory,
    contexts,
    log,
    context: () => {
      const maybeContext = contexts[0];
      if (maybeContext === undefined) {
        throw new Error("No fake audio context was constructed");
      }
      return maybeContext;
    },
  };
}

/** Every parameter call on every node of the context. */
export function allParamCalls(context: FakeContext): ParamCall[] {
  return context.nodes.flatMap((node) => [
    ...node.gain.calls,
    ...node.frequency.calls,
    ...node.Q.calls,
  ]);
}
