# `src/port.rs` 代码解读与常见问题

> 本文件记录了 `port.rs` 的设计思路、关键代码解释及常见疑问的解答。

---

## 一、文件概述

`port.rs` 是一个 Windows 平台专用的端口查询模块，通过调用 Windows IP Helper API（`GetExtendedTcpTable` / `GetExtendedUdpTable`）获取当前系统中所有 TCP/UDP 监听端口的占用信息，包括协议类型、本地地址、端口号和所属进程 PID。

**仅支持 Windows**，依赖 `windows` crate（微软官方的 Rust Win32 FFI 绑定）。

---

## 二、整体架构

```
get_port_entries()              // 公开入口，返回 Vec<PortEntry>
    ├── query_tcp4()            // IPv4 TCP 监听端口
    ├── query_tcp6()            // IPv6 TCP 监听端口
    ├── query_udp4()            // IPv4 UDP 监听端口
    └── query_udp6()            // IPv6 UDP 监听端口
         └── alloc_and_query()  // 通用两阶段查询辅助函数（内部复用）
```

四个 query 函数结构完全相同，区别只在于：
- 调用的 API 函数不同（TCP vs UDP）
- 地址族参数不同（IPv4 = 2, IPv6 = 23）
- 表类型常量不同（TCP_TABLE vs UDP_TABLE）
- 解析的结构体类型不同（MIB_TCPxxx vs MIB_UDPxxx）

---

## 三、核心数据结构

### PortEntry

```rust
pub struct PortEntry {
    pub proto: String,       // 协议标识："TCP" / "TCP6" / "UDP" / "UDP6"
    pub local_addr: String,  // 本地监听地址（如 "0.0.0.0" 或 "::"）
    pub port: u16,           // 监听端口号
    pub pid: u32,            // 占用该端口的进程 PID
}
```

---

## 四、关键函数详解

### 4.1 `alloc_and_query` — 两阶段查询模式

Windows 的 `GetExtendedTcpTable` / `GetExtendedUdpTable` 采用**两阶段调用**设计：

```
第 1 次：query_fn(&mut size, None)
          → 不传缓冲区，API 返回所需大小到 size 变量
          → 返回值通常为 122 (ERROR_INSUFFICIENT_BUFFER)，这不是错误

第 2 次：query_fn(&mut size, Some(指针))
          → 传入分配好的内存，API 往里面写入完整的表数据
          → 返回值 0 表示成功
```

**为什么需要两次调用？** 因为你事先不知道表有多大——系统可能开着几十个端口也可能只有几个。Windows API 设计成"你问我大小，我告诉你，你再分配"，避免 C 语言里常见的缓冲区大小不匹配问题。

### 4.2 闭包语法 `|size, buf| { ... }`

这是 Rust 的 **闭包（closure）**，等价于其他语言的 lambda 表达式：

| 语言 | 写法 |
|------|------|
| Rust | `\|size, buf\| { ... }` |
| JavaScript | `(size, buf) => { ... }` |
| Python | `lambda size, buf: ...` |
| Java | `(size, buf) -> { ... }` |

`alloc_and_query` 是泛型函数，接受闭包参数 `query_fn`，把"怎么查表"的决定权交给调用方：

- **`size`** → `*mut u32`（输出参数，API 会写入所需缓冲区大小）
- **`buf`** → `Option<*mut c_void>`（数据缓冲区指针，第一次调用时为 `None`）

四个 query 函数用同一个 `alloc_and_query`，只是传入不同的闭包（调不同的 API），避免了重复写两阶段查询的样板代码。这就是**策略模式**在 Rust 中的典型用法。

### 4.3 `Some(buf.as_mut_ptr() as *mut _)` 参数含义

逐层拆解：

```rust
Some( buf.as_mut_ptr()       as *mut _ )
 │        │                  │
 │        │                  └─ 强转为 "某种可变裸指针"（_ 让编译器自动推断目标类型）
 │        └─ 取 Vec<u8> 的可变裸指针（*mut u8），指向堆内存起始位置
 └─ 用 Option 包裹，因为闭包参数类型是 Option<*mut c_void>
```

Windows API 拿到这个指针后，直接往这块内存里**写入表数据**。

---

## 五、关于 `unsafe` 的说明

### 为什么必须用 unsafe？

代码中的 `unsafe` 是**无法避免的**，原因有三：

#### 1. 调用 Windows C API（FFI 边界）

`GetExtendedTcpTable` / `GetExtendedUdpTable` 是 Windows 的 C 语言 API，通过 `windows` crate（FFI 绑定）调用。Rust 编译器无法验证这些外部函数的安全性保证，所以调用它们必须标记 `unsafe`。

