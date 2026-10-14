# How We Got Here

This document records the Tauri app's direction and development. Current behavior is defined in the [desktop specification](docs/specs/desktop.md); build commands are in the [desktop guide](desktop/README.md).

## Product direction

The user needed to recover six-digit numeric passwords from PDFs, select files or folders through a modern UI, share work across CPU workers, and keep past outcomes. The repository was reused for a Tauri 2 desktop app with a Rust backend.

The user confirmed that the earlier .NET app stays on the back burner as historical code. Its [development notes](docs/archive/console-history.md), [guide](docs/archive/console-guide.md) and [PRD](docs/archive/console-prd.md) are archived. Active product documentation is about Tauri.

## Desktop implementation - 2026-10-14

1. Added the app under `desktop/`, with plain HTML, CSS and JavaScript views connected to Rust commands.
2. Kept the password scope at `000000` through `999999`, with a configurable inclusive search range.
3. Split work into distinct ranges of at most 1,000 candidates. A bounded pool shares those ranges across PDFs and within one PDF. Parsed PDFs are reused during a run.
4. Added a local SQLite database in the app-data directory for runs, settings, timestamps, per-file outcomes, recovered passwords and completed-range checkpoints.
5. Added cancellation, resume after interruption, and successful result reuse by content hash. Changed file content resets its checkpoints.
6. Kept recovery local and left source PDFs unchanged. Dependency approval and adopted decisions are in the [question ledger](docs/notes/tauri-app-questions.md).

## Verification and delivery

Recorded checks passed: 17 Rust unit tests, six UI logic tests, code review and a Windows x64 installer build with zero compiler warnings or errors. A review regression caught stale passwords on malformed replacement files and was fixed before delivery.

The implementation and delivery record were committed and pushed to `main` at `3d0944a`. [BRU-001](docs/issues/BRU-001-desktop.md) holds the evidence and remains in review. Native dialogs, rendered UI, actual persistence and restart/resume still need user-invoked integration review. Linux and macOS builds have not run.

## Documentation direction

The [README](README.md) owns desktop use, the [PRD](PRD.md) is the product overview, the [specification](docs/specs/desktop.md) owns behavior, and the [desktop guide](desktop/README.md) owns build commands. Historical material stays under `docs/archive/`.
