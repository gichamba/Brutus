---
id: BRU-001
title: Add the Brutus Tauri desktop app
kind: feature
status: in-review
owner: agent
acceptance_by: integration-review
delivery_target: main
depends_on: []
sources:
  - docs/specs/desktop.md
next_action: user-invoked integration review
---

## Outcome

A local Tauri app selects PDFs or folders, recovers numeric passwords with Rust workers, and keeps history.

## Scope

Implement D1 through D9 in desktop/. Preserve the console app and the existing PRD.md edit.

## Acceptance criteria

- [ ] AC1: Native file and folder selection discovers valid PDF inputs and displays input errors (D1).
- [ ] AC2: Distinct ranges run across files and within a file; password recovery and exhaustion are accurate (D2, D3, D6).
- [ ] AC3: Progress, cancellation and resume retain completed ranges (D4).
- [ ] AC4: History and successful hash reuse survive restarts, and changed sources reset checkpoints (D5).
- [ ] AC5: The UI supports keyboard use, narrow layouts, errors, empty states and searchable history (D7).
- [x] AC6: The Windows desktop build has zero errors and warnings; existing C# source stays intact (D8, D9).

## Verification plan

Pure unit tests for range coverage, job ownership, cancellation, resume, validation, PDF authentication and outcome classification. Static JavaScript checks and a Windows build. Native dialogs, rendered interactions, actual persistence and restart/resume need explicitly user-invoked integration review.

## Evidence

Implementation and code review passed, and the work is committed on main. AC1 through AC5 have implementation and local evidence but remain unproven through the real desktop flow until integration review.

- Changed files: desktop/ contains the Tauri shell, Rust service, scheduler, PDF authentication, SQLite history, static UI, tests, package locks and icons. README.md links the desktop instructions. .gitignore excludes generated Rust output. docs/specs/desktop.md and docs/notes/tauri-app-questions.md record the governing behavior and dependency approval.
- TDD: the initial range tests failed with three empty-range failures. Scheduling, cancellation and search tests then failed with five behavior failures before implementation; all passed after implementation. The review regression first failed because a malformed replacement retained `Some("123456")` instead of `None`, then passed after the error propagation fix.
- Final `cargo test --lib --locked`: 17 passed, 0 failed, 0 ignored, 0 filtered out. Tests are pure in-process checks; none start a database, application, browser or service.
- Final `npm test`: 6 passed, 0 failed, 0 skipped. `npm run check`, syntax checks for the view model and its tests, and static HTML ID reference checks passed.
- Final `npm run build -- --bundles nsis`: exit 0, optimized Windows x64 executable and installer built, zero compiler warnings or errors. The installer is desktop/src-tauri/target/release/bundle/nsis/Brutus_0.1.0_x64-setup.exe, 3,039,114 bytes.
- Build environment: automatic MSVC discovery first selected an incomplete Visual Studio 18 installation with missing `excpt.h`. The existing VS2019 developer environment supplied the working headers and libraries. Build helpers are under the user's temp directory at agents/Brutus/, not in the repository. An intermediate test build had five unused-code warnings before the service was wired; the final test and release builds have none.
- Candidate: SHA256 `aae7a5813edd14c6f36d79c4d316b3a35b63d86c4d230f472f0ff9a49fba85c3` of sorted desktop source/config/lock/icon paths and their SHA256 hashes, excluding Markdown and ignored outputs. Installer SHA256: `821487c507886fe816dd421b5f823bd17dd3fe7325970ad2657f0a14af8dd34c`.
- Initial implementation preservation: no diffs in Brutus.Core or Brutus.Console. The PRD.md diff was the original user edit. At that handoff, nothing had been committed, merged, pushed or deployed.
- Review: one read-only worker used the explicit `gpt-6.1-sol` fallback selector because the runtime exposes no Terra model. The worker found one changed-source error propagation defect. It was fixed, covered by the failing-then-passing regression, and the worker passed the correction. No remaining confirmed findings.
- Prose: humanizer review covered the new docs, UI copy and authored comments. Its advisory checker reported no findings for the desktop spec and README.
- Integration, browser and system checks: not run here; require user-invoked integration-review.

