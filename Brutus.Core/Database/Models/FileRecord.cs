namespace Brutus.Core.Database.Models;

/// <summary>
/// Represents a record in the file_list table.
/// </summary>
public class FileRecord
{
    /// <summary>The unique identifier for the file record.</summary>
    public int Id { get; set; }
    /// <summary>The absolute path to the PDF file.</summary>
    public string FilePath { get; set; } = string.Empty;
    /// <summary>The SHA256 hash of the file content.</summary>
    public string FileHash { get; set; } = string.Empty;
    /// <summary>The current processing status (e.g., PENDING, IN_PROGRESS, COMPLETED).</summary>
    public string Status { get; set; } = string.Empty;
    /// <summary>Whether the file has a password (0=unknown, 1=yes, 2=no).</summary>
    public int HasPassword { get; set; }
    /// <summary>The password if found.</summary>
    public string? PasswordFound { get; set; }
    /// <summary>The timestamp when processing started.</summary>
    public DateTime? StartedAt { get; set; }
    /// <summary>The timestamp when processing completed.</summary>
    public DateTime? CompletedAt { get; set; }
    /// <summary>The total time taken to process the file in minutes.</summary>
    public double? TotalTimeMinutes { get; set; }
    /// <summary>The ID of the instance currently processing the file.</summary>
    public string? InstanceId { get; set; }
}