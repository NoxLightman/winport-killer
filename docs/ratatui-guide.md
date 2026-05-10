# Ratatui 终端 UI 框架入门指南

> 从零开始理解 ratatui，配合 `src/ui.rs` 中的实际代码讲解。

---

## 一、ratatui 是什么？

**ratatui** 是 Rust 的终端 UI 库，用来在命令行里画出类似 GUI 的界面。

你见过的终端 GUI 程序基本都是这类库做的：

```
┌─────────────────────────────────┐
│  htop - 系统监控                 │  ← 终端里的"图形界面"
│  PID  USER   CPU%  MEM%         │
│  1234  root    5.2   3.1        │
│  5678  user   12.0   8.5        │
└─────────────────────────────────┘

┌─────────────────────────────────┐
│  vim / neovim 文本编辑器         │
│  ~                              │
│  │ fn main() {                  │
│  │     println!("hello");       │
│  │ }                            │
│  ~                              │
│  "ui.rs" 23L  512C              │
└─────────────────────────────────┘
```

**核心原理：** 终端本身只支持显示文字。ratatui 利用 Unicode 字符（`─│┌┐└┘█▀▄` 等）和 ANSI 转义码（控制颜色、光标位置）来模拟出"边框"、"表格"、"按钮"等视觉效果。

---

## 二、最简单的程序：Hello World

在理解你的代码之前，先看一个最小例子建立直觉：

```rust
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化后端（负责和真正的终端通信）
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // 2. 进入交替屏幕（画完再恢复原样）
    terminal.clear()?;

    // 3. 画一帧
    terminal.draw(|f| {
        // f 就是 Frame（画布），在这上面画画
        let area = f.area();  // 整个终端的大小

        use ratatui::widgets::{Paragraph, Block, Borders};
        let p = Paragraph::new("Hello Ratatui!")
            .block(Block::default().borders(Borders::ALL).title("Demo"));

        f.render_widget(p, area);  // 把组件画到指定区域
    })?;

    // 4. 等用户按任意键退出（实际程序这里会进入事件循环）
    std::io::stdin().read_line(&mut String::new())?;

    // 5. 恢复终端
    terminal.show_cursor()?;
    Ok(())
}
```

运行效果：
```
┌Demo──────────────────────────────┐
│                                  │
│           Hello Ratatui!          │
│                                  │
│                                  │
└──────────────────────────────────┘
```

**关键流程：**
```
初始化 → 清屏 → draw(|f| { 在 f 上画组件 }) → 循环等待输入 → 恢复终端
```

---

## 三、核心概念（6 个）

### 概念 1：Frame — 画布

`Frame` 代表**一帧画面**。每次调用 `terminal.draw()` 时，ratatui 给你一个干净的 `Frame`，你在上面画东西。

```rust
terminal.draw(|f| {
    // f 就是这一帧的画布
    // 你只能做两件事：
    f.render_widget(组件, 区域);              // 画静态组件
    f.render_stateful_widget(组件, 区域, 状态); // 画有交互状态的组件
})?;
```

**类比：** 就像游戏引擎每帧给你一个 Canvas，你往上面画精灵。

### 概念 2：Rect — 矩形区域

终端屏幕被切分成一个个矩形，每个矩形用 `Rect` 表示：

```rust
// Rect 包含四个值
pub struct Rect {
    pub x: u16,      // 左上角 x 坐标（列）
    pub y: u16,      // 左上角 y 坐标（行）
    pub width: u16,  // 宽度（多少列）
    pub height: u16, // 高度（多少行）
}

// 假设终端是 80列 × 24行
// f.area() = Rect { x:0, y:0, width:80, height:24 }
```

```
(0,0) ─────────────────────── (80,0)
      │                      │
      │   这就是 f.area()    │
      │   整个终端的区域      │
      │                      │
      │                      │
(0,24) ───────────────────── (80,24)
```

### 概念 3：Layout — 布局切割器

