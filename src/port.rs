use std::net::{Ipv4Addr, Ipv6Addr};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCPROW_OWNER_PID, MIB_TCP6ROW_OWNER_PID,
    MIB_UDPROW_OWNER_PID, MIB_UDP6ROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER,
    UDP_TABLE_OWNER_PID,
};
use windows::Win32::Networking::WinSock::AF_INET;

// AF_INET6 未在 windows crate 的 WinSock 模块中导出，手动定义
const AF_INET6: u32 = 23;

/// 一条端口监听记录
#[derive(Clone, Debug)]
pub struct PortEntry {
    pub proto: String,      // 协议：TCP / TCP6 / UDP / UDP6
    pub local_addr: String, // 本地监听地址（如 0.0.0.0、127.0.0.1）
    pub port: u16,          // 监听端口号
    pub pid: u32,           // 占用该端口的进程 PID
}

/// 获取系统中所有监听端口，覆盖 IPv4/IPv6 的 TCP/UDP 四种组合
pub fn get_port_entries() -> Vec<PortEntry> {
    let mut entries = Vec::new();
    entries.extend(get_tcp4_entries());
    entries.extend(get_tcp6_entries());
    entries.extend(get_udp4_entries());
    entries.extend(get_udp6_entries());
    entries
}

/// 获取 IPv4 TCP 监听端口
/// 调用 Windows API GetExtendedTcpTable，使用 TCP_TABLE_OWNER_PID_LISTENER
/// 只返回 LISTENING 状态的连接，避免 ESTABLISHED/TIME_WAIT 重复
fn get_tcp4_entries() -> Vec<PortEntry> {
    // 第一次调用：传入空缓冲区，获取所��缓冲区大小
    let mut size: u32 = 0;
    unsafe {
        let _ = GetExtendedTcpTable(
            None, &mut size, false, AF_INET.0 as u32, TCP_TABLE_OWNER_PID_LISTENER, 0,
        );
    }

    // 分配足够大的缓冲区
    let mut buf = vec![0u8; size as usize];

    // 第二次调用：传入缓冲区，获取实际数据
    let result = unsafe {
        GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut _), &mut size, false, AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_LISTENER, 0,
        )
    };
    if result != 0 {
        return Vec::new();
    }

    // 解析返回的表：第一个 u32 是行数，后面紧跟 MIB_TCPROW_OWNER_PID 结构体数组
    let table = buf.as_ptr() as *const u32;
    let num_entries = unsafe { *table };
    let rows = table.wrapping_add(1) as *const MIB_TCPROW_OWNER_PID;

    let mut entries = Vec::new();
    for i in 0..num_entries {
        let row = unsafe { &*rows.add(i as usize) };
        // dwLocalAddr 是网络字节序（big-endian），需转为本机字节序才能正确显示 IP
        let ip = u32::from_be(row.dwLocalAddr);
        // dwLocalPort 同样是网络字节序，低 16 位是端口号
        let port = u16::from_be(row.dwLocalPort as u16);
        entries.push(PortEntry {
            proto: "TCP".to_string(),
            local_addr: Ipv4Addr::from(ip).to_string(),
            port,
            pid: row.dwOwningPid,
        });
    }
    entries
}

/// 获取 IPv6 TCP 监听端口，逻辑与 IPv4 版本相同，使用 AF_INET6 和 MIB_TCP6ROW_OWNER_PID
fn get_tcp6_entries() -> Vec<PortEntry> {
    let mut size: u32 = 0;
    unsafe {
        let _ = GetExtendedTcpTable(
            None, &mut size, false, AF_INET6, TCP_TABLE_OWNER_PID_LISTENER, 0,
        );
    }

    let mut buf = vec![0u8; size as usize];
    let result = unsafe {
        GetExtendedTcpTable(
            Some(buf.as_mut_ptr() as *mut _), &mut size, false, AF_INET6,
            TCP_TABLE_OWNER_PID_LISTENER, 0,
        )
    };
    if result != 0 {
        return Vec::new();
    }

    let table = buf.as_ptr() as *const u32;
    let num_entries = unsafe { *table };
    let rows = table.wrapping_add(1) as *const MIB_TCP6ROW_OWNER_PID;

    let mut entries = Vec::new();
    for i in 0..num_entries {
        let row = unsafe { &*rows.add(i as usize) };
        // IPv6 地址存储在 ucLocalAddr 的 16 字节数组中，可直接构造 Ipv6Addr
        let ip = Ipv6Addr::from(row.ucLocalAddr);
        let port = u16::from_be(row.dwLocalPort as u16);
        entries.push(PortEntry {
            proto: "TCP6".to_string(),
            local_addr: ip.to_string(),
            port,
            pid: row.dwOwningPid,
        });
    }
    entries
}

/// 获取 IPv4 UDP 监听端口
/// UDP 没有 LISTENING 概念，使用 UDP_TABLE_OWNER_PID 获取所有 UDP 绑定
fn get_udp4_entries() -> Vec<PortEntry> {
    let mut size: u32 = 0;
    unsafe {
        let _ = GetExtendedUdpTable(
            None, &mut size, false, AF_INET.0 as u32, UDP_TABLE_OWNER_PID, 0,
        );
    }

    let mut buf = vec![0u8; size as usize];
    let result = unsafe {
        GetExtendedUdpTable(
            Some(buf.as_mut_ptr() as *mut _), &mut size, false, AF_INET.0 as u32,
            UDP_TABLE_OWNER_PID, 0,
        )
    };
    if result != 0 {
        return Vec::new();
    }

    let table = buf.as_ptr() as *const u32;
    let num_entries = unsafe { *table };
    let rows = table.wrapping_add(1) as *const MIB_UDPROW_OWNER_PID;

    let mut entries = Vec::new();
    for i in 0..num_entries {
        let row = unsafe { &*rows.add(i as usize) };
        let ip = u32::from_be(row.dwLocalAddr);
        let port = u16::from_be(row.dwLocalPort as u16);
        entries.push(PortEntry {
            proto: "UDP".to_string(),
            local_addr: Ipv4Addr::from(ip).to_string(),
            port,
            pid: row.dwOwningPid,
        });
    }
    entries
}

/// 获取 IPv6 UDP 监听端口，逻辑与 IPv4 版本相同，使用 AF_INET6 和 MIB_UDP6ROW_OWNER_PID
fn get_udp6_entries() -> Vec<PortEntry> {
    let mut size: u32 = 0;
    unsafe {
        let _ = GetExtendedUdpTable(
            None, &mut size, false, AF_INET6, UDP_TABLE_OWNER_PID, 0,
        );
    }

    let mut buf = vec![0u8; size as usize];
    let result = unsafe {
        GetExtendedUdpTable(
            Some(buf.as_mut_ptr() as *mut _), &mut size, false, AF_INET6,
            UDP_TABLE_OWNER_PID, 0,
        )
    };
    if result != 0 {
        return Vec::new();
    }

    let table = buf.as_ptr() as *const u32;
    let num_entries = unsafe { *table };
    let rows = table.wrapping_add(1) as *const MIB_UDP6ROW_OWNER_PID;

    let mut entries = Vec::new();
    for i in 0..num_entries {
        let row = unsafe { &*rows.add(i as usize) };
        let ip = Ipv6Addr::from(row.ucLocalAddr);
        let port = u16::from_be(row.dwLocalPort as u16);
        entries.push(PortEntry {
            proto: "UDP6".to_string(),
            local_addr: ip.to_string(),
            port,
            pid: row.dwOwningPid,
        });
    }
    entries
}
