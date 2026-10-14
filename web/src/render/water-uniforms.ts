/**
 * Water look constants and the uniform setter for the Water programs.
 *
 * The constants were tuned by eye from the UI-SPEC starting values on Dam
 * Break, Fountain and First Pour (43-06): higher foam speeds keep spreading
 * floor sheets blue while jets stay white, and a narrower, weaker highlight
 * turns per-particle glitter into small glints.
 */
import { CANVAS_RGB } from "./canvas";
import { PARTICLE_KERNEL_REACH } from "./contour";
import {
  DENSITY_SHADE_END,
  DENSITY_SHADE_FLOOR,
  DENSITY_SHADE_START,
} from "./density-shade";
import type { WaterPrograms } from "./water-programs";

type Vec3 = readonly [number, number, number];

/** World speed (m/s) where foam starts: the smoothstep lower edge. */
export const WATER_FOAM_SPEED_LOW = 3.5;
/** World speed (m/s) where foam saturates: the smoothstep upper edge. */
export const WATER_FOAM_SPEED_HIGH = 9.0;
/** Strongest blend toward the foam color, kept below flat white. */
export const WATER_FOAM_MAX = 0.85;
/** Largest backdrop offset in CSS px at the steepest field gradient. */
export const WATER_REFRACTION_PX = 10;
/** Steepness of the normal derived from the field gradient. */
export const WATER_NORMAL_SCALE = 6.0;
/** Beer-Lambert absorption per kernel-sum unit of thickness. */
export const WATER_ABSORPTION = 0.9;
/** Blinn-Phong exponent of the highlight. */
export const WATER_SHININESS = 64;
/** Highlight intensity added on top of the body color. */
export const WATER_SPECULAR_STRENGTH = 0.4;

const LIGHT: Vec3 = [-0.35, 0.55, 0.76];
const LIGHT_LENGTH = Math.hypot(...LIGHT);
/** Fixed light from the upper left, normalized, in clip orientation (y up). */
export const WATER_LIGHT_DIR: Vec3 = [
  LIGHT[0] / LIGHT_LENGTH,
  LIGHT[1] / LIGHT_LENGTH,
  LIGHT[2] / LIGHT_LENGTH,
];
/** Foam color `#F4F7FA` (theme foreground) as 0..1 RGB. */
export const WATER_FOAM_COLOR: Vec3 = [0xf4 / 255, 0xf7 / 255, 0xfa / 255];
/** Canvas background as 0..1 RGB, derived from the shared `CANVAS_COLOR`. */
export const WATER_BACKGROUND: Vec3 = [
  CANVAS_RGB[0] / 255,
  CANVAS_RGB[1] / 255,
  CANVAS_RGB[2] / 255,
];

/** Per-frame inputs of the shade pass. */
export type WaterShadeState = {
  readonly densityShading: boolean;
  readonly width: number;
  readonly height: number;
  readonly pixelRatio: number;
  readonly hasBackdrop: boolean;
};

/**
 * Looks up every location once and sets each uniform only when its value
 * changes. Each setter expects its program to be in use.
 */
export function createWaterUniforms(
  gl: WebGL2RenderingContext,
  programs: WaterPrograms,
) {
  const at = (program: WebGLProgram, name: string) =>
    gl.getUniformLocation(program, name);
  const { point, shade } = programs;
  const resolution = at(point, "uResolution");
  const pointScale = at(point, "uPointScale");
  const maxPoint = at(point, "uMaxPointSize");
  const foamLow = at(point, "uFoamLow");
  const foamHigh = at(point, "uFoamHigh");
  const shadeAt = {
    field: at(shade, "uField"),
    foam: at(shade, "uFoam"),
    backdrop: at(shade, "uBackdrop"),
    hasBackdrop: at(shade, "uHasBackdrop"),
    texel: at(shade, "uTexel"),
    background: at(shade, "uBackground"),
    refraction: at(shade, "uRefraction"),
    normalScale: at(shade, "uNormalScale"),
    shininess: at(shade, "uShininess"),
    specular: at(shade, "uSpecular"),
    absorption: at(shade, "uAbsorption"),
    lightDir: at(shade, "uLightDir"),
    foamMax: at(shade, "uFoamMax"),
    foamColor: at(shade, "uFoamColor"),
    shadeStart: at(shade, "uShadeStart"),
    shadeEnd: at(shade, "uShadeEnd"),
    shadeFloor: at(shade, "uShadeFloor"),
  };
  let maybePointKey: string | undefined;
  let maybeShadeSizeKey: string | undefined;
  let maybeDensity: boolean | undefined;
  let maybeHasBackdrop: boolean | undefined;

  const setStaticShade = () => {
    gl.uniform1i(shadeAt.field, 0);
    gl.uniform1i(shadeAt.foam, 1);
    gl.uniform1i(shadeAt.backdrop, 2);
    gl.uniform3f(shadeAt.background, ...WATER_BACKGROUND);
    gl.uniform1f(shadeAt.normalScale, WATER_NORMAL_SCALE);
    gl.uniform1f(shadeAt.shininess, WATER_SHININESS);
    gl.uniform1f(shadeAt.specular, WATER_SPECULAR_STRENGTH);
    gl.uniform1f(shadeAt.absorption, WATER_ABSORPTION);
    gl.uniform3f(shadeAt.lightDir, ...WATER_LIGHT_DIR);
    gl.uniform1f(shadeAt.foamMax, WATER_FOAM_MAX);
    gl.uniform3f(shadeAt.foamColor, ...WATER_FOAM_COLOR);
    gl.uniform1f(shadeAt.shadeStart, DENSITY_SHADE_START);
    gl.uniform1f(shadeAt.shadeEnd, DENSITY_SHADE_END);
  };

  return {
    point(width: number, height: number, ratio: number, maxPointSize: number) {
      if (maybePointKey === undefined) {
        gl.uniform1f(foamLow, WATER_FOAM_SPEED_LOW);
        gl.uniform1f(foamHigh, WATER_FOAM_SPEED_HIGH);
      }
      const key = [width, height, ratio, maxPointSize].join(";");
      if (key === maybePointKey) return;
      gl.uniform2f(resolution, width, height);
      gl.uniform1f(pointScale, PARTICLE_KERNEL_REACH * 2 * ratio);
      gl.uniform1f(maxPoint, maxPointSize);
      maybePointKey = key;
    },
    shade(state: WaterShadeState) {
      if (maybeShadeSizeKey === undefined) setStaticShade();
      const { width, height, pixelRatio } = state;
      const sizeKey = [width, height, pixelRatio].join(";");
      if (sizeKey !== maybeShadeSizeKey) {
        gl.uniform2f(shadeAt.texel, 1 / width, 1 / height);
        const offset = WATER_REFRACTION_PX * pixelRatio;
        gl.uniform2f(shadeAt.refraction, offset / width, offset / height);
        maybeShadeSizeKey = sizeKey;
      }
      if (state.densityShading !== maybeDensity) {
        // UI-D43: the Density shading control darkens the water body too.
        const floor = state.densityShading ? DENSITY_SHADE_FLOOR : 1;
        gl.uniform1f(shadeAt.shadeFloor, floor);
        maybeDensity = state.densityShading;
      }
      if (state.hasBackdrop !== maybeHasBackdrop) {
        gl.uniform1f(shadeAt.hasBackdrop, state.hasBackdrop ? 1 : 0);
        maybeHasBackdrop = state.hasBackdrop;
      }
    },
  };
}