这是 ratatui 最强大的部分——把大矩形切成小矩形。类似 CSS Flexbox 或 Android LinearLayout。

#### 基本用法

```rust
use ratatui::layout::{Layout, Direction, Constraint};

let chunks = Layout::default()
    .direction(Direction::Vertical)   // 从上到下切
    .constraints([
        Constraint::Length(3),   // 第1块：固定高 3 行
        Constraint::Min(10),     // 第2块：最少 10 行，剩余全给它
        Constraint::Length(1),   // 第3块：固定高 1 行
    ])
    .split(f.area());  // 对整个屏幕进行切割

// chunks[0] = 顶部 3 行的区域
// chunks[1] = 中间剩余空间的区域
// chunks[2] = 底部 1 行的区域
```

图示：
```
f.area() (80×24)
┌──────────────────────────┐
│ chunks[0] (80×3)         │  ← Length(3)
├──────────────────────────┤
│                          │
│ chunks[1] (80×20)        │  ← Min(10)，吃掉剩余空间
│                          │
├──────────────────────────┤
│ chunks[2] (80×1)         │  ← Length(1)
└──────────────────────────┘
```

#### 五种约束类型

| 约束 | 含义 | 示例 |
|------|------|------|
| `Length(n)` | 固定 n 行/列 | 标题栏、状态栏 |
| `Min(n)` | 至少 n，剩下的全给 | 主内容区 |
| `Max(n)` | 最多 n | 防止某区过大 |
| `Percentage(n)` | 占总量的 n% | 两栏各占 50% |
| `Ratio(m, n)` | 占 m/n | `Ratio(1, 3)` = 33.3% |

#### 水平切割示例

```rust
let chunks = Layout::default()
    .direction(Direction::Horizontal)  // 从左到右切！
    .constraints([
        Constraint::Percentage(30),  // 左侧 30%
        Constraint::Percentage(70),  // 右侧 70%
    ])
    .split(area);
```

```
┌────────────┬───────────────────────┐
│  30%       │        70%           │
│ chunks[0]  │     chunks[1]         │
└────────────┴───────────────────────┘
```

#### 嵌套布局（先竖后横）

```rust
// 先切成上下两块
let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)])
    .split(f.area());

// 再把下面那块切成左右两半
let horizontal = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(vertical[1]);
```

```
┌──────────────────────────┐
│     顶部标题 (3行)        │  vertical[0]
├──────────┬───────────────┤
│          │               │
│ 左侧 50%  │  右侧 50%     │  horizontal[0] / horizontal[1]
│          │               │
└──────────┴───────────────┘
```

**你项目中的用法就是这种三段式垂直切割：**

```rust
// src/ui.rs 第 12-19 行
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),   // 过滤栏
        Constraint::Min(5),      // 列表区
        Constraint::Length(1),   // 状态栏
    ])
    .split(f.area());
```

### 概念 4：Style — 样式

控制文字的颜色和修饰：

```rust
use ratatui::style::{Style, Color, Modifier};
```

#### 颜色

```rust
// 基础颜色（16色，所有终端都支持）
Color::Red
Color::Green
Color::Yellow
Color::Blue
Color::Magenta
Color::Cyan
Color::White
Color::Black
Color::DarkGray
Color::Gray

// 256 色（更多选择）
Color::Indexed(128)  // 256 色板中的第 128 号

// RGB 真彩色（支持真彩色的终端）
Color::Rgb(255, 100, 0)  // 自定义橙色
```

#### 修饰符

```rust
Modifier::BOLD       // 加粗
Modifier::DIM        // 暗淡/半透明
Modifier::ITALIC     // 斜体（部分终端支持）
Modifier::UNDERLINE  // 下划线
Modifier::REVERSED   // 反色（背景变前景）
Modifier::STRIKETHROUGH  // 删除线
```

#### 组合成 Style

