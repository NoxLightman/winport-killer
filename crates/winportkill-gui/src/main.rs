mod app;

use app::App;
use eframe::{NativeOptions, Renderer, egui};

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("WinPortKill"),
        renderer: Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "WinPortKill",
        options,
        Box::new(|_cc| Ok(Box::new(App::new()))),
    )
}
