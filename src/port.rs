use std::net::{Ipv4Addr, Ipv6Addr};
// Windows IP Helper API：用于查询 TCP/UDP 端口占用信息
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCPROW_OWNER_PID, MIB_TCP6ROW_OWNER_PID,
    MIB_UDPROW_OWNER_PID, MIB_UDP6ROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    UDP_TABLE_OWNER_PID,
};
use windows::Win32::Networking::WinSock::AF_INET;

/// IPv6 地址族常量（Windows SDK 中未直接导出，手动定义）
const AF_INET6: u32 = 23;

/// 单条端口占用记录，包含协议类型、本地地址、端口号和所属进程 PID
#[derive(Clone, Debug)]
pub struct PortEntry {
    pub proto: String,       // 协议标识："TCP" / "TCP6" / "UDP" / "UDP6"
    pub local_addr: String,  // 本地监听地址
    pub port: u16,           // 监听端口号
    pub pid: u32,            // 占用该端口的进程 PID
}

/// 获取当前系统中所有 TCP/UDP（IPv4 + IPv6）的端口占用列表
pub fn get_port_entries() -> Vec<PortEntry> {
    let mut entries = Vec::new();
    entries.extend(query_tcp4());
    entries.extend(query_tcp6());
    entries.extend(query_udp4());
    entries.extend(query_udp6());
    entries
}

/// 通用查表辅助函数：封装 Windows API 的两阶段调用模式
///
/// Windows 的 `GetExtendedTcpTable` / `GetExtendedUdpTable` 需要两步：
///   1. 先传空指针获取所需缓冲区大小
///   2. 再分配对应大小的缓冲区并填充数据
///
/// 返回包含完整表数据的 `Vec<u8>`，由调用方持有所有权并解析。
fn alloc_and_query<F>(query_fn: F) -> Option<Vec<u8>>
where
    F: Fn(*mut u32, Option<*mut std::ffi::c_void>) -> u32,
{
    let mut size: u32 = 0;
    // 第一次调用：仅获取所需缓冲区大小（buf=None 时 size 会被填入）
    if query_fn(&mut size, None) != 0 && size == 0 {
        return None;
    }

    let mut buf = vec![0u8; size as usize];
    // 第二次调用：将数据写入已分配的缓冲区
    if query_fn(&mut size, Some(buf.as_mut_ptr() as *mut _)) != 0 {
        return None;
    }

    Some(buf)
}

// ── IPv4 TCP ──────────────────────────────────────────────

/// 查询所有 IPv4 TCP 监听端口及其对应的 PID
fn query_tcp4() -> Vec<PortEntry> {
    alloc_and_query(|size, buf| unsafe {
        GetExtendedTcpTable(buf, size, false, AF_INET.0 as u32, TCP_TABLE_OWNER_PID_LISTENER, 0)
    })
    .map_or(Vec::new(), |buf| {
        // 表结构前 4 字节为 num_entries（u32），之后是连续的 MIB_TCPROW_OWNER_PID 结构体
        let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let rows = unsafe { buf.as_ptr().add(4) as *const MIB_TCPROW_OWNER_PID };
        (0..num_entries)
            .filter_map(|i| unsafe {
                let r = &*rows.add(i);
                let ip = u32::from_be(r.dwLocalAddr);
                let port = u16::from_be(r.dwLocalPort as u16);
                Some(PortEntry { proto: "TCP".into(), local_addr: Ipv4Addr::from(ip).to_string(), port, pid: r.dwOwningPid })
            })
            .collect()
    })
}

// ── IPv6 TCP ──────────────────────────────────────────────

/// 查询所有 IPv6 TCP 监听端口及其对应的 PID
fn query_tcp6() -> Vec<PortEntry> {
    alloc_and_query(|size, buf| unsafe {
        GetExtendedTcpTable(buf, size, false, AF_INET6, TCP_TABLE_OWNER_PID_LISTENER, 0)
    })
    .map_or(Vec::new(), |buf| {
        let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let rows = unsafe { buf.as_ptr().add(4) as *const MIB_TCP6ROW_OWNER_PID };
        (0..num_entries)
            .filter_map(|i| unsafe {
                let r = &*rows.add(i);
                let ip = Ipv6Addr::from(r.ucLocalAddr);
                let port = u16::from_be(r.dwLocalPort as u16);
                Some(PortEntry { proto: "TCP6".into(), local_addr: ip.to_string(), port, pid: r.dwOwningPid })
            })
            .collect()
    })
}

// ── IPv4 UDP ──────────────────────────────────────────────

/// 查询所有 IPv4 UDP 监听端口及其对应的 PID
fn query_udp4() -> Vec<PortEntry> {
    alloc_and_query(|size, buf| unsafe {
        GetExtendedUdpTable(buf, size, false, AF_INET.0 as u32, UDP_TABLE_OWNER_PID, 0)
    })
    .map_or(Vec::new(), |buf| {
        let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let rows = unsafe { buf.as_ptr().add(4) as *const MIB_UDPROW_OWNER_PID };
        (0..num_entries)
            .filter_map(|i| unsafe {
                let r = &*rows.add(i);
                let ip = u32::from_be(r.dwLocalAddr);
                let port = u16::from_be(r.dwLocalPort as u16);
                Some(PortEntry { proto: "UDP".into(), local_addr: Ipv4Addr::from(ip).to_string(), port, pid: r.dwOwningPid })
            })
            .collect()
    })
}

// ── IPv6 UDP ──────────────────────────────────────────────

/// 查询所有 IPv6 UDP 监听端口及其对应的 PID
fn query_udp6() -> Vec<PortEntry> {
    alloc_and_query(|size, buf| unsafe {
        GetExtendedUdpTable(buf, size, false, AF_INET6, UDP_TABLE_OWNER_PID, 0)
    })
    .map_or(Vec::new(), |buf| {
        let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let rows = unsafe { buf.as_ptr().add(4) as *const MIB_UDP6ROW_OWNER_PID };
        (0..num_entries)
            .filter_map(|i| unsafe {
                let r = &*rows.add(i);
                let ip = Ipv6Addr::from(r.ucLocalAddr);
                let port = u16::from_be(r.dwLocalPort as u16);
                Some(PortEntry { proto: "UDP6".into(), local_addr: ip.to_string(), port, pid: r.dwOwningPid })
            })
            .collect()
    })
}
