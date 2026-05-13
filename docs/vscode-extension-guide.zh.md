# VS Code 扩展指南

[English](./vscode-extension-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 范围

VS Code 集成位于 [.vscode-extension](../.vscode-extension)。

- TypeScript 扩展宿主
- webview sidebar UI
- 本地打包的 sidecar 二进制
- 真正的检查流程运行时仍要求 Windows

## 关键文件

- [`package.json`](../.vscode-extension/package.json)：manifest、命令、视图和设置声明
- [`src/extension.ts`](../.vscode-extension/src/extension.ts)：激活与命令注册
- [`src/sidecarManager.ts`](../.vscode-extension/src/sidecarManager.ts)：sidecar 生命周期
- [`src/apiClient.ts`](../.vscode-extension/src/apiClient.ts)：localhost HTTP client
- [`src/webviewProvider.ts`](../.vscode-extension/src/webviewProvider.ts)：webview 状态桥接和 UI
- [`src/types.ts`](../.vscode-extension/src/types.ts)：响应类型

## 构建与调试

在扩展目录下执行：

```powershell
cd .\.vscode-extension
npm.cmd run build
```

VS Code 任务链还会执行：

1. 停掉正在运行的 `winportkill` 进程
2. 把 `..\target\debug\winportkill.exe` 复制到 `.vscode-extension/bin/win32-x64/winportkill.exe`
3. 编译 TypeScript 扩展

提供的启动配置名为 `Run WinPortKill Extension`。

## 运行流程

1. 扩展因视图或刷新命令而激活。
2. `SidecarManager` 分配一个空闲 localhost 端口。
3. 启动 `winportkill.exe --serve <port>`。
4. 等待 `/health` 可用。
5. webview 请求 `ports` 或 `processes`。
6. 行级 kill 按钮调用 `POST /kill/{pid}`，然后刷新。

## 当前行为

- 两种视图：`ports` 和 `processes`
- 带 debounce 的过滤输入框
- Refresh 按钮
- 行级 kill 操作
- 窄宽度下切换为 card 布局
- 宽一些时使用 table 布局

## 当前缺口

- `winportkill.killSelected` 仍然只是 MVP 占位命令
- `package.json` 中声明了 `winportkill.refreshIntervalSeconds`，但当前 webview 代码还没有真正消费它
