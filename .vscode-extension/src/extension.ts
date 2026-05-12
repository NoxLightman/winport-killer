import * as vscode from "vscode";
import { SidecarManager } from "./sidecarManager";
import { WinPortKillViewProvider } from "./webviewProvider";

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  const sidecarManager = new SidecarManager(context);
  const provider = new WinPortKillViewProvider(context, sidecarManager);

  context.subscriptions.push(sidecarManager);
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider("winportkill.sidebar", provider)
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("winportkill.refresh", async () => {
      await provider.refresh();
    })
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("winportkill.killSelected", async () => {
      vscode.window.showInformationMessage(
        "Kill from the panel for MVP. Command wiring can be extended later."
      );
    })
  );
}

export function deactivate(): void {
  // Sidecar disposal is handled by subscriptions.
}
