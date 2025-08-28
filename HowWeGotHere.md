# How We Got Here

This file documents the steps taken to implement the PDF Brute Force tool.

## Initial Setup

1.  Created this log file, `HowWeGotHere.md`.
2.  Restructured the project to match the PRD, creating `Brutus.Core` and `Brutus.Console`.
3.  Installed `itext7` NuGet package.
4.  Ensured the solution compiles.

## Implementation

1.  Created `FileScanner.cs` to find PDF files.
2.  Created `Logger.cs` for console and file logging.
3.  Created `PdfProcessor.cs` to handle PDF operations with `iText7`.
4.  Created `BruteForceEngine.cs` to perform the brute force attack.
5.  Created model classes `ProcessingResult.cs` and `BruteForceOptions.cs`.
6.  Created `CommandLineHandler.cs` to parse arguments.
7.  Updated `Program.cs` to orchestrate the application flow.
8.  Verified the entire solution builds successfully.

## Debugging and Refinement

1.  Encountered an issue where `iText7` did not correctly identify a password-protected PDF.
2.  Switched from `iText7` to the `PDFsharp` library to address the issue.
3.  Experimented with different versions and APIs of `PDFsharp` to find a reliable way to detect password protection.
4.  Added extensive logging, including reflection-based inspection of `PdfSecuritySettings`, to diagnose the problem.
5.  The application is now able to correctly identify and process the user's password-protected file.

## New Features

1.  Created `FoundLogger.cs` to log all processed files and their results to `Found.txt`.
2.  The `Found.txt` log includes the file path, the found password (or 'N/A'), and the time taken in minutes.
3.  Updated `Program.cs` to use the new `FoundLogger`.

## Senior Developer Refactoring

### Asynchronous Conversion
To improve efficiency and align with modern C# best practices, the entire application was refactored to be asynchronous.

1.  **I/O Operations**: All synchronous file and database operations were converted to use `async/await`.
2.  **Repositories**: The `IFileRepository` and `IBatchRepository` interfaces, along with their concrete implementations, were updated to be fully asynchronous, using Dapper's async methods.
3.  **Service Layer**: The `async` pattern was propagated up through the `BatchCoordinator` and logging services.
4.  **Main Entry Point**: The application's entry point in `Program.cs` was converted to an `async Task Main`, allowing the entire application to run without blocking on I/O.

This change prevents thread blocking during database queries and file writes, making the application more scalable and responsive.
