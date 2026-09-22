import type { RenderMode } from "../render/mode";
import type {
  ProjectedBody,
  ProjectedParticle,
  ProjectedSample,
} from "./project";

const BACKGROUND = "#071018";
const BASIN_STROKE = "#94A3B8";
const RIGID_FILL = "#334155";
const RIGID_STROKE = "#CBD5E1";
const LABEL_FILL = "#F8FAFC";
const SOLID_BODY_STROKE_WIDTH = 2;
const LABEL_FONT_FACTOR = 0.55;
const LABEL_FONT_MAX = 28;
const LABEL_FONT_MIN = 8;

export type AnimatedSvgInput = {
  readonly title: string;
  readonly samples: readonly ProjectedSample[];
  readonly durationSeconds: number;
  readonly viewportWidth: number;
  readonly viewportHeight: number;
  readonly renderMode: RenderMode;
  readonly wireframeStrokeWidth: number;
};

type AnimatedAttribute = {
  readonly name: string;
  readonly values: readonly string[];
};

function formatDecimal(value: number, places: number): string {
  const scale = 10 ** places;
  return String(Math.round(value * scale) / scale);
}

function formatCoordinate(value: number): string {
  return formatDecimal(value, 1);
}

function formatAlpha(alpha: number): string {
  return formatDecimal(alpha, 3);
}

function particleFill(particle: ProjectedParticle): string {
  return `rgba(${particle.red},${particle.green},${particle.blue},${formatAlpha(particle.alpha)})`;
}

function escapeXml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function holdDefined<T>(values: readonly (T | undefined)[]): readonly T[] | undefined {
  const maybeFirst = values.find((value) => value !== undefined);
  if (maybeFirst === undefined) {
    return undefined;
  }

  let maybeCurrent: T | undefined;
  const forward = values.map((value) => {
    if (value !== undefined) {
      maybeCurrent = value;
    }
    return maybeCurrent;
  });
  return forward.map((value) => value ?? maybeFirst);
}

function attributeMarkup(
  attribute: AnimatedAttribute,
  duration: string,
): { readonly opening: string; readonly child: string } {
  const first = attribute.values[0] ?? "";
  const opening = ` ${attribute.name}="${first}"`;
  if (attribute.values.every((value) => value === first)) {
    return { opening, child: "" };
  }

  return {
    opening,
    child: `<animate attributeName="${attribute.name}" values="${attribute.values.join(";")}" dur="${duration}" repeatCount="indefinite" calcMode="linear"/>`,
  };
}

function shapeElement(
  tag: "circle" | "line" | "text",
  attributes: readonly AnimatedAttribute[],
  duration: string,
  maybeText?: string,
): string {
  const opening = [`<${tag}`];
  const children: string[] = [];
  for (const attribute of attributes) {
    const markup = attributeMarkup(attribute, duration);
    opening.push(markup.opening);
    if (markup.child.length > 0) {
      children.push(markup.child);
    }
  }

  if (children.length === 0 && maybeText === undefined) {
    return `${opening.join("")}/>`;
  }

  const body = maybeText === undefined ? children.join("") : `${children.join("")}${maybeText}`;
  return `${opening.join("")}>${body}</${tag}>`;
}

function presenceOpacity(present: readonly boolean[]): AnimatedAttribute | undefined {
  if (present.every((value) => value)) {
    return undefined;
  }

  return {
    name: "opacity",
    values: present.map((value) => (value ? "1" : "0")),
  };
}

function withOpacity(
  attributes: AnimatedAttribute[],
  present: readonly boolean[],
): AnimatedAttribute[] {
  const maybeOpacity = presenceOpacity(present);
  if (maybeOpacity === undefined) {
    return attributes;
  }

  return [...attributes, maybeOpacity];
}

function particleElement(
  samples: readonly ProjectedSample[],
  index: number,
  duration: string,
  renderMode: RenderMode,
  strokeWidth: string,
): string | undefined {
  const present = samples.map((sample) => sample.particles[index] !== undefined);
  const maybeHeld = holdDefined(samples.map((sample) => sample.particles[index]));
  if (maybeHeld === undefined) {
    return undefined;
  }

  const colorAttributes: AnimatedAttribute[] =
    renderMode === "wireframe"
      ? [
          { name: "fill", values: maybeHeld.map(() => "none") },
          { name: "stroke", values: maybeHeld.map(particleFill) },
          { name: "stroke-width", values: maybeHeld.map(() => strokeWidth) },
        ]
      : [{ name: "fill", values: maybeHeld.map(particleFill) }];

  return shapeElement(
    "circle",
    withOpacity(
      [
        { name: "cx", values: maybeHeld.map((particle) => formatCoordinate(particle.x)) },
        { name: "cy", values: maybeHeld.map((particle) => formatCoordinate(particle.y)) },
        { name: "r", values: maybeHeld.map((particle) => formatCoordinate(particle.radius)) },
        ...colorAttributes,
      ],
      present,
    ),
    duration,
  );
}

