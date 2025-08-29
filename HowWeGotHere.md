# How We Got Here

This file documents the key decisions and steps taken during the development of the PDF Brute Force tool, aiming to provide a clear historical log and prevent revisiting past challenges.

## Initial Setup

1.  Created this log file, `HowWeGotHere.md`.
2.  Restructured the project to match the PRD, creating `Brutus.Core` and `Brutus.Console`.
3.  Installed `itext7` NuGet package.
4.  Ensured the solution compiles.

## Core Implementation

1.  Created `FileScanner.cs` to find PDF files.
2.  Created `Logger.cs` for console and file logging.
3.  Created `PdfProcessor.cs` to handle PDF operations with `iText7`.
4.  Created `BruteForceEngine.cs` to perform the brute force attack, specifically targeting 6-digit numeric passwords (000000 to 999999).
5.  Created model classes `ProcessingResult.cs` and `BruteForceOptions.cs`.
6.  Created `CommandLineHandler.cs` to parse arguments.
7.  Updated `Program.cs` to orchestrate the application flow.
8.  Verified the entire solution builds successfully.

## Debugging and Refinement - PDF Library Choice

1.  Encountered an issue where `iText7` did not correctly identify a password-protected PDF.
2.  Switched from `iText7` to the `PDFsharp` library to address the issue.
3.  Experimented with different versions and APIs of `PDFsharp` to find a reliable way to detect password protection.
4.  Added extensive logging, including reflection-based inspection of `PdfSecuritySettings`, to diagnose the problem.
5.  The application is now able to correctly identify and process password-protected files.

## New Features & Project Evolution

1.  Created `FoundLogger.cs` to log all processed files and their results to `Found.txt`. This log includes the file path, the found password (or 'N/A'), and the time taken in minutes.
2.  Updated `Program.cs` to use the new `FoundLogger`.
3.  **Database Naming Convention**: Initially, the database was referred to as `progress.db` and later `Brutus.db`. A decision was made to standardize the database file name to `brutus.db` (all lowercase) across the project for consistency and clarity. This change was applied to all relevant code and documentation files (`PRD.md`, `README.md`).
4.  **Project Scope Clarification**: The initial "educational demonstration" angle was removed from the project's description. The tool's limitation to 6-digit numeric passwords is now explicitly stated as being by design to serve the specific purpose of efficiently cracking a batch of PDF files known to have this password pattern. This reflects the practical, problem-solving origin of the tool.
5.  **Build and Run Instructions**: Comprehensive instructions for building the project (using `dotnet build --configuration Release`) and running the compiled `Brutus.exe` directly were added to `README.md` to assist users who are not familiar with .NET development workflows.
6.  **Multi-Instance Logging Enhancement**: Addressed an `IOException` issue that occurred when multiple `Brutus.exe` instances attempted to write to the same `log.txt` and `Found.txt` files concurrently. The solution implemented involves creating a dedicated `Logs` directory. Within this directory, each `Brutus.exe` instance now generates its own unique log files (e.g., `log_someinstanceid.txt` and `Found_someinstanceid.txt`) by incorporating a unique instance ID (GUID) into the filenames. This prevents file access collisions and ensures independent logging for each running instance.

## Senior Developer Refactoring - Asynchronous Conversion

To improve efficiency and align with modern C# best practices, the entire application was refactored to be asynchronous.

1.  **I/O Operations**: All synchronous file and database operations were converted to use `async/await`.
2.  **Repositories**: The `IFileRepository` and `IBatchRepository` interfaces, along with their concrete implementations, were updated to be fully asynchronous, using Dapper's async methods.
3.  **Service Layer**: The `async` pattern was propagated up through the `BatchCoordinator` and logging services.
4.  **Main Entry Point**: The application's entry point in `Program.cs` was converted to an `async Task Main`, allowing the entire application to run without blocking on I/O.

This change prevents thread blocking during database queries and file writes, making the application more scalable and responsive.

## Gemini CLI's Contribution

The Gemini CLI played a significant role in the coding and refactoring process, assisting with various implementation tasks and ensuring adherence to project requirements.