```rust
// 只有前景色
Style::default().fg(Color::Yellow)

// 前景 + 背景
Style::default().fg(Color::White).bg(Color::DarkGray)

// 颜色 + 修饰
Style::default()
    .fg(Color::Cyan)
    .add_modifier(Modifier::BOLD)

// 完整组合
Style::default()
    .fg(Color::White)
    .bg(Color::Black)
    .add_modifier(Modifier::BOLD)
    .add_modifier(Modifier::UNDERLINE)
```

**你项目中的样式使用：**

```rust
// src/ui.rs — 协议颜色
let proto_color = match entry.proto.as_str() {
    "TCP" | "TCP6" => Color::Cyan,
    "UDP" | "UDP6" => Color::Green,
    _ => Color::DarkGray,
};

// src/ui.rs — 选中行样式（深灰底 + 加粗）
.highlight_style(
    Style::default()
        .bg(Color::DarkGray)
        .add_modifier(Modifier::BOLD),
)
```

### 概念 5：文本系统（Span + Line）

终端的一行文字可以**分段上不同颜色**，这就是 Span + Line 的设计：

```
一行文字（Line）:
┌──────────────────────────────────────────────────┐
│ [Span:青色] [Span:默认色] [Span:黄色] [Span:默认色] │
│   TCP         0.0.0.0       :80       nginx       │
└──────────────────────────────────────────────────┘
```

#### Span — 一段带样式的文字（最小单元）

```rust
use ratatui::text::{Span, Line};

// 带颜色的段
Span::styled("TCP", Style::default().fg(Color::Cyan))

// 不带样式的纯文本（继承父级样式）
Span::raw("    0.0.0.0   ")
```

#### Line — 一行 = 多个 Span 拼起来

```rust
// 用 vec 把多个 Span 组成一行
Line::from(vec![
    Span::styled("Proto", Style::default().add_modifier(Modifier::BOLD)),
    Span::raw("  "),
    Span::styled("Addr", Style::default().add_modifier(Modifier::BOLD)),
])

// 简单的一行（只有一个 Span）
Line::from("普通文字")
```

#### 格式化对齐

用 `{:<N}` 左对齐保证列对齐：

```rust
format!("{:<6}", "TCP")    // "TCP   " （占 6 格，左对齐）
format!("{:<14}", "0.0.0.0") // "0.0.0.0       " （占 14 格）
format!("{:<9}", ":80")     // ":80      " （占 9 格）
format!("{:<11.1}", 12.34)  // "12.3       " （11格宽，1位小数）
```

效果：
```
Proto  Addr             Port     PID
TCP    0.0.0.0          :80      1234
UDP6   ::               443      4
↑      ↑                ↑        ↑
6格    14格              9格      9格
```

**你项目中的实际用法：**

```rust
// src/ui.rs 第 81-88 行
ListItem::new(Line::from(vec![
    Span::styled(format!("{:<6}", entry.proto), Style::default().fg(proto_color)),  // 青色或绿色
    Span::raw(format!("{:<14}", entry.local_addr)),                                 // 默认色
    Span::styled(format!("{:<9}", entry.port), port_style),                         // 黄色或灰色
    Span::raw(format!("{:<9}", entry.pid)),                                         // 默认色
    Span::styled(format!("{:<11.1}", mem_mb), Style::default().fg(Color::Magenta)), // 品红
    Span::raw(entry.name.clone()),                                                   // 默认色
]))
```

### 概念 6：Widgets — 预制组件

ratatui 提供了很多现成的组件，像搭积木一样用：

#### Paragraph — 文本段落（最简单）

```rust
use ratatui::widgets::{Paragraph, Block, Borders};

let p = Paragraph::new("这是一段文字\n可以换行")
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default()
        .borders(Borders::ALL)
        .title("标题"));

f.render_widget(p, area);
```

效果：
```
┌标题─────────────────┐
│                     │
│  这是一段文字        │
│  可以换行            │
│                     │
└─────────────────────┘
```

**你项目中用在过滤栏和状态栏：**
- 过滤栏：`Paragraph::new(format!(" Filter: {}", filter_text))`
- 状态栏：`Paragraph::new(format!(" {}", msg))`

