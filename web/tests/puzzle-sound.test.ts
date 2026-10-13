import { describe, expect, it, vi } from "vitest";

import { NO_SOUND_SIGNALS, type PresentedSoundSignals } from "../src/audio/sound-signals";
import type { PuzzleFrameStatus, RenderFrame } from "../src/physics/frame";
import {
  SOUND_CONTROL_SELECTOR,
  createDisabledPuzzleSound,
  createPuzzleSound,
  type PuzzleSound,
  type SoundEventTarget,
} from "../src/player/puzzle-sound";
import { SOUND_STORAGE_KEY, createSoundPreferenceMemory } from "../src/player/sound-preference";
import { EMPTY_EDITOR } from "../src/puzzle/editor";
import type { PuzzleState } from "../src/puzzle/reduce-puzzle";
import { createFakeAudio, type FakeAudio, type ResumeBehavior } from "./sound-fake-context";

const GOAL_TARGET = 540;
const SPLASH: PresentedSoundSignals = {
  steps: 1,
  splashContacts: 20,
  impactCount: 0,
  impactImpulse: 0,
};

type FakeEvent = {
  readonly type: string;
  readonly isTrusted: boolean;
  readonly pointerType?: string;
  readonly key?: string;
  readonly target?: unknown;
};

type FakeEvents = SoundEventTarget & {
  hidden: boolean;
  readonly dispatch: (event: FakeEvent) => void;
  readonly listenerCount: () => number;
  readonly options: AddEventListenerOptions[];
};

function fakeEvents(): FakeEvents {
  const listeners = new Map<string, ((event: Event) => void)[]>();
  const options: AddEventListenerOptions[] = [];
  return {
    hidden: false,
    options,
    addEventListener(type, listener, maybeOptions) {
      listeners.set(type, [...(listeners.get(type) ?? []), listener]);
      options.push(maybeOptions ?? {});
    },
    removeEventListener(type, listener) {
      listeners.set(
        type,
        (listeners.get(type) ?? []).filter((entry) => entry !== listener),
      );
    },
    dispatch(event) {
      for (const listener of listeners.get(event.type) ?? []) {
        listener(event as unknown as Event);
      }
    },
    listenerCount: () => [...listeners.values()].reduce((sum, entries) => sum + entries.length, 0),
  };
}

function fakeStorage(initial: string | undefined) {
  const values = new Map<string, string>();
  if (initial !== undefined) {
    values.set(SOUND_STORAGE_KEY, initial);
  }
  return {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => {
      values.set(key, value);
    },
    stored: () => values.get(SOUND_STORAGE_KEY),
  };
}

type Harness = {
  readonly sound: PuzzleSound;
  readonly audio: FakeAudio;
  readonly events: FakeEvents;
  readonly storage: ReturnType<typeof fakeStorage>;
  readonly logError: ReturnType<typeof vi.fn>;
};

function harness(
  options: {
    readonly stored?: string;
    readonly unavailable?: boolean;
    readonly throwOnConstruct?: boolean;
    readonly resume?: ResumeBehavior;
  } = {},
): Harness {
  const audio = createFakeAudio({
    ...(options.throwOnConstruct === undefined
      ? {}
      : { throwOnConstruct: options.throwOnConstruct }),
    ...(options.resume === undefined ? {} : { resume: options.resume }),
  });
  const events = fakeEvents();
  const storage = fakeStorage(options.stored);
  const logError = vi.fn();
  const sound = createPuzzleSound({
    preference: createSoundPreferenceMemory(() => storage),
    maybeCreateContext: options.unavailable === true ? undefined : audio.factory,
    events,
    logError,
  });
  return { sound, audio, events, storage, logError };
}

function status(goalCount: number, outcome: "running" | "won" = "running"): PuzzleFrameStatus {
  const base = {
    goalCount,
    goalTarget: GOAL_TARGET,
    goalFull: 900,
    stepLimit: 720,
    peakHeldCount: 0,
    brimCount: 810,
  };
  return outcome === "won"
    ? { ...base, outcome: "won", maybeOutcomeStep: 100 }
    : { ...base, outcome: "running" };
}

