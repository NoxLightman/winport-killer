use crate::port::{self, PortEntry};
use crossterm::event::{KeyCode, KeyEvent};
use std::collections::{HashMap, HashSet};
use sysinfo::{ProcessesToUpdate, System};

/// 列表中每一行数据：进程 + 端口信息合并
/// 没有端口占用的进程，port 字段为 "-"
#[derive(Clone, Debug)]
pub struct Entry {
    pub proto: String,      // 协议（TCP/TCP6/UDP/UDP6），无端口时为空
    pub local_addr: String, // 监听地址，无端口时为空
    pub port: String,       // 端口号字符串，无端口时为 "-"
    pub pid: u32,           // 进程 PID
    pub name: String,       // 进程名
    pub memory: u64,        // 进程内存占用（字节）
}

/// 应用状态：包含所有数据、过滤、选中项等
pub struct App {
    pub entries: Vec<Entry>,         // 全量数据
    pub filtered: Vec<Entry>,        // 过滤后的数据
    pub filter: String,              // 过滤关键字
    pub filter_active: bool,         // 是否正在输入过滤关键字
    pub selected: usize,             // 当前选中行索引
    pub should_quit: bool,           // 是否退出
    pub status_msg: String,          // 状态栏消息（如 kill 结果）
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
        };
        // 初始化时加载一次数据
        app.refresh();
        app
    }

    /// 刷新数据：重新获取所有端口和进程信息，合并为统一列表
    pub fn refresh(&mut self) {
        // 从 sysinfo 获取全部进程列表
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        // 从 port 模块获取所有监听端口，按 PID 分组方便后续匹配
        let port_entries = port::get_port_entries();
        let mut port_map: HashMap<u32, Vec<&PortEntry>> = HashMap::new();
        for pe in &port_entries {
            port_map.entry(pe.pid).or_default().push(pe);
        }

        // 合成统一列表，去重：同一 PID 的 (proto, addr, port) 组合只保留一行
        self.entries.clear();
        let mut seen_ports: HashSet<(u32, String, String, u16)> = HashSet::new();
        for (pid, proc) in sys.processes() {
            let pid_u32 = pid.as_u32();
            let name = proc.name().to_string_lossy().to_string();
            let memory = proc.memory();

            if let Some(ports) = port_map.get(&pid_u32) {
                // 有端口的进程：每个端口生成一行
                for pe in ports {
                    let key = (pe.pid, pe.proto.clone(), pe.local_addr.clone(), pe.port);
                    if seen_ports.insert(key) {
                        self.entries.push(Entry {
                            proto: pe.proto.clone(),
                            local_addr: pe.local_addr.clone(),
                            port: pe.port.to_string(),
                            pid: pid_u32,
                            name: name.clone(),
                            memory,
                        });
                    }
                }
            } else {
                // 没有端口的进程：用 "-" 占位
                self.entries.push(Entry {
                    proto: String::new(),
                    local_addr: String::new(),
                    port: "-".to_string(),
                    pid: pid_u32,
                    name,
                    memory,
                });
            }
        }

        // 排序：有端口的进程排在前面，再按内存大小降序
        self.entries.sort_by(|a, b| {
            let a_has_port = a.port != "-";
            let b_has_port = b.port != "-";
            b_has_port.cmp(&a_has_port).then(b.memory.cmp(&a.memory))
        });

        // 排序后重新应用过滤
        self.apply_filter();
        self.status_msg.clear();
    }

    /// 根据当前 filter 关键字过滤 entries
    /// 搜索范围：PID、进程名、端口号、协议、地址
    fn apply_filter(&mut self) {
        let filter_lower = self.filter.to_lowercase();
        self.filtered = self
            .entries
            .iter()
            .filter(|e| {
                if filter_lower.is_empty() {
                    return true;
                }
                e.pid.to_string().contains(&filter_lower)
                    || e.name.to_lowercase().contains(&filter_lower)
                    || e.port.contains(&filter_lower)
                    || e.proto.to_lowercase().contains(&filter_lower)
                    || e.local_addr.contains(&filter_lower)
            })
            .cloned()
            .collect();
        // 选中行不能超出过滤后的列表长度
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    /// 处理键盘输入
    /// 过滤模式（filter_active）下只处理文字输入
    /// 正常模式下处理导航、kill、退出等快捷键
    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.filter_active {
            match key.code {
                // Enter 或 Esc 退出过滤模式
                KeyCode::Enter | KeyCode::Esc => {
                    self.filter_active = false;
                }
                // Backspace 删除最后一个字符并重新过滤
                KeyCode::Backspace => {
                    self.filter.pop();
                    self.apply_filter();
                }
                // 输入字符追加到过滤关键字并实时过滤
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
                // / 进入过滤模式
                self.filter_active = true;
            }
            KeyCode::Char('r') => {
                // r 手动刷新
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

    /// Kill 选中行对应的进程
    /// 通过 sysinfo 的 Process::kill() 发送终止信号
    /// 成功后自动刷新列表；失败提示可能需要管理员权限
    fn kill_selected(&mut self) {
        if let Some(entry) = self.filtered.get(self.selected) {
            let pid = entry.pid;
            let mut sys = System::new();
            sys.refresh_processes(ProcessesToUpdate::All, true);
            // sysinfo 的 PID 类型�� Pid，需要从 u32 查找对应的 Pid
            if let Some(pid_val) = sys.processes().iter().find_map(|(p, _)| {
                if p.as_u32() == pid { Some(*p) } else { None }
            }) {
                if let Some(proc) = sys.process(pid_val) {
                    if proc.kill() {
                        self.status_msg = format!("Killed PID {} ({})", pid, entry.name);
                        self.refresh();
                    } else {
                        self.status_msg = format!("Failed to kill PID {} (need admin?)", pid);
                    }
                }
            } else {
                self.status_msg = format!("PID {} not found", pid);
            }
        }
    }
}