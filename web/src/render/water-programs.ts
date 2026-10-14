/**
 * Links the Water programs. This is the only module that imports GLSL, so
 * the Bun export graph (canvas, mode, static layer) never sees a `?raw`
 * import.
 */
import { linkProgram } from "./gl-program";
import waterPointFragment from "./water-shaders/water-point.frag.glsl?raw";
import waterPointVertex from "./water-shaders/water-point.vert.glsl?raw";
import waterShadeFragment from "./water-shaders/water-shade.frag.glsl?raw";
import waterShadeVertex from "./water-shaders/water-shade.vert.glsl?raw";

/** The Water field (point splat) and shade programs. */
export type WaterPrograms = {
  readonly point: WebGLProgram;
  readonly shade: WebGLProgram;
};

/**
 * Compiles and links both Water programs. Returns undefined, after deleting
 * any program already linked, when either one fails.
 */
export function linkWaterPrograms(
  gl: WebGL2RenderingContext,
): WaterPrograms | undefined {
  const point = linkProgram(gl, waterPointVertex, waterPointFragment);
  if (point === undefined) {
    return undefined;
  }
  const shade = linkProgram(gl, waterShadeVertex, waterShadeFragment);
  if (shade === undefined) {
    gl.deleteProgram(point);
    return undefined;
  }
  return { point, shade };
}
