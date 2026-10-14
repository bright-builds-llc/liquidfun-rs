#version 300 es
precision highp float;

// Water field pass: the shaded-blob projection plus a per-particle foam
// weight from the optional speed lane (a constant 0 when the lane is absent).
in vec2 aPosition;
in float aRadius;
in vec4 aColor;
in float aSpeed;

uniform vec2 uResolution;
uniform float uPointScale;
uniform float uMaxPointSize;
uniform float uFoamLow;
uniform float uFoamHigh;

out vec4 vColor;
out float vFoam;

void main() {
  vec2 clip = (aPosition / uResolution) * 2.0 - 1.0;
  clip.y = -clip.y;
  gl_Position = vec4(clip, 0.0, 1.0);
  gl_PointSize = clamp(aRadius * uPointScale, 1.0, uMaxPointSize);
  vColor = aColor;
  // Ramping here keeps the half-float foam sums small.
  vFoam = smoothstep(uFoamLow, uFoamHigh, aSpeed);
}
