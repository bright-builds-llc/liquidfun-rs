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
