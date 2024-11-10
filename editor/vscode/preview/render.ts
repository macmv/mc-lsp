console.log("zzzz");

const canvas = document.getElementById("canvas") as any;
const gl = canvas.getContext("experimental-webgl") as WebGLRenderingContext;

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
  "#version 330" +
  "in vec3 pos;" +
  "out vec2 uv;" +
  "void main() {" +
  "  gl_Position = vec4(pos, 1.0);" +
  "  uv = pos.xy;" +
  "}";

const vertShader = gl.createShader(gl.VERTEX_SHADER);
if (!vertShader) {
  throw new Error("Failed to create vertex shader");
}
gl.shaderSource(vertShader, vertCode);
gl.compileShader(vertShader);

const fragCode =
  "#version 330" +
  "in vec2 uv;" +
  "void main() {" +
  "  gl_FragColor = vec4(1.0, 0.5, 0.0, 1.0);" +
  "}";

const fragShader = gl.createShader(gl.FRAGMENT_SHADER);
if (!fragShader) {
  throw new Error("Failed to create vertex shader");
}
gl.shaderSource(fragShader, fragCode);
gl.compileShader(fragShader);

const shaderProgram = gl.createProgram()!;
gl.attachShader(shaderProgram, vertShader);
gl.attachShader(shaderProgram, fragShader);
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
