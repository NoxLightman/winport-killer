# Ratatui Guide

English | [中文](./ratatui-guide.zh.md)

Back: [README](../README.md)

## Scope

This guide focuses on the root TUI under [`src`](../src).

- [`main.rs`](../src/main.rs): mode selection and terminal lifecycle
- [`app.rs`](../src/app.rs): state, refresh, filtering, kill behavior
- [`ui.rs`](../src/ui.rs): rendering

## Current TUI Flow

1. `main.rs` enters raw mode and alternate screen.
2. A `Terminal<CrosstermBackend>` is created.
3. `App::new()` loads the first snapshot.
4. The main loop draws one frame at a time.
5. Key events are forwarded to `App::handle_key()`.
6. Periodic refresh runs every 10 seconds.

## State Model

`App` stores:

- active view mode
- full and filtered port rows
- full and filtered process rows
- filter text
- selected row index
- stats for the current data sets
- status message and quit flag

## Rendering Layout

[`ui.rs`](../src/ui.rs) renders four vertical zones:

- top bar
- filter bar
- main table
- status line

It uses:

- `Layout` to split the screen
- `Paragraph` for labels and status
- `Table` for both ports and processes views
- `TableState` and `ScrollbarState` for selection and scroll position

## Key Bindings

- `q`: quit
- `k`: kill selected PID
- `/`: enter filter mode
- `r`: refresh
- `Tab`: switch view
- `Up` / `Down`: move selection
- `PageUp` / `PageDown`: jump
- `Home` / `End`: go to first or last row

## Why This Matters

The TUI is the simplest end-to-end client in the repo.

It is useful for:

- understanding the direct-core frontend path
- checking data shape changes before updating IDE clients
- validating interaction behavior without sidecar lifecycle complexity
