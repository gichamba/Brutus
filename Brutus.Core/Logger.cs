namespace Brutus.Core;

/// <summary>
/// A simple logger for writing messages to both the console and a log file.
/// </summary>
public class Logger
{
    private readonly string _logFilePath;

    /// <summary>
    /// Default constructor. Generates a new GUID for the instance ID.
    /// </summary>
    public Logger() : this(Guid.NewGuid().ToString()) { }

    /// <summary>
    /// Constructor that accepts an instance ID.
    /// </summary>
    /// <param name="instanceId">A unique identifier for the current application instance.</param>
    public Logger(string instanceId)
    {
        string logsDirectory = "Logs";
        Directory.CreateDirectory(logsDirectory);
        _logFilePath = Path.Combine(logsDirectory, $"log_{instanceId}.txt");
    }

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
        File.AppendAllText(_logFilePath, logMessage + Environment.NewLine);
    }
}