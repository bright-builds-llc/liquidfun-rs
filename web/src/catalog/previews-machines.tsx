/** Static SVG previews for the machine scenes. */
import {
  DRIP,
  PreviewFrame,
  RIGID,
  WATER,
} from "./preview-kit";

export function WaveTankPreview() {
  return (
    <PreviewFrame>
      <line x1="28" y1="22" x2="28" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="74" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="22" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="58" x2="64" y2="58" stroke={RIGID} stroke-width="3" />
      <line x1="64" y1="58" x2="64" y2="74" stroke={RIGID} stroke-width="3" />
      <rect x="68" y="52" width="60" height="22" fill={WATER} />
    </PreviewFrame>
  );
}

export function TeslaValvePreview() {
  const boundaries = [
    "M79.40 5.10 L79.40 8.70 A6.48 6.48 0 0 1 79.01 10.92 L78.90 11.22 A4.32 4.32 0 0 0 78.90 14.18 L79.53 15.90 A4.14 4.14 0 0 1 87.31 18.73 L78.90 41.82 A4.32 4.32 0 0 0 78.90 44.78 L79.53 46.50 A4.14 4.14 0 0 1 87.31 49.33 L78.90 72.42 A4.32 4.32 0 0 0 78.90 75.38 L79.01 75.68 A6.48 6.48 0 0 1 79.40 77.90 L79.40 81.50",
    "M77.24 5.10 L77.24 8.70 A4.32 4.32 0 0 1 76.98 10.18 L76.87 10.48 A6.48 6.48 0 0 0 76.87 14.92 L81.10 26.52 A4.32 4.32 0 0 1 81.10 29.48 L80.47 31.20 A4.14 4.14 0 0 0 72.69 34.03 L81.10 57.12 A4.32 4.32 0 0 1 81.10 60.08 L80.47 61.80 A4.14 4.14 0 0 0 72.69 64.63 L76.98 76.42 A4.32 4.32 0 0 1 77.24 77.90 L77.24 81.50",
  ];
  const islands = [
    "M81.56 16.63 A1.98 1.98 0 0 1 85.28 17.99 L82.78 24.84 L80.68 19.05 L81.56 16.63 Z",
    "M78.44 31.93 A1.98 1.98 0 0 0 74.72 33.29 L77.22 40.14 L79.32 34.35 L78.44 31.93 Z",
    "M81.56 47.23 A1.98 1.98 0 0 1 85.28 48.59 L82.78 55.44 L80.68 49.65 L81.56 47.23 Z",
    "M78.44 62.53 A1.98 1.98 0 0 0 74.72 63.89 L77.22 70.74 L79.32 64.95 L78.44 62.53 Z",
  ];
  const water: readonly [number, number][] = [[78.05, 8.41], [78.31, 10.85], [77.38, 12.84], [78.07, 14.76], [78.4, 17.3], [79.48, 19.88], [80.26, 22.35], [81.43, 24.77], [82.47, 27.09], [83, 28.85], [82.06, 31.03], [80.72, 33.59], [80, 36.06], [79.17, 38.52], [78.49, 41.1], [76.59, 43.08], [77.67, 44.91], [78.71, 47.22], [80.05, 49.81], [80.12, 52.36], [80.97, 54.91], [82.27, 57.27], [82.83, 59.2], [82.1, 61.2], [81.41, 63.58], [80.32, 66.03], [79.33, 68.54], [78.65, 71.07], [77.17, 73.51], [77.56, 75.2], [78.72, 76.98], [78.34, 79.17]];
  return (
    <PreviewFrame>
      <g fill="none" stroke={RIGID} stroke-width="1.08" stroke-linejoin="round" stroke-linecap="round">
        {boundaries.map((path) => <path d={path} />)}
      </g>
      <g fill={RIGID} stroke={RIGID} stroke-width="1.08" stroke-linejoin="round">
        {islands.map((path) => <path d={path} />)}
      </g>
      <g fill={WATER}>
        {water.map(([cx, cy]) => <circle cx={cx} cy={cy} r="0.16" />)}
      </g>
    </PreviewFrame>
  );
}

