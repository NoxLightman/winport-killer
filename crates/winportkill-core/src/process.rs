use crate::port::{self, PortEntry};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_TERMINATE, TerminateProcess,
    WaitForSingleObject,
};

#[derive(Clone, Debug, Serialize)]
pub struct PortViewEntry {
    pub proto: String,
    pub local_addr: String,
    pub port: String,
    pub pid: u32,
    pub name: String,
    pub memory: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProcessViewEntry {
    pub pid: u32,
    pub name: String,
    pub memory: u64,
    pub tcp_ports: usize,
    pub udp_ports: usize,
    pub ports: Vec<PortBinding>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PortBinding {
    pub proto: String,
    pub local_addr: String,
    pub port: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PortViewStats {
    pub total_rows: usize,
    pub total_procs: usize,
    pub tcp_count: usize,
    pub udp_count: usize,
    pub total_mem_bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProcessViewStats {
    pub total_procs: usize,
    pub procs_with_ports: usize,
    pub total_port_bindings: usize,
    pub tcp_count: usize,
    pub udp_count: usize,
    pub total_mem_bytes: u64,
}

#[derive(Clone, Debug)]
struct ProcessSnapshot {
    pid: u32,
    name: String,
    memory: u64,
    ports: Vec<PortEntry>,
}

pub type Entry = PortViewEntry;
pub type Stats = PortViewStats;

pub fn scan_ports() -> Vec<PortViewEntry> {
    snapshots_to_port_entries(scan_snapshots())
}

pub fn scan_processes() -> Vec<ProcessViewEntry> {
    snapshots_to_process_entries(scan_snapshots())
}

pub fn filter_ports(entries: &[PortViewEntry], keyword: &str) -> Vec<PortViewEntry> {
    let kw = keyword.to_lowercase();
    entries
        .iter()
        .filter(|entry| {
            kw.is_empty()
                || entry.pid.to_string().contains(&kw)
                || entry.name.to_lowercase().contains(&kw)
                || entry.port.contains(&kw)
                || entry.proto.to_lowercase().contains(&kw)
                || entry.local_addr.contains(&kw)
        })
        .cloned()
        .collect()
}

pub fn filter_processes(entries: &[ProcessViewEntry], keyword: &str) -> Vec<ProcessViewEntry> {
    let kw = keyword.to_lowercase();
    entries
        .iter()
        .filter(|entry| {
            kw.is_empty()
                || entry.pid.to_string().contains(&kw)
                || entry.name.to_lowercase().contains(&kw)
                || entry.ports.iter().any(|binding| {
                    binding.port.contains(&kw)
                        || binding.proto.to_lowercase().contains(&kw)
                        || binding.local_addr.contains(&kw)
                })
        })
        .cloned()
        .collect()
}

pub fn port_stats(entries: &[PortViewEntry]) -> PortViewStats {
    let unique_pids: HashSet<u32> = entries.iter().map(|entry| entry.pid).collect();
    let mut mem_by_pid: HashMap<u32, u64> = HashMap::new();
    for entry in entries {
        mem_by_pid.entry(entry.pid).or_insert(entry.memory);
    }

    PortViewStats {
        total_rows: entries.len(),
        total_procs: unique_pids.len(),
        tcp_count: entries
            .iter()
            .filter(|entry| entry.proto.starts_with("TCP"))
            .count(),
        udp_count: entries
            .iter()
            .filter(|entry| entry.proto.starts_with("UDP"))
            .count(),
        total_mem_bytes: mem_by_pid.values().sum(),
    }
}

pub fn process_stats(entries: &[ProcessViewEntry]) -> ProcessViewStats {
    ProcessViewStats {
        total_procs: entries.len(),
        procs_with_ports: entries
            .iter()
            .filter(|entry| !entry.ports.is_empty())
            .count(),
        total_port_bindings: entries.iter().map(|entry| entry.ports.len()).sum(),
        tcp_count: entries.iter().map(|entry| entry.tcp_ports).sum(),
        udp_count: entries.iter().map(|entry| entry.udp_ports).sum(),
        total_mem_bytes: entries.iter().map(|entry| entry.memory).sum(),
    }
}

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
            if sys.process(pid_key).is_none() {
                Err(format!("PID {} not found", pid))
            } else {
                terminate_process(pid).map(|()| name)
            }
        }
        None => Err(format!("PID {} not found", pid)),
    }
}

fn terminate_process(pid: u32) -> Result<(), String> {
    const SYNCHRONIZE_ACCESS: u32 = 0x0010_0000;
    let desired_access =
        PROCESS_ACCESS_RIGHTS(PROCESS_TERMINATE.0 | SYNCHRONIZE_ACCESS);
    let handle = unsafe { OpenProcess(desired_access, false, pid) }
        .map_err(|error| format!("Failed to open PID {}: {}", pid, error))?;

    let terminate_result = unsafe { TerminateProcess(handle, 1) };
    if let Err(error) = terminate_result {
        unsafe {
            let _ = CloseHandle(handle);
        }
        return Err(format!("Failed to kill PID {}: {}", pid, error));
    }

    let wait_result =
        unsafe { WaitForSingleObject(handle, wait_timeout_ms(Duration::from_secs(2))) };
    unsafe {
        let _ = CloseHandle(handle);
    }

    if wait_result == WAIT_OBJECT_0 {
        Ok(())
    } else {
        Err(format!("Timed out waiting for PID {} to exit", pid))
    }
}

fn wait_timeout_ms(duration: Duration) -> u32 {
    duration.as_millis().min(u128::from(u32::MAX)) as u32
}

pub fn scan() -> Vec<Entry> {
    scan_ports()
}

pub fn filter(entries: &[Entry], keyword: &str) -> Vec<Entry> {
    filter_ports(entries, keyword)
}

pub fn stats(entries: &[Entry]) -> Stats {
    port_stats(entries)
}

fn scan_snapshots() -> Vec<ProcessSnapshot> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut port_map: HashMap<u32, Vec<PortEntry>> = HashMap::new();
    for port_entry in port::get_port_entries() {
        port_map.entry(port_entry.pid).or_default().push(port_entry);
    }

