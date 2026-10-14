# Brutus desktop

## Purpose

Recover forgotten six-digit numeric passwords from the user's PDFs through a local desktop app. Let the user select inputs, watch recovery, and return to past outcomes.

This is the governing behavior specification for the Tauri product. The [PRD](../../PRD.md) is the product overview, the [README](../../README.md) describes use, and the [desktop guide](../../desktop/README.md) owns build commands.

## Requirements

- D1: The user can select PDF files or a folder through native dialogs. Folder discovery includes subfolders. Invalid inputs and unreadable paths produce visible errors.
- D2: The Rust backend tests zero-padded numeric passwords in an inclusive range within 000000 to 999999. The default is the whole range. Original PDFs remain unchanged.
- D3: A bounded, configurable worker pool shares distinct ranges across multiple PDFs and within one PDF. A recovered password stops remaining work for that PDF. One active run is allowed per desktop instance.
- D4: The UI shows per-file outcomes, progress, attempts, elapsed time and the worker limit. Cancellation stops scheduling and preserves completed ranges. Interrupted or cancelled work can resume without repeating completed ranges.
- D5: Local SQLite stores run inputs, settings, timestamps, per-file outcomes, recovered passwords, hashes and completed ranges across restarts. Startup marks unfinished runs interrupted. Successful results may be reused for identical content. A changed source resets that file's checkpoint.
- D6: A malformed PDF, unsupported encryption or unreadable file is an error, not a recovered password or an exhausted range. Other files continue after an individual error. Exhaustion is reported as not found in the selected range.
- D7: The UI has a restrained dark palette, clear file selection, a recovery view and searchable history with run details. Controls have labels, visible keyboard focus, accessible status messages and layouts for narrow and wide windows. Motion respects reduced-motion preferences.
- D8: The earlier C# console source and existing user edits stay intact as historical material. The active product is Tauri and uses its own app-data database. It is designed for Windows, macOS and Linux; only a Windows build is verified here.
- D9: Approved dependencies are Tauri 2, its dialog plugin, lopdf, rusqlite with bundled SQLite, serde, serde_json, sha2, tauri-build and the Tauri CLI. Use plain frontend assets and standard Rust threads.

## Scenarios

- Select a folder with two PDFs and a text file: discover the two PDFs, including PDFs in subfolders, and ignore the text file.
- Search 000000 to 002499: the scheduler creates 000000..000999, 001000..001999 and 002000..002499. Each candidate belongs to exactly one range.
- Recover password 001234: record that exact six-digit value and stop the file's remaining ranges. Other PDFs continue.
- Cancel after a range completes: save the completed range. Resume schedules only unfinished ranges. Partial ranges may be retried.
- Open a past run after restarting: show its saved paths, settings, status and outcomes. An unfinished run is interrupted and can resume.
- Select a renamed copy of a successfully recovered PDF: reuse the saved password by content hash and record the reused result.
- Encounter a malformed PDF or an inaccessible subfolder: show the error while retaining other discovered inputs and their outcomes.
- Resume after the source changes: discard that file's old range checkpoints and authenticate the new content.
- Enter a start greater than the end, or a worker count of zero: reject the settings before starting.

## Constraints

All recovery and history remain local. History contains recovered passwords as local plaintext. No speedup is guaranteed: PDF encryption and available CPU resources affect throughput. Checkpoint only completed ranges; a partial range can repeat after cancellation or a crash. Error messages must not claim success when history cannot be saved.

The worker setting accepts 1 through 32 workers. Work is split into ranges of at most 1,000 candidates. Sources reported above 100 MiB are rejected, and parsed PDFs remain in memory during a run. The desktop database is `brutus-desktop.db` in the user's app-data directory; it is separate from the legacy console database.

## External assumptions

The user supplies readable PDFs they own and a desktop with the required WebView runtime. These assumptions do not block implementation.

## Non-goals

Console database migration, coordinated desktop instances, GPU processing, nonnumeric passwords, PDF rewriting and network processing. Git delivery, publishing and deployment follow separate user instructions; the user later invoked merge-work for main delivery.

## Current state

The current app lives in `desktop/`, with a Tauri shell, Rust recovery service and static UI. Its code and delivery record were pushed to main. The earlier .NET code stays on the back burner; its documents are archived and do not govern active product work.

Unit checks, code review and the Windows installer build have recorded passing evidence in [BRU-001](../issues/BRU-001-desktop.md). Native dialogs, rendered UI, actual history persistence and restart/resume still need user-invoked integration review. Linux and macOS builds have not been verified here.

## Open questions

None. The approved decisions are recorded in [the question ledger](../notes/tauri-app-questions.md).
