# Port Layer Guide

English | [中文](./port-rs-guide.zh.md)

Back: [README](../README.md)

## Purpose

[`crates/winportkill-core/src/port.rs`](../crates/winportkill-core/src/port.rs) is the Windows-specific port query layer.

It is responsible for:

- collecting listening TCP and UDP bindings
- covering both IPv4 and IPv6
- returning protocol, local address, port, and owning PID

## Public Entry

The main public function is `get_port_entries()`.

It aggregates:

- `query_tcp4()`
- `query_tcp6()`
- `query_udp4()`
- `query_udp6()`

## Query Pattern

The code uses the standard Windows two-step buffer pattern:

1. call the API once to learn the required buffer size
2. allocate a `Vec<u8>`
3. call the API again to fill the buffer
4. cast the returned bytes to the appropriate row structs

The helper `alloc_and_query()` centralizes that flow.

## Important Windows APIs

- `GetExtendedTcpTable`
- `GetExtendedUdpTable`

Address families:

- `AF_INET` for IPv4
- `23` for IPv6

## Why `unsafe` Exists

`unsafe` is required here because the code:

- crosses the FFI boundary into Win32 APIs
- casts raw byte buffers into C-compatible row structures
- dereferences raw pointers returned from those buffers

The unsafe region is the unavoidable cost of directly reading Windows networking tables.

## Relationship To `process.rs`

`port.rs` only knows about bindings and owning PID values.

[`process.rs`](../crates/winportkill-core/src/process.rs) takes those raw rows and:

- joins them with process metadata
- deduplicates rows
- builds higher-level port and process views
- computes stats
