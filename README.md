# Brutus

Brutus is a Tauri 2 desktop app for recovering forgotten six-digit numeric passwords from PDFs. It has a Rust backend, a dark interface, and local work history.

## Recover PDFs

1. Open Brutus and select PDF files or a folder. Folders include PDFs in subfolders.
2. Choose the worker count and password range. The default range is `000000` through `999999`.
3. Select **Start recovery**. Watch per-file results, passwords tried, elapsed time and progress.
4. Use **Cancel run** to stop processing. Open **History** to inspect past outcomes or resume cancelled and interrupted runs.

Workers share distinct ranges across several files or within one file. A recovered password stops that file's remaining work. More workers use more CPU; speed depends on the PDF's encryption and the machine.

The app keeps the original PDFs unchanged. It tests exactly six-digit numeric passwords and reports when none are found in the selected range. Malformed, unsupported or unreadable PDFs appear as file errors.

## Build the desktop app

Install Node.js with npm, Rust 1.90 or newer, and the platform tools described in the [desktop build guide](desktop/README.md). On Windows, use a Visual Studio developer shell with working C++ tools and the Windows SDK.

From the repository root:

```powershell
cd desktop
npm ci
npm run build -- --bundles nsis
```

The Windows installer is written under `desktop/src-tauri/target/release/bundle/nsis/`. The executable is `desktop/src-tauri/target/release/brutus-desktop.exe`. These are local build outputs; they are not checked into the repository. Run the installer to install Brutus, or open the executable on a machine with WebView2 available.

For development, run `npm run dev` from `desktop/`. Linux and macOS build instructions are in the build guide; those builds have not been verified here.

## History and resume

Runs, selected inputs, settings, timestamps, recovered passwords and file outcomes are saved in a local SQLite database, `brutus-desktop.db`. On Windows its usual location is `%APPDATA%\app.brutus.desktop\brutus-desktop.db`.

Completed 1,000-candidate ranges are checkpointed. A partial range may repeat after cancellation or a crash. Resume rescans inputs and resets a file's checkpoints when its bytes change. Successful results can be reused for identical file content, even at another path.

Recovered passwords are stored locally as plaintext. Files reported above 100 MiB are rejected. Parsed PDFs stay in memory during a run, so large folders can need substantial memory. One recovery run is active per desktop instance; desktop instances do not share a scheduler.

## Documentation

- [Product overview](PRD.md)
- [Desktop requirements](docs/specs/desktop.md)
- [Build guide and developer checks](desktop/README.md)
- [Implementation and verification evidence](docs/issues/BRU-001-desktop.md)
- [Project history](HowWeGotHere.md)

The desktop implementation and Windows build have unit and code-review evidence. Native dialogs, rendered UI, actual history persistence and restart/resume still need user-invoked integration review.