function segmentElement(
  samples: readonly ProjectedSample[],
  index: number,
  duration: string,
  strokeWidth: string,
): string | undefined {
  const present = samples.map((sample) => sample.segments[index] !== undefined);
  const maybeHeld = holdDefined(samples.map((sample) => sample.segments[index]));
  if (maybeHeld === undefined) {
    return undefined;
  }

  return shapeElement(
    "line",
    withOpacity(
      [
        { name: "x1", values: maybeHeld.map((segment) => formatCoordinate(segment.x1)) },
        { name: "y1", values: maybeHeld.map((segment) => formatCoordinate(segment.y1)) },
        { name: "x2", values: maybeHeld.map((segment) => formatCoordinate(segment.x2)) },
        { name: "y2", values: maybeHeld.map((segment) => formatCoordinate(segment.y2)) },
        { name: "stroke", values: maybeHeld.map(() => BASIN_STROKE) },
        { name: "stroke-width", values: maybeHeld.map(() => strokeWidth) },
      ],
      present,
    ),
    duration,
  );
}

function labelFontSize(radius: number): number {
  return Math.min(radius * LABEL_FONT_FACTOR, LABEL_FONT_MAX);
}

function bodyElements(
  samples: readonly ProjectedSample[],
  index: number,
  duration: string,
  renderMode: RenderMode,
  strokeWidth: string,
): readonly string[] {
  const present = samples.map((sample) => sample.bodies[index] !== undefined);
  const maybeHeld = holdDefined(samples.map((sample) => sample.bodies[index]));
  if (maybeHeld === undefined) {
    return [];
  }

  const fill = renderMode === "wireframe" ? "none" : RIGID_FILL;
  const circle = shapeElement(
    "circle",
    withOpacity(
      [
        { name: "cx", values: maybeHeld.map((body) => formatCoordinate(body.x)) },
        { name: "cy", values: maybeHeld.map((body) => formatCoordinate(body.y)) },
        { name: "r", values: maybeHeld.map((body) => formatCoordinate(body.radius)) },
        { name: "fill", values: maybeHeld.map(() => fill) },
        { name: "stroke", values: maybeHeld.map(() => RIGID_STROKE) },
        { name: "stroke-width", values: maybeHeld.map(() => strokeWidth) },
      ],
      present,
    ),
    duration,
  );

  const maybeLabel = bodyLabel(maybeHeld, present, duration);
  if (maybeLabel === undefined) {
    return [circle];
  }

  return [circle, maybeLabel];
}

function bodyLabel(
  bodies: readonly ProjectedBody[],
  present: readonly boolean[],
  duration: string,
): string | undefined {
  const label = bodies.find((body) => body.label.length > 0)?.label ?? "";
  if (label.length === 0) {
    return undefined;
  }

  if (bodies.every((body) => labelFontSize(body.radius) < LABEL_FONT_MIN)) {
    return undefined;
  }

  return shapeElement(
    "text",
    withOpacity(
      [
        { name: "x", values: bodies.map((body) => formatCoordinate(body.x)) },
        { name: "y", values: bodies.map((body) => formatCoordinate(body.y)) },
        {
          name: "font-size",
          values: bodies.map((body) => formatCoordinate(labelFontSize(body.radius))),
        },
        { name: "fill", values: bodies.map(() => LABEL_FILL) },
        { name: "font-family", values: bodies.map(() => "sans-serif") },
        { name: "font-weight", values: bodies.map(() => "600") },
        { name: "text-anchor", values: bodies.map(() => "middle") },
        { name: "dominant-baseline", values: bodies.map(() => "middle") },
      ],
      present,
    ),
    duration,
    escapeXml(label),
  );
}

function outlineWidth(input: AnimatedSvgInput): string {
  if (input.renderMode === "wireframe") {
    return formatCoordinate(input.wireframeStrokeWidth);
  }

  return String(SOLID_BODY_STROKE_WIDTH);
}

/** Builds a standalone SMIL SVG that loops the sampled scene. */
export function buildAnimatedSvg(input: AnimatedSvgInput): string {
  if (input.samples.length === 0) {
    throw new Error("Animated SVG needs at least one sample");
  }

  const duration = `${input.durationSeconds}s`;
  const strokeWidth = outlineWidth(input);
  const particleCount = Math.max(...input.samples.map((sample) => sample.particles.length));
  const segmentCount = Math.max(...input.samples.map((sample) => sample.segments.length));
  const bodyCount = Math.max(...input.samples.map((sample) => sample.bodies.length));
  const width = formatCoordinate(input.viewportWidth);
  const height = formatCoordinate(input.viewportHeight);
  const elements = [
    `<rect width="100%" height="100%" fill="${BACKGROUND}"/>`,
    ...collectDefined(particleCount, (index) =>
      particleElement(input.samples, index, duration, input.renderMode, strokeWidth),
    ),
    ...collectDefined(segmentCount, (index) =>
      segmentElement(input.samples, index, duration, strokeWidth),
    ),
    ...collectMany(bodyCount, (index) =>
      bodyElements(input.samples, index, duration, input.renderMode, strokeWidth),
    ),
  ];

  return [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`,
    `<title>${escapeXml(input.title)}</title>`,
    ...elements,
    "</svg>",
  ].join("");
}

function collectDefined(
  count: number,
  read: (index: number) => string | undefined,
): string[] {
  const elements: string[] = [];
  for (let index = 0; index < count; index += 1) {
    const maybeElement = read(index);
    if (maybeElement !== undefined) {
      elements.push(maybeElement);
    }
  }
  return elements;
}

function collectMany(count: number, read: (index: number) => readonly string[]): string[] {
  const elements: string[] = [];
  for (let index = 0; index < count; index += 1) {
    elements.push(...read(index));
  }
  return elements;
}
