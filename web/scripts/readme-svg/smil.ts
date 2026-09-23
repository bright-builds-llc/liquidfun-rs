/**
 * Samples the SMIL subset emitted by `buildAnimatedSvg`.
 *
 * The README clips use linear, indefinite `<animate>` values. Sampling at a
 * time evaluates those keyframes the way the SVG animation does, then writes
 * a static document a rasterizer can draw.
 */

export type SvgAttribute = {
  readonly name: string;
  readonly value: string;
};

export type SvgContent =
  | { readonly kind: "element"; readonly element: SvgElement }
  | { readonly kind: "text"; readonly text: string };

export type SvgElement = {
  readonly tag: string;
  readonly attributes: readonly SvgAttribute[];
  readonly content: readonly SvgContent[];
};

export type ParsedAnimatedSvg = {
  readonly root: SvgElement;
  readonly durationSeconds: number;
  readonly width: number;
  readonly height: number;
};

export type SampleAnimatedSvgOptions = {
  readonly fontFamily?: string;
};

type Rgba = {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
  readonly alpha: number;
};

const KEYFRAME_SNAP = 1e-8;

/** Parses one standalone animated SVG from the README exporter. */
export function parseAnimatedSvg(source: string): ParsedAnimatedSvg {
  const scanner = new SvgScanner(source);
  const root = parseElement(scanner);
  if (root.tag !== "svg") {
    throw new Error("Animated SVG root must be <svg>.");
  }
  if (!scanner.isDone()) {
    throw new Error("Unexpected trailing content after </svg>.");
  }

  const durations = collectDurations(root);
  return {
    root,
    durationSeconds: sharedDuration(durations),
    width: requiredPositiveInteger(root, "width"),
    height: requiredPositiveInteger(root, "height"),
  };
}

/** Renders one static SVG at `timeSeconds` on the SMIL timeline. */
export function renderParsedSvg(
  parsed: ParsedAnimatedSvg,
  timeSeconds: number,
  options?: SampleAnimatedSvgOptions,
): string {
  if (!Number.isFinite(timeSeconds) || timeSeconds < 0) {
    throw new Error("SVG sample time must be a non-negative finite number.");
  }

  const sampled = omitOffscreenShapes(
    sampleElement(parsed.root, timeSeconds, options ?? {}),
    parsed.width,
    parsed.height,
  );
  return serializeElement(sampled);
}

/**
 * Drops circles and lines that miss the canvas.
 *
 * resvg 2.6.2 aborts when a shape's bounds do not intersect the viewport.
 * Particles that leave the frame are invisible there, so the raster omits them.
 */
function omitOffscreenShapes(element: SvgElement, width: number, height: number): SvgElement {
  const content: SvgContent[] = [];
  for (const item of element.content) {
    if (item.kind === "text") {
      content.push(item);
      continue;
    }
    if (!shapeIntersectsViewport(item.element, width, height)) {
      continue;
    }
    content.push({
      kind: "element",
      element: omitOffscreenShapes(item.element, width, height),
    });
  }
  return { tag: element.tag, attributes: element.attributes, content };
}

function shapeIntersectsViewport(element: SvgElement, width: number, height: number): boolean {
  const margin = 1;
  if (element.tag === "circle") {
    const cx = maybeNumberAttribute(element, "cx");
    const cy = maybeNumberAttribute(element, "cy");
    const radius = maybeNumberAttribute(element, "r");
    if (cx === undefined || cy === undefined || radius === undefined) {
      return true;
    }
    const reach = radius + (maybeNumberAttribute(element, "stroke-width") ?? 0) + margin;
    return cx + reach >= 0 && cy + reach >= 0 && cx - reach <= width && cy - reach <= height;
  }

  if (element.tag === "line") {
    const x1 = maybeNumberAttribute(element, "x1");
    const y1 = maybeNumberAttribute(element, "y1");
    const x2 = maybeNumberAttribute(element, "x2");
    const y2 = maybeNumberAttribute(element, "y2");
    if (x1 === undefined || y1 === undefined || x2 === undefined || y2 === undefined) {
      return true;
    }
    const pad = (maybeNumberAttribute(element, "stroke-width") ?? 0) + margin;
    const minX = Math.min(x1, x2) - pad;
    const maxX = Math.max(x1, x2) + pad;
    const minY = Math.min(y1, y2) - pad;
    const maxY = Math.max(y1, y2) + pad;
    return maxX >= 0 && maxY >= 0 && minX <= width && minY <= height;
  }

  return true;
}

function maybeNumberAttribute(element: SvgElement, name: string): number | undefined {
  const attribute = element.attributes.find((candidate) => candidate.name === name);
  if (attribute === undefined) {
    return undefined;
  }
  const value = Number(attribute.value);
  return Number.isFinite(value) ? value : undefined;
}

