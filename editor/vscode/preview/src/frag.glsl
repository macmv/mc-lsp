#version 300 es
precision highp float;

uniform sampler2D tex;

in vec2 f_uv;
in vec3 f_normal;

out vec4 frag;

void main() {
  if (texture(tex, f_uv).a < 0.25) {
    discard;
  }

  frag = texture(tex, f_uv);
}
