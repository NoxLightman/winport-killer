# WinPortKill egui GUI 测试指南

## 编译运行

```bash
cargo run -p winportkill-gui
```

或编译后直接运行：

```bash
cargo build -p winportkill-gui --release
target\release\winportkill-gui.exe
```

---

## 功能测试清单

### 1. 窗口基本显示

- [ ] 窗口标题显示 "WinPortKill"
- [ ] 顶部统计栏显示 Procs/TCP/UDP/Mem 数据
- [ ] 表格显示 Proto、Addr、Port、PID、Mem(MB)、Process 列
- [ ] TCP 行为青色，UDP 行为绿色，端口号为黄色

### 2. 选中与导航

- [ ] 点击表格行，选中行高亮（深蓝背景）
- [ ] 键盘 `↑` `↓` 上下切换选中行
- [ ] 选中行超出可视区域时能自动滚动

### 3. 过滤

- [ ] 底部 Filter 输入框输入 `tcp`，列表只显示 TCP 行
- [ ] 输入端口号如 `8080`，只显示匹配行
- [ ] 输入进程名如 `node`，只显示匹配行
- [ ] 清空输入框，列表恢复全部

### 4. Kill 进程

- [ ] 选中一行，点击右上角 **Kill (k)** 按钮
- [ ] 或按键盘 `k` 键
- [ ] 成功：底部显示绿色 "Killed PID xxx (进程名)"
- [ ] 失败：底部显示红色提示（需要管理员权限运行）
- [ ] kill 后列表自动刷新

### 5. 自动刷新

- [ ] 启动后等待 10 秒，列表自动刷新
- [ ] 点击右上角 **Refresh** 按钮，立即刷新
- [ ] 刷新后底部��示 "Refreshed"

### 6. 窗口缩放

- [ ] 拖动窗口边缘缩放，表格列宽度自适应
- [ ] 缩小到最小尺寸 600x400，界面不溢出

### 7. 管理员权限测试

> Kill 受保护进程（如系统服务）需要管理员权限

```bash
# 以管理员身份打开 PowerShell 后运行
target\release\winportkill-gui.exe
```

- [ ] 管理员模式下 kill 系统进程成功