function frame(stepIndex: number, frameStatus: PuzzleFrameStatus): RenderFrame {
  return {
    stepIndex,
    particleCount: 0,
    rigidShapeCount: 0,
    maxSpeed: 2,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array(),
    particleColors: new Uint8Array(),
    particleRadii: new Float32Array(),
    rigidSegments: new Float32Array(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
    maybePuzzle: frameStatus,
  };
}

function running(stepIndex: number, frameStatus: PuzzleFrameStatus): PuzzleState {
  return { kind: "running", stepIndex, status: frameStatus, editor: EMPTY_EDITOR, speed: 1 };
}

function building(): PuzzleState {
  return { kind: "building", status: status(0), editor: EMPTY_EDITOR, speed: 1 };
}

/** Presents one running frame at `stepIndex` with `goalCount` and the given signals. */
function presentRunning(
  sound: PuzzleSound,
  stepIndex: number,
  goalCount: number,
  signals: PresentedSoundSignals = NO_SOUND_SIGNALS,
  outcome: "running" | "won" = "running",
): void {
  const frameStatus = status(goalCount, outcome);
  sound.present(frame(stepIndex, frameStatus), signals, running(stepIndex, frameStatus));
}

const trustedTouchUp: FakeEvent = { type: "pointerup", isTrusted: true, pointerType: "touch" };

function constructions(audio: FakeAudio): number {
  return audio.log.filter((entry) => entry === "construct").length;
}

describe("createPuzzleSound state", () => {
  it("shows a stored on as enabled without constructing a context", () => {
    // Act
    const { sound, audio } = harness({ stored: "on" });

    // Assert
    expect(sound.enabled()).toBe(true);
    expect(sound.available()).toBe(true);
    expect(constructions(audio)).toBe(0);
  });

  it("stays off and writes nothing when Web Audio is missing", () => {
    // Arrange
    const { sound, storage } = harness({ unavailable: true });

    // Act
    sound.setEnabled(true);

    // Assert
    expect(sound.available()).toBe(false);
    expect(sound.enabled()).toBe(false);
    expect(storage.stored()).toBeUndefined();
  });

  it("constructs, resumes and stores on when turned on", () => {
    // Arrange
    const { sound, audio, storage } = harness();

    // Act
    sound.setEnabled(true);

    // Assert
    expect(constructions(audio)).toBe(1);
    expect(audio.context().resumeCalls).toBe(1);
    expect(sound.enabled()).toBe(true);
    expect(storage.stored()).toBe("on");
  });

  it("becomes unavailable and logs when construction throws", () => {
    // Arrange
    const { sound, storage, logError } = harness({ throwOnConstruct: true });

    // Act
    sound.setEnabled(true);

    // Assert
    expect(sound.available()).toBe(false);
    expect(sound.enabled()).toBe(false);
    expect(storage.stored()).not.toBe("on");
    expect(logError).toHaveBeenCalled();
  });

  it("stays off and failed without storing on when resume throws synchronously", () => {
    // Arrange
    const { sound, storage, logError } = harness({ resume: "throw" });

    // Act
    sound.setEnabled(true);

    // Assert
    expect(sound.enabled()).toBe(false);
    expect(sound.failed()).toBe(true);
    expect(sound.available()).toBe(true);
    expect(storage.stored()).toBeUndefined();
    expect(logError).toHaveBeenCalled();
  });

  it("stores off, stops voices and suspends when turned off", () => {
    // Arrange
    const { sound, audio, storage } = harness();
    sound.setEnabled(true);
    presentRunning(sound, 1, 0);
    presentRunning(sound, 2, 12);
    const context = audio.context();
    context.log.length = 0;

    // Act
    sound.setEnabled(false);

    // Assert
    expect(storage.stored()).toBe("off");
    expect(sound.enabled()).toBe(false);
    expect(context.log).toEqual(expect.arrayContaining(["stop:oscillator", "suspend"]));
    expect(context.log.indexOf("stop:oscillator")).toBeLessThan(context.log.indexOf("suspend"));
  });
});

describe("first-gesture unlock", () => {
  it("ignores an untrusted pointerup", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();

    // Act
    events.dispatch({ type: "pointerup", isTrusted: false, pointerType: "touch" });

    // Assert
    expect(constructions(audio)).toBe(0);
  });

  it("ignores a touch pointerdown", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();

    // Act
    events.dispatch({ type: "pointerdown", isTrusted: true, pointerType: "touch" });

    // Assert
    expect(constructions(audio)).toBe(0);
  });

  it("unlocks on a trusted touch pointerup", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();

    // Act
    events.dispatch(trustedTouchUp);

    // Assert
    expect(constructions(audio)).toBe(1);
    expect(audio.context().resumeCalls).toBe(1);
  });

  it("unlocks on a trusted mouse pointerdown", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();

    // Act
    events.dispatch({ type: "pointerdown", isTrusted: true, pointerType: "mouse" });

    // Assert
    expect(constructions(audio)).toBe(1);
  });

  it("unlocks on a trusted keydown other than Escape", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();

    // Act
    events.dispatch({ type: "keydown", isTrusted: true, key: "Escape" });
    const afterEscape = constructions(audio);
    events.dispatch({ type: "keydown", isTrusted: true, key: "a" });

    // Assert
    expect(afterEscape).toBe(0);
    expect(constructions(audio)).toBe(1);
  });

  it("ignores a gesture on a sound control", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();
    const closest = vi.fn((selector: string) => (selector === SOUND_CONTROL_SELECTOR ? {} : null));

    // Act
    events.dispatch({ ...trustedTouchUp, target: { closest } });

    // Assert
    expect(closest).toHaveBeenCalledWith(SOUND_CONTROL_SELECTOR);
    expect(constructions(audio)).toBe(0);
  });

  it("does nothing while sound is off", () => {
    // Arrange
    const { sound, audio, events } = harness();
    sound.attach();

    // Act
    events.dispatch(trustedTouchUp);

    // Assert
    expect(constructions(audio)).toBe(0);
  });

  it("registers passive capture listeners once even when attached twice", () => {
    // Arrange
    const { sound, events } = harness({ stored: "on" });

    // Act
    sound.attach();
    sound.attach();

    // Assert
    expect(events.listenerCount()).toBe(4);
    expect(events.options.slice(0, 3)).toEqual([
      { capture: true, passive: true },
      { capture: true, passive: true },
      { capture: true, passive: true },
    ]);
  });
});

