import type { JSX } from "solid-js";

import type { SceneId } from "./scenes";

const WATER = "#4DA3FF";
const ACCENT_WATER = "#39D3C7";
const MIX_RED = "#F87171";
const MIX_GREEN = "#3DDC97";
const RIGID = "#CBD5E1";
const JELLY = "#F4F7FA";
const CANVAS = "#071018";

export type ScenePreviewProps = {
  readonly sceneId: SceneId;
};

function PreviewFrame(props: { readonly children: JSX.Element }) {
  return (
    <svg viewBox="0 0 160 90" role="presentation" aria-hidden="true">
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

function ParticlesPreview() {
  return (
    <PreviewFrame>
      <line x1="28" y1="34" x2="28" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="74" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="74" x2="132" y2="34" stroke={RIGID} stroke-width="3" />
      <circle cx="80" cy="58" r="14" fill={ACCENT_WATER} />
      <circle cx="80" cy="28" r="8" fill="none" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

function LiquidTimerPreview() {
  return (
    <PreviewFrame>
      <path
        d="M40 18 L40 72 Q40 78 80 78 Q120 78 120 72 L120 18"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
      <line x1="48" y1="34" x2="96" y2="40" stroke={RIGID} stroke-width="2" />
      <line x1="64" y1="46" x2="112" y2="52" stroke={RIGID} stroke-width="2" />
      <line x1="48" y1="58" x2="96" y2="64" stroke={RIGID} stroke-width="2" />
      <rect x="56" y="20" width="48" height="10" fill={ACCENT_WATER} />
      <rect x="48" y="70" width="10" height="6" fill={ACCENT_WATER} />
      <rect x="66" y="70" width="10" height="6" fill={ACCENT_WATER} />
      <rect x="84" y="70" width="10" height="6" fill={ACCENT_WATER} />
      <rect x="102" y="70" width="10" height="6" fill={ACCENT_WATER} />
    </PreviewFrame>
  );
}

function VerticalWallBasin() {
  return (
    <>
      <line x1="28" y1="34" x2="28" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="74" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="74" x2="132" y2="34" stroke={RIGID} stroke-width="3" />
    </>
  );
}

function SurfaceTensionPreview() {
  return (
    <PreviewFrame>
      <VerticalWallBasin />
      <circle cx="52" cy="58" r="12" fill={MIX_RED} />
      <circle cx="80" cy="58" r="12" fill={MIX_GREEN} />
      <rect x="100" y="48" width="22" height="22" fill={WATER} />
      <circle cx="80" cy="26" r="8" fill="none" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

function ElasticParticlesPreview() {
  return (
    <PreviewFrame>
      <VerticalWallBasin />
      <ellipse cx="50" cy="58" rx="14" ry="11" fill={MIX_RED} />
      <ellipse cx="78" cy="58" rx="14" ry="11" fill={MIX_GREEN} />
      <g transform="rotate(-18 112 58)">
        <rect x="100" y="46" width="24" height="24" rx="4" fill={WATER} />
      </g>
      <circle cx="80" cy="26" r="8" fill="none" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

function RigidParticlesPreview() {
  return (
    <PreviewFrame>
      <VerticalWallBasin />
      <circle
        cx="50"
        cy="58"
        r="12"
        fill="none"
        stroke={MIX_RED}
        stroke-width="3"
      />
      <circle
        cx="78"
        cy="58"
        r="12"
        fill="none"
        stroke={MIX_GREEN}
        stroke-width="3"
      />
      <g transform="rotate(-18 112 58)">
        <rect
          x="100"
          y="46"
          width="24"
          height="24"
          fill="none"
          stroke={WATER}
          stroke-width="3"
        />
      </g>
      <circle cx="80" cy="26" r="8" fill="none" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

function SoupBasin() {
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

function SoupPreview() {
  return (
    <PreviewFrame>
      <SoupBasin />
    </PreviewFrame>
  );
}

function SoupStirrerPreview() {
  return (
    <PreviewFrame>
      <SoupBasin />
      <line
        x1="52"
        y1="30"
        x2="108"
        y2="30"
        stroke={RIGID}
        stroke-width="3"
      />
      <circle
        cx="80"
        cy="42"
        r="12"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
    </PreviewFrame>
  );
}

function ImpulsePreview() {
  return (
    <PreviewFrame>
      <rect
        x="36"
        y="18"
        width="88"
        height="54"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
      <circle cx="80" cy="45" r="16" fill={WATER} />
      <line
        x1="96"
        y1="38"
        x2="112"
        y2="28"
        stroke={JELLY}
        stroke-width="2"
      />
    </PreviewFrame>
  );
}

function WaveMachinePreview() {
  return (
    <PreviewFrame>
      <g transform="rotate(-12 80 50)">
        <rect
          x="40"
          y="24"
          width="80"
          height="48"
          fill="none"
          stroke={RIGID}
          stroke-width="3"
        />
        <rect x="46" y="42" width="68" height="24" fill={WATER} />
      </g>
    </PreviewFrame>
  );
}

function LiquidTumblerPreview() {
  return (
    <PreviewFrame>
      <line x1="58" y1="22" x2="58" y2="74" stroke={RIGID} stroke-width="4" />
      <line x1="58" y1="74" x2="102" y2="74" stroke={RIGID} stroke-width="4" />
      <line x1="102" y1="74" x2="102" y2="22" stroke={RIGID} stroke-width="4" />
      <rect x="62" y="46" width="36" height="28" fill={WATER} />
    </PreviewFrame>
  );
}

function TheoJansenPreview() {
  return (
    <PreviewFrame>
      <line x1="20" y1="74" x2="140" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="24" y1="40" x2="24" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="136" y1="40" x2="136" y2="74" stroke={RIGID} stroke-width="3" />
      <rect
        x="58"
        y="40"
        width="36"
        height="12"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
      <circle
        cx="76"
        cy="60"
        r="8"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
      <line x1="68" y1="52" x2="56" y2="70" stroke={RIGID} stroke-width="2" />
      <line x1="84" y1="52" x2="92" y2="70" stroke={RIGID} stroke-width="2" />
      <line x1="76" y1="52" x2="76" y2="68" stroke={RIGID} stroke-width="2" />
      <rect x="62" y="28" width="28" height="10" fill={WATER} />
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
    case "particles":
      return <ParticlesPreview />;
    case "liquid-timer":
      return <LiquidTimerPreview />;
    case "surface-tension":
      return <SurfaceTensionPreview />;
    case "elastic-particles":
      return <ElasticParticlesPreview />;
    case "rigid-particles":
      return <RigidParticlesPreview />;
    case "soup":
      return <SoupPreview />;
    case "soup-stirrer":
      return <SoupStirrerPreview />;
    case "impulse":
      return <ImpulsePreview />;
    case "wave-machine":
      return <WaveMachinePreview />;
    case "theo-jansen":
      return <TheoJansenPreview />;
    case "liquid-tumbler":
      return <LiquidTumblerPreview />;
  }
}
