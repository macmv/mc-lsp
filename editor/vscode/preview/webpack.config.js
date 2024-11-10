const path = require('path');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  entry: './index.js',
  output: {
    path: path.resolve(__dirname, 'out'),
    filename: 'index.js',
  },
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, ".")
    }),
  ],
  mode: 'development',
  experiments: {
    asyncWebAssembly: true
  }
};
