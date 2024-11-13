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

  async render(model: any) {
    for (const [i, element] of model.elements.entries()) {
      for (const f of ["up", "down", "east", "west", "north", "south"]) {
        if (element.faces[f] === undefined) {
          continue;
        }

        const face = element.faces[f];
        const texture = face.texture as any as string;

        const namespace = texture.split(":")[0];
        const rel_path = texture.split(":")[1];
        if (rel_path === undefined) {
          // Just... don't
          delete element.faces[f];
          continue;
        }

        const path = this.panel.webview.asWebviewUri(
          Uri.parse(
            `file://${vscode.workspace.rootPath}/src/main/resources/assets/${namespace}/textures/${rel_path}.png`
          )
        );

        face.texture = path.toString();
      }
    }

    await this.panel.webview.postMessage({
      RenderModel: { model },
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
