import * as vscode from "vscode";
import { ExtensionContext, Uri, WebviewPanel } from "vscode";
import { LanguageClient } from "vscode-languageclient/node";

export class Preview {
  client: LanguageClient;
  context: ExtensionContext;
  panel: WebviewPanel;

  constructor(
    client: LanguageClient,
    context: ExtensionContext,
    panel: WebviewPanel,
  ) {
    this.client = client;
    this.context = context;
    this.panel = panel;

    this.panel.onDidDispose(() => {
      this.panel = undefined!;
    });
  }

  async setup() {
    const renderSrc = this.panel.webview.asWebviewUri(
      Uri.joinPath(this.context.extensionUri, "preview", "out", "index.js"),
    );

    this.panel.webview.html = getWebviewContent(renderSrc);

    return new Promise((resolve) => {
      this.panel.webview.onDidReceiveMessage((message) => {
        console.log(`got the message ${message}`);
        resolve(null);
      });
    });
  }

  async render(uri: Uri) {
    const res: any = await this.client.sendRequest("mc-lsp/canonicalModel", {
      uri: uri.toString(),
    });

    await this.renderModel(res.model);
  }

  async renderModel(model: any) {
    for (const element of model.elements) {
      for (const f of ["up", "down", "east", "west", "north", "south"]) {
        const face = element.faces[f];
        if (face == null) {
          continue;
        }

        const texture = face.texture as any as string;
        if (texture == null) {
          continue;
        }

        const first = texture.split(":")[0];
        const second = texture.split(":")[1];
        let namespace: string;
        let rel_path: string;
        if (second == null) {
          namespace = "minecraft";
          rel_path = first;
        } else {
          namespace = first;
          rel_path = second;
        }

        const path = this.panel.webview.asWebviewUri(
          Uri.parse(
            `file://${vscode.workspace.rootPath}/src/main/resources/assets/${namespace}/textures/${rel_path}.png`,
          ),
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
