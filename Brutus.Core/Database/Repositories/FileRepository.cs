
using Dapper;
using Microsoft.Data.Sqlite;
using Brutus.Core.Database.Models;

namespace Brutus.Core.Database.Repositories;

/// <summary>
/// Provides data access methods for the file_list table.
/// </summary>
public class FileRepository(string connectionString, Logger logger) : IFileRepository
{
    private readonly Logger _logger = logger;
    /// <summary>
    /// Adds a new file to the database if it doesn't already exist.
    /// Uses 'INSERT OR IGNORE' to prevent duplicates based on the UNIQUE constraint on file_path.
    /// </summary>
    /// <param name="filePath">The absolute path of the file to add.</param>
    /// <param name="fileHash">The SHA256 hash of the file.</param>
    public async Task AddFileAsync(string filePath, string fileHash)
    {
        await using SqliteConnection? connection = new(connectionString);
        const string sql = "INSERT OR IGNORE INTO file_list (file_path, file_hash, status) VALUES (@FilePath, @FileHash, 'PENDING');";
        await connection.ExecuteAsync(sql, new { FilePath = filePath, FileHash = fileHash });
    }

    /// <summary>
    /// Gets the next file that needs processing.
    /// It prioritizes 'PENDING' files, but if none are available, it will look for
    /// 'IN_PROGRESS' files that still have pending batches.
    /// </summary>
    /// <returns>A Task representing the asynchronous operation, with a result of a FileRecord for the next file, or null if no work is available.</returns>
    public async Task<FileRecord> GetNextFileToProcessAsync()
    {
        await using SqliteConnection? connection = new(connectionString);
        // First, look for files that have never been touched.
        const string pendingSql = "SELECT id AS Id, file_path AS FilePath, file_hash AS FileHash, status AS Status, has_password AS HasPassword, password_found AS PasswordFound, started_at AS StartedAt, completed_at AS CompletedAt, total_time_minutes AS TotalTimeMinutes, instance_id AS InstanceId FROM file_list WHERE status = 'PENDING' ORDER BY id LIMIT 1;";
        FileRecord? file = await connection.QuerySingleOrDefaultAsync<FileRecord>(pendingSql);

        // If no pending files, look for in-progress files that have available batches.
        // This allows resuming work on files that were started but not finished.
        if (file == null)
        {
            const string inProgressSql = """
                SELECT id AS Id, file_path AS FilePath, file_hash AS FileHash, status AS Status, has_password AS HasPassword, password_found AS PasswordFound, started_at AS StartedAt, completed_at AS CompletedAt, total_time_minutes AS TotalTimeMinutes, instance_id AS InstanceId
                FROM file_list 
                WHERE status = 'IN_PROGRESS' AND id IN (
                    SELECT file_id FROM trial_batches WHERE status = 'PENDING'
                )
                ORDER BY id
                LIMIT 1;
            """;
            file = await connection.QuerySingleOrDefaultAsync<FileRecord>(inProgressSql);
        }

        return file;
    }

    /// <summary>
    /// Updates the status of a file (e.g., to 'IN_PROGRESS').
    /// </summary>
    /// <param name="fileId">The ID of the file to update.</param>
    /// <param name="status">The new status string.</param>
    public async Task UpdateFileStatusAsync(int fileId, string status)
    {
        await using SqliteConnection? connection = new(connectionString);
        const string sql = "UPDATE file_list SET status = @Status, started_at = @StartedAt WHERE id = @FileId;";
        await connection.ExecuteAsync(sql, new { Status = status, StartedAt = DateTime.Now, FileId = fileId });
    }

    /// <summary>
    /// Updates a file record with the final result after processing is complete.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    /// <param name="password">The password if found, or a status string ('NO_PASSWORD', 'FAILED').</param>
    /// <param name="durationMinutes">The total time spent processing the file.</param>
    public async Task UpdateFileResultAsync(int fileId, string? password, double durationMinutes)
    {
        await using SqliteConnection? connection = new(connectionString);
        const string sql = """
                          UPDATE file_list 
                          SET status = 'COMPLETED', 
                              password_found = @Password, 
                              completed_at = @CompletedAt, 
                              total_time_minutes = @Duration 
                          WHERE id = @FileId;
                      """;
        await connection.ExecuteAsync(sql, new { Password = password, CompletedAt = DateTime.Now, Duration = durationMinutes, FileId = fileId });
    }

    /// <summary>
    /// Retrieves a specific file record by its ID.
    /// </summary>
    /// <param name="fileId">The ID of the file to retrieve.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of the corresponding FileRecord.</returns>
    public async Task<FileRecord> GetFileByIdAsync(int fileId)
    {
        await using SqliteConnection? connection = new(connectionString);
        const string sql = "SELECT id AS Id, file_path AS FilePath, file_hash AS FileHash, status AS Status, has_password AS HasPassword, password_found AS PasswordFound, started_at AS StartedAt, completed_at AS CompletedAt, total_time_minutes AS TotalTimeMinutes, instance_id AS InstanceId FROM file_list WHERE id = @FileId;";
        return await connection.QuerySingleOrDefaultAsync<FileRecord>(sql, new { FileId = fileId });
    }

    /// <summary>
    /// Gets all file records matching a specific hash.
    /// </summary>
    /// <param name="fileHash">The SHA256 hash to search for.</param>
    /// <returns>A collection of matching file records.</returns>
    public async Task<IEnumerable<FileRecord>> GetByHashAsync(string fileHash)
    {
        await using SqliteConnection? connection = new(connectionString);
        const string sql = "SELECT id AS Id, file_path AS FilePath, file_hash AS FileHash, status AS Status, has_password AS HasPassword, password_found AS PasswordFound, started_at AS StartedAt, completed_at AS CompletedAt, total_time_minutes AS TotalTimeMinutes, instance_id AS InstanceId FROM file_list WHERE file_hash = @FileHash;";
        return await connection.QueryAsync<FileRecord>(sql, new { FileHash = fileHash });
    }
}
