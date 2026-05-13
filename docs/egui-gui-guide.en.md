# egui GUI Guide

English | [中文](./egui-gui-guide.zh.md)

Back: [README](../README.md)

## Scope

The native GUI lives in [crates/winportkill-gui](../crates/winportkill-gui).

- toolkit: `eframe` + `egui`
- backend: direct `winportkill-core` calls
- purpose: desktop GUI frontend without HTTP sidecar

## Run

```powershell
cargo run -p winportkill-gui
```

Release build:

```powershell
cargo build -p winportkill-gui --release
target\release\winportkill-gui.exe
```

## Current Behavior Checklist

- window title is `WinPortKill`
- ports and processes view are both available
- filter updates the visible rows
- row selection supports mouse and keyboard navigation
- kill requires confirmation
- refresh runs automatically every 10 seconds
- manual refresh is available

## Notes

- this GUI does not use the HTTP sidecar
- kill still targets the whole process
- protected targets may require elevation
