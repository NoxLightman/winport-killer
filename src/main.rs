mod app;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::time::{Duration, Instant};

/// WinPortKill — Windows 端口进程管理工具
#[derive(Parser)]
#[command(name = "winportkill", version, about = "Windows 端口进程管理工具")]
struct Cli {
    /// 启动 HTTP server 模式（为 IDE 插件提供 API）
    #[arg(long)]
    serve: Option<u16>,

    /// 仅输出一次 JSON 格式的端口列表（用于脚本集成）
    #[arg(long)]
    json: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // --json 模式：输出一次 JSON 后退出
    if cli.json {
        let entries = winportkill_core::scan();
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    // --serve 模式：启动 HTTP server
    if let Some(port) = cli.serve {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        println!("WinPortKill server running at http://{}", addr);
        println!("API: /ports  /stats  /kill/:pid  /ports/filter/:keyword  /ws");
        tokio::runtime::Runtime::new()?.block_on(async {
            let app = winportkill_server::create_app();
            let listener = tokio::net::TcpListener::bind(addr).await?;
            axum::serve(listener, app).await?;
            Ok::<_, Box<dyn std::error::Error>>(())
        })?;
        return Ok(());
    }

    // 默认：TUI 模式
    run_tui()?;
    Ok(())
}

fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let ui_tick = Duration::from_secs(1);
    let data_tick = Duration::from_secs(10);
    let mut last_ui_tick = Instant::now();
    let mut last_data_tick = Instant::now();

    while !app.should_quit {
        terminal.draw(|f| ui::draw(f, &app))?;

        let timeout = ui_tick.saturating_sub(last_ui_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        if last_ui_tick.elapsed() >= ui_tick {
            last_ui_tick = Instant::now();
        }

        if last_data_tick.elapsed() >= data_tick {
            app.refresh();
            last_data_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
