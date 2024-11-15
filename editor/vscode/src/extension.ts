import * as vscode from "vscode";
import {
  Executable,
  LanguageClient,
  RevealOutputChannelOn,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

import { Preview } from "./preview";

export async function activate(context: vscode.ExtensionContext) {
  console.log('Congratulations, your extension "mclsp" is now active!');

  let client: LanguageClient;
  let preview: Preview | undefined;

  vscode.window.onDidChangeActiveTextEditor(async (editor) => {
    if (
      editor !== undefined &&
      preview !== undefined &&
      editor.document.uri !== undefined
    ) {
      const uri = editor.document.uri;

      const res: any = await client.sendRequest("mc-lsp/canonicalModel", {
        uri: uri.toString(),
      });

      await preview.render(res.model);
    }
  });

  context.subscriptions.push(
    vscode.commands.registerCommand("mclsp.previewModel", async () => {
      if (preview === undefined) {
        const panel = vscode.window.createWebviewPanel(
          "previewModel",
          "Preview Model",
          {
            viewColumn: vscode.ViewColumn.Two,
            preserveFocus: true,
          },
          {
            enableScripts: true,
          },
        );

        preview = new Preview(context, panel);
        await preview.setup();
      } else {
        preview.panel.reveal(vscode.ViewColumn.Two, true);
      }

      const uri = vscode.window.visibleTextEditors[0].document.uri!;

      const res: any = await client.sendRequest("mc-lsp/canonicalModel", {
        uri: uri.toString(),
      });

      await preview.render(res.model);
    }),
  );

  const exec: Executable = {
    // The language server is bundled in the extension.
    command: context.asAbsolutePath("mc-lsp"),
    transport: TransportKind.stdio,
  };

  const serverOptions: ServerOptions = {
    run: exec,
    debug: exec,
  };

  const clientOptions = {
    documentSelector: [
      {
        scheme: "file",
        language: "mcmodel",
      },
    ],
    outputChannel: vscode.window.createOutputChannel("MC LSP"),
    revealOutputChannelOn: RevealOutputChannelOn.Info,
  };

  client = new LanguageClient(
    "mclsp",
    "MC LSP",

    serverOptions,
    clientOptions,
  );

  await client.start();
}

export function deactivate() {}
