/**
 * Phase 42 sound controls (UI-D29): the HUD speaker toggle and the "Sound"
 * section of the options drawer and form-factor row. Both read and write the
 * one shared sound state on `PuzzleController`, so they never disagree.
 * Neither is a HUD obstacle leaf; the `puzzle-options-slot` that holds the
 * toggle is.
 */
import "../styles/puzzle-sound.css";

import { Match, Show, Switch } from "solid-js";

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
 * carries on and off (D-01); unavailable renders disabled in the same box
 * (D-05). It is one element in every state, and unavailable uses
 * `aria-disabled` rather than `disabled`, so a click whose context creation
 * fails keeps keyboard focus on the button instead of dropping it to the body.
 */
export function PuzzleSoundToggle(props: { readonly controller: PuzzleController }) {
  const state = () => soundState(props.controller);
  const unavailable = () => state() === "unavailable";
  return (
    <button
      type="button"
      class="hud-button puzzle-sound-toggle"
      aria-disabled={unavailable() ? "true" : undefined}
      aria-pressed={state() === "on" ? "true" : "false"}
      aria-label={unavailable() ? SOUND_UNAVAILABLE_LABEL : SOUND_LABEL}
      title={unavailable() ? SOUND_UNAVAILABLE_LABEL : SOUND_HINT}
      aria-describedby={unavailable() ? undefined : SOUND_HELP_ID}
      data-sound-state={state()}
      onClick={() => {
        if (!unavailable()) {
          props.controller.setSoundEnabled(!props.controller.soundEnabled());
        }
      }}
    >
      <Switch>
        <Match when={state() === "on"}>
          <SoundOnIcon />
        </Match>
        <Match when={state() === "off"}>
          <SoundOffIcon />
        </Match>
        <Match when={unavailable()}>
          <SoundUnavailableIcon />
        </Match>
      </Switch>
    </button>
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
          onChange={(event) => {
            props.controller.setSoundEnabled(event.currentTarget.checked);
            // A failed unlock leaves the bound value unchanged, so Solid would
            // not rewrite the box the player just checked (IN-03).
            event.currentTarget.checked = available() && props.controller.soundEnabled();
          }}
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
