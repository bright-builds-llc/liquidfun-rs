import type { Page } from "@playwright/test";

/** What the fake audio context recorded on `window.__liquidfunAudioLog`. */
export type AudioLog = {
  readonly constructed: number;
  readonly states: readonly string[];
  readonly started: number;
  readonly closed: number;
};

const AUDIO_LOG_KEY = "__liquidfunAudioLog";
const SOUND_STORAGE_KEY = "liquidfun.puzzle-sound.v1";
const SEEDED_MARKER_KEY = "__liquidfunSoundPreferenceSeeded";

/**
 * Replaces `AudioContext` with a recording fake before any app code runs
 * (42-RESEARCH "Playwright: fake AudioContext via addInitScript"). Nodes
 * accept every call the engine makes and record only source starts.
 */
export async function installFakeAudio(page: Page): Promise<void> {
  await page.addInitScript((logKey) => {
    type Log = { constructed: number; states: string[]; started: number; closed: number };
    const log: Log = { constructed: 0, states: [], started: 0, closed: 0 };
    Object.defineProperty(window, logKey, { value: log, configurable: true });
    const param = () => ({
      value: 0,
      setValueAtTime() {
        return this;
      },
      linearRampToValueAtTime() {
        return this;
      },
      exponentialRampToValueAtTime() {
        return this;
      },
      setTargetAtTime() {
        return this;
      },
      cancelScheduledValues() {
        return this;
      },
    });
    const node = () => ({
      gain: param(),
      frequency: param(),
      Q: param(),
      detune: param(),
      threshold: param(),
      knee: param(),
      ratio: param(),
      attack: param(),
      release: param(),
      type: "",
      buffer: null,
      onended: null,
      connect(target: unknown) {
        return target;
      },
      disconnect() {},
      start() {
        log.started += 1;
      },
      stop() {},
    });
    class FakeAudioContext {
      state = "suspended";
      sampleRate = 48_000;
      destination = node();
      get currentTime() {
        return performance.now() / 1000;
      }
      constructor() {
        log.constructed += 1;
        log.states.push("constructed");
      }
      resume() {
        this.state = "running";
        log.states.push("running");
        return Promise.resolve();
      }
      suspend() {
        this.state = "suspended";
        log.states.push("suspended");
        return Promise.resolve();
      }
      close() {
        this.state = "closed";
        log.states.push("closed");
        log.closed += 1;
        return Promise.resolve();
      }
      createGain() {
        return node();
      }
      createOscillator() {
        return node();
      }
      createBiquadFilter() {
        return node();
      }
      createBufferSource() {
        return node();
      }
      createDynamicsCompressor() {
        return node();
      }
      createBuffer(_channels: number, length: number, sampleRate: number) {
        const data = new Float32Array(length);
        return { length, sampleRate, numberOfChannels: 1, getChannelData: () => data };
      }
    }
    Object.defineProperty(window, "AudioContext", { value: FakeAudioContext, configurable: true });
    Object.defineProperty(window, "webkitAudioContext", { value: undefined, configurable: true });
  }, AUDIO_LOG_KEY);
}

/** A browser without Web Audio: neither constructor exists (D-05). */
export async function installUnavailableAudio(page: Page): Promise<void> {
  await page.addInitScript(() => {
    Object.defineProperty(window, "AudioContext", { value: undefined, configurable: true });
    Object.defineProperty(window, "webkitAudioContext", { value: undefined, configurable: true });
  });
}

/**
 * Seeds the stored sound choice on the first navigation only, so a later
 * reload observes what the app itself wrote.
 */
export async function storeSoundPreference(page: Page, value: "on" | "off"): Promise<void> {
  await page.addInitScript(
    ({ key, marker, stored }) => {
      if (window.sessionStorage.getItem(marker) !== null) {
        return;
      }
      window.sessionStorage.setItem(marker, "1");
      window.localStorage.setItem(key, stored);
    },
    { key: SOUND_STORAGE_KEY, marker: SEEDED_MARKER_KEY, stored: value },
  );
}

/** The stored sound token, or null when nothing was written. */
export async function readStoredSound(page: Page): Promise<string | null> {
  return page.evaluate((key) => window.localStorage.getItem(key), SOUND_STORAGE_KEY);
}

/** A copy of the fake context's log; throws when the fake is not installed. */
export async function readAudioLog(page: Page): Promise<AudioLog> {
  const maybeLog = await page.evaluate((logKey) => {
    const maybeValue = (window as unknown as Record<string, AudioLog | undefined>)[logKey];
    return maybeValue === undefined
      ? null
      : {
          constructed: maybeValue.constructed,
          states: [...maybeValue.states],
          started: maybeValue.started,
          closed: maybeValue.closed,
        };
  }, AUDIO_LOG_KEY);
  if (maybeLog === null) {
    throw new Error("the fake audio context is not installed");
  }
  return maybeLog;
}