/** Parses and samples one animated SVG at `timeSeconds`. */
export function sampleAnimatedSvg(
  source: string,
  timeSeconds: number,
  options?: SampleAnimatedSvgOptions,
): string {
  return renderParsedSvg(parseAnimatedSvg(source), timeSeconds, options);
}

function sampleElement(
  element: SvgElement,
  timeSeconds: number,
  options: SampleAnimatedSvgOptions,
): SvgElement {
  const animated = new Map<string, string>();
  for (const item of element.content) {
    if (item.kind !== "element" || item.element.tag !== "animate") {
      continue;
    }
    const name = requiredAttribute(item.element, "attributeName");
    if (animated.has(name)) {
      throw new Error(`Duplicate SMIL animation for ${name}.`);
    }
    animated.set(name, sampleAnimate(item.element, timeSeconds));
  }

  const maybeFontFamily = options.fontFamily;
  if (
    maybeFontFamily !== undefined &&
    (animated.has("font-family") ||
      element.attributes.some((attribute) => attribute.name === "font-family"))
  ) {
    animated.set("font-family", maybeFontFamily);
  }

  const attributes = element.attributes.map((attribute) => {
    const replacement = animated.get(attribute.name);
    if (replacement === undefined) {
      return attribute;
    }
    return { name: attribute.name, value: replacement };
  });
  for (const [name, value] of animated) {
    if (attributes.some((attribute) => attribute.name === name)) {
      continue;
    }
    attributes.push({ name, value });
  }

  const content: SvgContent[] = [];
  for (const item of element.content) {
    if (item.kind === "text") {
      content.push(item);
      continue;
    }
    if (item.element.tag === "animate") {
      continue;
    }
    content.push({
      kind: "element",
      element: sampleElement(item.element, timeSeconds, options),
    });
  }

  return { tag: element.tag, attributes, content };
}

function sampleAnimate(element: SvgElement, timeSeconds: number): string {
  const name = requiredAttribute(element, "attributeName");
  const values = splitValues(name, requiredAttribute(element, "values"));
  const durationSeconds = parseDuration(requiredAttribute(element, "dur"));
  try {
    return interpolateValues(values, durationSeconds, timeSeconds);
  } catch (error) {
    const detail = error instanceof Error ? error.message : "interpolation failed";
    throw new Error(`Could not interpolate ${name}: ${detail}`);
  }
}

function splitValues(name: string, raw: string): readonly string[] {
  if (raw.length === 0) {
    throw new Error(`SMIL animation for ${name} has an empty values list.`);
  }
  const values = raw.split(";");
  if (values.some((value) => value.length === 0)) {
    throw new Error(`SMIL animation for ${name} has an empty value.`);
  }
  return values;
}

function interpolateValues(
  values: readonly string[],
  durationSeconds: number,
  timeSeconds: number,
): string {
  const first = values[0];
  if (first === undefined) {
    throw new Error("SMIL values list is empty.");
  }
  if (values.length === 1 || values.every((value) => value === first)) {
    return first;
  }

  const sample = keyframeSample(values.length, durationSeconds, timeSeconds);
  const left = values[sample.index];
  if (left === undefined) {
    throw new Error("SMIL keyframe index is outside the values list.");
  }
  if (sample.local === 0) {
    return left;
  }
  const right = values[sample.index + 1];
  if (right === undefined) {
    throw new Error("SMIL keyframe index is outside the values list.");
  }
  return interpolatePair(left, right, sample.local);
}

function keyframeSample(
  valueCount: number,
  durationSeconds: number,
  timeSeconds: number,
): { readonly index: number; readonly local: number } {
  const span = valueCount - 1;
  if (span <= 0 || durationSeconds <= 0) {
    return { index: 0, local: 0 };
  }

  const wrapped = timeSeconds % durationSeconds;
  const position = (wrapped / durationSeconds) * span;
  const nearest = Math.round(position);
  if (Math.abs(position - nearest) <= KEYFRAME_SNAP) {
    return { index: Math.min(Math.max(nearest, 0), span), local: 0 };
  }

  const index = Math.floor(position);
  if (index >= span) {
    return { index: span, local: 0 };
  }
  return { index, local: position - index };
}

