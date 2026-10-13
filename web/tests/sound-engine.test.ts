import { afterEach, describe, expect, it, vi } from "vitest";

import { createSoundEngine, type SoundEngine } from "../src/audio/sound-engine";
import { chimeNotes, impactParams } from "../src/audio/voices";
import {
  RESUME_ERROR,
  allParamCalls,
  createFakeAudio,
  type FakeAudio,
  type FakeContext,
  type ResumeBehavior,
} from "./sound-fake-context";

type Harness = {
  readonly audio: FakeAudio;
  readonly engine: SoundEngine;
  readonly onFailure: ReturnType<typeof vi.fn>;
};

function harness(
  options: { readonly resume?: ResumeBehavior; readonly throwOnConstruct?: boolean } = {},
): Harness {
  const audio = createFakeAudio(options);
  const onFailure = vi.fn();
  const engine = createSoundEngine({ createContext: audio.factory, onFailure });
  return { audio, engine, onFailure };
}

/** A running engine and its context. */
function running(): Harness & { readonly context: FakeContext } {
  const built = harness();
  built.engine.ensureRunning();
  return { ...built, context: built.audio.context() };
}

async function flushPromises(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

/** Six sounding voices: splash, impact and plink at 0, impact and plink at 0.06, splash at 0.1. */
function fillPool(engine: SoundEngine, context: FakeContext): void {
  context.currentTime = 0;
  engine.play([
    { kind: "splash", intensity: 0.5 },
    { kind: "impact", strength: 0.5 },
    { kind: "plink", frequency: 600 },
  ]);
  context.currentTime = 0.06;
  engine.play([
    { kind: "impact", strength: 0.5 },
    { kind: "plink", frequency: 700 },
  ]);
  context.currentTime = 0.1;
  engine.play([{ kind: "splash", intensity: 0.5 }]);
  context.currentTime = 0.115;
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("createSoundEngine before unlock", () => {
  it("never calls the factory on creation", () => {
    // Act
    const { audio, engine } = harness();

    // Assert
    expect(audio.contexts).toHaveLength(0);
    expect(engine.hasContext()).toBe(false);
  });

  it("creates no nodes when playing before ensureRunning", () => {
    // Arrange
    const { audio, engine } = harness();

    // Act
    engine.play([{ kind: "plink", frequency: 600 }]);

    // Assert
    expect(audio.log).toEqual([]);
    expect(engine.maybeNowSeconds()).toBeUndefined();
  });
});

describe("ensureRunning", () => {
  it("constructs the context once and resumes it on first use", () => {
    // Arrange
    const { audio, engine } = harness();

    // Act
    const ok = engine.ensureRunning();

    // Assert
    expect(ok).toBe(true);
    expect(audio.contexts).toHaveLength(1);
    expect(audio.context().resumeCalls).toBe(1);
    expect(engine.isRunning()).toBe(true);
  });

  it("does not construct again on a second call", () => {
    // Arrange
    const { audio, engine } = harness();
    engine.ensureRunning();

    // Act
    engine.ensureRunning();

    // Assert
    expect(audio.log.filter((entry) => entry === "construct")).toHaveLength(1);
  });

  it("does not stack resume calls while one is in flight", () => {
    // Arrange
    const { audio, engine } = harness({ resume: "pending" });
    engine.ensureRunning();

    // Act
    engine.ensureRunning();

    // Assert
    expect(audio.context().resumeCalls).toBe(1);
  });

  it("returns false and logs when construction throws", () => {
    // Arrange
    const errors = vi.spyOn(console, "error").mockImplementation(() => undefined);
    const { engine } = harness({ throwOnConstruct: true });

    // Act
    const ok = engine.ensureRunning();

    // Assert
    expect(ok).toBe(false);
    expect(engine.hasContext()).toBe(false);
    expect(errors).toHaveBeenCalledTimes(1);
  });

  it("reports a rejected resume to onFailure", async () => {
    // Arrange
    const { engine, onFailure } = harness({ resume: "reject" });

    // Act
    engine.ensureRunning();
    await flushPromises();

    // Assert
    expect(onFailure).toHaveBeenCalledWith(RESUME_ERROR);
  });
});

describe("master chain", () => {
  it("routes a 0.5 master gain through one compressor to the destination", () => {
    // Act
    const { context } = running();

    // Assert
    const gains = context.ofKind("gain");
    const compressors = context.ofKind("compressor");
    expect(gains).toHaveLength(1);
    expect(compressors).toHaveLength(1);
    expect(gains[0]?.gain.value).toBe(0.5);
    expect(gains[0]?.connections).toEqual([compressors[0]]);
    expect(compressors[0]?.connections).toEqual([context.destination]);
  });
});

describe("voice synthesis", () => {
  it("shares one noise buffer between splashes through bandpass filters", () => {
    // Arrange
    const { engine, context } = running();

    // Act
    engine.play([{ kind: "splash", intensity: 0.5 }]);
    context.currentTime = 0.1;
    engine.play([{ kind: "splash", intensity: 0.8 }]);

    // Assert
    expect(context.bufferCalls).toBe(1);
    expect(context.ofKind("bufferSource")).toHaveLength(2);
    const filters = context.ofKind("biquad");
    expect(filters).toHaveLength(2);
    expect(filters.every((filter) => filter.type === "bandpass")).toBe(true);
  });

  it("starts one sine oscillator at the plink frequency", () => {
    // Arrange
    const { engine, context } = running();

    // Act
    engine.play([{ kind: "plink", frequency: 784 }]);

    // Assert
    const oscillators = context.ofKind("oscillator");
    expect(oscillators).toHaveLength(1);
    expect(oscillators[0]?.type).toBe("sine");
    expect(oscillators[0]?.frequency.calls[0]).toMatchObject({ value: 784 });
    expect(oscillators[0]?.starts).toEqual([0]);
  });

  it("drops an impact's triangle pitch from startHz to endHz", () => {
    // Arrange
    const { engine, context } = running();
    const params = impactParams(0.5);

    // Act
    engine.play([{ kind: "impact", strength: 0.5 }]);

    // Assert
    const oscillator = context.ofKind("oscillator")[0];
    expect(oscillator?.type).toBe("triangle");
    expect(oscillator?.frequency.calls).toEqual([
      { method: "setValueAtTime", value: params.startHz, time: 0 },
      {
        method: "exponentialRampToValueAtTime",
        value: params.endHz,
        time: params.durationSeconds,
      },
    ]);
  });

  it("starts the chime's three sine notes at their offsets", () => {
    // Arrange
    const { engine, context } = running();
    context.currentTime = 2;

    // Act
    engine.play([{ kind: "chime" }]);

    // Assert
    const oscillators = context.ofKind("oscillator");
    expect(oscillators.map((node) => node.type)).toEqual(["sine", "sine", "sine"]);
    expect(oscillators.map((node) => node.frequency.calls[0]?.value)).toEqual(
      chimeNotes().map((note) => note.frequency),
    );
    expect(oscillators.map((node) => node.starts[0])).toEqual(
      chimeNotes().map((note) => 2 + note.offsetSeconds),
    );
  });

  it("never ramps exponentially to zero", () => {
    // Arrange
    const { engine, context } = running();

    // Act
    engine.play([
      { kind: "impact", strength: 1 },
      { kind: "splash", intensity: 1 },
      { kind: "plink", frequency: 600 },
    ]);
    engine.play([{ kind: "chime" }]);

    // Assert
    const ramps = allParamCalls(context).filter(
      (call) => call.method === "exponentialRampToValueAtTime",
    );
    expect(ramps.length).toBeGreaterThan(0);
    expect(ramps.every((call) => (call.value ?? 0) > 0)).toBe(true);
  });

  it("creates no nodes while the context is suspended", () => {
    // Arrange
    const { audio, engine } = harness({ resume: "pending" });
    engine.ensureRunning();
    const context = audio.context();
    const nodesBefore = context.nodes.length;

    // Act
    engine.play([{ kind: "plink", frequency: 600 }]);

    // Assert
    expect(context.nodes.length).toBe(nodesBefore);
  });

  it("creates no nodes while the context is interrupted", () => {
    // Arrange
    const { engine, context } = running();
    context.state = "interrupted";
    const nodesBefore = context.nodes.length;

    // Act
    engine.play([{ kind: "plink", frequency: 600 }]);

    // Assert
    expect(context.nodes.length).toBe(nodesBefore);
    expect(engine.isRunning()).toBe(false);
  });
});

describe("voice cap", () => {
  it("starts no node for a seventh non-chime voice", () => {
    // Arrange
    const { engine, context } = running();
    fillPool(engine, context);
    const sourcesBefore = context.sources().length;

    // Act
    engine.play([{ kind: "plink", frequency: 900 }]);

    // Assert
    expect(sourcesBefore).toBe(6);
    expect(context.sources()).toHaveLength(6);
  });

  it("stops the three oldest voices when the chime needs room", () => {
    // Arrange
    const { engine, context } = running();
    fillPool(engine, context);
    const [oldestSplash, oldestImpact, oldestPlink, ...newer] = context.sources();

    // Act
    engine.play([{ kind: "chime" }]);

    // Assert
    for (const evicted of [oldestSplash, oldestImpact, oldestPlink]) {
      expect(evicted?.stops.at(-1)).toBe(0.115);
    }
    for (const kept of newer) {
      expect(kept.stops).toHaveLength(1);
    }
    // Four run oscillators plus the three chime notes.
    expect(context.ofKind("oscillator")).toHaveLength(4 + 3);
  });
});

describe("stopAll and suspend", () => {
  it("stops every started source and disconnects its gain", () => {
    // Arrange
    const { engine, context } = running();
    engine.play([
      { kind: "splash", intensity: 0.5 },
      { kind: "plink", frequency: 600 },
    ]);
    const voiceGains = context.ofKind("gain").slice(1);

    // Act
    engine.stopAll();

    // Assert
    expect(context.sources().every((source) => source.stops.length === 2)).toBe(true);
    expect(voiceGains).toHaveLength(2);
    expect(voiceGains.every((gain) => gain.disconnects >= 1)).toBe(true);
    expect(voiceGains.every((gain) => gain.gain.value === 0)).toBe(true);
  });

  it("is safe without a context and twice in a row", () => {
    // Arrange
    const { engine } = harness();

    // Act
    const act = () => {
      engine.stopAll();
      engine.stopAll();
    };

    // Assert
    expect(act).not.toThrow();
  });

  it("stops voices before suspending and never closes", () => {
    // Arrange
    const { engine, context } = running();
    engine.play([{ kind: "plink", frequency: 600 }]);
    context.log.length = 0;

    // Act
    engine.suspend();

    // Assert
    expect(context.log.indexOf("stop:oscillator")).toBeGreaterThanOrEqual(0);
    expect(context.log.indexOf("stop:oscillator")).toBeLessThan(context.log.indexOf("suspend"));
    expect(context.suspendCalls).toBe(1);
    expect(context.closeCalls).toBe(0);
    expect(engine.hasContext()).toBe(true);
  });
});