```rust
// 直接调用 Windows C 函数
GetExtendedTcpTable(buf, size, false, AF_INET.0 as u32, TCP_TABLE_OWNER_PID_LISTENER, 0)
```

#### 2. 原始指针解引用与类型转换

Windows API 返回的是一块原始内存缓冲区（`Vec<u8>`），将其解析为 C 结构体数组需要手动偏移和类型转换：

```rust
// 将 Vec<u8> 的指针强转为 C 结构体数组指针
let rows = unsafe { buf.as_ptr().add(4) as *const MIB_TCPROW_OWNER_PID };
// 逐个解引用遍历
let r = &*rows.add(i);
```

#### 3. 内存布局假设

代码假设了 Windows 返回的缓冲区内存布局：
- 前 4 字节是 `u32` 条目数
- 之后紧跟着连续的 C 结构体数组

这种对 C 结构体内存布局的依赖，Rust 无法在编译期验证。

### unsafe 使用汇总

| 用途 | 位置 | 能否消除 |
|------|------|----------|
| FFI 调用 | `alloc_and_query` 闭包内 | **不能**，操作系统 C API |
| 指针类型强转 | `as *const MIB_xxx` | **不能**，C 结构体内存布局 |
| 指针解引用 | `&*rows.add(i)` | **不能**，同上 |
| 读条目数 | `u32::from_ne_bytes(...)` | 已消除，改用 safe 的 Vec 索引 |

### 剩余的 unsafe 能否完全消除？

**不能。** 只要通过 FFI 调用 Windows C API 并解析它返回的二进制结构体，就必须用 `unsafe`。这是 Rust 与 C 交互的边界成本。

如果想完全避免 unsafe，唯一的路是使用已经封装好的 safe Rust crate（如 `sysinfo`）来替代直接调用 Windows API，但那等于换了一套实现。

---

## 六、内存管理：从裸指针到 Vec<u8> 的演进

### 旧版本的问题

最初版本 `alloc_and_query` 返回裸指针 `(*const u8, usize)`，并使用 `std::mem::forget(buf)` 阻止内存释放：

```rust
// 旧版：泄漏所有权
let data_start = unsafe { buf.as_ptr().add(4) };
std::mem::forget(buf);  // 防止 buf 被 drop 导致 data_start 悬空
Some((data_start, num_entries))  // 返回裸指针，调用方负责"别用太久"
```

**问题：**
- `mem::forget` 导致几 KB 内存泄漏（直到进程退出才被 OS 回收）
- 返回裸指针没有所有权语义，调用方必须小心生命周期
- 不符合 Rust 的 RAII 惯例

### 新版本的改进

当前版本改为返回 `Option<Vec<u8>>`，让所有权清晰传递：

```rust
// 新版：正常 RAII
Some(buf)  // Vec<u8> 的所有权交给调用方，用完自动释放
```

| 对比项 | 旧版（裸指针 + forget） | 新版（Vec<u8>） |
|--------|------------------------|-----------------|
| 返回类型 | `Option<(*const u8, usize)>` | `Option<Vec<u8>>` |
| 内存所有权 | 泄漏，无人管理 | 由调用方持有，RAII 自动管理 |
| 内存泄漏 | 有（几 KB，进程退出后回收） | 无 |
| 安全性 | 裸指针可能悬空 | Vec 保证内存有效 |

---

## 七、`GetExtendedTcpTable` / `GetExtendedUdpTable` 内部原理

### 函数签名

```c
ULONG GetExtendedTcpTable(
    PVOID           pTcpTable,    // 输出缓冲区（存放结果）
    PDWORD          pdwSize,      // 输入/输出：缓冲区大小
    BOOL            bOrder,       // 是否按端口排序
    ULONG           ulAf,         // 地址族：AF_INET(2) 或 AF_INET6(23)
    TCP_TABLE_CLASS TableClass,   // 表类型（要查什么）
    ULONG           Reserved      // 保留，必须为 0
);
```

两个函数签名完全一致，只是查的表不同。

### 内部执行流程

```
用户态调用 GetExtendedTcpTable(...)
        │
        ▼
┌───────────────────────────────────────────────┐
│  第 1 阶段：pTcpTable == NULL（传 None）        │
│                                               │
│  1. 检查参数合法性                              │
│  2. 向内核查询当前 TCP 监听表的实际大小            │
│     → 内核遍历协议栈中的 TCP endpoint 列表       │
│     → 计算总字节数 = 4(头部) + N × sizeof(行)    │
│  3. 把大小写入 *pdwSize                        │
│  4. 返回 ERROR_INSUFFICIENT_BUFFER (122)        │
│     （这不是错误！是"你去分配内存吧"的信号）        │
└───────────────────────┬───────────────────────┘
                        │ 分配好内存后再次调用
                        ▼
┌───────────────────────────────────────────────┐
│  第 2 阶段：pTcpTable != NULL（传入指针）        │
│                                               │
│  1. 再次从内核获取最新的 TCP endpoint 数据       │
│     （两次调用之间可能有变化，内核会重新读取）     │
│  2. 对每个 TCP socket 填充一行结构体：            │
│     ┌──────────┬──────────┬────────┬──────┐     │
│     │ 本地地址  │ 本地端口  │ 状态   │ PID  │     │
│     └──────────┴──────────┴────────┴──────┘     │
│  3. 如果 bOrder==true，按端口号排序              │
│  4. 在缓冲区头部写入 num_entries (u32)          │
│  5. 返回 NO_ERROR (0)                           │
└───────────────────────────────────────────────┘
```

