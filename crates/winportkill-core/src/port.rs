use serde::Serialize;
use std::net::{Ipv4Addr, Ipv6Addr};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6ROW_OWNER_PID, MIB_TCPROW_OWNER_PID,
    MIB_UDP6ROW_OWNER_PID, MIB_UDPROW_OWNER_PID, TCP_TABLE_OWNER_PID_LISTENER, UDP_TABLE_OWNER_PID,
};
use windows::Win32::Networking::WinSock::AF_INET;

const AF_INET6: u32 = 23;

/// 单条端口占用记录
#[derive(Clone, Debug, Serialize)]
pub struct PortEntry {
    pub proto: String,
    pub local_addr: String,
    pub port: u16,
    pub pid: u32,
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

fn alloc_and_query<F>(query_fn: F) -> Option<Vec<u8>>
where
    F: Fn(*mut u32, Option<*mut std::ffi::c_void>) -> u32,
{
    let mut size: u32 = 0;
    if query_fn(&mut size, None) != 0 && size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    if query_fn(&mut size, Some(buf.as_mut_ptr() as *mut _)) != 0 {
        return None;
    }
    Some(buf)
}

fn query_tcp4() -> Vec<PortEntry> {
    alloc_and_query(|size, buf| unsafe {
        GetExtendedTcpTable(
            buf,
            size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_PID_LISTENER,
            0,
        )
    })
    .map_or(Vec::new(), |buf| {
        let num_entries = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let rows = unsafe { buf.as_ptr().add(4) as *const MIB_TCPROW_OWNER_PID };
        (0..num_entries)
            .filter_map(|i| unsafe {
                let r = &*rows.add(i);
                let ip = u32::from_be(r.dwLocalAddr);
                let port = u16::from_be(r.dwLocalPort as u16);
                Some(PortEntry {
                    proto: "TCP".into(),
                    local_addr: Ipv4Addr::from(ip).to_string(),
                    port,
                    pid: r.dwOwningPid,
                })
            })
            .collect()
    })
}

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
                Some(PortEntry {
                    proto: "TCP6".into(),
                    local_addr: ip.to_string(),
                    port,
                    pid: r.dwOwningPid,
                })
            })
            .collect()
    })
}

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
                Some(PortEntry {
                    proto: "UDP".into(),
                    local_addr: Ipv4Addr::from(ip).to_string(),
                    port,
                    pid: r.dwOwningPid,
                })
            })
            .collect()
    })
}

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
                Some(PortEntry {
                    proto: "UDP6".into(),
                    local_addr: ip.to_string(),
                    port,
                    pid: r.dwOwningPid,
                })
            })
            .collect()
    })
}
