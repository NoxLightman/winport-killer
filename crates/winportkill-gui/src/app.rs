use eframe::App as EframeApp;
use egui::{Color32, RichText};
use std::time::{Duration, Instant};
use winportkill_core::{scan, filter, kill, stats, Entry, Stats};

pub struct App {
    entries: Vec<Entry>,
    filtered: Vec<Entry>,
    stats: Stats,
    filter: String,
    selected: Option<usize>,
    status_msg: String,
    last_refresh: Instant,
    refresh_interval: Duration,
}

impl App {
    pub fn new() -> Self {
        let entries = scan();
        let stats = stats(&entries);
        Self {
            filtered: entries.clone(),
            entries,
            stats,
            filter: String::new(),
            selected: None,
            status_msg: String::new(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(10),
        }
    }

    fn refresh_data(&mut self) {
        self.entries = scan();
        self.stats = stats(&self.entries);
        self.apply_filter();
        self.last_refresh = Instant::now();
        self.status_msg.clear();
    }

    fn apply_filter(&mut self) {
        self.filtered = filter(&self.entries, &self.filter);
        if let Some(sel) = self.selected {
            if sel >= self.filtered.len() {
                self.selected = self.filtered.len().checked_sub(1);
            }
        }
    }

    fn kill_selected(&mut self) {
        if let Some(sel) = self.selected {
            if let Some(entry) = self.filtered.get(sel) {
                let pid = entry.pid;
                match kill(pid) {
                    Ok(name) => {
                        self.status_msg = format!("Killed PID {} ({})", pid, name);
                        self.refresh_data();
                    }
                    Err(msg) => {
                        self.status_msg = msg;
                    }
                }
            }
        }
    }
}

impl EframeApp for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 自动刷新
        if self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_data();
        }
        ctx.request_repaint_after(Duration::from_secs(1));

        // 顶部统计面板
        egui::TopBottomPanel::top("stats_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("WinPortKill").color(Color32::from_rgb(255, 220, 50)).size(18.0).strong());
                ui.separator();
                ui.label(RichText::new(format!("Procs: {}", self.stats.total_procs)).color(Color32::WHITE).strong());
                ui.label(RichText::new(format!("TCP: {}", self.stats.tcp_count)).color(Color32::from_rgb(0, 200, 255)).strong());
                ui.label(RichText::new(format!("UDP: {}", self.stats.udp_count)).color(Color32::from_rgb(0, 200, 0)).strong());
                let mem_mb = self.stats.total_mem_bytes as f64 / 1024.0 / 1024.0;
                let mem_str = if mem_mb >= 1024.0 { format!("{:.1} GB", mem_mb / 1024.0) } else { format!("{:.0} MB", mem_mb) };
                ui.label(RichText::new(format!("Mem: {}", mem_str)).color(Color32::from_rgb(200, 100, 255)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Refresh").clicked() {
                        self.refresh_data();
                        self.status_msg = "Refreshed".to_string();
                    }
                    if ui.button("Kill (k)").clicked() {
                        self.kill_selected();
                    }
                });
            });
        });

        // 底部过滤栏 + 状态
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                let response = ui.text_edit_singleline(&mut self.filter);
                if response.changed() {
                    self.apply_filter();
                }
                ui.separator();
                if self.status_msg.is_empty() {
                    ui.label(RichText::new("Select a row and press Kill or 'k'").color(Color32::GRAY));
                } else if self.status_msg.contains("Failed") || self.status_msg.contains("not found") {
                    ui.label(RichText::new(&self.status_msg).color(Color32::RED));
                } else {
                    ui.label(RichText::new(&self.status_msg).color(Color32::GREEN));
                }
            });
        });

        // 主表格区域
        egui::CentralPanel::default().show(ctx, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::exact(60.0))   // Proto
                .column(egui_extras::Column::exact(130.0))   // Addr
                .column(egui_extras::Column::exact(60.0))    // Port
                .column(egui_extras::Column::exact(60.0))    // PID
                .column(egui_extras::Column::exact(80.0))    // Mem
                .column(egui_extras::Column::remainder())    // Process
                .header(22.0, |mut header| {
                    header.col(|ui| { ui.strong("Proto"); });
                    header.col(|ui| { ui.strong("Addr"); });
                    header.col(|ui| { ui.strong("Port"); });
                    header.col(|ui| { ui.strong("PID"); });
                    header.col(|ui| { ui.strong("Mem(MB)"); });
                    header.col(|ui| { ui.strong("Process"); });
                })
                .body(|mut body| {
                    for (i, entry) in self.filtered.iter().enumerate() {
                        let is_selected = self.selected == Some(i);
                        let row_color = if is_selected {
                            Some(Color32::from_rgb(30, 60, 90))
                        } else {
                            None
                        };
                        body.row(20.0, |mut row| {
                            if let Some(color) = row_color {
                                row.col(|ui| {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, color);
                                });
                            }
                            let proto_color = match entry.proto.as_str() {
                                "TCP" | "TCP6" => Color32::from_rgb(0, 200, 255),
                                "UDP" | "UDP6" => Color32::from_rgb(0, 200, 0),
                                _ => Color32::GRAY,
                            };
                            let has_port = entry.port != "-";
                            let text_color = if has_port { Color32::WHITE } else { Color32::GRAY };
                            let port_color = if has_port { Color32::from_rgb(255, 220, 50) } else { Color32::GRAY };

                            row.col(|ui| { ui.label(RichText::new(&entry.proto).color(proto_color)); });
                            row.col(|ui| { ui.label(RichText::new(&entry.local_addr).color(text_color)); });
                            row.col(|ui| { ui.label(RichText::new(&entry.port).color(port_color)); });
                            row.col(|ui| { ui.label(RichText::new(entry.pid.to_string()).color(text_color)); });
                            let mem_mb = entry.memory as f64 / 1024.0 / 1024.0;
                            row.col(|ui| { ui.label(RichText::new(format!("{mem_mb:.1}")).color(Color32::from_rgb(200, 100, 255))); });
                            row.col(|ui| { ui.label(RichText::new(&entry.name).color(text_color)); });

                            // 点击选中
                            if row.response().clicked() {
                                self.selected = Some(i);
                            }
                        });
                    }
                });
        });

        // 键盘快捷键
        if ctx.input(|i| i.key_pressed(egui::Key::K)) {
            self.kill_selected();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            if let Some(sel) = self.selected {
                if sel + 1 < self.filtered.len() {
                    self.selected = Some(sel + 1);
                }
            } else if !self.filtered.is_empty() {
                self.selected = Some(0);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            if let Some(sel) = self.selected {
                if sel > 0 {
                    self.selected = Some(sel - 1);
                }
            }
        }
    }
}