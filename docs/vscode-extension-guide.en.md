# VS Code Extension Guide

English | [中文](./vscode-extension-guide.zh.md)

Back: [README](../README.md)

## Scope

The VS Code integration lives under [.vscode-extension](../.vscode-extension).

- TypeScript extension host
- webview sidebar UI
- bundled local sidecar binary
- Windows runtime requirement for the actual inspection flow

## Important Files

- [`package.json`](../.vscode-extension/package.json): manifest, commands, views, setting declaration
- [`src/extension.ts`](../.vscode-extension/src/extension.ts): activation and command registration
- [`src/sidecarManager.ts`](../.vscode-extension/src/sidecarManager.ts): sidecar lifecycle
- [`src/apiClient.ts`](../.vscode-extension/src/apiClient.ts): localhost HTTP client
- [`src/webviewProvider.ts`](../.vscode-extension/src/webviewProvider.ts): webview state bridge and UI
- [`src/types.ts`](../.vscode-extension/src/types.ts): response types

## Build And Debug

From the extension directory:

```powershell
cd .\.vscode-extension
npm.cmd run build
```

The VS Code tasks also:

1. stop any running `winportkill` process
2. copy `..\target\debug\winportkill.exe` into `.vscode-extension/bin/win32-x64/winportkill.exe`
3. compile the TypeScript extension

The provided launch configuration is `Run WinPortKill Extension`.

## Runtime Flow

1. The extension activates on the view or refresh command.
2. `SidecarManager` allocates a free localhost port.
3. It starts `winportkill.exe --serve <port>`.
4. It waits for `/health`.
5. The webview requests either `ports` or `processes`.
6. Row kill buttons call `POST /kill/{pid}` and then refresh.

## Current Behavior

- two view modes: `ports` and `processes`
- filter input with debounce
- refresh button
- per-row kill actions
- compact card layout on narrow widths
- table layout on wider widths

## Current Gaps

- `winportkill.killSelected` is still an MVP placeholder
- `winportkill.refreshIntervalSeconds` is declared in `package.json` but not yet consumed in the webview code
