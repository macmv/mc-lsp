import * as vscode from "vscode";
import {
  Executable,
  LanguageClient,
  RevealOutputChannelOn,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

export async function activate(context: vscode.ExtensionContext) {
  console.log('Congratulations, your extension "mclsp" is now active!');

  context.subscriptions.push(
    vscode.commands.registerCommand("mclsp.previewModel", () => {
      const panel = vscode.window.createWebviewPanel(
        "previewModel",
        "Preview Model",
        vscode.ViewColumn.Two,
        {}
      );
    })
  );

  const exec: Executable = {
    command: "/home/macmv/Desktop/programming/rust/mclsp/target/release/mc-lsp",
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

  const client = new LanguageClient(
    "mclsp",
    "MC LSP",

    serverOptions,
    clientOptions
  );

  await client.start();
}

export function deactivate() {}
