# Brutus desktop

A local Tauri app for recovering six-digit numeric passwords from PDFs. The Rust backend shares 1,000-candidate ranges across workers and files. The original C# console projects remain available.

## Build

Install Node.js, Rust 1.90 or newer, and the platform prerequisites listed in [Tauri's documentation](https://v2.tauri.app/start/prerequisites/). On Windows, use a Visual Studio developer shell with working C++ tools, the Windows SDK and WebView2.

From this directory:

```powershell
npm ci
npm run dev
```

Build a Windows installer:

```powershell
npm run build -- --bundles nsis
```

The installer is written to `src-tauri/target/release/bundle/nsis/`. A compiled executable is written to `src-tauri/target/release/brutus-desktop.exe`.

On Linux or macOS, use `npm run build` for the host's supported packages. These platform builds have not been verified in this workspace.

## Use

Select PDF files or a folder. Folders include subfolders; directory symlinks are skipped. Choose the worker count and inclusive numeric range, then start recovery. More workers consume more CPU and may not improve throughput for every encryption type.

Cancel to stop workers and keep completed ranges. Open History to inspect past runs or resume cancelled and interrupted work. A partial range may repeat. Resume rescans the selected inputs and resets a file's checkpoints when its content changes.

Successful results are reused by SHA256 content hash. Outcomes distinguish a recovered password, a PDF needing no password, an exhausted range, and a file error. Recovery authenticates the PDF's user password. Files larger than 100 MB produce an error. Parsed PDFs stay in memory during the run, so large folders can need substantial memory.

History is stored in `brutus-desktop.db` under Tauri's user app-data directory for `app.brutus.desktop`. On Windows this is normally `%APPDATA%\app.brutus.desktop`. The desktop app does not import or modify the console app's `brutus.db`. Recovered passwords are saved locally as plaintext.

## Development checks

```powershell
npm run check
npm test
cd src-tauri
cargo test --lib --locked
cargo build --locked
```

The unit tests do not start the app, a browser or a database. Native dialogs, rendered layouts, actual database persistence and restart/resume require a separate user-invoked integration review.
