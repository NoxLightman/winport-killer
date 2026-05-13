# Server 模式指南

[English](./server-mode-guide.en.md) | 中文

返回：[README](../README.zh.md)

## 构建与运行

```powershell
cargo build -p winportkill
cargo run -p winportkill -- --serve 3000
```

也可以直接运行已构建的二进制：

```powershell
target\debug\winportkill.exe --serve 3000
```

## 当前接口

- `GET /health`
- `GET /version`
- `GET /ports?filter=...`
- `GET /processes?filter=...`
- `GET /stats/ports?filter=...`
- `GET /stats/processes?filter=...`
- `POST /kill/{pid}`
- `GET /ws`

目前仍保留的兼容接口：

- `GET /ports/filter/{keyword}`
- `GET /stats`

## 示例

### 健康检查

```powershell
curl http://127.0.0.1:3000/health
```

### 端口视图

```powershell
curl http://127.0.0.1:3000/ports
curl "http://127.0.0.1:3000/ports?filter=8080"
```

### 进程视图

```powershell
curl http://127.0.0.1:3000/processes
curl "http://127.0.0.1:3000/processes?filter=node"
```

### 结束进程

```powershell
curl -X POST http://127.0.0.1:3000/kill/1234
```

kill 是针对整个进程，不是只释放某个端口。受保护目标可能需要管理员权限。

### WebSocket

```powershell
websocat ws://127.0.0.1:3000/ws
```

当前 WebSocket 每 10 秒推送一次端口视图风格的 payload。

## JSON 模式

如果你只需要一次性快照：

```powershell
cargo run -p winportkill -- --json
```

## 说明

- `main.rs` 打印的启动横幅略微过时，没有列出全部当前路由。
- 新的 IDE client 应优先使用 `/ports` 和 `/processes` 加查询参数过滤，而不是 legacy 接口。
