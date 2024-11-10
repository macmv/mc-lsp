console.log("zzzz");

const canvas = document.getElementById("canvas") as any;
const gl = canvas.getContext("webgl2") as WebGLRenderingContext;

const vertices = [-0.5, 0.5, 0.0, -0.5, -0.5, 0.0, 0.5, -0.5, 0.0];
const indices = [0, 1, 2];

const vertex_buffer = gl.createBuffer();
gl.bindBuffer(gl.ARRAY_BUFFER, vertex_buffer);
gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STATIC_DRAW);
gl.bindBuffer(gl.ARRAY_BUFFER, null);

const index_buffer = gl.createBuffer();
gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, index_buffer);
gl.bufferData(
  gl.ELEMENT_ARRAY_BUFFER,
  new Uint16Array(indices),
  gl.STATIC_DRAW,
);
gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);

const vertCode =
  "#version 300 es\n" +
  "in vec3 pos;\n" +
  "out vec2 uv;\n" +
  "void main() {\n" +
  "  gl_Position = vec4(pos, 1.0);\n" +
  "  uv = pos.xy;\n" +
  "}";

const v_shader = gl.createShader(gl.VERTEX_SHADER)!;
gl.shaderSource(v_shader, vertCode);
gl.compileShader(v_shader);
if (!gl.getShaderParameter(v_shader, gl.COMPILE_STATUS)) {
  throw gl.getShaderInfoLog(v_shader);
}

const fragCode =
  "#version 300 es\n" +
  "precision highp float;\n" +
  "in vec2 uv;\n" +
  "out vec4 frag;\n" +
  "void main() {\n" +
  "  frag = vec4(1.0, 0.5, 0.0, 1.0);\n" +
  "}";

const f_shader = gl.createShader(gl.FRAGMENT_SHADER)!;
gl.shaderSource(f_shader, fragCode);
gl.compileShader(f_shader);
if (!gl.getShaderParameter(f_shader, gl.COMPILE_STATUS)) {
  throw gl.getShaderInfoLog(f_shader);
}

const shaderProgram = gl.createProgram()!;
gl.attachShader(shaderProgram, v_shader);
gl.attachShader(shaderProgram, f_shader);
gl.linkProgram(shaderProgram);
gl.useProgram(shaderProgram);

gl.bindBuffer(gl.ARRAY_BUFFER, vertex_buffer);
gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, index_buffer);

const v_pos = gl.getAttribLocation(shaderProgram, "pos");
gl.vertexAttribPointer(v_pos, 3, gl.FLOAT, false, 0, 0);
gl.enableVertexAttribArray(v_pos);

gl.enable(gl.DEPTH_TEST);

gl.clearColor(0.5, 0.5, 0.5, 0.9);
gl.clear(gl.COLOR_BUFFER_BIT);
gl.viewport(0, 0, canvas.width, canvas.height);
gl.drawElements(gl.TRIANGLES, indices.length, gl.UNSIGNED_SHORT, 0);
