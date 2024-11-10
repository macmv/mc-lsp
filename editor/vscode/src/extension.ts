import * as vscode from "vscode";

export function activate(context: vscode.ExtensionContext) {
  console.log('Congratulations, your extension "mclsp" is now active!');

  const disposable = vscode.commands.registerCommand("mclsp.helloWorld", () => {
    vscode.window.showInformationMessage("Hello World from mclsp!");
  });

  context.subscriptions.push(disposable);
}

export function deactivate() {}
