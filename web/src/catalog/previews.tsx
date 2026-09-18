import type { JSX } from "solid-js";

import type { SceneId } from "./scenes";

const WATER = "#39D3C7";
const MIX_RED = "#F87171";
const RIGID = "#CBD5E1";
const JELLY = "#F4F7FA";
const CANVAS = "#071018";

export type ScenePreviewProps = {
  readonly sceneId: SceneId;
};

function PreviewFrame(props: { readonly children: JSX.Element }) {
  return (
    <svg
      class="catalog-preview"
      viewBox="0 0 160 90"
      role="presentation"
      aria-hidden="true"
    >
      <rect width="160" height="90" fill={CANVAS} />
      {props.children}
    </svg>
  );
}

function DamBreakPreview() {
  return (
    <PreviewFrame>
      <rect x="28" y="12" width="48" height="28" fill={WATER} />
      <line x1="22" y1="38" x2="22" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="22" y1="74" x2="138" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="138" y1="74" x2="138" y2="38" stroke={RIGID} stroke-width="3" />
      <circle cx="108" cy="52" r="10" fill="none" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

function FountainPreview() {
  return (
    <PreviewFrame>
      <circle cx="50" cy="22" r="3" fill={WATER} />
      <circle cx="62" cy="18" r="3" fill={WATER} />
      <circle cx="76" cy="20" r="3" fill={WATER} />
      <circle cx="88" cy="28" r="3" fill={WATER} />
      <circle cx="96" cy="40" r="3" fill={WATER} />
      <path
        d="M40 48 Q40 78 80 78 Q120 78 120 48"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
    </PreviewFrame>
  );
}

function FloatOrSinkPreview() {
  return (
    <PreviewFrame>
      <rect x="16" y="48" width="128" height="18" fill={WATER} />
      <rect x="40" y="36" width="16" height="10" fill={RIGID} />
      <rect x="92" y="54" width="28" height="16" fill={RIGID} />
    </PreviewFrame>
  );
}

function ColorMixerPreview() {
  return (
    <PreviewFrame>
      <circle cx="64" cy="46" r="22" fill={WATER} />
      <circle cx="96" cy="46" r="22" fill={MIX_RED} />
    </PreviewFrame>
  );
}

function JellyDropPreview() {
  return (
    <PreviewFrame>
      <rect x="28" y="62" width="44" height="8" rx="1" fill={RIGID} />
      <rect x="88" y="62" width="44" height="8" rx="1" fill={RIGID} />
      <rect x="58" y="16" width="44" height="36" rx="14" fill={JELLY} />
    </PreviewFrame>
  );
}

function WaterWheelPreview() {
  return (
    <PreviewFrame>
      <circle cx="80" cy="48" r="8" fill="none" stroke={RIGID} stroke-width="3" />
      <line x1="80" y1="20" x2="80" y2="40" stroke={RIGID} stroke-width="4" />
      <line x1="80" y1="56" x2="80" y2="76" stroke={RIGID} stroke-width="4" />
      <line x1="52" y1="48" x2="72" y2="48" stroke={RIGID} stroke-width="4" />
      <line x1="88" y1="48" x2="108" y2="48" stroke={RIGID} stroke-width="4" />
      <line
        x1="18"
        y1="32"
        x2="48"
        y2="44"
        stroke={WATER}
        stroke-width="3"
        stroke-dasharray="4 3"
      />
    </PreviewFrame>
  );
}

/** Static token-only SVG preview. Never starts a WASM session. */
export function ScenePreview(props: ScenePreviewProps) {
  switch (props.sceneId) {
    case "dam-break":
      return <DamBreakPreview />;
    case "fountain":
      return <FountainPreview />;
    case "float-or-sink":
      return <FloatOrSinkPreview />;
    case "color-mixer":
      return <ColorMixerPreview />;
    case "jelly-drop":
      return <JellyDropPreview />;
    case "water-wheel":
      return <WaterWheelPreview />;
  }
}
