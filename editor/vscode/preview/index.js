import("./pkg").then((_) => {
  // FIXME: Move this to rust.
  const vscode = acquireVsCodeApi();
  vscode.postMessage({ Ready: null });
});