export function WashingMachinePreview() {
  return (
    <PreviewFrame>
      <circle cx="80" cy="45" r="32" fill="none" stroke={RIGID} stroke-width="4" />
      <line x1="57" y1="22" x2="69" y2="34" stroke={RIGID} stroke-width="4" />
      <line x1="103" y1="22" x2="91" y2="34" stroke={RIGID} stroke-width="4" />
      <line x1="57" y1="68" x2="69" y2="56" stroke={RIGID} stroke-width="4" />
      <line x1="103" y1="68" x2="91" y2="56" stroke={RIGID} stroke-width="4" />
      <ellipse cx="80" cy="58" rx="18" ry="10" fill={WATER} />
    </PreviewFrame>
  );
}

export function StackedDripPreview() {
  const DRIP = "#40C4C4";
  return (
    <PreviewFrame>
      <line x1="18" y1="12" x2="18" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="18" y1="78" x2="142" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="142" y1="12" x2="142" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="112" y1="18" x2="112" y2="78" stroke={RIGID} stroke-width="3" />
      <rect x="36" y="18" width="48" height="10" fill={DRIP} />
      <rect x="56" y="28" width="5" height="8" fill={DRIP} />
      <line x1="40" y1="40" x2="88" y2="46" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="54" x2="76" y2="60" stroke={RIGID} stroke-width="3" />
      <line x1="40" y1="68" x2="88" y2="74" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

export function LiquidBubblerPreview() {
  return (
    <PreviewFrame>
      <line x1="28" y1="14" x2="28" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="78" x2="132" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="132" y1="14" x2="132" y2="78" stroke={RIGID} stroke-width="3" />
      <line x1="104" y1="20" x2="104" y2="66" stroke={RIGID} stroke-width="3" />
      <line x1="46" y1="32" x2="104" y2="32" stroke={RIGID} stroke-width="3" />
      <line x1="28" y1="50" x2="80" y2="50" stroke={RIGID} stroke-width="3" />
      <line x1="94" y1="50" x2="104" y2="50" stroke={RIGID} stroke-width="3" />
      <line x1="46" y1="68" x2="104" y2="68" stroke={RIGID} stroke-width="3" />
      <rect x="36" y="22" width="64" height="8" fill={DRIP} />
      <circle cx="37" cy="41" r="4" fill="none" stroke={RIGID} stroke-width="2" />
      <line x1="37" y1="37" x2="37" y2="45" stroke={RIGID} stroke-width="1.5" />
      <circle cx="87" cy="59" r="4" fill="none" stroke={RIGID} stroke-width="2" />
      <line x1="83" y1="59" x2="91" y2="59" stroke={RIGID} stroke-width="1.5" />
      <circle cx="37" cy="73" r="4" fill="none" stroke={RIGID} stroke-width="2" />
      <line x1="37" y1="69" x2="37" y2="77" stroke={RIGID} stroke-width="1.5" />
      <line x1="104" y1="75" x2="132" y2="74" stroke={RIGID} stroke-width="3" />
    </PreviewFrame>
  );
}

export function HydraulicFountainPreview() {
  return (
    <PreviewFrame>
      <line x1="20" y1="12" x2="20" y2="78" stroke={RIGID} stroke-width="8" />
      <line x1="20" y1="78" x2="140" y2="78" stroke={RIGID} stroke-width="8" />
      <line x1="140" y1="12" x2="140" y2="78" stroke={RIGID} stroke-width="8" />
      <rect x="28" y="56" width="104" height="18" fill={WATER} />
      <polygon points="16,50 72,54 72,40 16,32" fill={RIGID} />
      <polygon points="144,50 88,54 88,40 144,32" fill={RIGID} />
      <rect x="72" y="14" width="16" height="40" fill={WATER} />
    </PreviewFrame>
  );
}
