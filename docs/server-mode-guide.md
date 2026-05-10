# WinPortKill Server 模式测试指南

## 前置：编译

```bash
cargo build --release
```

产物在 `target/release/winportkill.exe`。

---

## 1. 启动 Server

```bash
winportkill.exe --serve 3000
```

输出：
```
WinPortKill server running at http://127.0.0.1:3000
API: /ports  /stats  /kill/:pid  /ports/filter/:keyword  /ws
```

---

## 2. 测试 REST API

### 获取全部端口列表

```bash
curl http://127.0.0.1:3000/ports
```

返回 JSON 数组，每项包含 `proto`, `local_addr`, `port`, `pid`, `name`, `memory`。

### 获取统计信息

```bash
curl http://127.0.0.1:3000/stats
```

返回：
```json
{
  "total_procs": 142,
  "tcp_count": 18,
  "udp_count": 7,
  "total_mem_bytes": 1234567890
}
```

### 按关键字过滤

```bash
# 按端口过滤
curl http://127.0.0.1:3000/ports/filter/8080

# 按进程名过滤
curl http://127.0.0.1:3000/ports/filter/node

# 按协议过滤
curl http://127.0.0.1:3000/ports/filter/tcp
```

### Kill 进程

```bash
curl -X POST http://127.0.0.1:3000/kill/1234
```

成功返回：
```json
{ "success": true, "message": "Killed PID 1234 (node.exe)" }
```

失败返回：
```json
{ "success": false, "message": "Failed to kill PID 1234 (need admin?)" }
```

> 注意：kill 受保护进程需要以管理员身份启动 server。

---

## 3. 测试 WebSocket

使用 [websocat](https://github.com/nickelc/websocat)：

```bash
websocat ws://127.0.0.1:3000/ws
```

每 10 秒会收到一次 JSON 格式的端口列表推送。

或者用浏览器控制台测试：

```javascript
const ws = new WebSocket("ws://127.0.0.1:3000/ws");
ws.onmessage = (e) => console.log(JSON.parse(e.data).length, "entries");
```

---

## 4. 测试 --json 模式

一次性输出 JSON 后退出，适合脚本集成：

```bash
winportkill.exe --json
```

配合 jq 使用：

```bash
# 只看 TCP 端口
winportkill.exe --json | jq '.[] | select(.proto == "TCP")'

# 按内存排序取前 10
winportkill.exe --json | jq 'sort_by(-.memory) | .[0:10]'
```

---

## 5. 验证 TUI 模式不受影响

```bash
winportkill.exe
```

确认 TUI 界面正常显示、过滤、kill 功能正常。

---

## 常见问题

| 问题 | 解决 |
|------|------|
| 端口被占用 | 换一个端口，如 `--serve 3001` |
| kill 返回 need admin | 以管理员身份运行 |
| curl 不是内部命令 | 用 PowerShell 的 `Invoke-WebRequest` 或安装 curl |
| WebSocket 连接失败 | 确认 server 正在运行，检查防火墙 |