/**
 * Page-session puzzle sound state and its browser glue (Phase 42). One
 * shared on/available/failed state backs the HUD toggle and the drawer
 * checkbox. The browser audio context is created only by the toggle's own
 * handler or by the first trusted activation gesture on a puzzle route
 * (D-04, D-17). This module is a pure consumer of presented frames (D-08):
 * every engine call is guarded, and a failure turns sound off for the
 * session without touching the stored preference or the run (D-16). Time
 * comes only from the engine's audio clock seconds.
 */

import { createSignal, type Accessor } from "solid-js";

import {
  browserContextFactory,
  createSoundEngine,
  type SoundContextFactory,
  type SoundEngine,
} from "../audio/sound-engine";
import type { PresentedSoundSignals } from "../audio/sound-signals";
import {
  INITIAL_TRIGGER_MEMORY,
  decideVoices,
  trackWithoutClock,
  type TriggerMemory,
} from "../audio/triggers";
import type { RenderFrame } from "../physics/frame";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import {
  browserSoundStorage,
  createSoundPreferenceMemory,
  type SoundPreferenceMemory,
} from "./sound-preference";

/** Clicks on these never unlock audio; the toggle's own handler does (UI-D35). */
export const SOUND_CONTROL_SELECTOR = ".puzzle-sound-toggle, .puzzle-sound-section input";

/** The document slice the glue listens to; tests pass a fake. */
export type SoundEventTarget = {
  addEventListener(
    type: string,
    listener: (event: Event) => void,
    options?: AddEventListenerOptions,
  ): void;
  removeEventListener(
    type: string,
    listener: (event: Event) => void,
    options?: EventListenerOptions,
  ): void;
  readonly hidden: boolean;
};

export type PuzzleSound = {
  readonly enabled: Accessor<boolean>;
  readonly available: Accessor<boolean>;
  /** True after a runtime audio failure turned sound off this session (UI-D36). */
  readonly failed: Accessor<boolean>;
  /** Call only from a click or change handler: turning on creates or resumes the context there. */
  readonly setEnabled: (on: boolean) => void;
  /** After dispatch and draw; never throws (D-08). */
  readonly present: (
    frame: RenderFrame,
    signals: PresentedSoundSignals,
    state: PuzzleState,
  ) => void;
  /** Generation change: silence active voices and reset trigger memory (D-10). */
  readonly stopVoices: () => void;
  /** Failure view: stop voices and suspend, keep listeners. */
  readonly pause: () => void;
  /** Puzzle route entry: add gesture and visibility listeners once. */
  readonly attach: () => void;
  /** Puzzle route exit: remove listeners, stop voices, suspend (context kept). */
  readonly leave: () => void;
};

export type PuzzleSoundDeps = {
  readonly preference: SoundPreferenceMemory;
  readonly maybeCreateContext: SoundContextFactory | undefined;
  readonly events: SoundEventTarget;
  readonly logError: (message: string, error: unknown) => void;
};

const FAILURE_MESSAGE = "Puzzle sound stopped after an audio error";
const GESTURE_TYPES = ["pointerdown", "pointerup", "keydown"] as const;
const GESTURE_OPTIONS: AddEventListenerOptions = { capture: true, passive: true };
const VISIBILITY_TYPE = "visibilitychange";

type ClosestTarget = { readonly closest?: unknown };

/** Duck-typed `closest`, so node tests can pass a plain target object. */
function isSoundControl(target: EventTarget | null): boolean {
  if (target === null || typeof target !== "object") {
    return false;
  }

  const maybeClosest = (target as ClosestTarget).closest;
  if (typeof maybeClosest !== "function") {
    return false;
  }

  return maybeClosest.call(target, SOUND_CONTROL_SELECTOR) != null;
}

/**
 * HTML activation-triggering input (Research Pitfall 1): trusted `keydown`
 * other than Escape, `pointerdown` only for a mouse and `pointerup` only for
 * touch or pen. Gestures on the sound controls never count (UI-D35).
 */
function isActivationGesture(event: Event): boolean {
  if (!event.isTrusted) {
    return false;
  }

  if (event.type === "keydown" && (event as KeyboardEvent).key === "Escape") {
    return false;
  }
  if (event.type === "pointerdown" || event.type === "pointerup") {
    const isMouse = (event as PointerEvent).pointerType === "mouse";
    if (isMouse !== (event.type === "pointerdown")) {
      return false;
    }
  }

  return !isSoundControl(event.target);
}

