#version 300 es
precision highp float;

// Water shade pass: refraction of the walls backdrop by the field gradient,
// Beer-Lambert tint in each particle's own color, Blinn-Phong highlights and
// speed foam. Every pixel depends on the current frame only (no time input).
uniform sampler2D uField;    // unit 0: (rgb * alpha, alpha) kernel sums
uniform sampler2D uFoam;     // unit 1: (foam * alpha, alpha) kernel sums
uniform sampler2D uBackdrop; // unit 2: stable walls raster, row 0 at the top
uniform float uHasBackdrop;
uniform vec2 uTexel;
uniform vec3 uBackground;
uniform vec2 uRefraction;    // UV offset per unit of normal tilt
uniform float uNormalScale;
uniform float uShininess;
uniform float uSpecular;
uniform float uAbsorption;
uniform vec3 uLightDir;
uniform float uFoamMax;
uniform vec3 uFoamColor;
uniform float uShadeStart;
uniform float uShadeEnd;
uniform float uShadeFloor;

in vec2 vUv;
out vec4 fragColor;

vec3 backdrop(vec2 uv) {
  vec3 procedural = uBackground + 0.035 * step(0.5, fract(uv.y * 24.0));
  vec3 walls = texture(uBackdrop, vec2(uv.x, 1.0 - uv.y)).rgb;
  return mix(procedural, walls, uHasBackdrop);
}

void main() {
  vec4 sum = texture(uField, vUv);
  // Same edge as shaded blob, so the outline holds when the view falls back.
  float edge = smoothstep(0.42, 0.62, sum.a);
  if (edge <= 0.0) {
    fragColor = vec4(uBackground, 1.0);
    return;
  }

  float dx = texture(uField, vUv + vec2(uTexel.x, 0.0)).a -
    texture(uField, vUv - vec2(uTexel.x, 0.0)).a;
  float dy = texture(uField, vUv + vec2(0.0, uTexel.y)).a -
    texture(uField, vUv - vec2(0.0, uTexel.y)).a;
  vec3 normal = normalize(vec3(-dx * uNormalScale, -dy * uNormalScale, 1.0));
  vec3 refracted = backdrop(vUv + normal.xy * uRefraction);

  vec3 color = sum.rgb / max(sum.a, 0.0001);
  // Same curve as the Density shading control in shaded blob.
  color *= mix(1.0, uShadeFloor, smoothstep(uShadeStart, uShadeEnd, sum.a));
  float transmit = exp(-uAbsorption * sum.a);
  vec3 body = mix(color, refracted * mix(vec3(1.0), color, 0.5), transmit);

  vec3 halfVector = normalize(uLightDir + vec3(0.0, 0.0, 1.0));
  float spec = uSpecular * pow(max(dot(normal, halfVector), 0.0), uShininess);

  vec4 foamSum = texture(uFoam, vUv);
  float foam = clamp(foamSum.r / max(foamSum.g, 0.0001), 0.0, 1.0);
  vec3 water = mix(body, uFoamColor, foam * uFoamMax) + spec * (1.0 - foam);
  fragColor = vec4(clamp(mix(uBackground, water, edge), 0.0, 1.0), 1.0);
}
