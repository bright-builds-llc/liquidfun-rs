/** Static SVG previews for the curiosity scenes. */
import { CANVAS, PreviewFrame, RIGID, WATER } from "./preview-kit";

export function CartesianDiverPreview() {
  return (
    <PreviewFrame>
      <rect x="68" y="18" width="24" height="62" fill={WATER} />
      <line x1="64" y1="12" x2="96" y2="12" stroke={RIGID} stroke-width="3" />
      <line x1="64" y1="82" x2="96" y2="82" stroke={RIGID} stroke-width="3" />
      <path d="M64 12 Q72 47 64 82" fill="none" stroke={RIGID} stroke-width="3" />
      <path d="M96 12 Q88 47 96 82" fill="none" stroke={RIGID} stroke-width="3" />
      <circle cx="80" cy="52" r="7" fill={CANVAS} stroke={RIGID} stroke-width="2" />
      <circle cx="80" cy="47.7" r="2.1" fill="none" stroke={RIGID} stroke-width="1.5" />
    </PreviewFrame>
  );
}

export function ArchimedesScrewPreview() {
  return (
    <PreviewFrame>
      <rect x="8" y="64" width="132" height="16" fill={WATER} />
      <rect x="95" y="31" width="35" height="10" fill={WATER} />
      <rect x="134" y="31" width="6" height="33" fill={WATER} />
      <polygon points="33.9,70 44.3,64 32.8,64" fill={WATER} />
      <polygon points="47.7,62 58.1,56 46.7,56" fill={WATER} />
      <polygon points="61.6,54 72,48 60.5,48" fill={WATER} />
      <polygon points="75.4,46 85.8,40 74.4,40" fill={WATER} />
      <polyline points="6,48 6,82 142,82 142,26" fill="none" stroke={RIGID} stroke-width="3" />
      <polyline points="93,36 93,42 132,42 132,30" fill="none" stroke={RIGID} stroke-width="3" />
      <line x1="20" y1="78" x2="93" y2="36" stroke={RIGID} stroke-width="3" />
      <line x1="15" y1="69.3" x2="99" y2="20.8" stroke={RIGID} stroke-width="3" />
      <line x1="20" y1="78" x2="15" y2="69.3" stroke={RIGID} stroke-width="3" />
      <line x1="99" y1="20.8" x2="104" y2="29.5" stroke={RIGID} stroke-width="3" />
      <line x1="33.9" y1="70" x2="32" y2="59.5" stroke={RIGID} stroke-width="2" />
      <line x1="47.7" y1="62" x2="45.9" y2="51.5" stroke={RIGID} stroke-width="2" />
      <line x1="61.6" y1="54" x2="59.7" y2="43.5" stroke={RIGID} stroke-width="2" />
      <line x1="75.4" y1="46" x2="73.6" y2="35.5" stroke={RIGID} stroke-width="2" />
      <line x1="89.3" y1="38" x2="87.4" y2="27.5" stroke={RIGID} stroke-width="2" />
    </PreviewFrame>
  );
}
