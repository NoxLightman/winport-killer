# Server Mode Guide

English | [中文](./server-mode-guide.zh.md)

Back: [README](../README.md)

## Build And Run

```powershell
cargo build -p winportkill
cargo run -p winportkill -- --serve 3000
```

You can also run the built binary directly:

```powershell
target\debug\winportkill.exe --serve 3000
```

## Current Endpoints

- `GET /health`
- `GET /version`
- `GET /ports?filter=...`
- `GET /processes?filter=...`
- `GET /stats/ports?filter=...`
- `GET /stats/processes?filter=...`
- `POST /kill/{pid}`
- `GET /ws`

Legacy compatibility routes still exist:

- `GET /ports/filter/{keyword}`
- `GET /stats`

## Examples

### Health

```powershell
curl http://127.0.0.1:3000/health
```

### Ports view

```powershell
curl http://127.0.0.1:3000/ports
curl "http://127.0.0.1:3000/ports?filter=8080"
```

### Processes view

```powershell
curl http://127.0.0.1:3000/processes
curl "http://127.0.0.1:3000/processes?filter=node"
```

### Kill a process

```powershell
curl -X POST http://127.0.0.1:3000/kill/1234
```

Kill is process-wide. Protected targets may require elevation.

### WebSocket

```powershell
websocat ws://127.0.0.1:3000/ws
```

The current WebSocket stream publishes a ports-style payload every 10 seconds.

## JSON Mode

If you only need a one-shot snapshot:

```powershell
cargo run -p winportkill -- --json
```

## Notes

- The startup banner printed by `main.rs` is slightly stale and does not list all current routes.
- New IDE clients should prefer `/ports` and `/processes` with query filters over legacy routes.
