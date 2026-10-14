#version 300 es
precision highp float;

// Splats the shaded-blob kernel into two additive targets: the color field
// and the kernel-weighted foam, so the shade pass can recover the mean foam.
in vec4 vColor;
in float vFoam;

layout(location = 0) out vec4 outField;
layout(location = 1) out vec4 outFoam;

void main() {
  vec2 point = gl_PointCoord * 2.0 - 1.0;
  float dist2 = dot(point, point);
  if (dist2 > 1.0) {
    discard;
  }
  float weight = 1.0 - dist2;
  weight = weight * weight;
  float alpha = vColor.a * weight;
  outField = vec4(vColor.rgb * alpha, alpha);
  outFoam = vec4(vFoam * alpha, alpha, 0.0, 0.0);
}
