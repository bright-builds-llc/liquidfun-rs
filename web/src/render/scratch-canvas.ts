const CANVAS_UNAVAILABLE = "Canvas 2D is unavailable";

type Scratch = {
  readonly canvas: HTMLCanvasElement;
  readonly context: CanvasRenderingContext2D;
};

const scratches = new Map<string, Scratch>();

function roundedSize(size: number): number {
  if (!Number.isFinite(size) || size <= 0) {
    return 1;
  }

  return Math.max(1, Math.ceil(size));
}

/** Reuses one offscreen canvas for a named particle-surface pass. */
export function scratchContext(
  name: string,
  width: number,
  height: number,
): CanvasRenderingContext2D {
  const pixelWidth = roundedSize(width);
  const pixelHeight = roundedSize(height);
  let maybeScratch = scratches.get(name);
  if (maybeScratch === undefined) {
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d", { willReadFrequently: true });
    if (context === null) {
      throw new Error(CANVAS_UNAVAILABLE);
    }
    maybeScratch = { canvas, context };
    scratches.set(name, maybeScratch);
  }

  if (
    maybeScratch.canvas.width !== pixelWidth ||
    maybeScratch.canvas.height !== pixelHeight
  ) {
    maybeScratch.canvas.width = pixelWidth;
    maybeScratch.canvas.height = pixelHeight;
  }

  return maybeScratch.context;
}

/** Clips a color buffer to a mask and blits the result in CSS pixels. */
export function compositeMasked(
  target: CanvasRenderingContext2D,
  color: HTMLCanvasElement,
  mask: HTMLCanvasElement,
  destWidth: number,
  destHeight: number,
): void {
  const out = scratchContext("surface-composite", destWidth, destHeight);
  out.clearRect(0, 0, out.canvas.width, out.canvas.height);
  out.globalCompositeOperation = "source-over";
  out.drawImage(color, 0, 0);
  out.globalCompositeOperation = "destination-in";
  out.drawImage(mask, 0, 0);
  out.globalCompositeOperation = "source-over";
  target.drawImage(out.canvas, 0, 0, destWidth, destHeight);
}
