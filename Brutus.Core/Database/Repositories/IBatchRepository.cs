using Brutus.Core.Database.Models;

namespace Brutus.Core.Database.Repositories;

/// <summary>
/// Defines the contract for batch data access operations.
/// </summary>
public interface IBatchRepository
{
    /// <summary>
    /// Creates all the necessary password batches for a given file.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    Task CreateBatchesForFileAsync(int fileId);

    /// <summary>
    /// Gets the next available batch for a file and marks it as checked out.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    /// <param name="instanceId">The ID of the instance checking out the batch.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of a TrialBatch object.</returns>
    Task<TrialBatch> GetNextBatchAsync(int fileId, string instanceId);

    /// <summary>
    /// Updates the status of a batch.
    /// </summary>
    /// <param name="batchId">The ID of the batch to update.</param>
    /// <param name="status">The new status.</param>
    Task UpdateBatchStatusAsync(int batchId, string status);

    /// <summary>
    /// Checks if all batches for a file are complete.
    /// </summary>
    /// <param name="fileId">The ID of the file to check.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of true if all batches are complete, otherwise false.</returns>
    Task<bool> AreAllBatchesCompleteAsync(int fileId);

    /// <summary>
    /// Resets any batches that have been checked out for too long.
    /// </summary>
    Task ResetStaleBatchesAsync();
}