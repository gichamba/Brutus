# Desktop build guide

This guide owns setup, build commands and developer checks for the Tauri app in this directory. For the recovery workflow and history behavior, see the [main README](../README.md). The desktop uses Tauri 2, a Rust backend, plain HTML/CSS/JavaScript and bundled SQLite.

## Prerequisites

Install Node.js with npm, Rust 1.90 or newer, and the platform prerequisites in [Tauri's documentation](https://v2.tauri.app/start/prerequisites/).

On Windows, install Visual Studio C++ build tools, the Windows SDK and WebView2. Use a Visual Studio developer shell with a complete C++ toolchain. A compiler executable without its headers and libraries is not enough. The recorded build used the existing VS2019 developer environment because the newer local installation was missing `excpt.h`.

For Linux and macOS, follow the platform sections of Tauri's prerequisite guide. Windows x64 is the platform built in this workspace. Linux and macOS builds have not been verified here.

## Develop

From the repository root:

```powershell
cd desktop
npm ci
npm run dev
```

The frontend files are embedded directly by Tauri. The app has no separate frontend development server.

## Package

From `desktop/`, build a Windows installer:

```powershell
npm run build -- --bundles nsis
```

The installer is written to `src-tauri/target/release/bundle/nsis/`. The Windows executable is written to `src-tauri/target/release/brutus-desktop.exe`.

On Linux or macOS, run `npm run build` from this directory to build the host's supported packages.

## Project layout

| Path | Purpose |
|---|---|
| `ui/` | Static recovery and history views, display logic and UI logic tests |
| `src-tauri/src/engine.rs` | Numeric ranges, job allocation and cancellable trials |
| `src-tauri/src/service.rs` | File discovery, bounded worker pool and checkpoints |
| `src-tauri/src/pdf.rs` | PDF user-password authentication through lopdf |
| `src-tauri/src/model.rs` | Settings and saved run/file records |
| `src-tauri/src/store.rs` | SQLite history and successful content-hash cache |
| `src-tauri/src/lib.rs` | Native commands and app-data setup |
| `src-tauri/tauri.conf.json` | Window, embedded frontend and packaging configuration |

## Developer checks

From `desktop/`:

```powershell
npm run check
npm test
cd src-tauri
cargo test --lib --locked
cargo build --release --locked
```

The unit tests do not start the app, a browser or a database. Native dialogs, rendered layouts, actual database persistence and restart/resume require a separate user-invoked integration review. Recorded results are in [BRU-001](../docs/issues/BRU-001-desktop.md).