describe("visibility", () => {
  it("stops voices and suspends when hidden", () => {
    // Arrange
    const { sound, audio, events } = harness();
    sound.attach();
    sound.setEnabled(true);
    presentRunning(sound, 1, 0);
    presentRunning(sound, 2, 12);
    const context = audio.context();
    context.log.length = 0;

    // Act
    events.hidden = true;
    events.dispatch({ type: "visibilitychange", isTrusted: true });

    // Assert
    expect(context.log).toEqual(["stop:oscillator", "disconnect:gain", "suspend"]);
  });

  it("resumes when visible with sound on and a context", () => {
    // Arrange
    const { sound, audio, events } = harness();
    sound.attach();
    sound.setEnabled(true);
    events.hidden = true;
    events.dispatch({ type: "visibilitychange", isTrusted: true });
    const resumesBefore = audio.context().resumeCalls;

    // Act
    events.hidden = false;
    events.dispatch({ type: "visibilitychange", isTrusted: true });

    // Assert
    expect(audio.context().resumeCalls).toBe(resumesBefore + 1);
  });

  it("resumes on a later gesture while a resume from visibility is still pending", () => {
    // Arrange
    const { sound, audio, events } = harness({ resume: "pending" });
    sound.attach();
    sound.setEnabled(true);
    events.hidden = true;
    events.dispatch({ type: "visibilitychange", isTrusted: true });
    events.hidden = false;
    events.dispatch({ type: "visibilitychange", isTrusted: true });
    const resumesBefore = audio.context().resumeCalls;

    // Act
    events.dispatch(trustedTouchUp);

    // Assert
    expect(audio.context().resumeCalls).toBe(resumesBefore + 1);
  });

  it("does not resume when visible with sound off", () => {
    // Arrange
    const { sound, audio, events } = harness();
    sound.attach();
    sound.setEnabled(true);
    sound.setEnabled(false);
    const resumesBefore = audio.context().resumeCalls;

    // Act
    events.hidden = false;
    events.dispatch({ type: "visibilitychange", isTrusted: true });

    // Assert
    expect(audio.context().resumeCalls).toBe(resumesBefore);
  });
});

