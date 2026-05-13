# Ratatui 指南

[English](./ratatui-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 范围

本文聚焦于根目录下的 TUI 实现，代码位于 [`src`](../src)。

- [`main.rs`](../src/main.rs)：模式选择和终端生命周期
- [`app.rs`](../src/app.rs)：状态、刷新、过滤和 kill 行为
- [`ui.rs`](../src/ui.rs)：渲染

## 当前 TUI 流程

1. `main.rs` 进入 raw mode 和 alternate screen。
2. 创建 `Terminal<CrosstermBackend>`。
3. `App::new()` 加载首个快照。
4. 主循环逐帧绘制 UI。
5. 键盘事件转发到 `App::handle_key()`。
6. 每 10 秒执行一次周期刷新。

## 状态模型

`App` 保存：

- 当前视图模式
- 完整和过滤后的端口行
- 完整和过滤后的进程行
- 过滤文本
- 当前选中行索引
- 当前数据集统计信息
- 状态消息和退出标志

## 渲染布局

[`ui.rs`](../src/ui.rs) 目前渲染四个垂直区域：

- 顶部栏
- 过滤栏
- 主表格
- 状态行

使用的关键组件包括：

- `Layout` 负责切分屏幕
- `Paragraph` 用于标签和状态文本
- `Table` 用于 ports 和 processes 两种视图
- `TableState` 与 `ScrollbarState` 负责选中和滚动位置

## 键位

- `q`：退出
- `k`：结束当前选中 PID
- `/`：进入过滤模式
- `r`：刷新
- `Tab`：切换视图
- `Up` / `Down`：移动选中
- `PageUp` / `PageDown`：跳跃移动
- `Home` / `End`：跳到首尾

## 为什么它重要

TUI 是仓库里最直接的一条端到端客户端路径。

它适合用于：

- 理解直接调用 core 的前端路径
- 在更新 IDE 客户端前确认数据结构变化
- 在不涉及 sidecar 生命周期复杂度的情况下验证交互行为
