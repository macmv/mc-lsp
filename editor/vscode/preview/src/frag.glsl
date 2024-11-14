#version 300 es
precision highp float;

uniform sampler2D tex;

in vec2 f_uv;
in vec3 f_normal;

out vec4 frag;

void main() {
  if (f_uv.x < 0.0 || f_uv.y < 0.0) {
    float c = mod(floor(f_uv.x * 16.0) + floor(f_uv.y * 16.0), 2.0);
    frag = vec4(c, 0.0, c, 1.0);
  } else {
    if (texture(tex, f_uv).a < 0.25) {
      discard;
    }

    frag = texture(tex, f_uv);
  }
}
