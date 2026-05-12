use crate::app::{App, ViewMode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

const ACCENT: Color = Color::Cyan;
const ACCENT_BG: Color = Color::Rgb(30, 60, 90);
const DIM_FG: Color = Color::Rgb(80, 80, 80);
const HEADER_BG: Color = Color::Rgb(40, 44, 52);
const HEADER_FG: Color = Color::White;
const BORDER_ACTIVE: Color = Color::Cyan;
const BORDER_IDLE: Color = Color::Rgb(60, 64, 72);
const TOP_BAR_BG: Color = Color::Rgb(20, 22, 28);
const TCP_COLOR: Color = Color::Cyan;
const UDP_COLOR: Color = Color::Green;
const PORT_COLOR: Color = Color::Yellow;
const MEM_COLOR: Color = Color::Magenta;

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(
        Block::new().style(Style::default().bg(TOP_BAR_BG)),
        f.area(),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(f.area());

    draw_top_bar(f, app, chunks[0]);
    draw_filter(f, app, chunks[1]);
    draw_table(f, app, chunks[2]);
    draw_status(f, app, chunks[3]);
}

fn draw_top_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28),
            Constraint::Length(16),
            Constraint::Min(20),
            Constraint::Length(32),
        ])
        .split(area);

    let title = ratatui::widgets::Paragraph::new(" WinPortKill ")
        .style(
            Style::default()
                .bg(TOP_BAR_BG)
                .fg(PORT_COLOR)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Left);
    f.render_widget(title, chunks[0]);

    let tabs = match app.view_mode {
        ViewMode::Ports => Line::from(vec![
            Span::styled(
                "[Ports]",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("Processes", Style::default().fg(DIM_FG)),
        ]),
        ViewMode::Processes => Line::from(vec![
            Span::styled("Ports", Style::default().fg(DIM_FG)),
            Span::raw(" "),
            Span::styled(
                "[Processes]",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]),
    };
    let tabs_widget = ratatui::widgets::Paragraph::new(tabs).alignment(Alignment::Center);
    f.render_widget(tabs_widget, chunks[1]);

    let stats_lines = match app.view_mode {
        ViewMode::Ports => {
            let mem_mb = app.port_stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
            vec![
                Line::from(vec![
                    Span::styled("Rows ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.port_stats.total_rows.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("Proc ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.port_stats.total_procs.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("TCP ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.port_stats.tcp_count.to_string(),
                        Style::default().fg(TCP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("UDP ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.port_stats.udp_count.to_string(),
                        Style::default().fg(UDP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("Mem {:.0} MB", mem_mb),
                        Style::default().fg(MEM_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
            ]
        }
        ViewMode::Processes => {
            let mem_mb = app.process_stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
            vec![
                Line::from(vec![
                    Span::styled("Proc ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.process_stats.total_procs.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("With Ports ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.process_stats.procs_with_ports.to_string(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Bindings ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.process_stats.total_port_bindings.to_string(),
                        Style::default().fg(PORT_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("TCP ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.process_stats.tcp_count.to_string(),
                        Style::default().fg(TCP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("UDP ", Style::default().fg(DIM_FG)),
                    Span::styled(
                        app.process_stats.udp_count.to_string(),
                        Style::default().fg(UDP_COLOR).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("Mem {:.0} MB", mem_mb),
                        Style::default().fg(MEM_COLOR).add_modifier(Modifier::BOLD),
                    ),
                ]),
            ]
        }
    };

    let stats = ratatui::widgets::Paragraph::new(stats_lines).alignment(Alignment::Left);
    f.render_widget(stats, chunks[2]);

    let help = ratatui::widgets::Paragraph::new("Tab switch view")
        .style(Style::default().fg(DIM_FG))
        .alignment(Alignment::Right);
    f.render_widget(help, chunks[3]);
}

fn draw_filter(f: &mut Frame, app: &App, area: Rect) {
    let border_color = if app.filter_active {
        BORDER_ACTIVE
    } else {
        BORDER_IDLE
    };

    let (filter_text, text_style) = if app.filter.is_empty() && !app.filter_active {
        (
            "Press / to filter...".to_string(),
            Style::default().fg(DIM_FG),
        )
    } else {
        (
            app.filter.clone(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));

    let paragraph = ratatui::widgets::Paragraph::new(Line::from(vec![
        Span::styled(" / ", Style::default().fg(ACCENT)),
        Span::styled(filter_text, text_style),
    ]))
    .block(block);
    f.render_widget(paragraph, area);
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    match app.view_mode {
        ViewMode::Ports => draw_ports_table(f, app, area),
        ViewMode::Processes => draw_processes_table(f, app, area),
    }
}

fn draw_ports_table(f: &mut Frame, app: &App, area: Rect) {
    let total = app.filtered_ports.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_IDLE))
        .title(format!(" Ports ({total}) "))
        .title_style(Style::default().fg(Color::White));

    let constraints = [
        Constraint::Length(7),
        Constraint::Length(18),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Min(10),
    ];

    let header = header_row(["Proto", "Addr", "Port", "PID", "Mem(MB)", "Process"]);
    let rows: Vec<Row> = app
        .filtered_ports
        .iter()
        .map(|entry| {
            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
            Row::new(vec![
                Cell::new(entry.proto.clone()).style(Style::default().fg(
                    if entry.proto.starts_with("TCP") {
                        TCP_COLOR
                    } else {
                        UDP_COLOR
                    },
                )),
                Cell::new(entry.local_addr.clone()).style(Style::default().fg(Color::White)),
                Cell::new(entry.port.clone()).style(Style::default().fg(PORT_COLOR)),
                Cell::new(entry.pid.to_string()).style(Style::default().fg(Color::White)),
                Cell::new(format!("{mem_mb:.1}")).style(Style::default().fg(MEM_COLOR)),
                Cell::new(entry.name.clone()).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    render_table(
        f,
        area,
        block,
        constraints,
        header,
        rows,
        app.selected,
        total,
    );
}

fn draw_processes_table(f: &mut Frame, app: &App, area: Rect) {
    let total = app.filtered_processes.len();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER_IDLE))
        .title(format!(" Processes ({total}) "))
        .title_style(Style::default().fg(Color::White));

    let constraints = [
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(10),
        Constraint::Min(20),
    ];

    let header = header_row(["PID", "TCP", "UDP", "Mem(MB)", "Process / Ports"]);
    let rows: Vec<Row> = app
        .filtered_processes
        .iter()
        .map(|entry| {
            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
            Row::new(vec![
                Cell::new(entry.pid.to_string()).style(Style::default().fg(Color::White)),
                Cell::new(entry.tcp_ports.to_string()).style(Style::default().fg(TCP_COLOR)),
                Cell::new(entry.udp_ports.to_string()).style(Style::default().fg(UDP_COLOR)),
                Cell::new(format!("{mem_mb:.1}")).style(Style::default().fg(MEM_COLOR)),
                Cell::new(format_process_summary(entry)).style(Style::default().fg(Color::White)),
            ])
        })
        .collect();

    render_table(
        f,
        area,
        block,
        constraints,
        header,
        rows,
        app.selected,
        total,
    );
}

fn header_row<'a, const N: usize>(headers: [&'a str; N]) -> Row<'a> {
    let cells = headers.into_iter().map(|header| {
        Cell::new(header).style(
            Style::default()
                .fg(HEADER_FG)
                .bg(HEADER_BG)
                .add_modifier(Modifier::BOLD),
        )
    });
    Row::new(cells).style(Style::default().bg(HEADER_BG))
}

fn render_table<const N: usize>(
    f: &mut Frame,
    area: Rect,
    block: Block<'static>,
    constraints: [Constraint; N],
    header: Row<'static>,
    rows: Vec<Row<'static>>,
    selected: usize,
    total: usize,
) {
    let table = Table::new(rows, constraints)
        .header(header)
        .block(block)
        .highlight_symbol("\u{2503} ")
        .highlight_spacing(HighlightSpacing::Always)
        .row_highlight_style(
            Style::default()
                .bg(ACCENT_BG)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );

    let mut state = TableState::default();
    if total > 0 {
        state.select(Some(selected));
        let visible_rows = area.height.saturating_sub(4) as usize;
        if visible_rows > 0 {
            let mid = visible_rows / 2;
            let offset = selected.saturating_sub(mid);
            *state.offset_mut() = offset;
        }
    }
    f.render_stateful_widget(table, area, &mut state);

    if total > 0 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(ACCENT))
            .track_style(Style::default().fg(Color::Rgb(40, 44, 52)));
        let mut scroll_state = ScrollbarState::new(total).position(selected);
        f.render_stateful_widget(scrollbar, area, &mut scroll_state);
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let keys = if app.filter_active {
        vec![("Enter/Esc", "Stop"), ("Type", "Filter")]
    } else {
        vec![
            ("q", "Quit"),
            ("k", "Kill"),
            ("/", "Filter"),
            ("r", "Refresh"),
            ("Tab", "View"),
            ("\u{2191}\u{2193}", "Nav"),
            ("PgUp/PgDn", "Jump"),
        ]
    };

    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default()
                .fg(Color::Rgb(150, 150, 160))
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(DIM_FG),
        ));
    }

    let line = if app.status_msg.is_empty() {
        Line::from(spans)
    } else {
        let msg_color = if app.status_msg.contains("Failed") || app.status_msg.contains("not found")
        {
            Color::Red
        } else {
            Color::Green
        };
        let mut msg_spans = vec![
            Span::styled(
                format!(" {} ", app.status_msg),
                Style::default().fg(msg_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("|", Style::default().fg(Color::Rgb(60, 64, 72))),
            Span::raw(" "),
        ];
        msg_spans.extend(spans);
        Line::from(msg_spans)
    };

    let paragraph = ratatui::widgets::Paragraph::new(line)
        .style(Style::default().bg(Color::Rgb(28, 30, 36)))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn format_process_summary(entry: &winportkill_core::ProcessViewEntry) -> String {
    if entry.ports.is_empty() {
        return format!("{}  |  No listening ports", entry.name);
    }

    let preview = entry
        .ports
        .iter()
        .take(2)
        .map(|port| format!("{} {}:{}", port.proto, port.local_addr, port.port))
        .collect::<Vec<_>>()
        .join(" | ");

    if entry.ports.len() > 2 {
        format!(
            "{}  |  {} | +{} more",
            entry.name,
            preview,
            entry.ports.len() - 2
        )
    } else {
        format!("{}  |  {}", entry.name, preview)
    }
}
