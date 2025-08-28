namespace Brutus.Core.Models;

/// <summary>
/// Represents the status of a file processing operation.
/// </summary>
public enum ProcessStatus
{
    /// <summary>The file was not password-protected and was skipped.</summary>
    Skipped,
    /// <summary>The password was found successfully.</summary>
    Success,
    /// <summary>The password was not found within the given range.</summary>
    Failure
}

/// <summary>
/// Represents the final result of a brute force attempt on a single file.
/// </summary>
/// <param name="FilePath">The path to the processed file.</param>
/// <param name="Status">The final status of the operation.</param>
/// <param name="FoundPassword">The password, if found.</param>
/// <param name="Duration">The total time taken for the operation.</param>
public record ProcessingResult(string FilePath, ProcessStatus Status, string? FoundPassword = null, TimeSpan? Duration = null);