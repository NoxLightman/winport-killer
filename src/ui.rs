use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// 主渲染函数：将界面分为三个区域（过滤栏、列表、状态栏）并分别绘制
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 过滤栏：3 行高（含边框）
            Constraint::Min(5),   // 列表区：至少 5 行，占满剩余空间
            Constraint::Length(1), // 状态栏：1 行高
        ])
        .split(f.area());

    draw_filter(f, app, chunks[0]);
    draw_list(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

/// 绘制过滤栏：显示当前过滤关键字或提示文字
/// 过滤模式激活时文字为黄色，否则为灰色提示
fn draw_filter(f: &mut Frame, app: &App, area: Rect) {
    let style = if app.filter_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let filter_text = if app.filter.is_empty() && !app.filter_active {
        "Press / to filter...".to_string()
    } else {
        app.filter.clone()
    };

    let paragraph = Paragraph::new(format!(" Filter: {}", filter_text))
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("WinPortKill"));
    f.render_widget(paragraph, area);
}

/// 绘制进程/端口列表：表头 + 数据行
/// 选中行有深灰背景 + 加粗 + ">>" 前缀
/// 颜色规则：TCP/TCP6 蓝色，UDP/UDP6 绿色，端口号黄色（无端口时灰色），内存紫色
fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    // 表头行
    let header = ListItem::new(Line::from(vec![
        Span::styled("Proto  ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Addr          ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Port     ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("PID      ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Mem(MB)    ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Process", Style::default().add_modifier(Modifier::BOLD)),
    ]));

    // 数据行：表头 + 所有过滤后的条目
    let items: Vec<ListItem> = std::iter::once(header)
        .chain(app.filtered.iter().map(|entry| {
            // 内存从字节转换为 MB
            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;

            // 协议颜色：TCP 系蓝色，UDP 系绿色，无协议灰色
            let proto_color = match entry.proto.as_str() {
                "TCP" | "TCP6" => Color::Cyan,
                "UDP" | "UDP6" => Color::Green,
                _ => Color::DarkGray,
            };

            // 端口颜色：有端口黄色，无端口（"-")灰色
            let port_style = if entry.port == "-" {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Yellow)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<6}", entry.proto), Style::default().fg(proto_color)),
                Span::raw(format!("{:<14}", entry.local_addr)),
                Span::styled(format!("{:<9}", entry.port), port_style),
                Span::raw(format!("{:<9}", entry.pid)),
                Span::styled(format!("{:<11.1}", mem_mb), Style::default().fg(Color::Magenta)),
                Span::raw(entry.name.clone()),
            ]))
        }))
        .collect();

    let list = List::new(items)
        // 选中行样式：深灰背景 + 加粗
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        // 选中行前缀符号
        .highlight_symbol(">> ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Processes ({}) ", app.filtered.len())),
        );

    // ListState 控制选中行，+1 是因为第 0 行是表头
    let mut state = ListState::default();
    if !app.filtered.is_empty() {
        state.select(Some(app.selected + 1));
    }
    f.render_stateful_widget(list, area, &mut state);
}

/// 绘制状态栏：显示操作结果消息 + 快捷键提示
/// 过滤模式时显示过滤相关提示，正常模式时显示完整快捷键列表
/// 消息颜色：成功绿色，失败红色，普通灰色
fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let help = if app.filter_active {
        "[Enter/Esc] Stop filtering  [Type] Filter"
    } else {
        "[q]Quit  [k]Kill  [/]Filter  [r]Refresh  [↑↓]Navigate"
    };

    // 如果有状态消息，拼接在快捷键提示前面
    let msg = if app.status_msg.is_empty() {
        help.to_string()
    } else {
        format!("{}  |  {}", app.status_msg, help)
    };

    // 消息颜色：包含 "Failed" 或 "not found" 为红色，有消息为绿色，无消息为灰色
    let style = if app.status_msg.contains("Failed") || app.status_msg.contains("not found") {
        Style::default().fg(Color::Red)
    } else if !app.status_msg.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let paragraph = Paragraph::new(format!(" {}", msg)).style(style);
    f.render_widget(paragraph, area);
}