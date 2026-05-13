# 端口层实现指南

[English](./port-rs-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 目的

[`crates/winportkill-core/src/port.rs`](../crates/winportkill-core/src/port.rs) 是 Windows 专用的端口查询层。

它负责：

- 收集监听中的 TCP 和 UDP 绑定
- 同时覆盖 IPv4 和 IPv6
- 返回协议、本地地址、端口和所属 PID

## 对外入口

主要公开函数是 `get_port_entries()`。

它会聚合：

- `query_tcp4()`
- `query_tcp6()`
- `query_udp4()`
- `query_udp6()`

## 查询模式

代码使用 Windows 常见的两阶段缓冲区模式：

1. 先调用一次 API，获取所需缓冲区大小
2. 分配 `Vec<u8>`
3. 再调用一次 API，填充缓冲区
4. 把返回字节转换成对应的行结构体

辅助函数 `alloc_and_query()` 统一封装了这个流程。

## 关键 Windows API

- `GetExtendedTcpTable`
- `GetExtendedUdpTable`

地址族：

- `AF_INET` 对应 IPv4
- `23` 对应 IPv6

## 为什么必须有 `unsafe`

这里必须使用 `unsafe`，因为代码需要：

- 穿过 FFI 边界调用 Win32 API
- 把原始字节缓冲区转换为兼容 C 的行结构体
- 解引用这些缓冲区上的裸指针

如果想直接读取 Windows 网络表，这部分 unsafe 成本基本不可避免。

## 与 `process.rs` 的关系

`port.rs` 只关心端口绑定和所属 PID。

[`process.rs`](../crates/winportkill-core/src/process.rs) 会基于这些原始行进一步：

- 关联进程元数据
- 去重
- 构建更高层的端口视图和进程视图
- 计算统计信息
