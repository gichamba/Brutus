using Brutus.Core.Database.Models;
using Brutus.Core.Database.Repositories;
using Brutus.Core.Security;

namespace Brutus.Core;

/// <summary>
/// Scans for PDF files, checks for duplicates using hashes, and adds new files to the database.
/// </summary>
public class FileScanner(IFileRepository fileRepository, Logger logger, FoundLogger foundLogger) {
    /// <summary>
    /// Scans a path for PDF files, processes them against the database, and adds new ones.
    /// </summary>
    /// <param name="path">The file or directory path to scan.</param>
    public async Task ScanAndAddFilesAsync(string path)
    {
        logger.LogInfo($"Scanning for PDF files in: {path}");
        List<string>? files = FindPdfFiles(path).ToList();
        logger.LogInfo($"Found {files.Count} PDF files. Calculating hashes...");

        foreach (string? file in files)
        {
            string? fileHash = FileHasher.ComputeSha256(file);
            IEnumerable<FileRecord>? existingFiles = await fileRepository.GetByHashAsync(fileHash);

            FileRecord? completedFile = existingFiles.FirstOrDefault(f => f.Status == "COMPLETED");

            if (completedFile != null)
            {
                logger.LogInfo($"DUPLICATE: File {file} already processed as {completedFile.FilePath}, password: {completedFile.PasswordFound}");
                foundLogger.Log(file, completedFile.PasswordFound, 0.00);
                continue;
            }

            await fileRepository.AddFileAsync(file, fileHash);
        }
        logger.LogInfo("Finished scanning and adding files.");
    }

    /// <summary>
    /// Finds all PDF files in a given path. The path can be a single file or a directory.
    /// If it's a directory, it searches recursively.
    /// </summary>
    /// <param name="path">The path to a file or directory.</param>
    /// <returns>An enumerable collection of strings, each representing the full path to a PDF file.</returns>
    private IEnumerable<string> FindPdfFiles(string path)
    {
        if (File.Exists(path))
        {
            if (path.EndsWith(".pdf", StringComparison.OrdinalIgnoreCase))
            {
                return [path];
            }
            return [];
        }

        if (Directory.Exists(path))
        {
            return Directory.EnumerateFiles(path, "*.pdf", SearchOption.AllDirectories);
        }

        return [];
    }
}