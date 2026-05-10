use crate::port::{self, PortEntry};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use sysinfo::{ProcessesToUpdate, System};

/// 列表中每一行数据：进程 + 端口信息合并
#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub proto: String,
    pub local_addr: String,
    pub port: String,
    pub pid: u32,
    pub name: String,
    pub memory: u64,
}

/// 统计信息
#[derive(Clone, Debug, Serialize)]
pub struct Stats {
    pub total_procs: usize,
    pub tcp_count: usize,
    pub udp_count: usize,
    pub total_mem_bytes: u64,
}

/// 扫描所有端口和进程，合并为统一列表
pub fn scan() -> Vec<Entry> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let port_entries = port::get_port_entries();
    let mut port_map: HashMap<u32, Vec<&PortEntry>> = HashMap::new();
    for pe in &port_entries {
        port_map.entry(pe.pid).or_default().push(pe);
    }

    let mut entries = Vec::new();
    let mut seen_ports: HashSet<(u32, String, String, u16)> = HashSet::new();
    for (pid, proc) in sys.processes() {
        let pid_u32 = pid.as_u32();
        let name = proc.name().to_string_lossy().to_string();
        let memory = proc.memory();

        if let Some(ports) = port_map.get(&pid_u32) {
            for pe in ports {
                let key = (pe.pid, pe.proto.clone(), pe.local_addr.clone(), pe.port);
                if seen_ports.insert(key) {
                    entries.push(Entry {
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
            entries.push(Entry {
                proto: String::new(),
                local_addr: String::new(),
                port: "-".to_string(),
                pid: pid_u32,
                name,
                memory,
            });
        }
    }

    entries.sort_by(|a, b| {
        let a_has_port = a.port != "-";
        let b_has_port = b.port != "-";
        b_has_port.cmp(&a_has_port).then(b.memory.cmp(&a.memory))
    });

    entries
}

/// 根据关键字过滤列表
pub fn filter(entries: &[Entry], keyword: &str) -> Vec<Entry> {
    let kw = keyword.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            if kw.is_empty() {
                return true;
            }
            e.pid.to_string().contains(&kw)
                || e.name.to_lowercase().contains(&kw)
                || e.port.contains(&kw)
                || e.proto.to_lowercase().contains(&kw)
                || e.local_addr.contains(&kw)
        })
        .cloned()
        .collect()
}

/// 终止指定 PID 的进程
/// 返回 Ok(进程名) 表示成功，Err 返回错误描述
pub fn kill(pid: u32) -> Result<String, String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let pid_val = sys.processes().iter().find_map(|(p, proc)| {
        if p.as_u32() == pid {
            Some((*p, proc.name().to_string_lossy().to_string()))
        } else {
            None
        }
    });

    match pid_val {
        Some((pid_key, name)) => {
            if let Some(proc) = sys.process(pid_key) {
                if proc.kill() {
                    Ok(name)
                } else {
                    Err(format!("Failed to kill PID {} (need admin?)", pid))
                }
            } else {
                Err(format!("PID {} not found", pid))
            }
        }
        None => Err(format!("PID {} not found", pid)),
    }
}

/// 从 entries 计算统计信息
pub fn stats(entries: &[Entry]) -> Stats {
    Stats {
        total_procs: entries.len(),
        tcp_count: entries.iter().filter(|e| e.proto.contains("TCP")).count(),
        udp_count: entries.iter().filter(|e| e.proto.contains("UDP")).count(),
        total_mem_bytes: entries.iter().map(|e| e.memory).sum(),
    }
}