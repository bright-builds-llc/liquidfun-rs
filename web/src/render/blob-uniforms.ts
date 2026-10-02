import { PARTICLE_KERNEL_REACH } from "./contour";
import {
  DENSITY_SHADE_END,
  DENSITY_SHADE_FLOOR,
  DENSITY_SHADE_START,
} from "./density-shade";

/** Owns program locations and updates only changed immutable render configuration. */
export function createBlobUniforms(
  gl: WebGL2RenderingContext,
  point: WebGLProgram,
  composite: WebGLProgram,
) {
  const resolution = gl.getUniformLocation(point, "uResolution");
  const pointScale = gl.getUniformLocation(point, "uPointScale");
  const maxPoint = gl.getUniformLocation(point, "uMaxPointSize");
  const field = gl.getUniformLocation(composite, "uField");
  const background = gl.getUniformLocation(composite, "uBackground");
  const shadeStart = gl.getUniformLocation(composite, "uShadeStart");
  const shadeEnd = gl.getUniformLocation(composite, "uShadeEnd");
  const shadeFloor = gl.getUniformLocation(composite, "uShadeFloor");
  let maybePointKey: string | undefined;
  let maybeDensity: boolean | undefined;
  return {
    point(width: number, height: number, ratio: number, maxPointSize: number) {
      const key = [width, height, ratio, maxPointSize].join(";");
      if (key === maybePointKey) return;
      gl.uniform2f(resolution, width, height);
      gl.uniform1f(pointScale, PARTICLE_KERNEL_REACH * 2 * ratio);
      gl.uniform1f(maxPoint, maxPointSize);
      maybePointKey = key;
    },
    composite(densityShading: boolean) {
      if (maybeDensity === undefined) {
        gl.uniform1i(field, 0);
        gl.uniform3f(background, 7 / 255, 16 / 255, 24 / 255);
        gl.uniform1f(shadeStart, DENSITY_SHADE_START);
        gl.uniform1f(shadeEnd, DENSITY_SHADE_END);
      }
      if (maybeDensity !== densityShading)
        gl.uniform1f(shadeFloor, densityShading ? DENSITY_SHADE_FLOOR : 1);
      maybeDensity = densityShading;
    },
  };
}
