#version 300 es
precision highp float;

// Full-screen quad for the water shade pass.
in vec2 aClip;
out vec2 vUv;

void main() {
  vUv = aClip * 0.5 + 0.5;
  gl_Position = vec4(aClip, 0.0, 1.0);
}
