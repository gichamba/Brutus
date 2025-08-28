namespace Brutus.Core.Database.Models;

/// <summary>
/// Represents a record in the trial_batches table.
/// </summary>
public class TrialBatch
{
    /// <summary>The unique identifier for the batch.</summary>
    public int Id { get; set; }
    /// <summary>The foreign key referencing the file this batch belongs to.</summary>
    public int file_id { get; set; }
    /// <summary>The 0-based index of this batch (0-99).</summary>
    public int batch_index { get; set; }
    /// <summary>The starting password for this batch (e.g., "000000").</summary>
    public string range_from { get; set; } = string.Empty;
    /// <summary>The ending password for this batch (e.g., "009999").</summary>
    public string range_to { get; set; } = string.Empty;
    /// <summary>The current status of this batch (e.g., PENDING, CHECKED_OUT, COMPLETED).</summary>
    public string status { get; set; } = string.Empty;
    /// <summary>The timestamp when the batch was checked out.</summary>
    public DateTime? checked_out_at { get; set; }
    /// <summary>The timestamp when the batch was completed.</summary>
    public DateTime? completed_at { get; set; }
    /// <summary>The ID of the instance that processed this batch.</summary>
    public string? instance_id { get; set; }
}