# JetBrains 插件指南

[English](./jetbrains-plugin-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 范围

本文档说明当前位于 [jetbrains-plugin](../jetbrains-plugin) 下的 WinPortKill JetBrains 插件 MVP。

- 目标：Windows 上的 JetBrains IDE
- UI：用于查看端口和进程的 Tool Window
- 后端：通过 HTTP 暴露的本地 sidecar 进程
- 状态：`runIde` 可运行，当前 MVP 支持列表、过滤和按 PID kill

## 关键文件

- [`build.gradle.kts`](../jetbrains-plugin/build.gradle.kts)：IntelliJ Platform 配置与 `runIde`
- [`src/main/resources/META-INF/plugin.xml`](../jetbrains-plugin/src/main/resources/META-INF/plugin.xml)：插件注册
- [`toolwindow/WinPortKillToolWindowFactory.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/toolwindow/WinPortKillToolWindowFactory.kt)：Tool Window 入口
- [`ui/WinPortKillPanel.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/ui/WinPortKillPanel.kt)：Swing UI 与响应式表格
- [`sidecar/SidecarManager.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/sidecar/SidecarManager.kt)：sidecar 生命周期与二进制查找
- [`api/ApiClient.kt`](../jetbrains-plugin/src/main/kotlin/dev/winportkill/jetbrains/api/ApiClient.kt)：localhost HTTP client

## 本地开发

前置条件：

- Windows
- 与 `jetbrains-plugin/gradle.properties` 匹配的 JDK
- `.vscode-extension/bin/win32-x64/winportkill.exe` 处存在 sidecar 二进制

启动沙箱 IDE：

```powershell
cd .\jetbrains-plugin
.\gradlew.bat runIde
```

`runIde` 会注入 `-Dwinportkill.dev.root=<repo-root>`，这样插件可以复用仓库中的 sidecar 二进制。

## Sidecar 流程

1. `WinPortKillPanel.refresh()` 请求一个 `ApiClient`。
2. `SidecarManager.ensureStarted()` 复用或启动 sidecar。
3. 它启动 `winportkill.exe --serve <port>`。
4. 等待 `/health == ok`。
5. 面板随后请求 ports、processes 或发送 kill 请求。

## UI 布局策略

插件使用一套基于宽度分档的表格系统，而不是单独切换到 card 模式。

- `WIDE`：完整列集
- `MEDIUM`：压缩次要列
- `COMPACT`：最小列集，细节放到底部 footer

当前阈值：

- `width <= 500`：`COMPACT`
- `501..860`：`MEDIUM`
- `> 860`：`WIDE`

## 打包说明

- 当 `.vscode-extension/bin/win32-x64/winportkill.exe` 存在时，`processResources` 会把它复制进插件资源
- 开发态优先使用仓库外置 sidecar 二进制
- 当前 MVP 在代码层面限制为 Windows
