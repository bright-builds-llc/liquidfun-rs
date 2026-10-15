/** Shared palette, frame and basin pieces for the static scene previews. */
import type { JSX } from "solid-js";

export const WATER = "#4DA3FF";
/** Fake Ferrofluid liquid; matches the Rust particle color. */
export const FERRO = "#8A84AD";
export const DRIP = "#F2B040";
export const ACCENT_WATER = "#39D3C7";
export const MIX_RED = "#F87171";
export const MIX_GREEN = "#3DDC97";
export const RIGID = "#CBD5E1";
export const JELLY = "#F4F7FA";
export const CANVAS = "#071018";

export function PreviewFrame(props: { readonly children: JSX.Element }) {
  return (
    <svg viewBox="0 0 160 90" role="presentation" aria-hidden="true">
      <rect width="160" height="90" fill={CANVAS} />
      {props.children}
    </svg>
  );
}

export function VerticalWallBasin() {
  return (
    <>
      <line x1="28" y1="34" x2="28" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="74" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="74" x2="132" y2="34" stroke={RIGID} stroke-width="3" />
    </>
  );
}

export function SoupBasin() {
  return (
    <>
      <line x1="28" y1="34" x2="28" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="74" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="74" x2="132" y2="34" stroke={RIGID} stroke-width="3" />
      <rect x="32" y="48" width="96" height="24" fill={WATER} />
      <circle cx="56" cy="56" r="6" fill={MIX_RED} />
      <rect x="78" y="52" width="10" height="10" fill={RIGID} />
      <rect x="100" y="54" width="10" height="10" fill={MIX_GREEN} />
      <line
        x1="40"
        y1="44"
        x2="52"
        y2="44"
        stroke={MIX_GREEN}
        stroke-width="2"
      />
      <line
        x1="110"
        y1="46"
        x2="122"
        y2="46"
        stroke={ACCENT_WATER}
        stroke-width="2"
      />
    </>
  );
}