### Merge delivery

- The user invoked merge-work, authorizing commits on main and a push to origin. The delivery target is now main. No other branches, worktrees, stashes or active claims needed combination or cleanup; there were no conflicts.
- Combined implementation commit: `4cfbf129c0de239ccc20f2e61556e331e618659b`. The following delivery-record commit changes Markdown only and preserves this tested source candidate.
- Main check: `cargo test --lib --locked` passed 17 tests, with 0 failures, 0 ignored and 0 filtered out. `npm test` passed 6 tests with 0 failures and 0 skipped; `npm run check` passed. `npm run build -- --bundles nsis` exited 0, producing the optimized Windows executable and NSIS installer with 0 compiler warnings and 0 compiler errors. Logs are under the user's temp directory at agents/Brutus/merge-unit-tests.log and agents/Brutus/merge-windows-build.log.
- Main source candidate hash: `1e9314c26223081910b8173cbfdd759875ddfcd474f58613e6fd9b333afd3b78`, using the source/config/lock/icon manifest method recorded above. Rebuilt installer: 3,036,841 bytes, SHA256 `a65d49d485a352c1cfbff79208ce6de7a8bea5218a6fdb182f36ad50c9017ea3`. This replaces the earlier working-tree installer at the same path.
- Before committing, review-comments and humanizer checked all 14 staged source files and recorded the receipt. Missing module, declaration, field and handler documentation was added. The commit guard twice misclassified non-secret data: a public PDF test fixture and a SQL column expression. The fixture now generates 000008 from numeric example 8, keeping fixed independent assertions. The SQL cache update uses its existing bound parameter ?2 for the same nullable value. No guard settings were changed.
- A focused read-only review on `4cfbf129` passed the two equivalent changes. The earlier recovery review remains applicable; no remaining confirmed findings. The worker used the explicit gpt-6.1-sol fallback because Terra was unavailable.
- Target reached: committed main. Implementation and its delivery record were pushed to origin/main at `3d0944ab060061bd3e3348e21d07f2e6978c25c6`. Push output confirmed `7c1cbbe..3d0944a main -> main`; the remote-head read matched local main. No deployment, tags or real-system checks ran.
- Integration, browser and system checks: not run here; require user-invoked integration review.

Remaining verification: native selection, actual multi-file recovery, rendered layout and keyboard use, cancellation through the UI, SQLite persistence, restart/resume and cache reuse. Linux and macOS builds have not run. The user owns the next integration-review invocation for BRU-001; no further dependency approval is needed.

## Timeline and comments

- 2026-10-14T13:16:52+03:00 | agent | in-progress | Claimed BRU-001 after the user approved dependencies. No other Issue claims exist.
- 2026-10-14T13:16:52+03:00 | agent | approach | Add static frontend and Tauri config, test the range scheduler first, implement Rust recovery and checkpoints, then finish UI and build. Main risks are encryption compatibility, cancellation races and persistence errors. One read-only review worker follows implementation; no system checks are authorized.
- 2026-10-14T13:44:12+03:00 | agent | in-review | Implementation passed final unit, static and Windows installer checks. Released the claim. Fresh review found a stale-password error-path defect; the fix passed its regression and focused re-review. Awaiting user-invoked integration review at the working tree.
- 2026-10-14T16:59:08+03:00 | agent | in-progress | The user invoked merge-work. Claimed delivery to main, including the original PRD edit. Inventory found one dirty main worktree, no stashes, no other claims and no other local or remote branches. Fetch confirmed main and origin/main at 7c1cbbe. Comment review added documentation only. Integration review remains pending.
- 2026-10-14T17:10:15+03:00 | agent | in-review | Committed the combined work as 4cfbf129 on main. All 23 unit tests passed and the Windows installer built with zero compiler warnings or errors. Focused review passed the bound SQL update and generated fixture. Released the delivery claim; user-invoked integration review remains the next product verification step.
