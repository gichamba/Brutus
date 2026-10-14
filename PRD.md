# Brutus desktop product overview

## Purpose

Brutus helps the user recover forgotten six-digit numeric passwords from their PDFs. The current product is a local Tauri 2 desktop app with a Rust backend.

The governing behavior is defined by requirements D1 through D9 in the [desktop specification](docs/specs/desktop.md). This overview points to that specification rather than maintaining a second copy of its requirements.

## Main workflow

The user selects PDF files or a folder, chooses a numeric range and worker count, and starts recovery. The app shows progress and per-file outcomes. History keeps earlier runs and lets the user resume cancelled or interrupted work.

| Area | Current requirement |
|---|---|
| Inputs | Native file and folder selection; recursive PDF discovery (D1) |
| Recovery | Six-digit, zero-padded numeric candidates; original PDFs unchanged (D2) |
| Parallel work | Bounded Rust workers share distinct ranges across files and within one file (D3) |
| Progress and control | Attempts, elapsed time, per-file outcomes, cancellation and completed-range resume (D4) |
| History | Local SQLite run records, checkpoints and successful results keyed by content hash (D5) |
| Errors | Separate file errors, unprotected PDFs, recovered passwords and range exhaustion (D6) |
| Interface | Dark recovery workspace, searchable history, keyboard controls and responsive layout (D7) |
| Compatibility | Windows, macOS and Linux design; Windows build evidence; console source retained (D8) |
| Stack | Tauri 2, Rust and plain HTML/CSS/JavaScript with the approved dependencies (D9) |

## Delivery and verification

The desktop app lives in `desktop/`. Its worker pool uses 1,000-candidate ranges. The final interval can be shorter. History is separate from the old console database and lives in the user's app-data directory.

Implementation and delivery evidence, including the passing unit tests and Windows installer build, is in [BRU-001](docs/issues/BRU-001-desktop.md). That Issue remains in review. Desktop integration, browser and persistence-through-restart checks have not run.

Build and packaging commands are owned by the [desktop build guide](desktop/README.md). Day-to-day use is described in the [README](README.md).

## Historical records

The earlier .NET app stays on the back burner as historical code. Its documents are preserved under `docs/archive/` and linked from the [project history](HowWeGotHere.md). Active product direction and requirements are about Tauri.
