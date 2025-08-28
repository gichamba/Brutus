# PDF Brute Force Tool - Product Requirements Document

## 1. Project Overview

### 1.1 Project Name
PDF Password Brute Force Tool

### 1.2 Purpose
Personal tool to recover forgotten 6-digit numeric passwords from my own PDF files. I had a collection of password-protected PDFs where I knew the passwords were 6-digit numbers but had forgotten or misplaced the specific passwords.

### 1.3 Target Audience
- Personal use for recovering forgotten passwords from owned PDF files
- Users with similar situations involving 6-digit numeric PDF passwords

### 1.4 Objectives
- Recover forgotten 6-digit numeric passwords from personal PDF files
- Process multiple files efficiently across multiple instances
- Avoid duplicate work on previously processed files
- Maintain and resume progress across sessions

## 2. Functional Requirements

### 2.1 Command Line Interface
The application must accept one command-line argument:
- **File Path**: Full or relative path to a single PDF file
- **Directory Path**: Full or relative path to a directory containing PDF files

#### 2.1.1 Usage Examples
```
Brutus.exe "C:\Documents\locked.pdf"
Brutus.exe ".\test-files\"
Brutus.exe "/home/user/documents/"
```

### 2.2 File Processing Logic

#### 2.2.1 Single File Mode
- Validate the file exists and has .pdf extension
- Check if the PDF is password protected
- If not password protected: Skip and log
- If password protected: Attempt brute force attack

#### 2.2.2 Directory Mode
- Recursively search all subdirectories
- Process all PDF files found (*.pdf extension)
- Apply same logic as single file mode to each PDF

### 2.3 Password Protection Detection
- Attempt to open PDF without password
- If successful: File is not password protected
- If exception/failure: File is likely password protected

### 2.4 Brute Force Attack Implementation

#### 2.4.1 Password Range
- **Start**: 000000
- **End**: 999999
- **Format**: 6-digit zero-padded numbers
- **Total Attempts**: 1,000,000 possible combinations

#### 2.4.2 Attack Strategy
- Batch-based processing (10,000 passwords per batch)
- Stop immediately when correct password is found
- Database-coordinated progress tracking
- Multi-instance support for parallel processing
- Resume capability for interrupted sessions

### 2.5 Logging Requirements

#### 2.5.1 Console Output
**Real-time display of:**
- Application startup message
- Current file being processed
- Password protection status
- Brute force progress (every 10,000 attempts)
- Success/failure results
- Summary statistics

#### 2.5.2 Log File Output (log.txt)
**Persistent logging of:**
- Timestamp for each operation
- File path and status
- Found passwords
- Processing duration
- Error messages

#### 2.5.3 Found Results File (Found.txt)
**Explicit tracking of ALL evaluated files:**
- Full file path
- Password found (or "NO_PASSWORD" if not password protected)
- Time taken to find password in minutes (decimal format)
- Tab-delimited format for easy parsing

#### 2.5.4 Log Entry Examples

**log.txt format:**
```
[2024-08-28 14:30:15] INFO: Processing file: C:\test\document1.pdf
[2024-08-28 14:30:16] INFO: File is not password protected - SKIPPED
[2024-08-28 14:30:20] INFO: Processing file: C:\test\secure.pdf
[2024-08-28 14:30:21] INFO: File is password protected - Starting brute force
[2024-08-28 14:32:45] SUCCESS: Password found for secure.pdf: 123456
```

**Found.txt format (tab-delimited):**
```
C:\test\document1.pdf	NO_PASSWORD	0.02
C:\test\secure.pdf	123456	2.40
C:\test\confidential.pdf	FAILED	5.00
```

### 2.6 Multi-Instance Support and Progress Tracking

#### 2.6.1 SQLite Database Schema
**Database File**: `progress.db` (created in application directory)

**Table: file_list**
```sql
CREATE TABLE file_list (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL,  -- SHA256 hash of file content
    status TEXT NOT NULL CHECK(status IN ('PENDING', 'IN_PROGRESS', 'COMPLETED', 'FAILED')),
    has_password INTEGER NOT NULL DEFAULT 0,  -- 0=unknown, 1=yes, 2=no
    password_found TEXT DEFAULT NULL,
    started_at DATETIME DEFAULT NULL,
    completed_at DATETIME DEFAULT NULL,
    total_time_minutes REAL DEFAULT NULL,
    instance_id TEXT DEFAULT NULL  -- Track which Brutus instance is working on it
);
```

