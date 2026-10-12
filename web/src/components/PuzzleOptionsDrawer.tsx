import type { JSX } from "solid-js";

import { RENDER_MODE_GROUPS, maybeParseRenderMode, type RenderMode } from "../render/mode";
import { GitHubSourceLink } from "./SiteHeader";
import { SiteFooter } from "./SiteFooter";
import {
  DrawerContent,
  DrawerDescription,
  DrawerLabel,
  DrawerTrigger,
} from "./ui/drawer";

const OPTIONS_LABEL = "Puzzle options";
const OPTIONS_DESCRIPTION = "Display options and project details for this puzzle.";

/** 44px HUD trigger for the puzzle options sheet; must render inside a `Drawer`. */
export function PuzzleOptionsTrigger() {
  return (
    <DrawerTrigger
      class="hud-button canvas-controls-trigger puzzle-options-trigger"
      aria-label="Puzzle options"
    >
      <OptionsIcon />
    </DrawerTrigger>
  );
}

/**
 * Bottom sheet holding only display options, the source link and the build
 * footer (D-24), so provenance stays reachable without any scene control.
 */
export function PuzzleOptionsSheet(props: { readonly children?: JSX.Element }) {
  return (
    <DrawerContent class="canvas-controls-sheet">
      <DrawerLabel>{OPTIONS_LABEL}</DrawerLabel>
      <DrawerDescription class="visually-hidden">{OPTIONS_DESCRIPTION}</DrawerDescription>
      <div class="canvas-controls-scroll">
        <div class="canvas-controls-body">
          {props.children}
          <GitHubSourceLink />
          <SiteFooter />
        </div>
      </div>
    </DrawerContent>
  );
}

/** Particle render-mode select; the only display option on puzzle routes (UI-D7). */
export function PuzzleRenderModeSelect(props: {
  readonly renderMode: RenderMode;
  readonly onRenderModeChange: (mode: RenderMode) => void;
}) {
  return (
    <label class="render-mode-control">
      Particles
      <select
        value={props.renderMode}
        onChange={(event) => {
          const maybeMode = maybeParseRenderMode(event.currentTarget.value);
          if (maybeMode !== undefined) {
            props.onRenderModeChange(maybeMode);
          }
        }}
      >
        {RENDER_MODE_GROUPS.map((group) => (
          <optgroup label={group.label}>
            {group.options.map((option) => (
              <option value={option.value}>{option.label}</option>
            ))}
          </optgroup>
        ))}
      </select>
    </label>
  );
}

function OptionsIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 8h16M4 16h16M8 6v4M16 14v4" />
    </svg>
  );
}
