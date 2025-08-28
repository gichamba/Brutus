using Brutus.Core.Database.Models;

namespace Brutus.Core.Database.Repositories;

/// <summary>
/// Defines the contract for file data access operations.
/// </summary>
public interface IFileRepository
{
    /// <summary>
    /// Adds a new file to the database.
    /// </summary>
    /// <param name="filePath">The path of the file to add.</param>
    /// <param name="fileHash">The SHA256 hash of the file.</param>
    Task AddFileAsync(string filePath, string fileHash);

    /// <summary>
    /// Gets the next available file to process.
    /// </summary>
    /// <returns>A Task representing the asynchronous operation, with a result of a FileRecord object.</returns>
    Task<FileRecord> GetNextFileToProcessAsync();

    /// <summary>
    /// Updates the status of a file.
    /// </summary>
    /// <param name="fileId">The ID of the file to update.</param>
    /// <param name="status">The new status.</param>
    Task UpdateFileStatusAsync(int fileId, string status);

    /// <summary>
    /// Updates the final result of a file processing attempt.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    /// <param name="password">The password if found.</param>
    /// <param name="durationMinutes">The total processing time.</param>
    Task UpdateFileResultAsync(int fileId, string? password, double durationMinutes);

    /// <summary>
    /// Gets a file record by its unique ID.
    /// </summary>
    /// <param name="fileId">The ID of the file.</param>
    /// <returns>A Task representing the asynchronous operation, with a result of a FileRecord object.</returns>
    Task<FileRecord> GetFileByIdAsync(int fileId);

    /// <summary>
    /// Gets all file records matching a specific hash.
    /// </summary>
    /// <param name="fileHash">The SHA256 hash to search for.</param>
    /// <returns>A collection of matching file records.</returns>
    Task<IEnumerable<FileRecord>> GetByHashAsync(string fileHash);
}
