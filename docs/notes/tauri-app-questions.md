# Tauri app questions

The user requested an update of this repository into a Tauri desktop app with a Rust backend, file or folder selection, parallel processing across files and number ranges, a modern UI, and saved work history with outcomes.

## Q1 - What work must the app perform?

Status: inferred.

Answer: preserve recovery of six-digit numeric passwords for the user's PDF files, from 000000 through 999999. The user requested a new app in the same repository, not a new password strategy.

Source: user request; PRD.md sections 1.2, 2.4 and 6.1; README.md; Brutus.Core/BruteForceEngine.cs.

Searched: PDF, password, Brutus, Tauri in project docs, all repository file paths, core processing code, git history, Codex session records and available Claude project paths. No earlier applicable desktop decision was found. One historical session matched this repo path but contained unrelated work.

Adoption: implementation outline below.

## Q2 - Which desktop platforms must be supported?

Status: answered for platform intent; inferred for local verification target.

Answer: retain cross-platform desktop design. Windows is the first local build target. Linux and macOS builds cannot be demonstrated on this Windows host.

Source: PRD.md sections 3.1 and 4.4; current workspace.

Searched: platform, cross-platform, Windows in PRD.md, README.md, historical records and tool availability.

Adoption: implementation outline below.

## Q3 - What information belongs in history?

Status: inferred.

Answer: save input paths, start and finish times, run settings, status, per-file outcomes and recovered passwords locally across restarts. Reuse successful results by file hash. Checkpoint completed ranges for resume after interruption.

Source: user request; PRD.md sections 2.5 through 2.7 and 4.3.

Searched: history, resume, outcome, database, hash in project docs and core code; Tauri and Rust PDF library documentation for implementation feasibility.

Adoption: implementation outline below.

## Q4 - What is the delivery boundary?

Status: inferred from the user's request.

Implement in the working tree. No commit, merge, push, or deployment was requested.

Source: user request and user instructions.

Searched: delivery and implement in the current request and user instructions.

Adoption: implementation outline below.

## Q5 - Which existing behavior and implementation should be retained?

Status: inferred.

Answer: add the desktop app within this repository. Preserve the C# console app and the user's existing PRD.md edit. Use a separate desktop database in the app data directory. Import of the console database is outside this first implementation; it was not requested.

Source: user request; git status and PRD.md diff; existing C# project files.

Searched: existing projects, database, compatibility and current edits in repository files and git diff.

Adoption: implementation outline below.

## Q6 - May the new app add the required dependencies?

Status: answered.

User answer on 2026-10-14: "Yes" to the proposed dependency set.

Proposed Rust dependencies: tauri 2, tauri-plugin-dialog 2, lopdf, rusqlite with bundled SQLite, serde, serde_json and sha2. Build dependency: tauri-build 2. Development dependency: @tauri-apps/cli 2.

Use native Rust threads for the shared worker pool and plain HTML, CSS and JavaScript for the UI. No frontend framework or extra concurrency package is needed.

Source: user instructions require approval before adding production dependencies. Tauri dialog documentation describes the native picker plugin. lopdf Document documentation provides password authentication methods.

Searched: package references, Cargo.toml, package.json, dependency approval in user instructions, https://v2.tauri.app/plugin/dialog/ and https://docs.rs/lopdf/latest/lopdf/struct.Document.html. No approval for this new dependency set was found.

Adoption: docs/specs/desktop.md, D9.

## Adopted implementation

The governing requirements are in [the desktop spec](../specs/desktop.md). Implementation and review evidence are in [BRU-001](../issues/BRU-001-desktop.md).

1. Add a Tauri desktop project with static frontend assets and a Rust backend. Keep existing C# projects intact.
2. Build a restrained dark UI with file and folder buttons, selected inputs, worker count, numeric range, live progress, cancellation and a history view. Cover empty, running, completed, cancelled and error states. Provide keyboard access and visible focus.
3. Scan folders recursively for PDFs. Use one bounded worker pool to claim distinct number ranges across files. Stop a file's remaining work when its password is found. Keep PDF parsing outside the per-password loop. Classify malformed PDFs separately from wrong passwords.
4. Store runs, file outcomes, content hashes and completed ranges in local SQLite. Mark unfinished runs interrupted at startup and allow resumption. Preserve source PDFs.
5. Use TDD for range coverage, scheduling, cancellation, result reuse and outcome classification. Run pure unit tests and a warning-free build. Application, browser and persistence-through-restart checks wait for explicitly user-invoked integration review.

Authorized implementation choices: UI styling, native worker threads, keeping the console app, desktop history location and the initial history fields.

No material owner decisions remain. The user approved the listed dependencies. Implementation is authorized in the working tree.