function interpolatePair(left: string, right: string, local: number): string {
  const maybeLeftNumber = maybeParseNumber(left);
  const maybeRightNumber = maybeParseNumber(right);
  if (maybeLeftNumber !== undefined && maybeRightNumber !== undefined) {
    return formatSampledNumber(maybeLeftNumber + (maybeRightNumber - maybeLeftNumber) * local);
  }

  const maybeLeftColor = maybeParseColor(left);
  const maybeRightColor = maybeParseColor(right);
  if (maybeLeftColor === undefined || maybeRightColor === undefined) {
    throw new Error(`unsupported values "${left}" and "${right}".`);
  }
  if (maybeLeftColor === "none" || maybeRightColor === "none") {
    throw new Error(`cannot blend none with a paint value ("${left}" to "${right}").`);
  }

  return formatRgba({
    red: maybeLeftColor.red + (maybeRightColor.red - maybeLeftColor.red) * local,
    green: maybeLeftColor.green + (maybeRightColor.green - maybeLeftColor.green) * local,
    blue: maybeLeftColor.blue + (maybeRightColor.blue - maybeLeftColor.blue) * local,
    alpha: maybeLeftColor.alpha + (maybeRightColor.alpha - maybeLeftColor.alpha) * local,
  });
}

function maybeParseNumber(value: string): number | undefined {
  if (!/^-?(?:\d+\.\d+|\d+|\.\d+)$/.test(value)) {
    return undefined;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : undefined;
}

function maybeParseColor(value: string): Rgba | "none" | undefined {
  if (value === "none") {
    return "none";
  }

  const rgba = /^rgba\((\d+),(\d+),(\d+),(\d+(?:\.\d+)?)\)$/.exec(value);
  if (rgba !== null) {
    return rgbaChannels(rgba[1], rgba[2], rgba[3], rgba[4]);
  }

  const hex = /^#([0-9a-fA-F]{6})$/.exec(value);
  if (hex !== null) {
    const digits = hex[1];
    if (digits === undefined) {
      return undefined;
    }
    return {
      red: Number.parseInt(digits.slice(0, 2), 16),
      green: Number.parseInt(digits.slice(2, 4), 16),
      blue: Number.parseInt(digits.slice(4, 6), 16),
      alpha: 1,
    };
  }

  return undefined;
}

function rgbaChannels(
  red: string | undefined,
  green: string | undefined,
  blue: string | undefined,
  alpha: string | undefined,
): Rgba | undefined {
  if (red === undefined || green === undefined || blue === undefined || alpha === undefined) {
    return undefined;
  }
  return {
    red: Number(red),
    green: Number(green),
    blue: Number(blue),
    alpha: Number(alpha),
  };
}

function formatSampledNumber(value: number): string {
  const rounded = Math.round(value * 1000) / 1000;
  if (Object.is(rounded, -0)) {
    return "0";
  }
  return String(rounded);
}

function formatRgba(color: Rgba): string {
  return `rgba(${clampChannel(color.red)},${clampChannel(color.green)},${clampChannel(color.blue)},${formatSampledNumber(color.alpha)})`;
}

function clampChannel(value: number): number {
  return Math.min(255, Math.max(0, Math.round(value)));
}

function collectDurations(element: SvgElement): number[] {
  const durations: number[] = [];
  visitElements(element, (current) => {
    if (current.tag !== "animate") {
      return;
    }
    assertLinearIndefinite(current);
    durations.push(parseDuration(requiredAttribute(current, "dur")));
  });
  return durations;
}

function visitElements(element: SvgElement, visit: (element: SvgElement) => void): void {
  visit(element);
  for (const item of element.content) {
    if (item.kind === "element") {
      visitElements(item.element, visit);
    }
  }
}

function sharedDuration(durations: readonly number[]): number {
  const first = durations[0];
  if (first === undefined) {
    return 0;
  }
  if (durations.some((duration) => duration !== first)) {
    throw new Error("Animated SVG mixes SMIL durations.");
  }
  return first;
}

function assertLinearIndefinite(element: SvgElement): void {
  const calcMode = requiredAttribute(element, "calcMode");
  if (calcMode !== "linear") {
    throw new Error(`Unsupported SMIL calcMode "${calcMode}".`);
  }
  const repeatCount = requiredAttribute(element, "repeatCount");
  if (repeatCount !== "indefinite") {
    throw new Error(`Unsupported SMIL repeatCount "${repeatCount}".`);
  }
}

function parseDuration(raw: string): number {
  const match = /^(\d+(?:\.\d+)?)(ms|s)$/.exec(raw);
  if (match === null) {
    throw new Error(`Unsupported SMIL duration "${raw}".`);
  }
  const magnitudeText = match[1];
  const unit = match[2];
  const magnitude = magnitudeText === undefined ? Number.NaN : Number(magnitudeText);
  if (!Number.isFinite(magnitude) || magnitude <= 0) {
    throw new Error(`Unsupported SMIL duration "${raw}".`);
  }
  return unit === "ms" ? magnitude / 1000 : magnitude;
}

function requiredPositiveInteger(element: SvgElement, name: string): number {
  const raw = requiredAttribute(element, name);
  if (!/^\d+$/.test(raw)) {
    throw new Error(`SVG ${name} must be a positive integer.`);
  }
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`SVG ${name} must be a positive integer.`);
  }
  return value;
}

