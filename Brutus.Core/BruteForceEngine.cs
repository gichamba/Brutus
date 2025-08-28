namespace Brutus.Core;

/// <summary>
/// Handles the actual password testing logic.
/// </summary>
public class BruteForceEngine(PdfProcessor pdfProcessor, Logger logger)
{
    /// <summary>
    /// Attempts to crack a PDF password by testing the entire 6-digit numeric range.
    /// This is a convenience method that calls CrackPasswordRange with the full default range.
    /// </summary>
    /// <param name="filePath">The path to the PDF file.</param>
    /// <returns>The found password, or null if not found.</returns>
    public string? CrackPassword(string filePath)
    {
        return CrackPasswordRange(filePath, "000000", "999999");
    }

    /// <summary>
    /// Attempts to crack a PDF password by testing a specific numeric range.
    /// </summary>
    /// <param name="filePath">The path to the PDF file.</param>
    /// <param name="startPassword">The starting 6-digit password string (e.g., "010000").</param>
    /// <param name="endPassword">The ending 6-digit password string (e.g., "019999").</param>
    /// <returns>The found password, or null if not found in the specified range.</returns>
    public string? CrackPasswordRange(string filePath, string startPassword, string endPassword)
    {
        int start = int.Parse(startPassword);
        int end = int.Parse(endPassword);

        for (int i = start; i <= end; i++)
        {
            string password = i.ToString("D6");
            if (i % 1000 == 0) // Log progress more frequently for batch processing.
            {
                logger.LogInfo($"Testing range {startPassword}-{endPassword}, current: {password}");
            }

            if (pdfProcessor.TryOpenWithPassword(filePath, password))
            {
                return password;
            }
        }
        return null;
    }
}