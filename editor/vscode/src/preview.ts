import { readFileSync } from "fs";
import { ExtensionContext, Uri, WebviewPanel } from "vscode";

export class Preview {
  context: ExtensionContext;
  panel: WebviewPanel;

  constructor(context: ExtensionContext, panel: WebviewPanel) {
    this.context = context;
    this.panel = panel;
  }

  setup() {
    const renderSrc = this.panel.webview.asWebviewUri(
      Uri.joinPath(this.context.extensionUri, "preview", "out", "index.js")
    );

    this.panel.webview.html = getWebviewContent(renderSrc);
  }

  render(model: Uri) {
    // TODO: Fetch this from the language server, which will build the canonical model format.
    const content = readFileSync(model.fsPath);

    this.panel.webview.postMessage({
      RenderModel: {
        model: content,
      },
    });
  }
}

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
