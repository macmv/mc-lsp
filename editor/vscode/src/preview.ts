import { ExtensionContext, Uri, WebviewPanel } from "vscode";

export const setupPreview = (
  context: ExtensionContext,
  panel: WebviewPanel,
) => {
  const renderSrc = panel.webview.asWebviewUri(
    Uri.joinPath(context.extensionUri, "preview", "out", "render.js"),
  );
  const vertSrc = panel.webview.asWebviewUri(
    Uri.joinPath(context.extensionUri, "preview", "vert.glsl"),
  );
  const fragSrc = panel.webview.asWebviewUri(
    Uri.joinPath(context.extensionUri, "preview", "frag.glsl"),
  );

  panel.webview.html = getWebviewContent(renderSrc, vertSrc, fragSrc);
};

const getWebviewContent = (previewSrc: Uri, vertSrc: Uri, fragSrc: Uri) => {
  return `<!DOCTYPE html>
  <html lang="en">
  <head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Preview Model</title>
  </head>
  <body>
    <canvas id="canvas" width=800 height=800 />
    <script>
      var g_vertCode = fetch("${vertSrc}").then((res) => res.text());
      var g_fragCode = fetch("${fragSrc}").then((res) => res.text());
    </script>
    <script src="${previewSrc}"/>
  </body>
  </html>`;
};
