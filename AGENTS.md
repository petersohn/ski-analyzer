# Context

The purpose of this app is to display a recorded GPX track for skiing, and display it on top of the ski area. Ski area data is downloaded from OpenStreetMap.

# Project structure

- `src-tauri/ski-analyzer-lib`: A library written in Rust that contains business logic about processing ski areas and GPX tracks.
- `src-tauri`: A Tauri app connecting the library with the front end.
- `src`: The front end responsible for display written in Angular.

## Responsibilities

- Anything related to downloading ski areas from OpenStreetMap and analyzing GPX tracks should be in `ski-analyzer-lib`.
- Configuration and persisting of data is done in `src-tauri`.
- The front end is only responsible for displaying. It should not directly access the file system.

# Building and testing

**Important:** After every change, build the entire project and run tests.

## Library

Issue these commands in `src-tauri/ski-analyzer-lib`.

- **build:** `cargo build`
- **test:** `cargo test`

## Tauri app

Run this from the project root.

- **build the entire app:** `pnpm tauri build`
- **build the front end:** `pnpm build`
