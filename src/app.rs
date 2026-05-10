use crossterm::event::{KeyCode, KeyEvent};
use winportkill_core::{Entry, Stats, scan, filter, kill, stats};

/// 应用状态
pub struct App {
    pub entries: Vec<Entry>,
    pub filtered: Vec<Entry>,
    pub filter: String,
    pub filter_active: bool,
    pub selected: usize,
    pub should_quit: bool,
    pub status_msg: String,
    pub stats: Stats,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            entries: Vec::new(),
            filtered: Vec::new(),
            filter: String::new(),
            filter_active: false,
            selected: 0,
            should_quit: false,
            status_msg: String::new(),
            stats: Stats {
                total_procs: 0,
                tcp_count: 0,
                udp_count: 0,
                total_mem_bytes: 0,
            },
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.entries = scan();
        self.stats = stats(&self.entries);
        self.apply_filter();
        self.status_msg.clear();
    }

    fn apply_filter(&mut self) {
        self.filtered = filter(&self.entries, &self.filter);
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.filter_active {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    self.filter_active = false;
                }
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.apply_filter();
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.apply_filter();
                }
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('k') => self.kill_selected(),
            KeyCode::Char('/') => {
                self.filter_active = true;
            }
            KeyCode::Char('r') => {
                self.refresh();
                self.status_msg = "Refreshed".to_string();
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered.len() {
                    self.selected += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + 10).min(self.filtered.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    fn kill_selected(&mut self) {
        if let Some(entry) = self.filtered.get(self.selected) {
            let pid = entry.pid;
            match kill(pid) {
                Ok(name) => {
                    self.status_msg = format!("Killed PID {} ({})", pid, name);
                    self.refresh();
                }
                Err(msg) => {
                    self.status_msg = msg;
                }
            }
        }
    }
}