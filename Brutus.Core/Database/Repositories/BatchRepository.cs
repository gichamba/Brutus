using Dapper;
using Microsoft.Data.Sqlite;
using Brutus.Core.Database.Models;

namespace Brutus.Core.Database.Repositories;

/// <summary>
/// Provides data access methods for the trial_batches table.
/// </summary>
public class BatchRepository(string connectionString) : IBatchRepository
{
    /// <summary>
    /// Populates the database with 100 password batches for a given file.
    /// Each batch covers a range of 10,000 passwords.
    /// </summary>
    /// <param name="fileId">The ID of the file to create batches for.</param>
    public async Task CreateBatchesForFileAsync(int fileId)
    {
        await using SqliteConnection connection = new(connectionString);
        // The password range is 0-999999, so we create 100 batches of 10,000.
        for (int i = 0; i < 100; i++)
        {
            const string sql = """
                INSERT OR IGNORE INTO trial_batches (file_id, batch_index, range_from, range_to, status)
                VALUES (@FileId, @BatchIndex, @RangeFrom, @RangeTo, 'PENDING');
            """;
            await connection.ExecuteAsync(sql, new
            {
                FileId = fileId,
                BatchIndex = i,
                RangeFrom = (i * 10000).ToString("D6"),
                RangeTo = ((i + 1) * 10000 - 1).ToString("D6"),
            });
        }
    }

    /// <summary>
    /// Atomically gets the next available batch for a file and marks it as 'CHECKED_OUT'.
    /// The RETURNING clause makes this an atomic operation, preventing race conditions.
    /// </summary>
    /// <param name="fileId">The ID of the file to get a batch for.</param>
    /// <param name="instanceId">The ID of the instance checking out the batch.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of the next available TrialBatch, or null if none are available.</returns>
    public async Task<TrialBatch> GetNextBatchAsync(int fileId, string instanceId)
    {
        await using SqliteConnection connection = new(connectionString);
        const string sql = """
            UPDATE trial_batches
            SET status = 'CHECKED_OUT', checked_out_at = @CheckedOutAt, instance_id = @InstanceId
            WHERE id = (
                SELECT id FROM trial_batches
                WHERE file_id = @FileId AND status = 'PENDING'
                ORDER BY batch_index
                LIMIT 1
            )
            RETURNING *;
        """;
        return await connection.QuerySingleOrDefaultAsync<TrialBatch>(sql, new { CheckedOutAt = DateTime.Now, InstanceId = instanceId, FileId = fileId });
    }

    /// <summary>
    /// Updates the status of a specific batch.
    /// </summary>
    /// <param name="batchId">The ID of the batch to update.</param>
    /// <param name="status">The new status (e.g., 'COMPLETED').</param>
    public async Task UpdateBatchStatusAsync(int batchId, string status)
    {
        await using SqliteConnection connection = new(connectionString);
        const string sql = "UPDATE trial_batches SET status = @Status, completed_at = @CompletedAt WHERE id = @BatchId;";
        await connection.ExecuteAsync(sql, new { Status = status, CompletedAt = DateTime.Now, BatchId = batchId });
    }

    /// <summary>
    /// Checks if all batches for a given file are marked as 'COMPLETED'.
    /// </summary>
    /// <param name="fileId">The ID of the file to check.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of true if all batches are complete, otherwise false.</returns>
    public async Task<bool> AreAllBatchesCompleteAsync(int fileId)
    {
        await using SqliteConnection connection = new(connectionString);
        const string sql = "SELECT COUNT(*) FROM trial_batches WHERE file_id = @FileId AND status != 'COMPLETED';";
        int count = await connection.ExecuteScalarAsync<int>(sql, new { FileId = fileId });
        return count == 0;
    }

    /// <summary>
    /// Resets batches that were checked out for more than 10 minutes.
    /// This handles cases where an instance crashes or fails without gracefully shutting down.
    /// </summary>
    public async Task ResetStaleBatchesAsync()
    {
        await using SqliteConnection connection = new(connectionString);
        const string sql = "UPDATE trial_batches SET status = 'PENDING', checked_out_at = NULL, instance_id = NULL WHERE status = 'CHECKED_OUT' AND checked_out_at < @StaleTime;";
        await connection.ExecuteAsync(sql, new { StaleTime = DateTime.Now.AddMinutes(-10) });
    }
}