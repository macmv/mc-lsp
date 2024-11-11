#version 300 es

uniform mat4 proj;
uniform mat4 view;
uniform mat4 model;

in vec3 pos;
in vec2 uv;
in vec3 normal;

out vec2 f_uv;
out vec3 f_normal;

void main() {
  gl_Position = proj * view * model * vec4(pos, 1);

  f_uv = uv;
  f_normal = normal;
}