describe("leave", () => {
  it("removes every listener, stops voices and suspends", () => {
    // Arrange
    const { sound, audio, events } = harness();
    sound.attach();
    sound.setEnabled(true);
    sound.setEnabled(false);
    sound.setEnabled(true);
    const context = audio.context();
    const resumesBefore = context.resumeCalls;
    context.log.length = 0;

    // Act
    sound.leave();
    events.hidden = false;
    events.dispatch({ type: "visibilitychange", isTrusted: true });
    events.dispatch(trustedTouchUp);

    // Assert
    expect(events.listenerCount()).toBe(0);
    expect(context.log).toContain("suspend");
    expect(context.resumeCalls).toBe(resumesBefore);
  });
});

describe("present", () => {
  it("starts no voice while building", () => {
    // Arrange
    const { sound, audio } = harness();
    sound.setEnabled(true);

    // Act
    sound.present(frame(0, status(20)), SPLASH, building());

    // Assert
    expect(audio.context().sources()).toHaveLength(0);
  });

  it("starts a plink when the goal count rises during a run", () => {
    // Arrange
    const { sound, audio } = harness();
    sound.setEnabled(true);
    presentRunning(sound, 1, 0);

    // Act
    presentRunning(sound, 2, 12);

    // Assert
    const oscillators = audio.context().ofKind("oscillator");
    expect(oscillators).toHaveLength(1);
    expect(oscillators[0]?.type).toBe("sine");
  });

  it("plays no chime after a mid-run unlock when the win was already tracked", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();
    presentRunning(sound, 1, 0);
    presentRunning(sound, 2, 600, NO_SOUND_SIGNALS, "won");
    const constructedBefore = constructions(audio);
    events.dispatch(trustedTouchUp);

    // Act
    presentRunning(sound, 3, 600, NO_SOUND_SIGNALS, "won");

    // Assert
    expect(constructedBefore).toBe(0);
    expect(audio.context().sources()).toHaveLength(0);
  });

  it("keeps clean splash memory across a mid-run unlock", () => {
    // Arrange
    const { sound, audio, events } = harness({ stored: "on" });
    sound.attach();
    presentRunning(sound, 1, 0, SPLASH);
    presentRunning(sound, 2, 6, SPLASH);
    presentRunning(sound, 3, 12, SPLASH);
    events.dispatch(trustedTouchUp);

    // Act
    presentRunning(sound, 4, 12, SPLASH);

    // Assert
    const context = audio.context();
    expect(context.ofKind("bufferSource")).toHaveLength(1);
    const oscillators = context.ofKind("oscillator");
    expect(oscillators.map((node) => node.type)).toEqual(["triangle"]);
  });

  it("logs and turns sound off in memory when the engine throws", () => {
    // Arrange
    const { sound, audio, storage, logError } = harness();
    sound.setEnabled(true);
    audio.context().createOscillator = () => {
      throw new Error("node failed");
    };
    presentRunning(sound, 1, 0);

    // Act
    const act = () => presentRunning(sound, 2, 12);

    // Assert
    expect(act).not.toThrow();
    expect(logError).toHaveBeenCalled();
    expect(sound.enabled()).toBe(false);
    expect(sound.failed()).toBe(true);
    expect(storage.stored()).toBe("on");
  });

  it("clears the failure when sound is turned on again", () => {
    // Arrange
    const { sound, audio } = harness();
    sound.setEnabled(true);
    audio.context().createOscillator = () => {
      throw new Error("node failed");
    };
    presentRunning(sound, 1, 0);
    presentRunning(sound, 2, 12);

    // Act
    sound.setEnabled(true);

    // Assert
    expect(sound.failed()).toBe(false);
    expect(sound.enabled()).toBe(true);
  });
});

describe("createDisabledPuzzleSound", () => {
  it("is unavailable and every method is a no-op", () => {
    // Arrange
    const sound = createDisabledPuzzleSound();

    // Act
    const act = () => {
      sound.setEnabled(true);
      sound.attach();
      sound.present(frame(1, status(0)), NO_SOUND_SIGNALS, running(1, status(0)));
      sound.stopVoices();
      sound.pause();
      sound.leave();
    };

    // Assert
    expect(act).not.toThrow();
    expect(sound.available()).toBe(false);
    expect(sound.enabled()).toBe(false);
    expect(sound.failed()).toBe(false);
  });
});