#### Block — 边框容器（装饰器）

`Block` 本身不显示内容，它是一个**带边框和标题的外框**，包裹其他组件：

```rust
Block::default()
    .borders(Borders::ALL)                    // 四周边框
    // .borders(Borders::TOP | Borders::LEFT)  // 也可以只画部分边框
    .title("WinPortKill")                     // 左上角标题
    // .title_style(Style::default().fg(Color::Red))  // 标题颜色
```

边框选项：
```
Borders::NONE      ┌────────┐  Borders::ALL
Borders::LEFT      │────────┘  Borders::TOP | Borders::LEFT
Borders::RIGHT     │        │  Borders::BOTTOM | Borders::RIGHT
Borders::TOP       └────────┘  Borders::LEFT | Borders::RIGHT
Borders::BOTTOM
```

#### List + ListItem + ListState — 可选中的列表（最重要）

这三个**必须一起用**：

```
List（整个列表组件）
│
├── ListItem #0  ← 表头（加粗）
├── ListItem #1  ← 数据行
├── ListItem #2  ← 数据行 ★ ListState 说选中这行
├── ListItem #3  ← 数据行
└── ...
```

##### Step 1: 准备数据行

```rust
use ratatui::widgets::{List, ListItem, ListState};

let items = vec![
    ListItem::new("第一行"),
    ListItem::new("第二行"),
    ListItem::new("第三行"),
];
```

也可以每行用 `Line`（多颜色）：

```rust
let items = vec![
    ListItem::new(Line::from(vec![
        Span::styled("TCP", Color::Cyan),
        Span::raw(" :80 "),
    ])),
];
```

##### Step 2: 创建 List 并设置样式

```rust
let list = List::new(items)
    // 选中时整行的外观
    .highlight_style(
        Style::default()
            .bg(Color::DarkGray)          // 深灰背景
            .add_modifier(Modifier::BOLD)  // 加粗字体
    )
    // 选中行前面显示的符号
    .highlight_symbol(">> ")
    // 外层边框
    .block(Block::default()
        .borders(Borders::ALL)
        .title(" Processes (42) ")        // 标题带数量
    );
```

未选中时：
```
┌ Processes (42) ──────────────┐
│ TCP    0.0.0.0    80    4    │
│ UDP    127.0.0.1  5353  1234 │
│ TCP6   ::         443   4    │
└───────────────────────────────┘
```

选中第 2 行时：
```
┌ Processes (42) ──────────────┐
│ TCP    0.0.0.0    80    4    │
│ UDP    127.0.0.1  5353  1234 │
│ >> TCP6   ::      443   4    │  ← 深灰底 + 加粗 + ">>"
└───────────────────────────────┘
```

##### Step 3: 设置选中状态并渲染

```rust
let mut state = ListState::default();
state.select(Some(2));  // 选中索引为 2 的行（第 3 行）

// 注意：必须用 render_stateful_widget，不是 render_widget！
f.render_stateful_widget(list, area, &mut state);
```

**为什么有两个 render 方法？**

| 方法 | 参数 | 适用场景 |
|------|------|----------|
| `render_widget(widget, area)` | 2 个 | 静态展示，不需要交互 |
| `render_stateful_widget(widget, area, &mut state)` | 3 个 | 有交互状态（选中/滚动/输入） |

`Paragraph` 是静态的 → 用 `render_widget`
`List` 是可交互的 → 用 `render_stateful_widget`

##### ListState 还能做什么？

```rust
let mut state = ListState::default();

state.select(Some(3));        // 选中第 3 行
state.select(None);            // 取消选中
state.selected();              // 查询当前选中了哪行 → Option<usize>
```

**你项目中的用法（注意 +1）：**

```rust
// src/ui.rs 第 108-112 行
let mut state = ListState::default();
if !app.filtered.is_empty() {
    state.select(Some(app.selected + 1));
    //                                   ^^^
    // 因为 items[0] 是表头，所以实际数据的索引要 +1
    // app.selected=0 → 选中 items[1]（第一条数据）
}
f.render_stateful_widget(list, area, &mut state);
```

