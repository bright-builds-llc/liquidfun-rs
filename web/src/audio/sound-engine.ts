/**
 * The only Web Audio code in the playground (Phase 42). One page-session
 * context is created lazily inside a user activation (D-04), routed through
 * a 0.5 master gain and a compressor (D-09), and never closed: turning sound
 * off stops every voice and suspends it for reuse. The context factory is
 * injected so node-environment tests need no browser.
 */

import { EMPTY_VOICE_POOL, admitVoice, type ActiveVoice, type VoicePool } from "./voice-pool";
import {
  chimeNotes,
  impactParams,
  plinkParams,
  splashParams,
  voiceDurationSeconds,
  type ChimeNote,
  type VoiceRequest,
} from "./voices";

/** The slice of the browser audio context the engine uses. */
export type SoundContext = Pick<
  AudioContext,
  | "state"
  | "currentTime"
  | "sampleRate"
  | "destination"
  | "resume"
  | "suspend"
  | "createGain"
  | "createOscillator"
  | "createBiquadFilter"
  | "createBufferSource"
  | "createDynamicsCompressor"
  | "createBuffer"
>;
export type SoundContextFactory = () => SoundContext;

type WebkitWindow = Window & { readonly webkitAudioContext?: typeof AudioContext };

/** The browser constructor as a factory, or undefined when Web Audio is unavailable (D-05). Never constructs. */
export function browserContextFactory(): SoundContextFactory | undefined {
  const maybeCtor = window.AudioContext ?? (window as WebkitWindow).webkitAudioContext;
  return typeof maybeCtor === "function" ? () => new maybeCtor() : undefined;
}

export type SoundEngine = {
  readonly hasContext: () => boolean;
  readonly isRunning: () => boolean;
  /**
   * Creates the context on first use and resumes it; call only inside a
   * trusted user activation. Always reaches `context.resume()` unless already
   * running, because a resume queued without activation may never settle and
   * only a resume called inside a real activation can start the clock.
   * False when construction threw (logged).
   */
  readonly ensureRunning: () => boolean;
  /**
   * Resumes an existing context outside a user activation (for example when
   * the tab becomes visible). Never constructs, and skips the call while an
   * earlier resume is still pending so background resumes do not stack.
   */
  readonly resumeWithoutGesture: () => void;
  readonly maybeNowSeconds: () => number | undefined;
  /** Starts admitted voices; a no-op unless running. Synchronous node errors propagate to the caller. */
  readonly play: (requests: readonly VoiceRequest[]) => void;
  readonly stopAll: () => void;
  /** Stops every voice, then suspends; the context is kept for reuse. */
  readonly suspend: () => void;
};

export type SoundEngineOptions = {
  readonly createContext: SoundContextFactory;
  /** Asynchronous resume or suspend failures. */
  readonly onFailure: (error: unknown) => void;
  /** Where a failed construction is logged; defaults to `console.error`. */
  readonly maybeLogError?: (message: string, error: unknown) => void;
};

const MASTER_GAIN = 0.5;
/** Exponential ramps cannot reach 0, so envelopes start and end here (Research Pitfall 5). */
const SILENT_GAIN = 0.0001;
const NOISE_SECONDS = 0.25;
const IMPACT_ATTACK_SECONDS = 0.003;
const PLINK_ATTACK_SECONDS = 0.005;
const CHIME_ATTACK_SECONDS = 0.01;
const CONSTRUCTION_FAILED_MESSAGE =
  "Puzzle sound is unavailable: the audio context could not be created";

/** A constructed context with its master chain and lazily built noise buffer. */
type LiveContext = {
  readonly context: SoundContext;
  readonly master: GainNode;
  maybeNoise: AudioBuffer | undefined;
};

/** Nodes of one started pool entry. */
type StartedVoice = {
  readonly sources: readonly AudioScheduledSourceNode[];
  readonly gain: GainNode;
};

function createLiveContext(context: SoundContext): LiveContext {
  const master = context.createGain();
  master.gain.value = MASTER_GAIN;
  const compressor = context.createDynamicsCompressor();
  master.connect(compressor);
  compressor.connect(context.destination);
  return { context, master, maybeNoise: undefined };
}

