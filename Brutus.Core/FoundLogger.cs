namespace Brutus.Core;

/// <summary>
/// A specific logger for writing final results to a structured tab-delimited file (Found.txt).
/// </summary>
public class FoundLogger
{
    private readonly string _logFilePath;

    public FoundLogger(string instanceId)
    {
        string logsDirectory = "Logs";
        Directory.CreateDirectory(logsDirectory);
        _logFilePath = Path.Combine(logsDirectory, $"Found_{instanceId}.txt");

        // Write header to the tab-delimited file if it's new.
        if (!File.Exists(_logFilePath))
        {
            File.WriteAllText(_logFilePath, "File_Path\tPassword\tTime_Minutes" + Environment.NewLine);
        }
    }
    /// <summary>
    /// Logs a single result entry to the Found.txt file.
    /// </summary>
    /// <param name="filePath">The path of the file that was processed.</param>
    /// <param name="password">The password if found, or a status string ('N/A', 'FAILED').</param>
    /// <param name="durationMinutes">The time taken to process the file.</param>
    public void Log(string filePath, string? password, double durationMinutes)
    {
        string durationString = durationMinutes.ToString("F2");
        string logMessage = $"{filePath}\t{password ?? "N/A"}\t{durationString}";
        File.AppendAllText(_logFilePath, logMessage + Environment.NewLine);
    }
}