### 数据来源

```
┌────────────────────────────────────────────────────┐
│                   Windows 内核                      │
│                                                    │
│  应用程序 (Rust 程序)                               │
│       │                                           │
│       ▼  syscall                                   │
│  ┌──────────────────────────────────────────┐     │
│  │  tcpip.sys (TCP/IP 协议栈驱动)            │     │
│  │                                          │     │
│  │  内部维护 TCB 表，每个监听 socket 一条：    │     │
│  │                                          │     │
│  │  [0] 0.0.0.0:80      PID=4 (System)      │     │
│  │  [1] 127.0.0.1:5353  PID=1234 (mDNS)      │     │
│  │  [2] 0.0.0.0:443     PID=4 (System)      │     │
│  │  ...                                      │     │
│  │                                          │     │
│  │  API 把这张表拷贝到用户态缓冲区             │     │
│  └──────────────────────────────────────────┘     │
└────────────────────────────────────────────────────┘
```

### 四种调用组合

代码中一共调用 4 次，排列组合如下：

```
              IPv4 (AF_INET=2)          IPv6 (AF_INET6=23)
┌──────────────────────────┬──────────────────────────┐
│ GetExtendedTcpTable      │ GetExtendedTcpTable      │
│ + TCP_TABLE_OWNER_PID_   │ + TCP_TABLE_OWNER_PID_   │
│   LISTENER               │   LISTENER               │
│ → MIB_TCPROW_OWNER_PID   │ → MIB_TCP6ROW_OWNER_PID  │
├──────────────────────────┼──────────────────────────┤
│ GetExtendedUdpTable      │ GetExtendedUdpTable      │
│ + UDP_TABLE_OWNER_PID    │ + UDP_TABLE_OWNER_PID    │
│ → MIB_UDPROW_OWNER_PID   │ → MIB_UDP6ROW_OWNER_PID  │
└──────────────────────────┴──────────────────────────┘
```

### 关键参数：TableClass

```rust
// TCP 用这两个：
TCP_TABLE_OWNER_PID_LISTENER   // 只查 LISTEN 状态（本程序只用这个）
// 还有：TCP_TABLE_BASIC_LISTENER（不带 PID）、TCP_TABLE_OWNER_PID_ALL（所有状态）

// UDP 用这个：
UDP_TABLE_OWNER_PID            // UDP 没有状态概念，不需要区分 LISTENER/ALL
```

### 返回值含义（COM 风格 HRESULT）

| 返回值 | 含义 | 处理方式 |
|--------|------|----------|
| `0` (NO_ERROR) | 成功 | 继续使用数据 |
| `122` (ERROR_INSUFFICIENT_BUFFER) | 缓冲区太小 | **不是错！** 是让调用方去分配内存 |
| 其他非零 | 真正的错误 | 返回 `None` |

代码中的判断逻辑：
```rust
if query_fn(&mut size, None) != 0 && size == 0 {
    return None;
}
// 第一次调用返回 122 且 size > 0 → 正常流程，继续分配
// 只有返回非零且 size == 0 时才是真正的失败
```

---

## 八、网络字节序转换

Windows API 返回的地址和端口字段都是**网络字节序（大端，Big-Endian）**，而 x86/x64 是小端架构，需要转换：

```rust
// IPv4 地址：u32 大端 → 主机字节序
let ip = u32::from_be(r.dwLocalAddr);

// 端口号：u16 大端 → 主机字节序
let port = u16::from_be(r.dwLocalPort as u16);

// IPv6 地址：ucLocalAddr 是 [u8; 16] 字节数组，已经是标准格式，无需转换
let ip = Ipv6Addr::from(r.ucLocalAddr);
```

---

## 九、文件依赖

| 依赖 | 用途 |
|------|------|
| `std::net::{Ipv4Addr, Ipv6Addr}` | IP 地址类型转换与格式化输出 |
| `windows::Win32::NetworkManagement::IpHelper` | Windows IP Helper API 绑定 |
| `windows::Win32::Networking::WinSock::AF_INET` | IPv4 地址族常量 |
