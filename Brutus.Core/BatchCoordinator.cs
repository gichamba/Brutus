using Brutus.Core.Database.Models;
using Brutus.Core.Database.Repositories;

namespace Brutus.Core;

/// <summary>
/// Orchestrates the distribution of work between multiple instances.
/// This class acts as the central nervous system for the multi-instance coordination,
/// ensuring that files and password batches are processed efficiently and without duplication.
/// </summary>
public class BatchCoordinator(IFileRepository fileRepository, IBatchRepository batchRepository)
{
    /// <summary>
    /// Gets the next available file to be processed. This includes resetting stale batches
    /// before fetching the next pending or in-progress file.
    /// </summary>
    /// <returns>A Task representing the asynchronous operation, with a result of a FileRecord object or null if no work is available.</returns>
    public async Task<FileRecord?> GetNextFileAsync()
    {
        // First, reset any batches that were checked out by instances that may have crashed or exited.
        await batchRepository.ResetStaleBatchesAsync();
        return await fileRepository.GetNextFileToProcessAsync();
    }

    /// <summary>
    /// Checks out the next available password batch for a given file.
    /// </summary>
    /// <param name="fileId">The ID of the file to process.</param>
    /// <param name="instanceId">The unique ID of the instance checking out the batch.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of a TrialBatch object or null if no batches are available.</returns>
    public async Task<TrialBatch?> CheckoutNextBatchAsync(int fileId, string instanceId)
    {
        return await batchRepository.GetNextBatchAsync(fileId, instanceId);
    }

    /// <summary>
    /// Marks a specific batch as completed.
    /// </summary>
    /// <param name="batchId">The ID of the batch to mark as complete.</param>
    public async Task MarkBatchAsCompletedAsync(int batchId)
    {
        await batchRepository.UpdateBatchStatusAsync(batchId, "COMPLETED");
    }

    /// <summary>
    /// Checks if all batches for a specific file have been completed.
    /// </summary>
    /// <param name="fileId">The ID of the file to check.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of true if all batches are complete, otherwise false.</returns>
    public async Task<bool> CheckIfFileIsCompletedAsync(int fileId)
    {
        return await batchRepository.AreAllBatchesCompleteAsync(fileId);
    }

    /// <summary>
    /// Marks a file as fully completed, recording the final result.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    /// <param name="password">The password if found, otherwise a status like 'NO_PASSWORD' or 'FAILED'.</param>
    /// <param name="durationMinutes">The total time taken to process the file.</param>
    public async Task MarkFileAsCompletedAsync(int fileId, string? password, double durationMinutes)
    {
        await fileRepository.UpdateFileResultAsync(fileId, password, durationMinutes);
    }

    /// <summary>
    /// Marks a file as 'IN_PROGRESS' once processing begins.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    public async Task MarkFileAsInProgressAsync(int fileId)
    {
        await fileRepository.UpdateFileStatusAsync(fileId, "IN_PROGRESS");
    }

    /// <summary>
    /// Creates the 100 password batches for a given file in the database.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    public async Task CreateBatchesForFileAsync(int fileId)
    {
        await batchRepository.CreateBatchesForFileAsync(fileId);
    }
}