---

## 四、完整渲染流程（以你的项目为例）

把所有概念串起来，看看一次完整的绘制过程：

```
用户按下方向键 / 数据刷新
        │
        ▼
terminal.draw(|f| {
    │
    ├── ① Layout 把屏幕切成 3 块
    │   chunks[0] = 过滤栏区域 (80×3)
    │   chunks[1] = 列表区域   (80×20)
    │   chunks[2] = 状态栏区域 (80×1)
    │
    ├── ② draw_filter(f, app, chunks[0])
    │   创建 Paragraph ("Filter: ...")
    │   加 Block 边框 + "WinPortKill" 标题
    │   f.render_widget(paragraph, chunks[0])
    │
    ├── ③ draw_list(f, app, chunks[1])
    │   │
    │   ├── 构造表头 Line（加粗的 Proto/Addr/Port/PID/Mem/Process）
    │   │
    │   ├── 遍历 app.filtered，每条数据构造一个 Line：
    │   │   ├── Span::styled(proto, 青色/绿色)
    │   │   ├── Span::raw(addr)
    │   │   ├── Span::styled(port, 黄色/灰色)
    │   │   ├── Span::raw(pid)
    │   │   ├── Span::styled(mem_mb, 品红)
    │   │   └── Span::raw(name)
    │   │
    │   ├── 表头 + 所有数据行 → Vec<ListItem>
    │   │
    │   ├── 创建 List：
    │   │   ├── highlight_style = 深灰底+加粗
    │   │   ├── highlight_symbol = ">> "
    │   │   └── Block 边框 + "Processes (N)" 标题
    │   │
    │   ├── ListState.select(selected + 1)
    │   └── f.render_stateful_widget(list, chunks[1], &mut state)
    │
    └── ④ draw_status(f, app, chunks[2])
        创建 Paragraph ("[q]Quit [k]Kill ..." 或操作消息)
        根据内容选颜色（绿/红/灰）
        f.render_widget(paragraph, chunks[2])
})
        │
        ▼
ratatui 计算差异，只刷新变化的部分到终端
```

---

## 五、常用组件速查

| 组件 | 导入路径 | 用途 | 是否需要 State |
|------|---------|------|---------------|
| `Paragraph` | `widgets::Paragraph` | 多行文本 | 否 |
| `Block` | `widgets::Block` | 边框+标题容器 | 否 |
| `List` | `widgets::List` | 可选中列表 | **是** (`ListState`) |
| `ListItem` | `widgets::ListItem` | 列表中的一行 | — |
| `Table` | `widgets::Table` | 表格（自动对齐列） | **是** (`TableState`) |
| `BarChart` | `widgets::BarChart` | 柱状图 | 否 |
| `Sparkline` | `widgets::Sparkline` | 迷你折线图（`▁▂▃▅▇▃▄`） | 否 |
| `Canvas` | `widgets::Canvas` | 自由绘图（点/线/矩形） | 否 |
| `Clear` | `widgets::Clear` | 清空区域 | 否 |

---

## 六、学习建议

1. **先跑通 Hello World** — 理解 `draw` 闭包和 `Frame` 的关系
2. **玩 Layout** — 尝试不同的 `Direction` 和 `Constraint` 组合，看怎么切分屏幕
3. **用 List 做一个可上下键选择的菜单** — 理解 `render_stateful_widget` 和 `ListState`
4. **给你的 List 加上颜色** — 理解 `Span` + `Line` + `Style` 的组合
5. **加入键盘事件循环** — 让界面真正动起来（按上下键移动选中行）

> 注：键盘事件处理不在 `ui.rs` 里，而是在主事件循环中（通常在 `app.rs` 或 `main.rs` 中）。`ui.rs` 只负责"根据当前状态画出一帧"，不处理输入。