**Table: trial_batches**
```sql
CREATE TABLE trial_batches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    batch_index INTEGER NOT NULL,  -- 0-based index (0-99)
    range_from TEXT NOT NULL,      -- e.g., "000000"
    range_to TEXT NOT NULL,        -- e.g., "009999"
    status TEXT NOT NULL CHECK(status IN ('PENDING', 'CHECKED_OUT', 'COMPLETED')),
    checked_out_at DATETIME DEFAULT NULL,
    completed_at DATETIME DEFAULT NULL,
    instance_id TEXT DEFAULT NULL,
    FOREIGN KEY (file_id) REFERENCES file_list(id),
    UNIQUE(file_id, batch_index)
);
```

#### 2.6.2 Password Range Distribution
- **Total Range**: 000000 to 999999 (1,000,000 passwords)
- **Batch Count**: 100 batches per file
- **Batch Size**: 10,000 passwords per batch
- **Batch Distribution**:
  - Batch 0: 000000-009999
  - Batch 1: 010000-019999
  - ...
  - Batch 99: 990000-999999

#### 2.6.3 Multi-Instance Coordination
**Instance Behavior (revised):**
1. Generate unique instance ID (GUID or hostname+PID)
2. **Scan and hash files**, handle duplicates as above
3. Check database for next available work:
   - Find first file with status 'PENDING' 
   - If no pending files, find first file with status 'IN_PROGRESS' that has uncompleted batches
4. Checkout next available batch for the selected file
5. Mark batch as 'CHECKED_OUT' with instance_id and timestamp
6. Process the batch range
7. Update batch status to 'COMPLETED' when done
8. Check if all batches for file are complete, update file status accordingly

#### 2.6.4 Resume Capability
- On startup, application checks for 'CHECKED_OUT' batches older than 10 minutes
- Resets stale checkouts back to 'PENDING' status
- Continues processing from where previous instances left off
- Database ensures atomic operations to prevent race conditions
- **Hash-based duplicate detection** prevents reprocessing completed files even if moved or renamed

### 2.7 File Hash-Based Duplicate Detection

#### 2.7.1 Hash Calculation and Storage
- **Hash Algorithm**: SHA256 of file content
- **Hash Column**: `file_hash` (TEXT NOT NULL) in file_list table
- **Hash Timing**: Calculate immediately after file discovery, before any processing

#### 2.7.2 Duplicate Detection Logic
**For each discovered PDF file:**

1. **Calculate SHA256 hash** of file content
2. **Query database** for existing record with same hash:
   ```sql
   SELECT id, file_path, status, password_found 
   FROM file_list 
   WHERE file_hash = ?
   ```
3. **Handle based on result:**

**Case A: Hash does not exist in database**
- Insert new record with calculated hash
- Set status to 'PENDING' 
- Continue with normal processing

**Case B: Hash exists, status is NOT 'COMPLETED'**
- Insert new record with same hash (different file path)
- Set status to 'PENDING'
- Continue with normal processing
- **Rationale**: File content is same but may be in different location, and previous attempt was incomplete

**Case C: Hash exists, status IS 'COMPLETED'**
- **Console Output**: "File already processed: [current_path] (same as [existing_path]) - Password: [password_found]"
- **Log to log.txt**: "DUPLICATE: File [current_path] already processed, password: [password_found]"
- **Log to Found.txt**: Current file path with already-found password and 0.00 time
- **Do NOT insert** new record in database
- **Skip processing** entirely

## 3. Technical Requirements

