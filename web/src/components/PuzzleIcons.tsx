import type { JSX } from "solid-js";

/**
 * Puzzle-local inline icons (Phase 39 UI-SPEC): a 24-unit viewBox in a 22 px
 * box, stroked with `currentColor` at width 2 with round caps and joins.
 * Filled shapes set their own fill. Hand-drawn; every icon is decorative and
 * its button carries the accessible name.
 */
function Icon(props: { readonly children: JSX.Element }) {
  return (
    <svg class="puzzle-icon" viewBox="0 0 24 24" aria-hidden="true">
      {props.children}
    </svg>
  );
}

/** Filled triangle (Phase 38 Play path). */
export function PlayIcon() {
  return (
    <Icon>
      <path d="M8 6v12l10-6z" fill="currentColor" stroke="none" />
    </Icon>
  );
}

/** Filled rounded square. */
export function StopIcon() {
  return (
    <Icon>
      <rect x="7" y="7" width="10" height="10" rx="1.5" fill="currentColor" stroke="none" />
    </Icon>
  );
}

/** Eraser: a body turned 45° with its rubber tip, over a base line. */
export function ClearIcon() {
  return (
    <Icon>
      <path d="M3 15l8-8 6 6-8 8z" />
      <path d="M7 11l6 6" />
      <path d="M9 21h11" />
    </Icon>
  );
}

/** A 300° arc whose upper-left arrowhead points counterclockwise. */
export function RotateCounterclockwiseIcon() {
  return (
    <Icon>
      <path d="M5.07 16A8 8 0 1 0 5.07 8" />
      <path d="M8.9 7L5.07 8L4 4.1" />
    </Icon>
  );
}

/** Mirror image of the counterclockwise icon. */
export function RotateClockwiseIcon() {
  return (
    <Icon>
      <path d="M18.93 16A8 8 0 1 1 18.93 8" />
      <path d="M15.1 7L18.93 8L20 4.1" />
    </Icon>
  );
}

/** Trash can: lid, handle, tapered bin and two inner lines. */
export function RemoveIcon() {
  return (
    <Icon>
      <path d="M4 7h16" />
      <path d="M9 7V4h6v3" />
      <path d="M6 7l1 13h10l1-13" />
      <path d="M10 11v6M14 11v6" />
    </Icon>
  );
}

/** Filled rounded bar. */
export function PlankIcon() {
  return (
    <Icon>
      <rect x="2" y="9.5" width="20" height="5" rx="1.5" fill="currentColor" stroke="none" />
    </Icon>
  );
}

/** Valve: a shorter filled gate bar under a small clock face with one hand. */
export function ValveIcon() {
  return (
    <Icon>
      <rect x="4" y="15" width="16" height="5" rx="1.5" fill="currentColor" stroke="none" />
      <circle cx="12" cy="7.5" r="4.5" />
      <path d="M12 5.5v2l1.5 1" />
    </Icon>
  );
}

/** Five points, outer radius 10 and inner radius 4.5, centred at (12, 12). */
const STAR_PATH =
  "M12 2L14.65 8.36L21.51 8.91L16.28 13.39L17.88 20.09L12 16.5L6.12 20.09L7.72 13.39L2.49 8.91L9.35 8.36Z";

/** Star glyph; its row colors it (UI-D12). */
export function StarIcon(props: { readonly filled: boolean }) {
  return (
    <svg class="puzzle-star-glyph" viewBox="0 0 24 24" aria-hidden="true">
      <path d={STAR_PATH} fill={props.filled ? "currentColor" : "none"} />
    </svg>
  );
}
