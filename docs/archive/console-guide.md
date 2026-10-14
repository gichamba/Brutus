# Historical C# console guide

The earlier console source remains in `Brutus.Core/`, `Brutus.Console/` and `Brutus.sln`. The user keeps it on the back burner as historical code. Active product use and development are described in the [Tauri README](../../README.md) and [desktop build guide](../../desktop/README.md).

The commands below are historical references for the retained console source. Its earlier requirements are preserved in the [console PRD](console-prd.md), and its development notes are in the [console history](console-history.md).

## Build

Use the .NET 9 SDK. From the repository root:

```powershell
dotnet restore Brutus.sln
dotnet build Brutus.sln --configuration Release
```

The executable is written to `Brutus.Console/bin/Release/net9.0/Brutus.exe`.

## Run

Pass a PDF file or a directory as the single command-line argument:

```powershell
Brutus.Console\bin\Release\net9.0\Brutus.exe "C:\Documents\locked.pdf"
Brutus.Console\bin\Release\net9.0\Brutus.exe "C:\Documents\PDFs"
```

The console searches six-digit numeric passwords from `000000` through `999999`. It uses SQLite to coordinate batches between console instances.

Its default `brutus.db` path is relative to the working directory. Each console instance writes `Logs/log_<instance-id>.txt` and `Logs/Found_<instance-id>.txt` there. The latter records tab-delimited file paths, password outcomes and durations.

These console files are separate from the desktop app's `brutus-desktop.db`. The desktop does not import the console database or log files.

## Verification

The Tauri work left this C# source intact. Its console build and real processing flows were not rerun during the desktop implementation or this documentation update.
