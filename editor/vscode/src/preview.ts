import * as vscode from "vscode";
import { readFileSync } from "fs";
import { ExtensionContext, Uri, WebviewPanel } from "vscode";

export class Preview {
  context: ExtensionContext;
  panel: WebviewPanel;

  constructor(context: ExtensionContext, panel: WebviewPanel) {
    this.context = context;
    this.panel = panel;
  }

  async setup() {
    const renderSrc = this.panel.webview.asWebviewUri(
      Uri.joinPath(this.context.extensionUri, "preview", "out", "index.js")
    );

    this.panel.webview.html = getWebviewContent(renderSrc);

    return new Promise((resolve) => {
      this.panel.webview.onDidReceiveMessage((message) => {
        console.log(`got the message ${message}`);
        resolve(null);
      });
    });
  }

  async render(model: Uri) {
    // TODO: Fetch this from the language server, which will build the canonical model format.
    const content = JSON.parse(readFileSync(model.fsPath).toString());

    for (const [key, texture] of Object.entries(content.textures)) {
      const namespace = (texture as any).split(":")[0];
      const rel_path = (texture as any).split(":")[1];
      const path = this.panel.webview.asWebviewUri(
        Uri.parse(
          `file://${vscode.workspace.rootPath}/src/main/resources/assets/${namespace}/textures/${rel_path}.png`
        )
      );

      content.textures[key] = path.toString();
    }

    await this.panel.webview.postMessage({
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