/** Creates the shared puzzle sound; never constructs the audio context. */
export function createPuzzleSound(deps: PuzzleSoundDeps): PuzzleSound {
  const isAvailable = deps.maybeCreateContext !== undefined;
  const [available, setAvailable] = createSignal(isAvailable);
  const [enabled, setEnabledSignal] = createSignal(isAvailable && deps.preference.on());
  const [failed, setFailed] = createSignal(false);
  let memory: TriggerMemory = INITIAL_TRIGGER_MEMORY;
  let attached = false;

  function disableAfterFailure(error: unknown): void {
    deps.logError(FAILURE_MESSAGE, error);
    const alreadyFailed = failed();
    setEnabledSignal(false);
    setFailed(true);
    if (alreadyFailed) {
      return;
    }
    try {
      maybeEngine?.suspend();
    } catch (stopError) {
      deps.logError(FAILURE_MESSAGE, stopError);
    }
  }

  const maybeEngine: SoundEngine | undefined =
    deps.maybeCreateContext === undefined
      ? undefined
      : createSoundEngine({
          createContext: deps.maybeCreateContext,
          onFailure: disableAfterFailure,
          maybeLogError: deps.logError,
        });

  function guarded(action: (engine: SoundEngine) => void): void {
    if (maybeEngine === undefined) {
      return;
    }
    try {
      action(maybeEngine);
    } catch (error) {
      disableAfterFailure(error);
    }
  }

  /** Creates or resumes the context; a failed construction makes sound unavailable (D-05). */
  function unlock(engine: SoundEngine): boolean {
    if (engine.ensureRunning()) {
      return true;
    }

    setAvailable(false);
    setEnabledSignal(false);
    return false;
  }

  function setEnabled(on: boolean): void {
    if (!on) {
      deps.preference.set(false);
      setEnabledSignal(false);
      guarded((engine) => engine.suspend());
      return;
    }

    if (!available()) {
      return;
    }
    setFailed(false);
    guarded((engine) => {
      if (unlock(engine)) {
        deps.preference.set(true);
        setEnabledSignal(true);
      }
    });
  }

  const onGesture = (event: Event) => {
    if (!enabled() || !available() || deps.events.hidden || !isActivationGesture(event)) {
      return;
    }
    guarded((engine) => {
      if (!engine.isRunning()) {
        unlock(engine);
      }
    });
  };

  const onVisibilityChange = () => {
    if (deps.events.hidden) {
      guarded((engine) => engine.suspend());
      return;
    }
    if (!enabled()) {
      return;
    }
    guarded((engine) => {
      if (engine.hasContext()) {
        engine.ensureRunning();
      }
    });
  };

  function present(frame: RenderFrame, signals: PresentedSoundSignals, state: PuzzleState): void {
    try {
      const maybeNowSeconds = maybeEngine?.maybeNowSeconds();
      if (maybeEngine === undefined || maybeNowSeconds === undefined) {
        // No audio clock yet: track the run without inventing a time (D-15).
        memory = trackWithoutClock(memory, { state, frame });
        return;
      }

      const decision = decideVoices(memory, { state, frame, signals, nowSeconds: maybeNowSeconds });
      memory = decision.memory;
      if (decision.requests.length === 0 || !enabled() || deps.events.hidden) {
        return;
      }
      if (maybeEngine.isRunning()) {
        maybeEngine.play(decision.requests);
      }
    } catch (error) {
      disableAfterFailure(error);
    }
  }

  function attach(): void {
    if (attached) {
      return;
    }
    attached = true;
    for (const type of GESTURE_TYPES) {
      deps.events.addEventListener(type, onGesture, GESTURE_OPTIONS);
    }
    deps.events.addEventListener(VISIBILITY_TYPE, onVisibilityChange);
  }

  function leave(): void {
    if (attached) {
      attached = false;
      for (const type of GESTURE_TYPES) {
        deps.events.removeEventListener(type, onGesture, GESTURE_OPTIONS);
      }
      deps.events.removeEventListener(VISIBILITY_TYPE, onVisibilityChange);
    }
    memory = INITIAL_TRIGGER_MEMORY;
    guarded((engine) => engine.suspend());
  }

  return {
    enabled,
    available,
    failed,
    setEnabled,
    present,
    stopVoices: () => {
      memory = INITIAL_TRIGGER_MEMORY;
      guarded((engine) => engine.stopAll());
    },
    pause: () => guarded((engine) => engine.suspend()),
    attach,
    leave,
  };
}

const never = () => false;
const ignore = () => undefined;

/** Sound that is never available; the default for runtimes built outside the browser session. */
export function createDisabledPuzzleSound(): PuzzleSound {
  return {
    enabled: never,
    available: never,
    failed: never,
    setEnabled: ignore,
    present: ignore,
    stopVoices: ignore,
    pause: ignore,
    attach: ignore,
    leave: ignore,
  };
}

/** Browser instance: stored preference, `browserContextFactory()`, `document`, `console.error`. */
export function createBrowserPuzzleSound(): PuzzleSound {
  return createPuzzleSound({
    preference: createSoundPreferenceMemory(browserSoundStorage),
    maybeCreateContext: browserContextFactory(),
    events: document,
    logError: (message, error) => console.error(message, error),
  });
}