function noiseBuffer(live: LiveContext): AudioBuffer {
  if (live.maybeNoise !== undefined) {
    return live.maybeNoise;
  }

  const { context } = live;
  const buffer = context.createBuffer(
    1,
    Math.ceil(context.sampleRate * NOISE_SECONDS),
    context.sampleRate,
  );
  const data = buffer.getChannelData(0);
  // Audio-only randomness: it never touches the simulation.
  for (let index = 0; index < data.length; index += 1) {
    data[index] = Math.random() * 2 - 1;
  }
  live.maybeNoise = buffer;
  return buffer;
}

/** A fast attack from silence to `peak`, then an exponential decay back to silence. */
function envelope(
  param: AudioParam,
  startSeconds: number,
  peak: number,
  attackSeconds: number,
  durationSeconds: number,
): void {
  param.setValueAtTime(SILENT_GAIN, startSeconds);
  param.exponentialRampToValueAtTime(Math.max(SILENT_GAIN, peak), startSeconds + attackSeconds);
  param.exponentialRampToValueAtTime(SILENT_GAIN, startSeconds + durationSeconds);
}

function voiceGain(live: LiveContext): GainNode {
  const gain = live.context.createGain();
  gain.connect(live.master);
  return gain;
}

function startSplash(live: LiveContext, intensity: number, now: number): StartedVoice {
  const params = splashParams(intensity);
  const source = live.context.createBufferSource();
  source.buffer = noiseBuffer(live);
  const filter = live.context.createBiquadFilter();
  filter.type = "bandpass";
  filter.frequency.setValueAtTime(params.filterHz, now);
  filter.Q.setValueAtTime(params.q, now);
  const gain = voiceGain(live);
  envelope(gain.gain, now, params.gain, params.attackSeconds, params.durationSeconds);
  source.connect(filter);
  filter.connect(gain);
  source.start(now);
  source.stop(now + params.durationSeconds);
  return { sources: [source], gain };
}

function startTone(
  live: LiveContext,
  type: OscillatorType,
  frequency: number,
  startSeconds: number,
  durationSeconds: number,
): { readonly oscillator: OscillatorNode; readonly gain: GainNode } {
  const oscillator = live.context.createOscillator();
  oscillator.type = type;
  oscillator.frequency.setValueAtTime(frequency, startSeconds);
  const gain = voiceGain(live);
  oscillator.connect(gain);
  oscillator.start(startSeconds);
  oscillator.stop(startSeconds + durationSeconds);
  return { oscillator, gain };
}

function startImpact(live: LiveContext, strength: number, now: number): StartedVoice {
  const params = impactParams(strength);
  const { oscillator, gain } = startTone(
    live,
    "triangle",
    params.startHz,
    now,
    params.durationSeconds,
  );
  oscillator.frequency.exponentialRampToValueAtTime(params.endHz, now + params.durationSeconds);
  envelope(gain.gain, now, params.gain, IMPACT_ATTACK_SECONDS, params.durationSeconds);
  return { sources: [oscillator], gain };
}

function startPlink(live: LiveContext, frequency: number, now: number): StartedVoice {
  const params = plinkParams(frequency);
  const { oscillator, gain } = startTone(
    live,
    "sine",
    params.frequency,
    now,
    params.durationSeconds,
  );
  envelope(gain.gain, now, params.gain, PLINK_ATTACK_SECONDS, params.durationSeconds);
  return { sources: [oscillator], gain };
}

function startChimeNote(live: LiveContext, note: ChimeNote, now: number): StartedVoice {
  const startSeconds = now + note.offsetSeconds;
  const { oscillator, gain } = startTone(
    live,
    "sine",
    note.frequency,
    startSeconds,
    note.durationSeconds,
  );
  envelope(gain.gain, startSeconds, note.gain, CHIME_ATTACK_SECONDS, note.durationSeconds);
  return { sources: [oscillator], gain };
}

/** Silences one voice now: cancel its envelope, mute, stop and disconnect. */
function silence(voice: StartedVoice, now: number): void {
  voice.gain.gain.cancelScheduledValues(now);
  voice.gain.gain.value = 0;
  for (const source of voice.sources) {
    source.stop(now);
  }
  voice.gain.disconnect();
}