### 3.1 Platform and Framework
- **Target Framework**: .NET 5.0 or later (C# 9 features)
- **Platform**: Cross-platform (Windows, Linux, macOS)
- **Architecture**: Console Application

### 3.2 External Dependencies
- **PDF Library**: iTextSharp 7 or PdfSharp for PDF manipulation
- **Database**: SQLite (Microsoft.Data.Sqlite) for progress tracking
- **File System**: System.IO for file and directory operations
- **Logging**: Built-in System.IO for file logging

### 3.3 Project Structure
```
PdfBruteForcer.sln
├── PdfBruteForcer.Core/
│   ├── Database/
│   │   ├── DatabaseManager.cs
│   │   ├── Models/
│   │   │   ├── FileRecord.cs
│   │   │   └── TrialBatch.cs
│   │   └── Repositories/
│   │       ├── IFileRepository.cs
│   │       ├── FileRepository.cs
│   │       ├── IBatchRepository.cs
│   │       └── BatchRepository.cs
│   ├── Security/
│   │   └── FileHasher.cs
│   ├── PdfProcessor.cs
│   ├── BruteForceEngine.cs
│   ├── BatchCoordinator.cs
│   ├── Logger.cs
│   ├── FileScanner.cs
│   └── Models/
│       ├── ProcessingResult.cs
│       └── BruteForceOptions.cs
└── PdfBruteForcer.Console/
    ├── Program.cs
    └── CommandLineHandler.cs
```

**Output**: `Brutus.exe`

### 3.4 Architecture Design

#### 3.4.1 Core Library (PdfBruteForcer.Core)
**Separation of concerns for future library conversion:**
- `DatabaseManager`: SQLite database initialization and connection management
- `FileRepository`/`BatchRepository`: Data access layer for file and batch tracking
- `FileHasher`: SHA256 hash calculation for duplicate detection
- `BatchCoordinator`: Multi-instance coordination and batch assignment
- `PdfProcessor`: PDF file validation and password testing
- `BruteForceEngine`: Password generation and attack execution for assigned batches
- `FileScanner`: Directory traversal, PDF discovery, hash calculation, and database population
- `Logger`: Centralized logging to console, log.txt, and Found.txt

#### 3.4.2 Console Application (PdfBruteForcer.Console)
**Minimal Program.cs implementation:**
- Command line argument parsing
- Instance ID generation
- Core library instantiation with database coordination
- Exception handling and user feedback
- Graceful shutdown with batch cleanup

### 3.5 Programming Best Practices

#### 3.5.1 SOLID Principles
- **Single Responsibility**: Each class has one clear purpose
- **Open/Closed**: Extensible for future password strategies
- **Liskov Substitution**: Interface-based design
- **Interface Segregation**: Focused interfaces
- **Dependency Inversion**: Dependency injection ready

#### 3.5.2 C# 9 Features Utilization
- Record types for result models
- Pattern matching enhancements
- Init-only properties
- Top-level programs (minimal Program.cs)

#### 3.5.3 Error Handling
- Comprehensive exception handling
- Graceful degradation for inaccessible files
- User-friendly error messages
- Proper resource disposal (using statements)

#### 3.5.4 Performance Considerations
- Async/await for I/O operations
- Memory-efficient file processing
- Progress reporting without performance impact
- Cancellation token support for interruption

## 4. Non-Functional Requirements

### 4.1 Performance
- **Maximum Memory Usage**: 100MB per instance during operation
- **Concurrent Instances**: Support unlimited simultaneous Brutus.exe instances
- **Database Concurrency**: SQLite with WAL mode for multi-reader/single-writer access
- **File Processing**: Handle files up to 100MB
- **Progress Reporting**: Update every 1,000 password attempts
- **Batch Timeout**: 10 minutes before resetting stale checkouts
- **Database Locking**: Maximum 5-second wait for database access

### 4.2 Reliability
- **Error Recovery**: Continue processing remaining files after failures
- **Data Integrity**: All successful results must be logged
- **Resource Management**: Proper cleanup of PDF file handles

### 4.3 Security Considerations
- **Personal Use Only**: Tool designed for recovering passwords from your own files
- **No Network Operations**: Purely local file processing
- **Secure Memory**: Clear password variables after use
- **File Hash Privacy**: SHA256 hashes stored locally only

### 4.4 Usability
- **Clear Documentation**: Comprehensive usage instructions
- **Progress Indication**: Visual feedback during long operations
- **Error Messages**: Actionable error descriptions
- **Cross-Platform**: Consistent behavior across operating systems

## 5. Input/Output Specifications

### 5.1 Input Validation
- Verify argument count (exactly 1 required)
- Validate file/directory existence
- Check file permissions and accessibility
- Validate PDF file format

### 5.2 Output Format

#### 5.2.1 Console Output Example
```
Brutus PDF Password Recovery Tool v1.0 [Instance: DESKTOP-ABC123-1234]
======================================================================

Initializing database...
Scanning for PDF files in: C:\MyPDFs\
Found 5 PDF files...
Calculating file hashes...

Processing file: document1.pdf
Hash calculated: a1b2c3d4...
Status: Not password protected - logging to Found.txt

Processing file: secure.pdf
Hash calculated: e5f6g7h8...
New file - added to database for processing

File already processed: C:\MyPDFs\backup\secure.pdf (same as C:\MyPDFs\secure.pdf) - Password: 123456

Assigned file [ID: 2]: secure.pdf  
Status: Password protected - checking out batch 12 (120000-129999)
Progress: [████████████████████] 123,456/129,999 (95.0%)
SUCCESS: Password found: 123456 (logged to Found.txt)

No more work available. Instance shutting down.

Session Summary:
===============
Files discovered: 5
New files processed: 2
Duplicate files skipped: 1
Passwords recovered: 1
Batches completed: 1
Total time: 00:03:15
```

#### 5.2.2 Log File Format (log.txt)
```
[2024-08-28 14:30:15] INFO: Brutus instance DESKTOP-ABC123-1234 started
[2024-08-28 14:30:15] INFO: Database initialized successfully
[2024-08-28 14:30:15] INFO: Scanning directory: C:\MyPDFs\
[2024-08-28 14:30:16] INFO: Found 5 PDF files, calculating hashes...
[2024-08-28 14:30:17] INFO: Hash calculated for document1.pdf: a1b2c3d4e5f6...
[2024-08-28 14:30:17] INFO: File not password protected - logged to Found.txt
[2024-08-28 14:30:18] INFO: Hash calculated for secure.pdf: e5f6g7h8i9j0...
[2024-08-28 14:30:18] INFO: New file added to database for processing
[2024-08-28 14:30:19] DUPLICATE: File C:\MyPDFs\backup\secure.pdf already processed, password: 123456
[2024-08-28 14:30:20] INFO: Assigned file ID 2: secure.pdf
[2024-08-28 14:30:20] INFO: Checked out batch 12 (120000-129999)
[2024-08-28 14:32:45] SUCCESS: Password found - File: secure.pdf, Password: 123456
[2024-08-28 14:32:45] INFO: File ID 2 completed successfully
[2024-08-28 14:32:46] INFO: Instance shutting down gracefully
```

#### 5.2.3 Found Results File (Found.txt)
```
File_Path	Password	Time_Minutes
C:\MyPDFs\document1.pdf	NO_PASSWORD	0.02
C:\MyPDFs\secure.pdf	123456	2.40
C:\MyPDFs\backup\secure.pdf	123456	0.00
C:\MyPDFs\confidential.pdf	FAILED	15.25
```

## 6. Constraints and Limitations

### 6.1 Password Scope
- **Limited to 6-digit numeric passwords only**
- Does not support alphabetic or special character passwords
- Batch-based sequential testing (no optimization algorithms)

### 6.2 PDF Compatibility
- Supports standard PDF encryption only
- May not work with custom or enterprise encryption
- Limited to password-based protection (not certificate-based)

### 6.3 Performance Limitations
- SQLite database limits concurrent write operations
- No GPU acceleration
- Linear search algorithm within each batch

### 6.4 Multi-Instance Constraints
- Maximum practical instances limited by system resources
- Database contention may slow batch coordination
- Network file systems may cause database locking issues

## 7. Future Enhancements

### 7.1 Library Conversion
- Package as NuGet library
- Support for custom password dictionaries
- Pluggable password generation strategies
- Alternative database backends (SQL Server, PostgreSQL)

### 7.2 Advanced Features
- Thread-pool based batch processing within single instance
- Dictionary-based attacks
- Custom character set support
- GUI version with real-time progress monitoring
- REST API for distributed processing coordination

## 8. Testing Requirements

### 8.1 Unit Tests
- PDF detection and validation
- SHA256 hash calculation and duplicate detection
- Password generation sequence
- Database operations (CRUD for files and batches)
- Batch coordination and instance management
- File system operations
- Logging functionality (all three log types)

### 8.2 Integration Tests
- Multi-instance coordination
- Database concurrency and locking
- End-to-end file processing with resume capability
- Directory traversal with database population
- Error handling scenarios

### 8.3 Test Data
- Sample PDFs with known passwords in different batch ranges
- Identical PDF files with different names/locations (hash collision testing)
- Various directory structures
- Edge cases (corrupted files, permission issues, database corruption)
- Concurrent access simulation

## 9. Deployment and Distribution

### 9.1 Build Configuration
- Release configuration for distribution
- Self-contained deployment option
- Cross-platform compatibility testing

### 9.2 Documentation
- README.md with setup instructions
- Usage examples and tutorials

### 9.3 Packaging
- Console executable
- Sample test files