import { ExtensionContext, Uri, WebviewPanel } from "vscode";

export const setupPreview = (
  context: ExtensionContext,
  panel: WebviewPanel,
) => {
  const renderSrc = panel.webview.asWebviewUri(
    Uri.joinPath(context.extensionUri, "preview", "out", "index.js"),
  );

  panel.webview.html = getWebviewContent(renderSrc);
};

const getWebviewContent = (previewSrc: Uri) => {
  return `<!DOCTYPE html>
  <html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Preview Model</title>
  </head>
  <body>
    <canvas id="canvas" width=800 height=800 />
    <script src="${previewSrc}"/>
  </body>
  </html>`;
};
