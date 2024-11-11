#version 300 es

uniform mat4 proj;
uniform mat4 view;
uniform mat4 model;

in vec4 pos;

out vec2 f_uv;

void main() {
  gl_Position = proj * view * model * pos;

  f_uv = pos.xy;
}