/** The pool entries' voices, one per admitted entry (a chime maps its three entries to its notes). */
function startAdmitted(
  live: LiveContext,
  request: VoiceRequest,
  admitted: readonly ActiveVoice[],
  now: number,
): readonly (readonly [ActiveVoice, StartedVoice])[] {
  switch (request.kind) {
    case "splash":
      return admitted.map((entry) => [entry, startSplash(live, request.intensity, now)] as const);
    case "impact":
      return admitted.map((entry) => [entry, startImpact(live, request.strength, now)] as const);
    case "plink":
      return admitted.map((entry) => [entry, startPlink(live, request.frequency, now)] as const);
    case "chime": {
      const notes = chimeNotes();
      return admitted.flatMap((entry, index) => {
        const maybeNote = notes[index];
        return maybeNote === undefined
          ? []
          : [[entry, startChimeNote(live, maybeNote, now)] as const];
      });
    }
  }
}

function defaultLogError(message: string, error: unknown): void {
  console.error(message, error);
}

/** Creates the engine; the factory is called only by the first `ensureRunning`. */
export function createSoundEngine(options: SoundEngineOptions): SoundEngine {
  const logError = options.maybeLogError ?? defaultLogError;
  let maybeLive: LiveContext | undefined;
  let constructionFailed = false;
  let resumeInFlight = false;
  let pool: VoicePool = EMPTY_VOICE_POOL;
  const started = new Map<number, StartedVoice>();

  const isRunning = () => maybeLive?.context.state === "running";

  function resume(context: SoundContext, fromGesture: boolean): void {
    if (context.state === "running" || (resumeInFlight && !fromGesture)) {
      return;
    }

    resumeInFlight = true;
    const failed = (error: unknown) => {
      resumeInFlight = false;
      options.onFailure(error);
    };
    try {
      context.resume().then(() => {
        resumeInFlight = false;
      }, failed);
    } catch (error) {
      failed(error);
    }
  }

  function ensureRunning(): boolean {
    if (constructionFailed) {
      return false;
    }
    if (maybeLive === undefined) {
      try {
        maybeLive = createLiveContext(options.createContext());
      } catch (error) {
        constructionFailed = true;
        logError(CONSTRUCTION_FAILED_MESSAGE, error);
        return false;
      }
    }

    resume(maybeLive.context, true);
    return true;
  }

  function resumeWithoutGesture(): void {
    if (maybeLive !== undefined) {
      resume(maybeLive.context, false);
    }
  }

  function register(id: number, voice: StartedVoice): void {
    started.set(id, voice);
    const [first] = voice.sources;
    if (first === undefined) {
      return;
    }
    first.onended = () => {
      voice.gain.disconnect();
      if (started.get(id) === voice) {
        started.delete(id);
      }
    };
  }

  function play(requests: readonly VoiceRequest[]): void {
    const live = maybeLive;
    if (live === undefined || !isRunning()) {
      return;
    }

    const now = live.context.currentTime;
    for (const request of requests) {
      const admission = admitVoice(pool, request.kind, now, voiceDurationSeconds(request));
      pool = admission.pool;
      for (const evicted of admission.evicted) {
        const maybeVoice = started.get(evicted.id);
        started.delete(evicted.id);
        if (maybeVoice !== undefined) {
          silence(maybeVoice, now);
        }
      }
      // Each voice is registered only after its sources started (Research Pitfall 6).
      for (const [entry, voice] of startAdmitted(live, request, admission.admitted, now)) {
        register(entry.id, voice);
      }
    }
  }

  /** Silences every voice; the first node error is rethrown after all were tried. */
  function stopAll(): void {
    const voices = [...started.values()];
    started.clear();
    pool = { ...pool, voices: [] };
    const live = maybeLive;
    if (live === undefined) {
      return;
    }

    const now = live.context.currentTime;
    let maybeError: { readonly error: unknown } | undefined;
    for (const voice of voices) {
      try {
        silence(voice, now);
      } catch (error) {
        maybeError ??= { error };
      }
    }
    if (maybeError !== undefined) {
      throw maybeError.error;
    }
  }

  function suspend(): void {
    try {
      // A suspended clock freezes scheduled notes, so stop first (Research Pitfall 8).
      stopAll();
    } finally {
      // A resume still pending when suspended must not block the next one.
      resumeInFlight = false;
      maybeLive?.context.suspend().catch(options.onFailure);
    }
  }

  return {
    hasContext: () => maybeLive !== undefined,
    isRunning,
    ensureRunning,
    resumeWithoutGesture,
    maybeNowSeconds: () => maybeLive?.context.currentTime,
    play,
    stopAll,
    suspend,
  };
}
