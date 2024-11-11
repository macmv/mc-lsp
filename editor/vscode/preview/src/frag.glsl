#version 300 es
precision highp float;

in vec2 f_uv;

out vec4 frag;

void main() {
  frag = vec4(f_uv.x, f_uv.y, 1, 1);
}
