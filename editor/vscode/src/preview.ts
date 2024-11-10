import { ExtensionContext, Uri, WebviewPanel } from "vscode";

export const setupPreview = (
  context: ExtensionContext,
  panel: WebviewPanel,
) => {
  const onDiskPath = Uri.joinPath(
    context.extensionUri,
    "out",
    "preview",
    "render.js",
  );
  const previewSrc = panel.webview.asWebviewUri(onDiskPath);

  panel.webview.html = getWebviewContent(previewSrc);
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
    <script src="${previewSrc}"/>
  </body>
  </html>`;
};
