namespace Brutus.Console;

/// <summary>
/// Handles parsing of command-line arguments.
/// </summary>
public class CommandLineHandler
{
    /// <summary>
    /// Gets the file or directory path from the command-line arguments.
    /// It expects exactly one argument.
    /// </summary>
    /// <param name="args">The command-line arguments.</param>
    /// <returns>The path if valid, otherwise null.</returns>
    public string? GetPath(string[] args)
    {
        if (args.Length != 1)
        {
            System.Console.WriteLine("Error: Invalid number of arguments.");
            System.Console.WriteLine("Usage: Brutus.exe <file_path_or_directory_path>");
            return null;
        }

        string path = args[0];
        if (!File.Exists(path) && !Directory.Exists(path))
        {
            System.Console.WriteLine($"Error: The specified path does not exist: {path}");
            return null;
        }

        return path;
    }
}
