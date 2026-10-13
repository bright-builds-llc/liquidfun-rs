/**
 * Phase 42 sound controls (UI-D29): the HUD speaker toggle and the "Sound"
 * section of the options drawer and form-factor row. Both read and write the
 * one shared sound state on `PuzzleController`, so they never disagree.
 * Neither is a HUD obstacle leaf; the `puzzle-options-slot` that holds the
 * toggle is.
 */
import "../styles/puzzle-sound.css";

import { Show } from "solid-js";

import type { PuzzleController } from "../puzzle/controller";
import { SoundOffIcon, SoundOnIcon, SoundUnavailableIcon } from "./PuzzleIcons";

export const SOUND_LABEL = "Sound";
export const SOUND_EFFECTS_LABEL = "Sound effects";
export const SOUND_HINT = "No sound on iPhone? Check the silent switch.";
export const SOUND_UNAVAILABLE_LABEL = "Sound unavailable in this browser";
export const SOUND_UNAVAILABLE_LINE =
  "Sound is unavailable in this browser. The puzzle still works without it.";
export const SOUND_FAILED_NOTE =
  "Sound stopped after an audio error. Turn on Sound effects to try again.";
/** Id of the visually hidden hint paragraph that describes the HUD toggle (UI-D34). */
export const SOUND_HELP_ID = "puzzle-sound-help";

type SoundState = "on" | "off" | "unavailable";

function soundState(controller: PuzzleController): SoundState {
  if (!controller.soundAvailable()) {
    return "unavailable";
  }
  return controller.soundEnabled() ? "on" : "off";
}

/**
 * 44 px HUD toggle. The accessible name stays "Sound" while `aria-pressed`
 * carries on and off (D-01); unavailable renders disabled in the same box (D-05).
 */
export function PuzzleSoundToggle(props: { readonly controller: PuzzleController }) {
  return (
    <Show
      when={props.controller.soundAvailable()}
      fallback={
        <button
          type="button"
          class="hud-button puzzle-sound-toggle"
          disabled
          aria-pressed="false"
          aria-label={SOUND_UNAVAILABLE_LABEL}
          title={SOUND_UNAVAILABLE_LABEL}
          data-sound-state="unavailable"
        >
          <SoundUnavailableIcon />
        </button>
      }
    >
      <button
        type="button"
        class="hud-button puzzle-sound-toggle"
        aria-pressed={props.controller.soundEnabled() ? "true" : "false"}
        aria-label={SOUND_LABEL}
        title={SOUND_HINT}
        aria-describedby={SOUND_HELP_ID}
        data-sound-state={props.controller.soundEnabled() ? "on" : "off"}
        onClick={() => props.controller.setSoundEnabled(!props.controller.soundEnabled())}
      >
        <Show when={props.controller.soundEnabled()} fallback={<SoundOffIcon />}>
          <SoundOnIcon />
        </Show>
      </button>
    </Show>
  );
}

/**
 * "Sound" section with the mirror checkbox, the silent-switch hint (D-11) or
 * the unavailable line, and the note shown after a runtime audio failure (UI-D36).
 */
export function PuzzleSoundSection(props: { readonly controller: PuzzleController }) {
  const available = () => props.controller.soundAvailable();
  return (
    <section
      class="puzzle-sound-section"
      aria-labelledby="puzzle-sound-heading"
      data-sound-state={soundState(props.controller)}
    >
      <h3 id="puzzle-sound-heading">{SOUND_LABEL}</h3>
      <label class="player-check">
        <input
          type="checkbox"
          aria-describedby="puzzle-sound-hint"
          checked={available() && props.controller.soundEnabled()}
          disabled={!available()}
          onChange={(event) => props.controller.setSoundEnabled(event.currentTarget.checked)}
        />
        {SOUND_EFFECTS_LABEL}
      </label>
      <Show when={props.controller.soundFailed()}>
        <p id="puzzle-sound-note" class="puzzle-sound-note">
          {SOUND_FAILED_NOTE}
        </p>
      </Show>
      <p id="puzzle-sound-hint" class="puzzle-sound-hint">
        {available() ? SOUND_HINT : SOUND_UNAVAILABLE_LINE}
      </p>
    </section>
  );
}
