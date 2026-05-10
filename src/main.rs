mod app;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 开启 raw mode：关闭终端默认行为（按键回显、Ctrl+C 直接退出等），
    // 让程序完全掌控输入输出
    enable_raw_mode()?;

    // 切换到备用屏幕缓冲区，退出时原终端内容自动恢复，不会留下乱码
    execute!(std::io::stdout(), EnterAlternateScreen)?;

    // 创建 ratatui 终端后端，基于 crossterm 实现跨平台渲染
    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // 初始化应用状态（首次 refresh 加载所有端口和进程数据）
    let mut app = App::new();

    // UI 刷新间隔：每秒重绘（时钟按秒更新）
    let ui_tick = Duration::from_secs(1);
    // 进程数据刷新间隔：每 10 秒重新拉取端口和进程数据
    let data_tick = Duration::from_secs(10);
    let mut last_ui_tick = Instant::now();
    let mut last_data_tick = Instant::now();

    // 主循环：绘制界面 → 等待输入 → 刷新数据，周而复始
    while !app.should_quit {
        // 将当前 app 状态渲染到屏幕
        terminal.draw(|f| ui::draw(f, &app))?;

        // poll 超时取距离下次 UI tick 的剩余时间
        let timeout = ui_tick.saturating_sub(last_ui_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Windows 上 crossterm 会对每次按键同时触发 Press 和 Release 事件，
                // 只处理 Press 避免按键被识别两次
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }

        // 每秒触发一次重绘周期（时钟会自动更新）
        if last_ui_tick.elapsed() >= ui_tick {
            last_ui_tick = Instant::now();
        }

        // 每 10 秒重新拉取进程数据
        if last_data_tick.elapsed() >= data_tick {
            app.refresh();
            last_data_tick = Instant::now();
        }
    }

    // 退出清理：恢复终端到正常状态
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