function requiredAttribute(element: SvgElement, name: string): string {
  const attribute = element.attributes.find((candidate) => candidate.name === name);
  if (attribute === undefined) {
    throw new Error(`SVG <${element.tag}> is missing ${name}.`);
  }
  return attribute.value;
}

function serializeElement(element: SvgElement): string {
  const attributes = element.attributes
    .map((attribute) => ` ${attribute.name}="${escapeAttribute(attribute.value)}"`)
    .join("");
  if (element.content.length === 0) {
    return `<${element.tag}${attributes}/>`;
  }

  const inner = element.content
    .map((item) => (item.kind === "text" ? item.text : serializeElement(item.element)))
    .join("");
  return `<${element.tag}${attributes}>${inner}</${element.tag}>`;
}

function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;");
}

class SvgScanner {
  private index = 0;

  constructor(private readonly source: string) {}

  isDone(): boolean {
    this.skipWhitespace();
    return this.index >= this.source.length;
  }

  parseElement(): SvgElement {
    this.skipWhitespace();
    this.expect("<");
    if (this.startsWith("/")) {
      throw new Error(`Unexpected closing tag at SVG offset ${this.index}.`);
    }
    if (this.startsWith("!") || this.startsWith("?")) {
      throw new Error(`Unsupported SVG markup at offset ${this.index}.`);
    }

    const tag = this.readName();
    const attributes = this.parseAttributes();
    this.skipWhitespace();
    if (this.startsWith("/>")) {
      this.expect("/>");
      return { tag, attributes, content: [] };
    }

    this.expect(">");
    const content: SvgContent[] = [];
    const closing = `</${tag}>`;
    while (!this.startsWith(closing)) {
      if (this.index >= this.source.length) {
        throw new Error(`Unclosed <${tag}>.`);
      }
      if (this.startsWith("<")) {
        content.push({ kind: "element", element: this.parseElement() });
        continue;
      }
      content.push({ kind: "text", text: this.readText() });
    }
    this.expect(closing);
    return { tag, attributes, content };
  }

  private parseAttributes(): SvgAttribute[] {
    const attributes: SvgAttribute[] = [];
    while (true) {
      this.skipWhitespace();
      if (this.startsWith("/>") || this.startsWith(">") || this.index >= this.source.length) {
        return attributes;
      }
      const name = this.readName();
      this.skipWhitespace();
      this.expect("=");
      this.skipWhitespace();
      const value = this.readQuoted();
      if (attributes.some((attribute) => attribute.name === name)) {
        throw new Error(`Duplicate SVG attribute ${name}.`);
      }
      attributes.push({ name, value });
    }
  }

  private readQuoted(): string {
    this.expect('"');
    const start = this.index;
    const end = this.source.indexOf('"', start);
    if (end === -1) {
      throw new Error("Unclosed SVG attribute.");
    }
    this.index = end + 1;
    return this.source.slice(start, end);
  }

  private readName(): string {
    const start = this.index;
    while (this.index < this.source.length && isNameCharacter(this.source[this.index] ?? "")) {
      this.index += 1;
    }
    if (start === this.index) {
      throw new Error(`Expected a name at SVG offset ${start}.`);
    }
    return this.source.slice(start, this.index);
  }

  private readText(): string {
    const start = this.index;
    const end = this.source.indexOf("<", start);
    if (end === -1) {
      throw new Error("Unclosed SVG text.");
    }
    this.index = end;
    return this.source.slice(start, end);
  }

  private skipWhitespace(): void {
    while (this.index < this.source.length && isWhitespace(this.source[this.index] ?? "")) {
      this.index += 1;
    }
  }

  private startsWith(token: string): boolean {
    return this.source.startsWith(token, this.index);
  }

  private expect(token: string): void {
    if (!this.startsWith(token)) {
      throw new Error(`Expected "${token}" at SVG offset ${this.index}.`);
    }
    this.index += token.length;
  }
}

function parseElement(scanner: SvgScanner): SvgElement {
  return scanner.parseElement();
}

function isNameCharacter(character: string): boolean {
  const code = character.charCodeAt(0);
  return (
    (code >= 65 && code <= 90) ||
    (code >= 97 && code <= 122) ||
    (code >= 48 && code <= 57) ||
    character === ":" ||
    character === "_" ||
    character === "-"
  );
}

function isWhitespace(character: string): boolean {
  return character === " " || character === "\n" || character === "\r" || character === "\t";
}
