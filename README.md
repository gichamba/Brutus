# PDF Password Brute Force Tool

## Overview

This was a classic case of needing a specific tool and building it on the spot. I had to recover a 6-digit password from a batch of PDFs, so I spun this up in about 30 minutes. The PRD was drafted with some help from Claude, and Gemini CLI handled the coding. Just another day at the office.

This is a C# console application that performs a multi-instance, database-coordinated brute-force attack on PDF files protected with 6-digit numeric passwords.

This is a simple tool that serves a (very) specific purpose.

## Features

- **Brute Force Attack**: Attempts to find the password of a protected PDF file by trying every combination from `000000` to `999999`.
- **Directory Scanning**: Can process a single PDF file or recursively scan an entire directory for PDF files.
- **Multi-Instance Coordination**: Multiple instances of `Brutus.exe` can be run simultaneously to work together on the same set of files. Work is coordinated through a central SQLite database (`progress.db`).
- **Session Resume**: If an instance is stopped, its work can be automatically picked up by another instance after a timeout. The application can be stopped and restarted, and it will resume where the collective of instances left off.
- **File Hash-Based Duplicate Detection**: Avoids reprocessing already completed files by calculating and storing SHA256 hashes of file content.
- **Logging**:
    - **Console**: Real-time progress of the current instance.
    - **`log.txt`**: A detailed, timestamped log of all operations.
    - **`Found.txt`**: A clean, tab-delimited summary of all processed files and the passwords that were found.

## Usage

To run the application, open a command prompt or terminal, navigate to the directory containing `Brutus.exe`, and provide the path to a PDF file or a directory.

### Process a single file:
```shell
Brutus.exe "C:\Path\To\Your\locked.pdf"
```

### Process all PDF files in a directory (and its subdirectories):
```shell
Brutus.exe "C:\Path\To\Your\Documents"
```

The application will create a `progress.db` file in the same directory to track progress.

## Core Limitation: 6-Digit Numeric Passwords Only

**This tool is intentionally limited and will ONLY work for passwords that are exactly six digits long and contain only numbers (e.g., `123456`, `987654`, `000000`).**

- It **cannot** find passwords containing letters.
- It **cannot** find passwords containing symbols.
- It **cannot** find passwords that are shorter or longer than 6 digits.

This limitation is by design to keep the scope of the educational demonstration focused and manageable. It effectively highlights the vulnerability of a very specific, weak password pattern.