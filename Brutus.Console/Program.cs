
using Brutus.Core;
using System.Diagnostics;
using Brutus.Console;
using Brutus.Core.Database;
using Brutus.Core.Database.Repositories;
using Brutus.Core.Database.Models;

return await Main(args);

async Task<int> Main(string[] args)
{
    // --- 1. Initialization ---
    CommandLineHandler commandLineHandler = new();
    string? path = commandLineHandler.GetPath(args);
    if (path == null) return 1; // Exit with an error code.

    string instanceId = System.Guid.NewGuid().ToString();
    Logger logger = new();
    FoundLogger foundLogger = new();
    logger.LogInfo($"Brutus instance {instanceId} started.");

    DatabaseManager databaseManager = new();
    databaseManager.InitializeDatabase();
    logger.LogInfo("Database initialized.");

    string connectionString = databaseManager.GetConnectionString();
    FileRepository fileRepository = new(connectionString, logger);
    BatchRepository batchRepository = new(connectionString);
    BatchCoordinator batchCoordinator = new(fileRepository, batchRepository);
    FileScanner fileScanner = new(fileRepository, logger, foundLogger);
    PdfProcessor pdfProcessor = new(logger);
    BruteForceEngine bruteForceEngine = new(pdfProcessor, logger);

    // --- 2. File Population ---
    await fileScanner.ScanAndAddFilesAsync(path);

    // --- 3. Main Processing Loop ---
    logger.LogInfo("Checking for available work...");
    Stopwatch fileStopwatch = new();

    while (true)
    {
        FileRecord? fileToProcess = await batchCoordinator.GetNextFileAsync();

        if (fileToProcess == null)
        {
            logger.LogInfo("No more pending files available. Instance shutting down.");
            break;
        }

        logger.LogInfo($"--- Assigned File [ID: {fileToProcess.Id}]: {fileToProcess.FilePath} ---");
        fileStopwatch.Restart();

        if (!pdfProcessor.IsPasswordProtected(fileToProcess.FilePath))
        {
            logger.LogSkip("File is not password protected.");
            await batchCoordinator.MarkFileAsCompletedAsync(fileToProcess.Id, "NO_PASSWORD", 0);
            foundLogger.Log(fileToProcess.FilePath, "NO_PASSWORD", 0);
            continue;
        }

        logger.LogInfo("File is password protected. Preparing for brute force.");
        await batchCoordinator.MarkFileAsInProgressAsync(fileToProcess.Id);
        await batchCoordinator.CreateBatchesForFileAsync(fileToProcess.Id);

        // --- 3a. Batch Processing Loop ---
        while (true)
        { 
            TrialBatch? batch = await batchCoordinator.CheckoutNextBatchAsync(fileToProcess.Id, instanceId);
            if (batch == null)
            {
                logger.LogInfo($"No more available batches for file {fileToProcess.Id}. Moving to next file.");
                break;
            }

            logger.LogInfo($"Processing Batch {batch.batch_index} ({batch.range_from} - {batch.range_to}) for file {fileToProcess.Id}");
            
            string? password = bruteForceEngine.CrackPasswordRange(fileToProcess.FilePath, batch.range_from, batch.range_to);

            if (password != null)
            {
                fileStopwatch.Stop();
                logger.LogSuccess($"Password FOUND: {password}");
                await batchCoordinator.MarkBatchAsCompletedAsync(batch.Id);
                await batchCoordinator.MarkFileAsCompletedAsync(fileToProcess.Id, password, fileStopwatch.Elapsed.TotalMinutes);
                foundLogger.Log(fileToProcess.FilePath, password, fileStopwatch.Elapsed.TotalMinutes);
                break;
            }
            else
            {
                logger.LogInfo($"Password not in batch {batch.batch_index}.");
                await batchCoordinator.MarkBatchAsCompletedAsync(batch.Id);
            }
        }
    }

    logger.LogInfo("Application completed.");
    return 0; // Success
}
