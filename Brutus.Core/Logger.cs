namespace Brutus.Core;

/// <summary>
/// A simple logger for writing messages to both the console and a log file.
/// </summary>
/// <param name="logFilePath">The path to the log file. Defaults to "log.txt".</param>
public class Logger(string logFilePath = "log.txt")
{
    /// <summary>
    /// Logs an informational message.
    /// </summary>
    public void LogInfo(string message)
    {
        Log("INFO", message);
    }

    /// <summary>
    /// Logs a success message.
    /// </summary>
    public void LogSuccess(string message)
    {
        Log("SUCCESS", message);
    }

    /// <summary>
    /// Logs an error message.
    /// </summary>
    public void LogError(string message)
    {
        Log("ERROR", message);
    }

    /// <summary>
    /// Logs a message for a skipped operation.
    /// </summary>
    public void LogSkip(string message)
    {
        Log("SKIP", message);
    }

    /// <summary>
    /// Private helper method to format and write the log message.
    /// </summary>
    private void Log(string level, string message)
    {
        string logMessage = $"[{DateTime.Now:yyyy-MM-dd HH:mm:ss}] {level}: {message}";
        Console.WriteLine(logMessage);
        File.AppendAllText(logFilePath, logMessage + Environment.NewLine);
    }
}
