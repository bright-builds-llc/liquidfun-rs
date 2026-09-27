const TAU = Math.PI * 2;

/** Screen angle of the upward vertex. Canvas y grows downward. */
const POINT_UP = -TAU / 4;

export type WireVertex = {
  readonly x: number;
  readonly y: number;
};

/**
 * Equilateral triangle whose vertices sit on the particle circle.
 *
 * The first vertex points up on the canvas. The other two follow in
 * counter-clockwise order.
 */
export function triangleWireVertices(
  x: number,
  y: number,
  radius: number,
): readonly [WireVertex, WireVertex, WireVertex] {
  return [
    vertexAt(x, y, radius, 0),
    vertexAt(x, y, radius, 1),
    vertexAt(x, y, radius, 2),
  ];
}

function vertexAt(
  x: number,
  y: number,
  radius: number,
  index: number,
): WireVertex {
  const angle = POINT_UP + (index * TAU) / 3;
  return {
    x: x + Math.cos(angle) * radius,
    y: y + Math.sin(angle) * radius,
  };
}
