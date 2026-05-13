# egui GUI 指南

[English](./egui-gui-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 范围

原生 GUI 位于 [crates/winportkill-gui](../crates/winportkill-gui)。

- 工具栈：`eframe` + `egui`
- 后端：直接调用 `winportkill-core`
- 目的：不依赖 HTTP sidecar 的桌面 GUI 前端

## 运行

```powershell
cargo run -p winportkill-gui
```

Release 构建：

```powershell
cargo build -p winportkill-gui --release
target\release\winportkill-gui.exe
```

## 当前行为检查项

- 窗口标题为 `WinPortKill`
- 同时提供 ports 和 processes 两种视图
- 过滤输入会更新可见行
- 行选中支持鼠标和键盘导航
- kill 需要确认
- 每 10 秒自动刷新
- 提供手动刷新入口

## 说明

- 这个 GUI 不经过 HTTP sidecar
- kill 仍然是针对整个进程
- 受保护目标可能需要管理员权限
