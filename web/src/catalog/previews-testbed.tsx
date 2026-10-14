/** Static SVG previews for the testbed scenes. */
import {
  ACCENT_WATER,
  JELLY,
  MIX_GREEN,
  MIX_RED,
  PreviewFrame,
  RIGID,
  SoupBasin,
  VerticalWallBasin,
  WATER,
} from "./preview-kit";

export function ParticlesPreview() {
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

export function LiquidTimerPreview() {
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

export function SurfaceTensionPreview() {
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

export function ElasticParticlesPreview() {
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

export function RigidParticlesPreview() {
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

export function SoupPreview() {
  return (
    <PreviewFrame>
      <SoupBasin />
    </PreviewFrame>
  );
}

export function SoupStirrerPreview() {
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

export function ImpulsePreview() {
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

export function DrawingParticlesPreview() {
  return (
    <PreviewFrame>
      <line x1="36" y1="70" x2="124" y2="70" stroke={RIGID} stroke-width="3" />
      <line x1="36" y1="28" x2="36" y2="70" stroke={RIGID} stroke-width="3" />
      <line x1="124" y1="28" x2="124" y2="70" stroke={RIGID} stroke-width="3" />
      <line x1="36" y1="28" x2="58" y2="28" stroke={RIGID} stroke-width="3" />
      <line x1="102" y1="28" x2="124" y2="28" stroke={RIGID} stroke-width="3" />
      <circle cx="52" cy="58" r="2" fill={WATER} />
      <circle cx="58" cy="56" r="2" fill={WATER} />
      <circle cx="64" cy="58" r="2" fill={WATER} />
      <circle cx="96" cy="48" r="4" fill={MIX_GREEN} />
      <circle cx="104" cy="50" r="4" fill={MIX_GREEN} />
      <circle cx="100" cy="56" r="4" fill={MIX_GREEN} />
    </PreviewFrame>
  );
}

export function SparkyPreview() {
  return (
    <PreviewFrame>
      <rect
        x="48"
        y="16"
        width="64"
        height="62"
        fill="none"
        stroke={RIGID}
        stroke-width="3"
      />
      <circle cx="80" cy="30" r="8" fill="none" stroke={RIGID} stroke-width="2" />
      <circle cx="68" cy="50" r="8" fill="none" stroke={RIGID} stroke-width="2" />
      <circle cx="92" cy="50" r="8" fill="none" stroke={RIGID} stroke-width="2" />
      <circle cx="80" cy="42" r="2" fill={MIX_RED} />
      <circle cx="86" cy="40" r="2" fill={MIX_GREEN} />
      <circle cx="74" cy="40" r="2" fill={ACCENT_WATER} />
      <circle cx="80" cy="36" r="2" fill={JELLY} />
    </PreviewFrame>
  );
}

export function LiquidTumblerPreview() {
  return (
    <PreviewFrame>
      <line x1="58" y1="22" x2="58" y2="74" stroke={RIGID} stroke-width="4" />
      <line x1="58" y1="74" x2="102" y2="74" stroke={RIGID} stroke-width="4" />
      <line x1="102" y1="74" x2="102" y2="22" stroke={RIGID} stroke-width="4" />
      <rect x="62" y="46" width="36" height="28" fill={WATER} />
    </PreviewFrame>
  );
}

export function TheoJansenPreview() {
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
