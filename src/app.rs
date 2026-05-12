use crossterm::event::{KeyCode, KeyEvent};
use winportkill_core::{
    PortViewEntry, PortViewStats, ProcessViewEntry, ProcessViewStats, filter_ports,
    filter_processes, kill, port_stats, process_stats, scan_ports, scan_processes,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Ports,
    Processes,
}

pub struct App {
    pub view_mode: ViewMode,
    pub port_entries: Vec<PortViewEntry>,
    pub filtered_ports: Vec<PortViewEntry>,
    pub process_entries: Vec<ProcessViewEntry>,
    pub filtered_processes: Vec<ProcessViewEntry>,
    pub filter: String,
    pub filter_active: bool,
    pub selected: usize,
    pub should_quit: bool,
    pub status_msg: String,
    pub port_stats: PortViewStats,
    pub process_stats: ProcessViewStats,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            view_mode: ViewMode::Ports,
            port_entries: Vec::new(),
            filtered_ports: Vec::new(),
            process_entries: Vec::new(),
            filtered_processes: Vec::new(),
            filter: String::new(),
            filter_active: false,
            selected: 0,
            should_quit: false,
            status_msg: String::new(),
            port_stats: PortViewStats {
                total_rows: 0,
                total_procs: 0,
                tcp_count: 0,
                udp_count: 0,
                total_mem_bytes: 0,
            },
            process_stats: ProcessViewStats {
                total_procs: 0,
                procs_with_ports: 0,
                total_port_bindings: 0,
                tcp_count: 0,
                udp_count: 0,
                total_mem_bytes: 0,
            },
        };
        app.refresh();
        app
    }

    pub fn refresh(&mut self) {
        self.port_entries = scan_ports();
        self.process_entries = scan_processes();
        self.apply_filter();
        self.port_stats = port_stats(&self.filtered_ports);
        self.process_stats = process_stats(&self.filtered_processes);
        self.status_msg.clear();
    }

    pub fn current_len(&self) -> usize {
        match self.view_mode {
            ViewMode::Ports => self.filtered_ports.len(),
            ViewMode::Processes => self.filtered_processes.len(),
        }
    }

    pub fn current_pid(&self) -> Option<u32> {
        match self.view_mode {
            ViewMode::Ports => self
                .filtered_ports
                .get(self.selected)
                .map(|entry| entry.pid),
            ViewMode::Processes => self
                .filtered_processes
                .get(self.selected)
                .map(|entry| entry.pid),
        }
    }

    pub fn apply_filter(&mut self) {
        self.filtered_ports = filter_ports(&self.port_entries, &self.filter);
        self.filtered_processes = filter_processes(&self.process_entries, &self.filter);
        if self.selected >= self.current_len() {
            self.selected = self.current_len().saturating_sub(1);
        }
    }

    pub fn switch_view(&mut self, view_mode: ViewMode) {
        if self.view_mode != view_mode {
            self.view_mode = view_mode;
            if self.selected >= self.current_len() {
                self.selected = self.current_len().saturating_sub(1);
            }
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
                    self.port_stats = port_stats(&self.filtered_ports);
                    self.process_stats = process_stats(&self.filtered_processes);
                }
                KeyCode::Char(c) => {
                    self.filter.push(c);
                    self.apply_filter();
                    self.port_stats = port_stats(&self.filtered_ports);
                    self.process_stats = process_stats(&self.filtered_processes);
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
            KeyCode::Tab => {
                let next = match self.view_mode {
                    ViewMode::Ports => ViewMode::Processes,
                    ViewMode::Processes => ViewMode::Ports,
                };
                self.switch_view(next);
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.current_len() {
                    self.selected += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + 10).min(self.current_len().saturating_sub(1));
            }
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                self.selected = self.current_len().saturating_sub(1);
            }
            _ => {}
        }
    }

    fn kill_selected(&mut self) {
        if let Some(pid) = self.current_pid() {
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