    let mut snapshots: Vec<ProcessSnapshot> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessSnapshot {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().to_string(),
            memory: process.memory(),
            ports: port_map.remove(&pid.as_u32()).unwrap_or_default(),
        })
        .collect();

    snapshots.sort_by(|a, b| {
        let a_has_ports = !a.ports.is_empty();
        let b_has_ports = !b.ports.is_empty();
        b_has_ports
            .cmp(&a_has_ports)
            .then(b.memory.cmp(&a.memory))
            .then(a.name.cmp(&b.name))
    });

    snapshots
}

fn snapshots_to_port_entries(snapshots: Vec<ProcessSnapshot>) -> Vec<PortViewEntry> {
    let mut entries = Vec::new();
    let mut seen_ports: HashSet<(u32, String, String, u16)> = HashSet::new();

    for snapshot in snapshots {
        for port in snapshot.ports {
            let key = (
                snapshot.pid,
                port.proto.clone(),
                port.local_addr.clone(),
                port.port,
            );
            if seen_ports.insert(key) {
                entries.push(PortViewEntry {
                    proto: port.proto,
                    local_addr: port.local_addr,
                    port: port.port.to_string(),
                    pid: snapshot.pid,
                    name: snapshot.name.clone(),
                    memory: snapshot.memory,
                });
            }
        }
    }

    entries.sort_by(|a, b| {
        b.memory
            .cmp(&a.memory)
            .then(a.pid.cmp(&b.pid))
            .then(a.proto.cmp(&b.proto))
            .then(a.local_addr.cmp(&b.local_addr))
            .then(a.port.cmp(&b.port))
    });

    entries
}

fn snapshots_to_process_entries(snapshots: Vec<ProcessSnapshot>) -> Vec<ProcessViewEntry> {
    let mut entries: Vec<ProcessViewEntry> = snapshots
        .into_iter()
        .map(|snapshot| {
            let mut seen_ports: HashSet<(String, String, u16)> = HashSet::new();
            let mut ports = Vec::new();
            let mut tcp_ports = 0usize;
            let mut udp_ports = 0usize;

            for port in snapshot.ports {
                let key = (port.proto.clone(), port.local_addr.clone(), port.port);
                if seen_ports.insert(key) {
                    if port.proto.starts_with("TCP") {
                        tcp_ports += 1;
                    } else if port.proto.starts_with("UDP") {
                        udp_ports += 1;
                    }

                    ports.push(PortBinding {
                        proto: port.proto,
                        local_addr: port.local_addr,
                        port: port.port.to_string(),
                    });
                }
            }

            ports.sort_by(|a, b| {
                a.proto
                    .cmp(&b.proto)
                    .then(a.local_addr.cmp(&b.local_addr))
                    .then(a.port.cmp(&b.port))
            });

            ProcessViewEntry {
                pid: snapshot.pid,
                name: snapshot.name,
                memory: snapshot.memory,
                tcp_ports,
                udp_ports,
                ports,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_has_ports = !a.ports.is_empty();
        let b_has_ports = !b.ports.is_empty();
        b_has_ports
            .cmp(&a_has_ports)
            .then(b.memory.cmp(&a.memory))
            .then(a.name.cmp(&b.name))
    });

    entries
}
